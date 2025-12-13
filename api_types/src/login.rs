use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LoginRequest {
    #[validate(email(code = "email_invalido"))]
    pub email: String,
    #[validate(length(min = 8, message = "Senha deve ter pelo menos 8 caracteres"))]
    pub password: String,
}
