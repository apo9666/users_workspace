use application::auth::Auth;
use memory::repository::credential::MemoryCredentialRepository;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let credential_repository = Arc::new(MemoryCredentialRepository::new());
    let auth = Auth::new(credential_repository.clone());

    auth.signup(&application::port::credential_repository::Credential {
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
