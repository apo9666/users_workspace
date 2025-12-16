use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    App, HttpRequest, HttpServer, Responder,
    http::{self, header},
    middleware::Logger,
    post, web,
};
use actix_web_httpauth::extractors::bearer::{self, BearerAuth};
use api_types::{
    error::ErrorResponse, login::LoginRequest, signup::SignupRequest, totp::TotpSetupResponse,
};
use application::auth::{Auth, LoginResult};
use env_logger::{Env, init_from_env};
use jwt_auth_tokens::JwtAuthTokens;
use log::info;
use memory::repository::user::MemoryUserRepository;
use totp::Totp;
use webauthn_rs::{WebauthnBuilder, prelude::Url};

#[post("/signup")]
async fn greet(data: web::Data<AppState>, body: web::Json<SignupRequest>) -> impl Responder {
    data.auth
        .signup(body.name.clone(), body.email.clone(), body.password.clone())
        .await
        .unwrap();
    format!("Hello")
}

#[post("/login")]
async fn login(data: web::Data<AppState>, body: web::Json<LoginRequest>) -> impl Responder {
    match data
        .auth
        .login(body.email.clone(), body.password.clone())
        .await
    {
        Ok(result) => web::Json(result),
        Err(e) => {
            info!("Login error: {}", e);
            web::Json(LoginResult {
                mfa_verification_token: None,
                mfa_registration_token: None,
                access_token: None,
                refresh_token: None,
            })
        }
    }
}

#[post("/mfa/totp/setup")]
async fn totp_setup(data: web::Data<AppState>, auth: BearerAuth) -> impl Responder {
    info!("{}", auth.token());

    match data.auth.mfa_totp_setup(auth.token().to_string()).await {
        Ok(result) => web::Json(TotpSetupResponse {
            qr_code_url: result,
        }),
        Err(e) => {
            info!("Totp setup error: {}", e);
            web::Json(TotpSetupResponse {
                qr_code_url: "deu ruim".to_string(),
            })
        }
    }
    // let headers = req.headers();

    // if let Some(auth_header_value) = headers.get(header::AUTHORIZATION) {
    //     if let Ok(auth_str) = auth_header_value.to_str() {
    //         println!("Authorization header: {}", auth_str);
    //         // Process the token (e.g., validate it)
    //         // ...
    //     }
    // } else {
    //     // No Authorization header found
    //     return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    // }

    // match data
    //     .auth
    //     .login(body.email.clone(), body.password.clone())
    //     .await
    // {
    //     Ok(result) => web::Json(result),
    //     Err(e) => {
    //         info!("Login error: {}", e);
    //         web::Json(LoginResult {
    //             mfa_verification_token: None,
    //             mfa_registration_token: None,
    //             access_token: None,
    //             refresh_token: None,
    //         })
    //     }
    // }
}

struct AppState {
    auth: Arc<Auth>,
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    init_from_env(Env::default().default_filter_or("info"));

    let credential_repository = Arc::new(MemoryUserRepository::new());
    let jwt_auth = Arc::new(JwtAuthTokens {});
    let totp = Arc::new(Totp {});
    let hsm_store = Arc::new(memory::hsm_store::MemoryHsmStore::new());
    let webauthn = Arc::new(
        WebauthnBuilder::new("localhost", &Url::parse("http://localhost:3000").unwrap())
            .unwrap()
            .build()
            .unwrap(),
    );

    let auth = Arc::new(Auth::new(
        credential_repository.clone(),
        jwt_auth.clone(),
        totp.clone(),
        hsm_store.clone(),
        webauthn.clone(),
    ));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8000")
            .allowed_methods(vec!["GET", "POST", "DELETE", "PUT"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState { auth: auth.clone() }))
            .app_data(bearer::Config::default())
            .service(greet)
            .service(login)
            .service(totp_setup)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
