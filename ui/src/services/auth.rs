use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SignupRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub message: Option<String>,
}

pub async fn login(req: LoginRequest) -> reqwest::Result<reqwest::Response> {
    let client = Client::new();
    client
        .post("http://localhost:8080/login")
        .json(&req)
        .send()
        .await
}

pub async fn signup(req: SignupRequest) -> reqwest::Result<reqwest::Response> {
    let client = Client::new();
    client
        .post("http://localhost:8080/signup")
        .json(&req)
        .send()
        .await
}
