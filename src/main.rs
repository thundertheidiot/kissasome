use std::env;

use actix_files::Files;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use actix_web::web::Data;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use migration::Migrator;
use migration::MigratorTrait;
use sea_orm::{Database, DatabaseConnection};
use serde::Serialize;

use crate::user::{get, login, register};

pub mod entity;
pub mod user;

#[derive(Serialize)]
struct Message {
    message: String,
}

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[get("/hello")]
async fn hello() -> impl Responder {
    let message = Message {
        message: String::from("hello world"),
    };

    HttpResponse::Ok().json(message)
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let conn = env::var("DATABASE_URL")?;
    let conn = Database::connect(&conn).await?;
    Migrator::up(&conn, None).await?;

    let session_key = env::var("SESSION_KEY")?;
    let session_key = Key::from(&session_key.as_bytes());

    let state = AppState { conn };

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                session_key.clone(),
            ))
            .service(
                web::scope("/api")
                    .service(hello)
                    .service(register)
                    .service(login)
                    .service(get),
            )
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
