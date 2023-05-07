use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use bcrypt::{verify};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use dotenv::dotenv;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
struct AuthRequest {
    password: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    token: String,
}

async fn authenticate(data: web::Json<AuthRequest>) -> impl Responder {
    let hashed_password = env::var("APP_PASSWORD_HASH").expect("APP_PASSWORD_HASH not set");
    if verify(&data.password, &hashed_password).unwrap()
     {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");
        let header = Header::default();
        let claims = Claims { sub: "user".to_string(), exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize };
        let token = encode(&header, &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap();
        HttpResponse::Ok().json(token)
    } else {
        HttpResponse::Unauthorized().body("Invalid password")
    }
}

async fn chat_gpt(client: web::Data<Client>, data: web::Json<ChatRequest>) -> impl Responder {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");
    let validation = Validation { leeway: 60, ..Validation::default() };
    let token_data = decode::<Claims>(&data.token, &DecodingKey::from_secret(secret.as_ref()), &validation);

    if token_data.is_ok() {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let response = client.post("https:api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model" : "gpt-3.5-turbo",
                "messages": [{"role": "user", "content": *data.message}],
                "max_tokens": 50
            }))
            .send()
            .await;

        match response {
            Ok(response) => HttpResponse::Ok().json(response.json::<serde_json::Value>().await.unwrap()),
            Err(_) => HttpResponse::InternalServerError().body("Error calling OpenAI API"),
        }
    } else {
        eprintln!("Error: {:?}", token_data.err());
        return HttpResponse::Unauthorized().body("Invalid token");
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
            .service(web::resource("/chat").route(web::post().to(chat_gpt)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}