use crate::{
    app::Route,
    components::{
        auth_card::AuthCard,
        ui::{loading_spinner::LoadingSpinner, server_error::ServerError},
    },
    context::user::{User, UserAction, UserContext},
    services::auth::{totp_registration_finish, totp_registration_start},
};
use api_types::totp::TotpVerifyRequest;
use qrcode::render::svg;
use serde::{Deserialize, Serialize};
use url::Url;
use web_sys::{Element, HtmlInputElement};
use yew::{platform::spawn_local, prelude::*};
use yew_router::{
    Routable,
    hooks::{use_location, use_navigator},
};

#[derive(Clone, PartialEq)]
enum TotpState {
    Loading,
    Error(String),
    Ready { svg_content: String, secret: String },
}

#[derive(Deserialize, Serialize)]
struct LoginQuery {
    return_to: Option<String>,
}

#[component]
pub fn TotpPage() -> Html {
    let navigator = use_navigator().expect("Navigator not found");
    let user_context = use_context::<UserContext>().expect("no user ctx found");
    let location = use_location().expect("Location not found");
    let totp_state = use_state(|| TotpState::Loading);
    let otp_code = use_state(|| String::new());
    let is_submitting = use_state(|| false);
    let error_msg = use_state(|| Option::<String>::None);
    let svg_ref = use_node_ref();

    let mfa_token = user_context
        .state
        .user
        .as_ref()
        .and_then(|user| user.mfa_registration_token.as_ref().cloned())
        .expect("no mfa registration token");

    // --- EFFECT 1: Buscar dados e processar com a lib URL ---
    {
        let totp_state = totp_state.clone();
        let mfa_token = mfa_token.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let new_state = match totp_registration_start(&mfa_token).await {
                    Ok(res) => {
                        // Usando a lib 'url' para extrair o segredo de forma segura
                        let secret = Url::parse(&res.qr_code_url)
                            .ok()
                            .and_then(|u| {
                                u.query_pairs()
                                    .find(|(key, _)| key == "secret")
                                    .map(|(_, val)| val.into_owned())
                            })
                            .unwrap_or_else(|| "N/A".to_string());

                        match qrcode::QrCode::new(res.qr_code_url.as_bytes()) {
                            Ok(code) => {
                                let svg_content = code
                                    .render()
                                    .min_dimensions(200, 200)
                                    .dark_color(svg::Color("#000000ff"))
                                    .light_color(svg::Color("#ffffffff"))
                                    .build();
                                TotpState::Ready {
                                    svg_content,
                                    secret,
                                }
                            }
                            Err(e) => TotpState::Error(format!("Erro ao gerar QR Code: {}", e)),
                        }
                    }
                    Err(e) => TotpState::Error(format!("Erro de conexão: {}", e)),
                };
                totp_state.set(new_state);
            });
            || ()
        });
    }

    // --- EFFECT 2: Injetar SVG ---
    {
        let totp_state_val = (*totp_state).clone();
        let svg_ref = svg_ref.clone();
        use_effect_with(totp_state_val, move |state| {
            if let TotpState::Ready { svg_content, .. } = state {
                if let Some(element) = svg_ref.cast::<Element>() {
                    element.set_inner_html(svg_content);
                }
            }
            || ()
        });
    }

    // --- HANDLERS ---
    let on_input = {
        let otp_code = otp_code.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let val = input
                .value()
                .chars()
                .filter(|c| c.is_digit(10))
                .take(6)
                .collect::<String>();
            otp_code.set(val);
            error_msg.set(None); // Limpa erro ao digitar
        })
    };

    let on_submit = {
        let navigator = navigator.clone();
        let location = location.clone();
        let user_context = user_context.clone();
        let is_submitting = is_submitting.clone();
        let otp_code = (*otp_code).clone();
        let mfa_token = mfa_token.clone();
        let error_msg = error_msg.clone();

        Callback::from(move |_| {
            let navigator = navigator.clone();
            let location = location.clone();
            let user_context = user_context.clone();
            let is_submitting = is_submitting.clone();
            let mfa_token = mfa_token.clone();
            let code = otp_code.clone();
            let error_msg = error_msg.clone();

            spawn_local(async move {
                is_submitting.set(true);
                match totp_registration_finish(&mfa_token, TotpVerifyRequest { code }).await {
                    Ok(resp) => {
                        let user = User {
                            name: "test".to_string(),
                            email: "email".to_string(),
                            mfa_registration_token: None,
                            mfa_verification_token: None,
                            access_token: Some(resp.access_token),
                            refresh_token: Some(resp.refresh_token),
                        };
                        user_context.state.dispatch(UserAction::Set(user.clone()));

                        let query = location
                            .query::<LoginQuery>()
                            .unwrap_or(LoginQuery { return_to: None });

                        match (user.refresh_token, user.access_token) {
                            (Some(_), Some(_)) => match query.return_to {
                                Some(path) => match <Route as Routable>::recognize(&path) {
                                    Some(route) => navigator.push(&route),
                                    None => navigator.push(&Route::Home),
                                },
                                None => navigator.push(&Route::Home),
                            },
                            _ => error_msg.set(Some("Resposta do servidor inválida".to_string())),
                        }
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Código inválido: {}", e)));
                    }
                }
                is_submitting.set(false);
            });
        })
    };

    html! {
        <AuthCard title="Segurança da Conta">
            <div class="totp-setup-wrapper">
                {
                    match &*totp_state {
                        TotpState::Loading => html! { <LoadingSpinner /> },
                        TotpState::Error(msg) => html! { <ServerError message={msg.clone()} /> },
                        TotpState::Ready { secret, .. } => html! {
                            <>
                                <div class="step-section">
                                    <div class="step-header">
                                        <span class="step-number">{"1."}</span>
                                        <span class="step-title">{"Vincular Aplicativo"}</span>
                                        <span class="step-desc">
                                            {"Abra seu app autenticador e escaneie o código abaixo."}
                                        </span>
                                    </div>

                                    <div class="qr-display">
                                        <div class="qr-frame" ref={svg_ref} />

                                        <div class="manual-key-info">
                                            <span>{"Chave Manual"}</span>
                                            <code>{secret}</code>
                                        </div>
                                    </div>
                                </div>

                                <hr style="border: 0; border-top: 1px solid var(--border); margin: 8px 0;" />

                                <div class="verification-section">
                                    <div class="step-header">
                                        <span class="step-number">{"2."}</span>
                                        <span class="step-title">{"Confirmar Registro"}</span>
                                        <span class="step-desc">{"Digite o código de 6 dígitos gerado pelo app."}</span>
                                    </div>

                                    <div class="input-group">
                                        <input
                                            type="text"
                                            inputmode="numeric"
                                            placeholder="000000"
                                            maxlength="6"
                                            value={(*otp_code).clone()}
                                            oninput={on_input}
                                        />
                                        <button
                                            class="btn-confirm"
                                            onclick={on_submit}
                                            disabled={otp_code.len() != 6 || *is_submitting}
                                        >
                                            if *is_submitting {
                                                {"Validando..."}
                                            } else {
                                                {"Ativar"}
                                            }
                                        </button>
                                    </div>

                                    if let Some(msg) = &*error_msg {
                                        <p style="color: var(--error); font-size: 0.8rem; margin-top: 8px; text-align: center;">
                                            {msg}
                                        </p>
                                    }
                                </div>
                            </>
                        }
                    }
                }
            </div>
        </AuthCard>
    }
}
