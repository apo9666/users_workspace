use uuid::Uuid;

#[derive(Debug)]
pub struct SignupInput {
    pub name: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct SignupOutput {
    pub user_id: Uuid,
}
