use std::{
    error,
    fmt::{self, Display, Formatter},
};

use crate::{AppState, Error, post::image::image_to_webp};
use crate::{
    entity::{
        posts::{self},
        users,
    },
    user::UserResponse,
};
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_session::Session;
use actix_web::{HttpResponse, Responder, ResponseError, get, http::StatusCode, post, web::Data};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QueryOrder};
use serde::Serialize;

mod image;

type Result<T> = std::result::Result<T, Error<PostError>>;

#[derive(Debug)]
enum PostError {
    InvalidImage,
    UserNotFound,
    CaptionTooLong,
}

impl From<PostError> for Error<PostError> {
    fn from(err: PostError) -> Self {
        Self::Custom(err)
    }
}

impl Display for PostError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidImage => write!(f, "image is invalid"),
            Self::UserNotFound => write!(f, "couldn't find linked user"),
            Self::CaptionTooLong => write!(f, "the caption is too long, max 200 characters"),
        }
    }
}

impl ResponseError for PostError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

impl error::Error for PostError {}

#[derive(MultipartForm)]
struct CreatePostForm {
    text: Option<Text<String>>,
    #[multipart(limit = "20MB")]
    image: TempFile,
}

#[derive(Serialize)]
struct PostResponse {
    id: i64,
    user: UserResponse,
    image: String,
    text: Option<String>,
}

impl TryFrom<(posts::Model, Option<users::Model>)> for PostResponse {
    type Error = Error<PostError>;

    fn try_from((post, user): (posts::Model, Option<users::Model>)) -> Result<Self> {
        match user {
            Some(user) => Ok(Self {
                id: post.id,
                user: user.into(),
                image: format!("/images/{}.webp", post.image),
                text: post.text,
            }),
            None => Err(PostError::UserNotFound.into()),
        }
    }
}

#[post("/post/create")]
async fn create(
    MultipartForm(form): MultipartForm<CreatePostForm>,
    state: Data<AppState>,
    session: Session,
) -> Result<impl Responder> {
    // check that text is max 200 chars
    if let Some(text) = form.text.as_ref().map(|t| &t.0)
        && text.len() > 200
    {
        return Err(PostError::CaptionTooLong.into());
    }

    match session.get::<i64>("user_id")? {
        Some(uid) => {
            let user = users::Entity::find_by_id(uid).one(&state.conn).await?;

            let webp = image_to_webp(form.image, &state).await?;
            let post = posts::ActiveModel {
                user_id: Set(uid),
                text: Set(form.text.map(|t| t.0)),
                image: Set(webp),
                ..Default::default()
            };

            let post = post.insert(&state.conn).await?;

            Ok(HttpResponse::Ok().json(PostResponse::try_from((post, user))?))
        }
        _ => Err(Error::Unauthorized),
    }
}

#[get("/post/timeline")]
async fn timeline(state: Data<AppState>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(
        posts::Entity::find()
            .order_by_desc(posts::Column::CreatedAt)
            .find_also_related(users::Entity)
            .all(&state.conn)
            .await?
            .into_iter()
            .map(PostResponse::try_from)
            .collect::<Result<Vec<PostResponse>>>()?,
    ))
}
