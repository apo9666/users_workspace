use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, http, middleware::Logger, post, web};
use actix_web_httpauth::extractors::bearer::{self, BearerAuth};
use actix_web_validator::Json;
use api_types::{
    error::ErrorResponse,
    login::LoginRequest,
    signup::{SignupRequest, SignupResponse},
    totp::{TotpSetupResponse, TotpVerifyRequest, TotpVerifyResponse},
};
use application::auth::Auth;
use env_logger::{Env, init_from_env};
use jwt_auth_tokens::JwtAuthTokens;
use log::info;
use memory::repository::user::MemoryUserRepository;
use totp::Totp;
use webauthn_rs::{WebauthnBuilder, prelude::Url};

#[post("/signup")]
async fn greet(data: web::Data<AppState>, body: Json<SignupRequest>) -> impl Responder {
    match data
        .auth
        .signup(body.name.clone(), body.email.clone(), body.password.clone())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(SignupResponse {}),
        Err(e) => {
            info!("Signup error: {}", e);
            HttpResponse::BadRequest().json(ErrorResponse {
                message: "Erro ao cadastrar usuário".to_string(),
            })
        }
    }
}

#[post("/login")]
async fn login(data: web::Data<AppState>, body: Json<LoginRequest>) -> impl Responder {
    match data
        .auth
        .login(body.email.clone(), body.password.clone())
        .await
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            info!("Login error: {}", e);
            HttpResponse::Unauthorized().json(ErrorResponse {
                message: "Usuário ou senha inválidos".to_string(),
            })
        }
    }
}

#[post("/mfa/totp/registration/start")]
async fn totp_registration_start(data: web::Data<AppState>, auth: BearerAuth) -> impl Responder {
    match data
        .auth
        .start_totp_registration(auth.token().to_string())
        .await
    {
        Ok(result) => HttpResponse::Ok().json(TotpSetupResponse {
            qr_code_url: result,
        }),
        Err(e) => {
            info!("Totp registration start error: {}", e);
            HttpResponse::BadRequest().json(ErrorResponse {
                message: "Erro ao iniciar registro TOTP".to_string(),
            })
        }
    }
}

#[post("/mfa/totp/registration/finish")]
async fn totp_registration_finish(
    data: web::Data<AppState>,
    auth: BearerAuth,
    body: Json<TotpVerifyRequest>,
) -> impl Responder {
    match data
        .auth
        .finish_totp_registration(auth.token().to_string(), body.code.clone())
        .await
    {
        Ok((refresh_token, access_token)) => HttpResponse::Ok().json(TotpVerifyResponse {
            refresh_token,
            access_token,
        }),
        Err(e) => {
            info!("Totp registration finish error: {}", e);
            HttpResponse::BadRequest().json(ErrorResponse {
                message: "Erro ao terminar registro TOTP".to_string(),
            })
        }
    }
}

#[post("/mfa/totp/verify")]
async fn totp_verify(
    data: web::Data<AppState>,
    auth: BearerAuth,
    body: Json<TotpVerifyRequest>,
) -> impl Responder {
    info!("{}", auth.token());

    match data
        .auth
        .start_totp_registration(auth.token().to_string())
        .await
    {
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
            .app_data(
                actix_web_validator::JsonConfig::default().error_handler(|err, _req| {
                    let response = ErrorResponse {
                        message: err.to_string(),
                    };

                    actix_web::error::InternalError::from_response(
                        err,
                        HttpResponse::BadRequest().json(response),
                    )
                    .into()
                }),
            )
            .service(greet)
            .service(login)
            .service(totp_registration_start)
            .service(totp_registration_finish)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
