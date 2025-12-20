use api_types::{
    error::ErrorResponse,
    login::{LoginRequest, LoginResponse},
    signup::SignupRequest,
    totp::{TotpSetupResponse, TotpVerifyRequest, TotpVerifyResponse},
};
use reqwest::Client;

pub async fn login(req: LoginRequest) -> Result<LoginResponse, String> {
    let client = Client::new();
    let response = client
        .post("http://localhost:8080/login")
        .json(&req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        response
            .json::<LoginResponse>()
            .await
            .map_err(|_| "Erro ao processar resposta do servidor".to_string())
    } else {
        let error_msg = response
            .json::<ErrorResponse>()
            .await
            .map(|e| e.message)
            .unwrap_or_else(|_| "Falha desconhecida no cadastro".to_string());

        Err(error_msg)
    }
}

pub async fn signup(req: SignupRequest) -> reqwest::Result<reqwest::Response> {
    let client = Client::new();
    client
        .post("http://localhost:8080/signup")
        .json(&req)
        .send()
        .await
}

pub async fn totp_registration_start(auth_token: &str) -> Result<TotpSetupResponse, String> {
    let client = Client::new();
    let response = client
        .post("http://localhost:8080/mfa/registration/totp/start")
        .bearer_auth(auth_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        response
            .json::<TotpSetupResponse>()
            .await
            .map_err(|_| "Erro ao processar resposta do servidor".to_string())
    } else {
        let error_msg = response
            .json::<ErrorResponse>()
            .await
            .map(|e| e.message)
            .unwrap_or_else(|_| "Falha desconhecida no cadastro".to_string());

        Err(error_msg)
    }
}

pub async fn totp_registration_finish(
    auth_token: &str,
    req: TotpVerifyRequest,
) -> Result<TotpVerifyResponse, String> {
    let client = Client::new();
    let response = client
        .post("http://localhost:8080/mfa/registration/totp/finish")
        .json(&req)
        .bearer_auth(auth_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        response
            .json::<TotpVerifyResponse>()
            .await
            .map_err(|_| "Erro ao processar resposta do servidor".to_string())
    } else {
        let error_msg = response
            .json::<ErrorResponse>()
            .await
            .map(|e| e.message)
            .unwrap_or_else(|_| "Falha desconhecida no cadastro".to_string());

        Err(error_msg)
    }
}

pub async fn webauthn_registration_start(auth_token: &str) -> Result<String, String> {
    let client = Client::new();
    let response = client
        .post("http://localhost:8080/mfa/registration/webauthn/start")
        .bearer_auth(auth_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        response
            .text()
            .await
            .map_err(|_| "Erro ao processar resposta do servidor".to_string())
    } else {
        let error_msg = response
            .json::<ErrorResponse>()
            .await
            .map(|e| e.message)
            .unwrap_or_else(|_| "Falha desconhecida no cadastro".to_string());

        Err(error_msg)
    }
}
