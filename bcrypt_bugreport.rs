use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use bcrypt::{verify};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use dotenv::dotenv;
use std::env;
//use chrono::Duration;

#[derive(Deserialize)]
struct AuthRequest {
    password: String,
}

async fn authenticate(data: web::Json<AuthRequest>) -> impl Responder {
    let hashed_password = env::var("APP_PASSWORD_HASH").expect("APP_PASSWORD_HASH not set");
    if verify(&data.password, &hashed_password).unwrap()
    {
        //some task
    } else {
        HttpResponse::Unauthorized().body("Invalid password")
    }
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let client = Client::new();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(client.clone()))
            .service(web::resource("/authenticate").route(web::post().to(authenticate)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}