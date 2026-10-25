use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
	message: String,
}

#[get("/hello")]
async fn hello() -> impl Responder {
	let message = Message { message: String::from("hello world") };

	HttpResponse::Ok().json(message)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| App::new()
					.service(web::scope("/api")
							 .service(hello))
					.service(Files::new("/", "./dist")
							 .index_file("index.html")
							 .prefer_utf8(true)
					)
	)
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}
