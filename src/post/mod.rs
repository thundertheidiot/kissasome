use std::{
    error,
    fmt::{self, Display, Formatter},
};

use crate::entity::posts;
use crate::{AppState, Error, post::image::image_to_webp};
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
}

impl Display for PostError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "")
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
    user_id: i64,
    image: String,
    text: Option<String>,
}

impl From<posts::Model> for PostResponse {
    fn from(model: posts::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            image: format!("/images/{}.webp", model.image),
            text: model.text,
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
        return Ok(HttpResponse::Unauthorized().into());
    }

    match session.get::<i64>("user_id")? {
        Some(uid) => {
            let webp = image_to_webp(form.image, &state).await?;
            let post = posts::ActiveModel {
                user_id: Set(uid),
                text: Set(form.text.map(|t| t.0)),
                image: Set(webp),
                ..Default::default()
            };

            let post = post.insert(&state.conn).await?;

            Ok(HttpResponse::Ok().json(PostResponse::from(post)))
        }
        _ => Ok(HttpResponse::Unauthorized().into()),
    }
}

#[get("/post/timeline")]
async fn timeline(state: Data<AppState>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(
        posts::Entity::find()
            .order_by_desc(posts::Column::CreatedAt)
            .all(&state.conn)
            .await?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<PostResponse>>(),
    ))
}
