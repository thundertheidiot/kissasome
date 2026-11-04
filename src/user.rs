use crate::{AppState, entity::users};
use actix_session::Session;
use actix_web::{
    HttpResponse, Responder, get, post,
    web::{Data, Json},
};
use anyhow::anyhow;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

fn validate_password(password: &str) -> bool {
    password.len() > 7
        && password.chars().any(|c| c.is_ascii_digit())
        && password.chars().any(char::is_uppercase)
        && password.chars().any(char::is_lowercase)
}

pub async fn try_register(
    conn: &DatabaseConnection,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    if !validate_password(password) {
        return Err(anyhow!("password doesn't meet requirements"));
    }

    let argon2 = Argon2::default();

    let bytes = password.as_bytes();
    let password_hash = argon2.hash_password(bytes)?.to_string();

    let user = users::ActiveModel {
        username: Set(username.into()),
        password_hash: Set(password_hash),
        ..Default::default()
    };

    let _ = user.insert(conn).await?;

    Ok(())
}

async fn try_login(
    conn: &DatabaseConnection,
    username: &str,
    password: &str,
) -> anyhow::Result<Option<users::Model>> {
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
            Ok(Some(user))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[derive(Deserialize)]
struct ApiUserRequest {
    username: String,
    password: String,
}

#[post("/user/register")]
async fn register(body: Json<ApiUserRequest>, state: Data<AppState>) -> impl Responder {
    match try_register(&state.conn, &body.username, &body.password).await {
        Ok(()) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::BadRequest().body(format!("registration failed: {e:?}")),
    }
}

#[post("/user/login")]
async fn login(
    body: Json<ApiUserRequest>,
    state: Data<AppState>,
    session: Session,
) -> impl Responder {
    match try_login(&state.conn, &body.username, &body.password).await {
        Ok(Some(user)) => match session.insert("user_id", user.id) {
            Ok(()) => HttpResponse::Ok(),
            _ => HttpResponse::BadRequest(),
        },
        _ => HttpResponse::Unauthorized(),
    }
}

#[derive(Serialize)]
struct GetUserResp {
    id: i64,
    username: String,
}

#[get("/user/get")]
async fn get(state: Data<AppState>, session: Session) -> impl Responder {
    if let Ok(Some(uid)) = session.get::<i64>("user_id")
        && let Ok(Some(user)) = users::Entity::find_by_id(uid).one(&state.conn).await
    {
        HttpResponse::Ok().json(GetUserResp {
            id: user.id,
            username: user.username,
        })
    } else {
        HttpResponse::Unauthorized().into()
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
