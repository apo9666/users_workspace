use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct SignupRequest {
    #[validate(length(
        min = 3,
        message = "O nome deve ter no mínimo 3 caracteres",
        code = "name_too_short"
    ))]
    pub name: String,

    #[validate(email(message = "E-mail inválido", code = "invalid_email"))]
    pub email: String,

    #[validate(custom(function = "validate_password"))]
    pub password: String,
}

/// Custom validation for password complexity:
/// - Minimum 8 characters
/// - At least one uppercase letter
/// - At least one special character
fn validate_password(password: &str) -> Result<(), ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    let has_min_length = password.len() >= 8;

    if has_uppercase && has_special && has_min_length {
        Ok(())
    } else {
        let mut error = ValidationError::new("weak_password");
        error.message = Some(
            "A senha deve ter no mínimo 8 caracteres, uma letra maiúscula e um caractere especial"
                .into(),
        );
        Err(error)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SignupResponse {}
