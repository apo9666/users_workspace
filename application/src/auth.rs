use crate::{
    domain::{claims::Claims, user::User},
    port::{
        for_auth_tokens::ForAuthTokens, for_totp::ForTotp, hsm_store::HSMStore,
        user_repository::UserRepository,
    },
};
use bcrypt::{DEFAULT_COST, hash, verify};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use webauthn_rs::{
    Webauthn,
    prelude::{
        PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
        RegisterPublicKeyCredential,
    },
};

const WEBAUTHN_REG_STATE: &str = "webauthn/reg/state";
const WEBAUTHN_AUTH_STATE: &str = "webauthn/auth/state";

pub struct LoginResult {
    pub mfa_token: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

pub struct Auth {
    user_repository: Arc<dyn UserRepository>,
    for_auth_tokens: Arc<dyn ForAuthTokens>,
    for_totp: Arc<dyn ForTotp>,
    hsm_store: Arc<dyn HSMStore>,
    webauthn: Arc<Webauthn>,
}

impl Auth {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        for_auth_tokens: Arc<dyn ForAuthTokens>,
        for_totp: Arc<dyn ForTotp>,
        hsm_store: Arc<dyn HSMStore>,
        webauthn: Arc<Webauthn>,
    ) -> Self {
        Self {
            user_repository,
            for_auth_tokens,
            for_totp,
            hsm_store,
            webauthn,
        }
    }

    pub async fn signup(
        &self,
        name: String,
        username: String,
        password: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cred = User::new(username, name, hash(password, DEFAULT_COST)?);

        self.user_repository.save(cred).await.unwrap();
        Ok(())
    }

    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<LoginResult, Box<dyn std::error::Error>> {
        let credential = self
            .user_repository
            .find_username(username.clone())
            .await?
            .ok_or_else(|| "Invalid username or password".to_string())?;

        if !verify(password, &credential.password)? {
            return Err("Invalid username or password".into());
        }

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");

        if credential.otp_secret.is_some() {
            let exp = (since_the_epoch.as_secs() + 600) as usize; // 10 minutes from now
            let mfa_token = self
                .for_auth_tokens
                .create_token(Claims {
                    token_type: "mfa".to_string(),
                    sub: username.clone(),
                    exp,
                })
                .await?;

            return Ok(LoginResult {
                mfa_token: Some(mfa_token),
                access_token: None,
                refresh_token: None,
            });
        }

        let exp = (since_the_epoch.as_secs() + 604800) as usize; // 7 days from now
        let refresh_token = self
            .for_auth_tokens
            .create_token(Claims {
                token_type: "refresh".to_string(),
                sub: username.clone(),
                exp,
            })
            .await?;

        let exp = (since_the_epoch.as_secs() + 600) as usize; // 10 minutes from now
        let access_token = self
            .for_auth_tokens
            .create_token(Claims {
                token_type: "access".to_string(),
                sub: username.clone(),
                exp,
            })
            .await?;

        return Ok(LoginResult {
            mfa_token: None,
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
        });
    }

    pub async fn mfa_totp_setup(
        &self,
        mfa_token: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let claims = self
            .for_auth_tokens
            .validate_token(mfa_token, "access".to_string())
            .await?;

        let Some(mut credential) = self.user_repository.find_username(claims.sub).await? else {
            return Err("Invalid username or password".to_string().into());
        };

        let (secret, auth_url) = self
            .for_totp
            .auth_url(credential.username.clone(), "TODO_ISSUER".to_string())
            .await?;

        credential.otp_secret = Some(secret);
        self.user_repository.save(credential).await?;

        Ok(auth_url)
    }

    pub async fn start_passkey_registration(
        &self,
        user_id: Uuid,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let Some(user) = self.user_repository.find_id(user_id).await? else {
            return Err("Credential not found".into());
        };
        self.hsm_store.set(user.id, WEBAUTHN_REG_STATE, "")?;
        let credential_ids = user.pass_keys.iter().map(|k| k.cred_id().clone()).collect();

        let (ccr, reg_state) = self
            .webauthn
            .start_passkey_registration(user_id, &user.username, &user.name, Some(credential_ids))
            .expect("Failed to start registration.");

        let json_reg_state = serde_json::to_string(&reg_state)?;
        self.hsm_store
            .set(user.id, WEBAUTHN_REG_STATE, &json_reg_state)?;

        let ccr = serde_json::to_string(&ccr)?;

        return Ok(ccr);
    }

    pub async fn finish_passkey_registration(
        &self,
        user_id: Uuid,
        req: RegisterPublicKeyCredential,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reg_state_str = self
            .hsm_store
            .get(user_id, WEBAUTHN_REG_STATE)?
            .ok_or_else(|| "could not find webauthn registration key".to_string())?;

        let reg_state: PasskeyRegistration = serde_json::from_str(&reg_state_str)?;
        self.hsm_store.set(user_id, WEBAUTHN_REG_STATE, "")?;

        let sk = self
            .webauthn
            .finish_passkey_registration(&req, &reg_state)?;

        let mut user = self
            .user_repository
            .find_id(user_id)
            .await?
            .ok_or_else(|| "user not found".to_string())?;

        user.pass_keys.push(sk);
        self.user_repository.save(user).await?;

        Ok(())
    }

    pub async fn start_passkey_authentication(
        &self,
        user_id: Uuid,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.hsm_store.set(user_id, WEBAUTHN_AUTH_STATE, "")?;
        let user = self
            .user_repository
            .find_id(user_id)
            .await?
            .ok_or_else(|| "user not found".to_string())?;

        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(&user.pass_keys)?;

        let json_auth_state = serde_json::to_string(&auth_state)?;
        self.hsm_store
            .set(user_id, WEBAUTHN_AUTH_STATE, &json_auth_state)?;

        let rcr = serde_json::to_string(&rcr)?;
        Ok(rcr)
    }

    pub async fn finish_passkey_authentication(
        &self,
        user_id: Uuid,
        auth: PublicKeyCredential,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let auth_state_str = self
            .hsm_store
            .get(user_id, WEBAUTHN_AUTH_STATE)?
            .ok_or_else(|| "could not find webauthn registration key".to_string())?;

        self.hsm_store.set(user_id, WEBAUTHN_AUTH_STATE, "")?;
        let auth_state: PasskeyAuthentication = serde_json::from_str(&auth_state_str)?;

        let auth_result = self
            .webauthn
            .finish_passkey_authentication(&auth, &auth_state)?;

        let mut user = self
            .user_repository
            .find_id(user_id)
            .await?
            .ok_or_else(|| "user not found".to_string())?;

        user.pass_keys.iter_mut().for_each(|k| {
            k.update_credential(&auth_result);
        });
        self.user_repository.save(user).await?;

        Ok(())
    }
}
