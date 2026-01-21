use std::{str::FromStr, sync::Arc};

use actix_cors::Cors;
use actix_web::{
    App, Error, HttpMessage, HttpResponse, HttpServer, Responder,
    body::{EitherBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http,
    middleware::{Logger, Next, from_fn},
    get, post, web,
};
use actix_web_httpauth::extractors::bearer::{self, BearerAuth};
use actix_web_validator::Json;
use api_types::{
    error::ErrorResponse,
    login::{LoginRequest, LoginResponse},
    signup::{SignupRequest, SignupResponse},
    totp::{TotpSetupResponse, TotpVerifyRequest, TotpVerifyResponse},
};
use contracts::auth::{
    login::LoginInput,
    passkey::PasskeyStartRegistrationInput,
    signup::SignupInput,
    totp::{TOTPFinishRegistrationInput, TOTPStartRegistrationInput},
};
use env_logger::{Env, init_from_env};
use log::info;
use serde::{Deserialize, Serialize};

#[post("/signup")]
async fn greet(data: web::Data<AppState>, body: Json<SignupRequest>) -> impl Responder {
    match data
        .auth
        .signup(SignupInput {
            name: body.name.clone(),
            username: body.email.clone(),
            password: body.password.clone(),
        })
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginResult {
    pub mfa_registration_token: Option<String>,
    pub mfa_verification_token: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[post("/login")]
async fn login(data: web::Data<AppState>, body: Json<LoginRequest>) -> impl Responder {
    match data
        .auth
        .login(LoginInput {
            username: body.email.clone(),
            password: body.password.clone(),
        })
        .await
    {
        Ok(result) => HttpResponse::Ok().json(LoginResponse {
            mfa_registration_token: result.mfa_registration_token,
            mfa_verification_token: result.mfa_verification_token,
            access_token: result.access_token,
            refresh_token: result.refresh_token,
        }),
        Err(e) => {
            info!("Login error: {}", e);
            HttpResponse::Unauthorized().json(ErrorResponse {
                message: "Usuário ou senha inválidos".to_string(),
            })
        }
    }
}

#[get("/.well-known/jwks.json")]
async fn jwks(data: web::Data<AppState>) -> impl Responder {
    match data.auth.get_jwks().await {
        Ok(jwks_json) => HttpResponse::Ok()
            .content_type("application/json")
            .body(jwks_json),
        Err(e) => {
            info!("Jwks fetch error: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Erro ao buscar JWKS".to_string(),
            })
        }
    }
}

#[post("/totp/start")]
async fn totp_registration_start(data: web::Data<AppState>, auth: BearerAuth) -> impl Responder {
    match data
        .auth
        .start_totp_registration(TOTPStartRegistrationInput {
            mfa_token: auth.token().to_string(),
        })
        .await
    {
        Ok(result) => HttpResponse::Ok().json(TotpSetupResponse {
            qr_code_url: result.auth_url,
        }),
        Err(e) => {
            info!("Totp registration start error: {}", e);
            HttpResponse::BadRequest().json(ErrorResponse {
                message: "Erro ao iniciar registro TOTP".to_string(),
            })
        }
    }
}

#[post("/totp/finish")]
async fn totp_registration_finish(
    data: web::Data<AppState>,
    auth: BearerAuth,
    body: Json<TotpVerifyRequest>,
) -> impl Responder {
    match data
        .auth
        .finish_totp_registration(TOTPFinishRegistrationInput {
            mfa_token: auth.token().to_string(),
            code: body.code.clone(),
        })
        .await
    {
        Ok(output) => HttpResponse::Ok().json(TotpVerifyResponse {
            refresh_token: output.refresh_token,
            access_token: output.access_token,
        }),
        Err(e) => {
            info!("Totp registration finish error: {}", e);
            HttpResponse::BadRequest().json(ErrorResponse {
                message: "Erro ao terminar registro TOTP".to_string(),
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    // aud: String, // Optional. Audience
    pub exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    // iat: usize, // Optional. Issued at (as UTC timestamp)
    // iss: String, // Optional. Issuer
    // nbf: usize,  // Optional. Not Before (as UTC timestamp)
    pub sub: String, // Optional. Subject (whom token refers to)
    pub token_type: String,
}

#[post("/webauthn/start")]
async fn webauthn_registration_start(
    data: web::Data<AppState>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    match data
        .auth
        .start_passkey_registration(PasskeyStartRegistrationInput {
            user_id: Uuid::from_str(&claims.sub).unwrap(),
        })
        .await
    {
        Ok(result) => HttpResponse::Ok().body(result.challenge),
        Err(e) => {
            info!("Webauthn registration start error: {}", e);
            HttpResponse::BadRequest().json(ErrorResponse {
                message: "Erro ao iniciar registro TOTP".to_string(),
            })
        }
    }
}

async fn protected_mfa_registration_route(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, Error> {
    // Alterado aqui

    let data = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("AppState missing"))?;

    // Exemplo de extração do token (ajuste conforme sua lógica)
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_val) = auth_header {
        let token = auth_val.to_str().unwrap_or("").replace("Bearer ", "");

        match data.auth.validate_token(&token, "mfa_registration").await {
            Ok(claims) => {
                // SUCESSO: Usamos .map_into_left_body()
                req.extensions_mut().insert(claims);
                let res = next.call(req).await?;
                Ok(res.map_into_left_body())
            }
            Err(e) => {
                // ERRO DE VALIDAÇÃO: Usamos .map_into_right_body()
                let res = HttpResponse::Unauthorized().body(format!("Unauthorized: {}", e));
                let s_res = req.into_response(res);
                Ok(s_res.map_into_right_body())
            }
        }
    } else {
        // SEM HEADER: Usamos .map_into_right_body()
        let res = HttpResponse::Unauthorized().finish();
        let s_res = req.into_response(res);
        Ok(s_res.map_into_right_body())
    }
}

struct AppState {
    auth: Arc<dyn contracts::auth::Component>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_from_env(Env::default().default_filter_or("info"));

    let auth = Arc::new(auth::AuthComponent::new());

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
            .service(
                web::scope("/mfa/registration")
                    .wrap(from_fn(protected_mfa_registration_route))
                    .service(totp_registration_start)
                    .service(totp_registration_finish)
                    .service(webauthn_registration_start),
            )
            .service(greet)
            .service(login)
            .service(jwks)
            .service(totp_registration_start)
            .service(totp_registration_finish)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
