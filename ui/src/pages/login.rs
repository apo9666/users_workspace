use std::collections::HashMap;

use api_types::login::LoginRequest;
use serde::{Deserialize, Serialize};
use validator::Validate;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::{
    Routable,
    hooks::{use_location, use_navigator},
};

use crate::{
    app::Route,
    components::{
        auth_card::AuthCard,
        ui::{
            button::Button,
            input_field::{Field, InputField},
            server_error::ServerError,
        },
    },
    context::user::{User, UserAction, UserContext},
    services::auth::login,
    utils::validator::{get_validation_errors, sync_field_error},
};

#[derive(Deserialize, Serialize)]
struct LoginQuery {
    return_to: Option<String>,
}

#[component]
pub fn LoginPage() -> Html {
    let navigator = use_navigator().expect("Navigator not found");
    let location = use_location().expect("Location not found");
    let user_ctx = use_context::<UserContext>().expect("no user ctx found");
    let is_loading = use_state(|| false);
    let server_error = use_state(|| String::new());

    let email = use_state(Field::default);
    let password = use_state(Field::default);

    let handle_login = {
        let navigator = navigator.clone();
        let location = location.clone();
        let user_ctx = user_ctx.clone();
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
            let navigator = navigator.clone();
            let location = location.clone();
            let user_ctx = user_ctx.clone();

            spawn_local(async move {
                match login(req).await {
                    Ok(resp) => {
                        let user = User {
                            name: "test".to_string(),
                            email: "email".to_string(),
                            mfa_registration_token: resp.mfa_registration_token,
                            mfa_verification_token: resp.mfa_verification_token,
                            access_token: resp.access_token,
                            refresh_token: resp.refresh_token,
                        };
                        user_ctx.state.dispatch(UserAction::Set(user.clone()));

                        let query = location
                            .query::<LoginQuery>()
                            .unwrap_or(LoginQuery { return_to: None });

                        match (user.mfa_registration_token, user.access_token) {
                            (Some(_), _) => {
                                let _ = navigator.push_with_query(&Route::Totp, &query);
                            }
                            (None, Some(_)) => match query.return_to {
                                Some(path) => match <Route as Routable>::recognize(&path) {
                                    Some(route) => navigator.push(&route),
                                    None => navigator.push(&Route::Home),
                                },
                                None => navigator.push(&Route::Home),
                            },
                            _ => server_error.set("Resposta do servidor inválida".to_string()),
                        }
                    }
                    Err(err_msg) => {
                        server_error.set(err_msg);
                    }
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
