use crate::entities::user::User;
use crate::ports::user_repository::UserRepository;
use bcrypt::{DEFAULT_COST, hash};
use contracts::auth::{
    error::AuthError,
    signup::{SignupInput, SignupOutput},
};
use std::sync::Arc;

pub struct SignupUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl SignupUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, input: SignupInput) -> Result<SignupOutput, AuthError> {
        let password_hash = hash(&input.password, DEFAULT_COST).map_err(AuthError::BcryptError)?;

        let user = User::new(&input.username, &input.name, &password_hash);

        self.user_repository
            .save(user.clone())
            .await
            .map_err(AuthError::SaveUserError)?;
        Ok(SignupOutput { user_id: user.id })
    }
}
