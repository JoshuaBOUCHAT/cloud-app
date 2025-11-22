use std::fs::read_to_string;

use actix_web::HttpResponse;

#[actix_web::get("/openapi.yaml")]
async fn openapi_yaml() -> HttpResponse {
    match read_to_string("openapi.yaml") {
        Ok(file) => HttpResponse::Ok()
            .content_type("application/yaml")
            .body(file),
        Err(err) => HttpResponse::NotFound()
            .body(format!("An error occurs when opening the yaml file: {err}")),
    }
}
