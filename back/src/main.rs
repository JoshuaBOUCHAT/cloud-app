use std::{fmt::Debug, time::Duration};

use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    rt::time::sleep,
    web::{self, get},
};
use serde::Serialize;
pub mod models;
pub mod services;
pub mod shared;

#[derive(Serialize)]
struct PingResponse {
    message: String,
}

use sqlx::Executor;
use sqlx::mysql::MySqlPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    sleep(Duration::from_secs(5)).await;
    let pool = MySqlPoolOptions::new()
        .max_connections(6)
        .connect(env!("DATABASE_URL"))
        .await
        .expect("Can't connect to DB");

    let rows = sqlx::query("SHOW TABLES")
        .fetch_all(&pool)
        .await
        .expect("Impossible de lister les tables");

    println!("Now listing tables");
    for row in rows {
        println!("Table: {:?}", row);
    }
    println!("Finish listing tables");

    let pool_data = web::Data::new(pool.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .service(web::scope("").route("/ping", get().to(handle_ping)))
    })
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
