use std::collections::HashMap;

use validator::Validate;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::ui::input_field::{Field, InputField},
    services::auth::signup,
    utils::validator::get_validation_errors,
};
use api_types::signup::SignupRequest;

#[function_component(SignupPage)]
pub fn signup_page() -> Html {
    let name = use_state(Field::default);
    let email = use_state(Field::default);
    let password = use_state(Field::default);
    let confirm = use_state(Field::default);

    let server_error = use_state(|| String::new());

    let handle_signup = {
        let name = name.clone();
        let email = email.clone();
        let password = password.clone();
        let confirm = confirm.clone();
        let server_error = server_error.clone();

        move |_: MouseEvent| {
            server_error.set("".to_string());

            let req = SignupRequest {
                name: (*name).value.clone(),
                email: (*email).value.clone(),
                password: (*password).value.clone(),
            };

            let mut error_map = match req.validate() {
                Ok(_) => HashMap::new(),
                Err(errs) => get_validation_errors(errs),
            };

            if (*password).value != (*confirm).value {
                error_map.insert("confirm".to_string(), "As senhas não conferem".to_string());
            }

            let sync = |field_handle: &UseStateHandle<Field>, key: &str| {
                let new_error = error_map.get(key).cloned();

                if field_handle.error != new_error {
                    let mut f = (**field_handle).clone();
                    f.error = new_error;
                    field_handle.set(f);
                }
            };

            sync(&name, "name");
            sync(&email, "email");
            sync(&password, "password");
            sync(&confirm, "confirm");

            if !error_map.is_empty() {
                return;
            }

            let server_error = server_error.clone();
            spawn_local(async move {
                match signup(req).await {
                    Ok(res) if res.status().is_success() => {
                        server_error.set(format!("Conta criada com sucesso"));
                    }
                    Ok(res) => server_error.set(format!("Erro no servidor: {}", res.status())),
                    Err(e) => server_error.set(format!("Erro de conexão: {}", e)),
                }
            });
        }
    };

    html! {
        <div class="login-container">
            <div class="login-box">
                <h1>{"Criar Conta"}</h1>

                <InputField label="Nome:" field={name} input_type="text" placeholder="Seu nome" />
                <InputField label="Email:" field={email} input_type="email" placeholder="email@exemplo.com" />
                <InputField label="Senha:" field={password} input_type="password" placeholder="Crie uma senha" />
                <InputField label="Confirmar Senha:" field={confirm} input_type="password" placeholder="Crie uma senha" />

                <button onclick={handle_signup} class="login-btn">
                    {"Cadastrar"}
                </button>

                if !server_error.is_empty() {
                    <p class="status-message">{ (*server_error).clone() }</p>
                }
            </div>
        </div>
    }
}
