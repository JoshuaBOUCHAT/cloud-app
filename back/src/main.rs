use std::{sync::LazyLock, time::Duration};

use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    rt::time::sleep,
    web::{self, get, post},
};
use bb8_redis::RedisConnectionManager;
use serde::Serialize;
pub mod models;
pub mod services;
pub mod shared;

#[derive(Serialize)]
struct PingResponse {
    message: String,
}

const RESET: bool = false;

use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};

use crate::{
    models::{auth_model::Claims, user_model::User},
    services::auth_service,
    shared::SQLable,
};

static DB_POOL: LazyLock<Pool<MySql>> = std::sync::LazyLock::new(|| {
    MySqlPoolOptions::new()
        .max_connections(6)
        .connect_lazy(&std::env::var("DATABASE_URL").expect("DATABASE_URL not define !"))
        .expect("Can't connect to DB")
});
use tokio::runtime;

// Redis pool global initialisé de manière synchrone
static REDIS_POOL: LazyLock<bb8::Pool<RedisConnectionManager>> = LazyLock::new(|| {
    // Runtime temporaire juste pour bloquer l'initialisation async
    let manager = RedisConnectionManager::new("redis://redis/").unwrap();
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("can't build runtime for creating redis");
    rt.block_on(async { bb8::Pool::builder().build(manager).await.unwrap() })
});

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    sleep(Duration::from_secs(5)).await;
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

    let pool_data = web::Data::new(pool.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .service(web::scope("/auth").route("/login", post().to(auth_service::login)))
            .service(web::scope("/public").route("/ping", get().to(handle_ping)))
            .service(web::scope("").route("/login_test", post().to(login_test)))
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
