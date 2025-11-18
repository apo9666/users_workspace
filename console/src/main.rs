use application::{auth::Auth, domain::credential::Credential};
use env_logger::{Builder, Target};
use jwt_auth_tokens::JwtAuthTokens;
use memory::repository::credential::MemoryCredentialRepository;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    Builder::new().target(Target::Stdout).init();

    let credential_repository = Arc::new(MemoryCredentialRepository::new());
    let jwt_auth = Arc::new(JwtAuthTokens {});
    let auth = Auth::new(credential_repository.clone(), jwt_auth.clone());

    auth.signup(&Credential {
        username: "user1".to_string(),
        password: "password123".to_string(),
    })
    .await
    .unwrap();

    let result = auth
        .login(&"user1".to_string(), &"password123".to_string())
        .await
        .unwrap();

    println!("{}", result);
}
