use std::{env, fmt::Debug, sync::LazyLock, time::Duration};

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

use sqlx::{MySql, Pool, mysql::MySqlPoolOptions, pool::PoolConnection};

use crate::{models::user_model::User, shared::SQLable};

static DB_POOL: LazyLock<Pool<MySql>> = std::sync::LazyLock::new(|| {
    MySqlPoolOptions::new()
        .max_connections(6)
        .connect_lazy(&std::env::var("DATABASE_URL").expect("DATABASE_URL not define !"))
        .expect("Can't connect to DB")
});

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    sleep(Duration::from_secs(5)).await;
    let pool = MySqlPoolOptions::new()
        .max_connections(6)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not define !"))
        .await
        .expect("Can't connect to DB");

    up_all_table(&pool)
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
async fn up_all_table(conn: &Pool<MySql>) -> Result<(), Box<dyn std::error::Error>> {
    User::up(conn).await?;

    Ok(())
}
