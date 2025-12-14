use std::collections::HashMap;

use api_types::login::LoginRequest;
use validator::Validate;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::{
        auth_card::AuthCard,
        ui::{
            button::Button,
            input_field::{Field, InputField},
            server_error::ServerError,
        },
    },
    services::auth::login,
    utils::validator::{get_validation_errors, sync_field_error},
};

#[component]
pub fn LoginPage() -> Html {
    let is_loading = use_state(|| false);
    let server_error = use_state(|| String::new());

    let email = use_state(Field::default);
    let password = use_state(Field::default);

    let handle_login = {
        let is_loading = is_loading.clone();
        let server_error = server_error.clone();
        let email = email.clone();
        let password = password.clone();

        move |_: MouseEvent| {
            if *is_loading {
                return;
            }
            server_error.set("".to_string());

            let req = LoginRequest {
                email: (*email).value.clone(),
                password: (*password).value.clone(),
            };

            let error_map = match req.validate() {
                Ok(_) => HashMap::new(),
                Err(errs) => get_validation_errors(errs),
            };

            sync_field_error(&email, "email", &error_map);
            sync_field_error(&password, "password", &error_map);

            if !error_map.is_empty() {
                return;
            }

            is_loading.set(true);

            let is_loading = is_loading.clone();
            let server_error = server_error.clone();

            spawn_local(async move {
                match login(req).await {
                    Ok(res) if res.status().is_success() => {
                        server_error.set(format!("Login com sucesso"));
                    }
                    Ok(res) => server_error.set(format!("Erro no servidor: {}", res.status())),
                    Err(e) => server_error.set(format!("Erro de conexão: {}", e)),
                }
                is_loading.set(false);
            });
        }
    };

    html! {
        <AuthCard title="Login">
            <InputField label="Email:" field={email} input_type="email" placeholder="email@exemplo.com" />
            <InputField label="Senha:" field={password} input_type="password" placeholder="Sua senha" />

            <Button label="Entrar" onclick={handle_login} is_loading={*is_loading} />

            <ServerError message={(*server_error).clone()} />
        </AuthCard>
    }
}
