use api_types::{login::LoginRequest, signup::SignupRequest};
use reqwest::Client;

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
