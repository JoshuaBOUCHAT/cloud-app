use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    cookie::Key,
    web::{self, get},
};
use serde::Serialize;
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};
use std::sync::LazyLock;

use crate::{
    auth::{
        auth_models::{claims::Claims, token::TokenAble},
        auth_service::{login, logout, refresh_token, register, verify},
    },
    services::openapi_service::openapi_yaml,
    utils::redis_utils::{REDIS_POOL, init_redis_pool},
};

pub mod auth;
pub mod constants;
pub mod errors;
pub mod models;
pub mod services;
pub mod shared;
pub mod utils;

pub const SECRET: &[u8; 44] = b"laOOVyHM6s3IcgDAty1O7AXAdRZR6eaaQi65v3qhVRg=";

#[derive(Serialize)]
struct PingResponse {
    message: String,
}

static DB_POOL: LazyLock<Pool<MySql>> = std::sync::LazyLock::new(|| {
    MySqlPoolOptions::new()
        .max_connections(6)
        .connect_lazy(&std::env::var("DATABASE_URL").expect("DATABASE_URL not define !"))
        .expect("Can't connect to DB")
});
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let rows = sqlx::query("SHOW TABLES")
        .fetch_all(&*DB_POOL)
        .await
        .expect("Impossible de lister les tables");
    println!("initialising redis cell");
    init_redis_pool().await;
    REDIS_POOL.wait();
    println!("initialising successe");

    println!("Now listing tables");
    for row in rows {
        println!("Table: {:?}", row);
    }
    println!("Finish listing tables");

    let claim = Claims::new_user_claim(4);
    if let Ok(ser) = claim.encode() {
        println!("Claims serialisation: {}", ser.as_ref());
        println!("Claims as json: {}", serde_json::to_string(&claim).unwrap());
        let decoded_claim = Claims::decode(ser.as_ref()).expect("Deserialisation should not fail");
        println!(
            "Decoded as json: {}",
            serde_json::to_string(&claim).unwrap()
        );
    }

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(
                        b"THE VERY VERY VERY SECURE KEY TO ALLOW A VERY VERY VERY SECRET SESSION",
                    ),
                )
                .cookie_secure(true) // HTTPS seulement
                .cookie_http_only(true) // pas accessible via JS
                .cookie_same_site(actix_web::cookie::SameSite::Strict) // protège contre CSRF basique
                .session_lifecycle(
                    actix_session::config::PersistentSession::default()
                        .session_ttl(time::Duration::days(7)), // 7 jours
                )
                .build(),
            )
            .service(
                web::scope("/auth")
                    .service(login)
                    .service(register)
                    .service(verify)
                    .service(refresh_token)
                    .service(logout),
            )
            .service(openapi_yaml)
            .service(web::scope("/public").route("/ping", get().to(handle_ping)))
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
