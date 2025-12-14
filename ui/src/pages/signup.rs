use std::collections::HashMap;
use validator::Validate;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{components::ui::input_field::InputField, services::auth::signup};
use api_types::signup::SignupRequest;

#[function_component(SignupPage)]
pub fn signup_page() -> Html {
    let name_ref = use_node_ref();
    let email_ref = use_node_ref();
    let password_ref = use_node_ref();
    let confirm_ref = use_node_ref();

    let field_errors = use_state(|| HashMap::<String, String>::new());
    let signup_status = use_state(|| String::new());

    let handle_signup = {
        let name_ref = name_ref.clone();
        let email_ref = email_ref.clone();
        let password_ref = password_ref.clone();
        let confirm_ref = confirm_ref.clone();
        let field_errors = field_errors.clone();
        let signup_status = signup_status.clone();

        move |_: MouseEvent| {
            field_errors.set(HashMap::new());
            signup_status.set("".to_string());

            let name = name_ref
                .cast::<HtmlInputElement>()
                .map(|el| el.value())
                .unwrap_or_default();
            let email = email_ref
                .cast::<HtmlInputElement>()
                .map(|el| el.value())
                .unwrap_or_default();
            let password = password_ref
                .cast::<HtmlInputElement>()
                .map(|el| el.value())
                .unwrap_or_default();
            let confirm = confirm_ref
                .cast::<HtmlInputElement>()
                .map(|el| el.value())
                .unwrap_or_default();

            let mut errors = HashMap::new();

            // 1. Validação Manual (Confirmação de Senha)
            if password != confirm {
                errors.insert("confirm".to_string(), "As senhas não conferem".to_string());
            }

            // 2. Validação via Struct (validator crate)
            let req = SignupRequest {
                name,
                email: email.clone(),
                password,
            };

            if let Err(errs) = req.validate() {
                for (field, field_errs) in errs.field_errors() {
                    if let Some(first_err) = field_errs.first() {
                        let msg = first_err
                            .message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "Campo inválido".to_string());
                        errors.insert(field.to_string(), msg);
                    }
                }
            }

            if !errors.is_empty() {
                field_errors.set(errors);
                return;
            }

            // 3. Chamada da API
            let signup_status = signup_status.clone();
            spawn_local(async move {
                match signup(req).await {
                    Ok(res) if res.status().is_success() => {
                        signup_status.set(format!("Conta criada com sucesso para {}", email));
                    }
                    Ok(res) => signup_status.set(format!("Erro no servidor: {}", res.status())),
                    Err(e) => signup_status.set(format!("Erro de conexão: {}", e)),
                }
            });
        }
    };

    let clear_error = |field: &'static str| {
        let field_errors = field_errors.clone();
        Callback::from(move |_: InputEvent| {
            if field_errors.contains_key(field) {
                let mut current = (*field_errors).clone();
                current.remove(field);
                field_errors.set(current);
            }
        })
    };

    html! {
        <div class="login-container">
            <div class="login-box">
                <h1>{"Criar Conta"}</h1>

                <InputField label="Nome:" input_type="text" placeholder="Seu nome" input_ref={name_ref.clone()} name="name" error={field_errors.get("name").cloned()} oninput={clear_error("name")} />
                <InputField label="Email:" input_type="email" placeholder="email@exemplo.com" input_ref={email_ref.clone()} name="email" error={field_errors.get("email").cloned()} oninput={clear_error("email")} />
                <InputField label="Senha:" input_type="password" placeholder="Crie uma senha" input_ref={password_ref.clone()} name="password" error={field_errors.get("password").cloned()} oninput={clear_error("password")} />
                <InputField label="Confirmar Senha:" input_type="password" placeholder="Crie uma senha" input_ref={confirm_ref.clone()} name="confirm" error={field_errors.get("confirm").cloned()} oninput={clear_error("confirm")} />

                <button onclick={handle_signup} class="login-btn">
                    {"Cadastrar"}
                </button>

                if !signup_status.is_empty() {
                    <p class="status-message">{ (*signup_status).clone() }</p>
                }
            </div>
        </div>
    }
}
