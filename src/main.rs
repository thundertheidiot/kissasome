#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![feature(trait_alias)]
use std::env;
use std::path::PathBuf;

use actix_files::Files;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use actix_web::middleware::from_fn;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use migration::Migrator;
use migration::MigratorTrait;
use sea_orm::{Database, DatabaseConnection};

pub mod entity;
pub mod error;
pub mod master_password;
pub mod post;
pub mod user;

pub use error::Error;
#[derive(Debug, Clone)]
struct AppState {
    image_path: PathBuf,
    conn: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let conn = env::var("DATABASE_URL")?;
    let conn = Database::connect(&conn).await?;
    Migrator::up(&conn, None).await?;

    let session_key = env::var("SESSION_KEY")?;
    let session_key = Key::from(session_key.as_bytes());

    let image_path: PathBuf = env::var("IMAGE_PATH")
        .unwrap_or("./images".to_string())
        .into();

    let state = AppState { image_path, conn };

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .wrap(from_fn(master_password::basic_auth))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                session_key.clone(),
            ))
            .service(
                web::scope("/api")
                    .service(user::register)
                    .service(user::login)
                    .service(user::logout)
                    .service(user::get)
                    .service(post::create)
                    .service(post::timeline),
            )
            .service(Files::new("/images", "./images"))
            .service(
                Files::new("/", "./dist")
                    .index_file("index.html")
                    .prefer_utf8(true),
            )
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await?;

    Ok(())
}
