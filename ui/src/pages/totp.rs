use crate::{
    components::ui::loading_spinner::LoadingSpinner, components::ui::server_error::ServerError,
    context::user::UserContext, services::auth::totp_setup,
};
use qrcode::render::svg;
use web_sys::Element;
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone, PartialEq)]
enum TotpState {
    Loading,
    Error(String),
    Ready(String), // Contém a string SVG do QR Code
}

#[component]
pub fn TotpPage() -> Html {
    let user_context = use_context::<UserContext>().expect("no user ctx found");
    let totp_state = use_state(|| TotpState::Loading);
    let svg_ref = use_node_ref();

    let mfa_token = user_context
        .state
        .user
        .as_ref()
        .and_then(|user| user.mfa_registration_token.as_ref().cloned())
        .expect("no mfa registration token");

    // --- EFFECT 1: Buscar dados e Gerar SVG ---
    {
        let totp_state = totp_state.clone();
        let mfa_token = mfa_token.clone();

        use_effect_with((), move |_| {
            if matches!(*totp_state, TotpState::Loading) {
                spawn_local(async move {
                    let new_state = match totp_setup(&mfa_token).await {
                        Ok(res) => match qrcode::QrCode::new(res.qr_code_url.as_bytes()) {
                            Ok(code) => {
                                let svg_content = code
                                    .render()
                                    .min_dimensions(220, 220)
                                    .dark_color(svg::Color("#000000ff"))
                                    .light_color(svg::Color("#ffffffff"))
                                    .build();
                                TotpState::Ready(svg_content)
                            }
                            Err(e) => TotpState::Error(format!("Erro ao gerar QR Code: {}", e)),
                        },
                        Err(e) => TotpState::Error(format!("Erro de conexão/servidor: {}", e)),
                    };
                    totp_state.set(new_state);
                });
            }
            || ()
        });
    }

    // --- EFFECT 2: Injetar o SVG no DOM (necessário para injetar HTML bruto em Yew) ---
    {
        let svg_content_to_inject = match &*totp_state {
            TotpState::Ready(content) => Some(content.clone()),
            _ => None,
        };
        let svg_ref = svg_ref.clone();

        use_effect_with(svg_content_to_inject, move |svg_content| {
            if let Some(url) = svg_content {
                if let Some(element) = svg_ref.cast::<Element>() {
                    // Injeta a string SVG completa (<svg>...</svg>)
                    element.set_inner_html(url);
                }
            }
            || ()
        });
    }

    html! {
        <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 300px;">
            <h3>{"Configuração de Segundo Fator (2FA)"}</h3>
            {
                match &*totp_state {
                    TotpState::Loading => html! { <LoadingSpinner /> },

                    TotpState::Ready(_) => html! {
                        <div style="animation: fadeIn 0.5s; text-align: center;">
                            // Esta div atua como contêiner para a injeção do SVG
                            <div
                                ref={svg_ref}
                                class="totp-qr-code-container"
                            >
                            </div>
                            <p><small>{"Escaneie o código acima no seu app autenticador."}</small></p>
                        </div>
                    },

                    TotpState::Error(msg) => html! {
                        <ServerError message={msg.clone()} />
                    },
                }
             }
        </div>
    }
}
