use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    web::{self, get},
};
use serde::Serialize;

#[derive(Serialize)]
struct PingResponse {
    message: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(web::scope("").route("/ping", get().to(handle_ping))))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}

async fn handle_ping() -> impl Responder {
    let response = PingResponse {
        message: "pong".to_string(),
    };
    HttpResponse::Ok().json(response)
}
