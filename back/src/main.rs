use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    cookie::Key,
    middleware,
    rt::time::sleep,
    web::{self, get, post},
};
use serde::Serialize;
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};
use std::{sync::LazyLock, time::Duration};

use crate::{
    models::{auth_model::Claims, user_model::User},
    services::auth_service,
    shared::SQLable,
};

pub mod errors;
pub mod middlewares;
pub mod models;
pub mod services;
pub mod shared;
pub mod utils;

#[derive(Serialize)]
struct PingResponse {
    message: String,
}

const RESET: bool = false;

static DB_POOL: LazyLock<Pool<MySql>> = std::sync::LazyLock::new(|| {
    MySqlPoolOptions::new()
        .max_connections(6)
        .connect_lazy(&std::env::var("DATABASE_URL").expect("DATABASE_URL not define !"))
        .expect("Can't connect to DB")
});

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = MySqlPoolOptions::new()
        .max_connections(6)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not define !"))
        .await
        .expect("Can't connect to DB");

    if RESET {
        down_all_table().await.expect("down all table failed");
    }

    up_all_table()
        .await
        .expect("error while uping all the tables !");

    let rows = sqlx::query("SHOW TABLES")
        .fetch_all(&pool)
        .await
        .expect("Impossible de lister les tables");

    println!("Now listing tables");
    for row in rows {
        println!("Table: {:?}", row);
    }
    println!("Finish listing tables");

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
                .cookie_same_site(actix_web::cookie::SameSite::Lax) // protège contre CSRF basique
                .session_lifecycle(
                    actix_session::config::PersistentSession::default()
                        .session_ttl(time::Duration::days(7)), // 7 jours
                )
                .build(),
            )
            .service(
                web::scope("/auth")
                    .route("/login", post().to(auth_service::login))
                    .route("/register", post().to(auth_service::register))
                    .route("/verify", post().to(auth_service::verify)),
            )
            .service(web::scope("/public").route("/ping", get().to(handle_ping)))
            .service(
                web::scope("")
                    .wrap(middleware::from_fn(
                        middlewares::auth_middleware::auth_middleware,
                    ))
                    .route("/login_test", get().to(login_test)),
            )
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
async fn up_all_table() -> Result<(), Box<dyn std::error::Error>> {
    User::up().await?;

    Ok(())
}
async fn down_all_table() -> Result<(), Box<dyn std::error::Error>> {
    User::down().await?;

    Ok(())
}
async fn login_test(claim: Claims) -> HttpResponse {
    let message = if claim.admin {
        format!("Hi admin n°{}", claim.id)
    } else {
        format!("Hi user n°{}", claim.id)
    };
    HttpResponse::Ok().json(message)
}
