use std::fmt::{self, Display, Formatter};

use crate::{AppState, Error, entity::users};
use actix_session::Session;
use actix_web::{
    HttpResponse, Responder, ResponseError, get,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Error<UserError>>;

#[derive(Debug)]
enum UserError {
    PasswordLength,
    PasswordDigitMissing,
    PasswordMixedLetters,
    NonUniqueUsername,
    InvalidCredentials,
    UserNotFound,
}

impl Display for UserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::PasswordLength => write!(f, "password must be at least 8 characters long"),
            Self::PasswordDigitMissing => write!(f, "password must contain at least one digit"),
            Self::PasswordMixedLetters => write!(
                f,
                "password must contain at least one lowercase and one uppercase letter"
            ),
            Self::NonUniqueUsername => write!(f, "username must be unique"),
            Self::InvalidCredentials => write!(f, "username or password is incorrect"),
            Self::UserNotFound => write!(f, "this user doesn't exist"),
        }
    }
}

impl ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::UserNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

impl From<UserError> for Error<UserError> {
    fn from(err: UserError) -> Self {
        Self::Custom(err)
    }
}

fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        Err(UserError::PasswordLength.into())
    } else if !password.chars().any(|c| c.is_ascii_digit()) {
        Err(UserError::PasswordDigitMissing.into())
    } else if !(password.chars().any(char::is_uppercase)
        && password.chars().any(char::is_lowercase))
    {
        Err(UserError::PasswordMixedLetters.into())
    } else {
        Ok(())
    }
}

async fn try_register(
    conn: &DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<users::Model> {
    validate_password(password)?;

    if let Ok(Some(_user)) = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await
    {
        return Err(UserError::NonUniqueUsername.into());
    }

    let argon2 = Argon2::default();

    let bytes = password.as_bytes();
    let password_hash = argon2.hash_password(bytes)?.to_string();

    let user = users::ActiveModel {
        username: Set(username.into()),
        password_hash: Set(password_hash),
        ..Default::default()
    };

    Ok(user.insert(conn).await?)
}

async fn try_login(
    conn: &DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<users::Model> {
    if let Some(user) = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await?
    {
        let hash = PasswordHash::new(&user.password_hash)?;

        if Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
        {
            Ok(user)
        } else {
            Err(UserError::InvalidCredentials.into())
        }
    } else {
        Err(UserError::InvalidCredentials.into())
    }
}

#[derive(Deserialize)]
struct ApiUserRequest {
    username: String,
    password: String,
}

#[post("/user/register")]
async fn register(
    body: Json<ApiUserRequest>,
    state: Data<AppState>,
    session: Session,
) -> Result<impl Responder> {
    let user = try_register(&state.conn, &body.username, &body.password).await?;
    session.insert("user_id", user.id)?;
    Ok(HttpResponse::Created().finish())
}

#[post("/user/login")]
async fn login(
    body: Json<ApiUserRequest>,
    state: Data<AppState>,
    session: Session,
) -> Result<impl Responder> {
    let user = try_login(&state.conn, &body.username, &body.password).await?;
    session.insert("user_id", user.id)?;
    Ok(HttpResponse::Ok())
}

#[post("/user/logout")]
async fn logout(session: Session) -> Result<impl Responder> {
    if session.contains_key("user_id") {
        session
            .remove("user_id")
            .ok_or(Error::Internal("session error"))?;
        Ok(HttpResponse::Ok())
    } else {
        Ok(HttpResponse::Unauthorized())
    }
}

#[derive(Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
}

impl From<users::Model> for UserResponse {
    fn from(model: users::Model) -> Self {
        Self {
            id: model.id,
            username: model.username,
        }
    }
}

#[get("/user/get")]
async fn get(state: Data<AppState>, session: Session) -> impl Responder {
    if let Ok(Some(uid)) = session.get::<i64>("user_id")
        && let Ok(Some(user)) = users::Entity::find_by_id(uid).one(&state.conn).await
    {
        let resp: UserResponse = user.into();
        HttpResponse::Ok().json(resp)
    } else {
        HttpResponse::Unauthorized().into()
    }
}

#[get("/user/get/{user}")]
async fn get_by_id(path: Path<i64>, state: Data<AppState>) -> Result<impl Responder> {
    match users::Entity::find_by_id(path.into_inner())
        .one(&state.conn)
        .await?
    {
        Some(user) => {
            let resp: UserResponse = user.into();
            Ok(HttpResponse::Ok().json(resp))
        }
        None => Err(UserError::UserNotFound.into()),
    }
}

mod test {
    #![allow(unused_imports)]

    use migration::{Migrator, MigratorTrait};
    use sea_orm::Database;

    use crate::user::try_register;

    #[tokio::test]
    async fn unique_usernames() -> anyhow::Result<()> {
        let conn = Database::connect("sqlite::memory:").await?;
        Migrator::up(&conn, None).await?;

        try_register(&conn, "user", "Password1").await?;
        try_register(&conn, "user", "Password1")
            .await
            .expect_err("doesn't fail on unique username");

        Ok(())
    }

    #[tokio::test]
    async fn password_requirements() -> anyhow::Result<()> {
        let conn = Database::connect("sqlite::memory:").await?;
        Migrator::up(&conn, None).await?;

        try_register(&conn, "user", "short")
            .await
            .expect_err("too short should fail");
        try_register(&conn, "user", "Short1")
            .await
            .expect_err("too short should fail");

        try_register(&conn, "user", "longenoughpassword")
            .await
            .expect_err("long without other requirements should fail");
        try_register(&conn, "user", "Longenoughpassword")
            .await
            .expect_err("long without number should fail");
        try_register(&conn, "user", "longenoughpassword1")
            .await
            .expect_err("long without uppercase should fail");

        try_register(&conn, "user", "Longenoughpassword1")
            .await
            .expect("should fullfill requirements");

        Ok(())
    }
}
