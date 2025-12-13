use std::collections::HashMap;
use validator::Validate;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::services::auth::signup;
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

    // Use AttrValue para evitar problemas de lifetime com o macro html!
    let render_input_group = |label: &'static str,
                              input_type: &'static str,
                              placeholder: &'static str,
                              n_ref: NodeRef,
                              field_name: &'static str| {
        let error_msg = field_errors.get(field_name).cloned();
        let input_class = if error_msg.is_some() {
            "form-input input-error"
        } else {
            "form-input"
        };

        html! {
            <div class="form-group">
                <label>{label}</label>
                <input
                    ref={n_ref}
                    type={input_type}
                    placeholder={placeholder} // Agora funciona porque é &'static str
                    class={input_class}
                />
                if let Some(msg) = error_msg {
                    <span class="error-message">{msg}</span>
                }
            </div>
        }
    };

    html! {
        <div class="login-container">
            <div class="login-box">
                <h1>{"Criar Conta"}</h1>

                { render_input_group("Nome:", "text", "Seu nome", name_ref, "name") }
                { render_input_group("Email:", "email", "email@exemplo.com", email_ref, "email") }
                { render_input_group("Senha:", "password", "Crie uma senha", password_ref, "password") }
                { render_input_group("Confirmar Senha:", "password", "Repita sua senha", confirm_ref, "confirm") }

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
