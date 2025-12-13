use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::services::auth::{SignupRequest, signup};

#[component]
pub fn SignupPage() -> Html {
    let name_ref = NodeRef::default();
    let email_ref = NodeRef::default();
    let password_ref = NodeRef::default();
    let confirm_ref = NodeRef::default();
    let signup_status = use_state(|| String::new());

    let handle_signup = {
        let name_ref = name_ref.clone();
        let email_ref = email_ref.clone();
        let password_ref = password_ref.clone();
        let confirm_ref = confirm_ref.clone();
        let signup_status = signup_status.clone();

        move |_: MouseEvent| {
            let name = name_ref
                .cast::<HtmlInputElement>()
                .and_then(|el| Some(el.value()))
                .unwrap_or_default();
            let email = email_ref
                .cast::<HtmlInputElement>()
                .and_then(|el| Some(el.value()))
                .unwrap_or_default();
            let password = password_ref
                .cast::<HtmlInputElement>()
                .and_then(|el| Some(el.value()))
                .unwrap_or_default();
            let confirm = confirm_ref
                .cast::<HtmlInputElement>()
                .and_then(|el| Some(el.value()))
                .unwrap_or_default();

            // Verificações básicas
            if name.is_empty() || email.is_empty() || password.is_empty() || confirm.is_empty() {
                signup_status.set("Por favor, preencha todos os campos".to_string());
                return;
            }

            if password != confirm {
                signup_status.set("As senhas não conferem".to_string());
                return;
            }

            let signup_status = signup_status.clone();

            spawn_local(async move {
                let req = SignupRequest {
                    name,
                    email: email.clone(),
                    password,
                };

                match signup(req).await {
                    Ok(response) => {
                        if response.status().is_success() {
                            signup_status.set(format!("Conta criada com sucesso: {}", email));
                        } else {
                            signup_status
                                .set(format!("Erro ao criar conta: {}", response.status()));
                        }
                    }
                    Err(e) => signup_status.set(format!("Erro na conexão: {}", e)),
                }
            });
        }
    };

    html! {
        <div class="login-container">
            <div class="login-box">
                <h1>{"Criar Conta"}</h1>

                <div class="form-group">
                    <label>{"Nome:"}</label>
                    <input
                        ref={name_ref.clone()}
                        type="text"
                        placeholder="Seu nome"
                        class="form-input"
                    />
                </div>

                <div class="form-group">
                    <label>{"Email:"}</label>
                    <input
                        ref={email_ref.clone()}
                        type="email"
                        placeholder="email@exemplo.com"
                        class="form-input"
                    />
                </div>

                <div class="form-group">
                    <label>{"Senha:"}</label>
                    <input
                        ref={password_ref.clone()}
                        type="password"
                        placeholder="Crie uma senha"
                        class="form-input"
                    />
                </div>

                <div class="form-group">
                    <label>{"Confirmar Senha:"}</label>
                    <input
                        ref={confirm_ref.clone()}
                        type="password"
                        placeholder="Repita sua senha"
                        class="form-input"
                    />
                </div>

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
