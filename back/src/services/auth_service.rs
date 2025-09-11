use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};

use crate::models::auth_model::{Claims, Token};

#[derive(Serialize, Deserialize)]
pub struct LoginInfo {
    mail: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64, // durée de vie en secondes
}

pub async fn login(login_info: web::Json<LoginInfo>) -> HttpResponse {
    /*Des verification */
    if login_info.mail != "joshuabouchat@gmail.com" && login_info.password != "root" {
        return HttpResponse::Unauthorized().json("identifiant invalide");
    }

    let claims = Claims::new(0, true);
    let access_token: Token = Token::try_from(&claims).unwrap(); // ton JWT
    let refresh_token = uuid::Uuid::new_v4(); // opaque token UUID ou aléatoire

    // 3️⃣ Retourner en JSON
    HttpResponse::Ok().json(LoginResponse {
        access_token: access_token.as_ref().to_string(),
        refresh_token: refresh_token.to_string(),
        expires_in: 3600, // par ex 1h
    })
}
