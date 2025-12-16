use crate::{
    components::ui::server_error::ServerError, context::user::UserContext,
    services::auth::totp_setup,
};
use qrcode::render::svg;
use web_sys::Element; // Importar Element de web_sys
use yew::{platform::spawn_local, prelude::*};

#[component]
pub fn TotpPage() -> Html {
    let user_context = use_context::<UserContext>().expect("no user ctx found");
    let qr_code_url = use_state(|| None::<String>);
    let is_loading = use_state(|| true);
    let server_error = use_state(|| String::new());
    let svg_ref = use_node_ref();

    // 1. Obter o mfa_registration_token
    let mfa_token = user_context
        .state
        .user
        .as_ref()
        .and_then(|user| user.mfa_registration_token.as_ref().cloned())
        .expect("no mfa registration token");

    // --- EFFECT 1: Buscar dados, gerar SVG, e atualizar estado ---
    {
        let qr_code_url = qr_code_url.clone();
        let is_loading = is_loading.clone();
        let server_error = server_error.clone();
        let mfa_token = mfa_token.clone(); // Clonar para o bloco async

        use_effect_with((), move |_| {
            spawn_local(async move {
                match totp_setup(&mfa_token).await {
                    Ok(res) => {
                        // Resposta de sucesso. `res.qr_code_url` é o URI 'otpauth://...'

                        // 2. Lógica de Geração do QR Code SVG
                        match qrcode::QrCode::new(
                            res.qr_code_url.as_bytes(), // Converter a string URI para bytes
                        ) {
                            Ok(code) => {
                                let image = code
                                    .render()
                                    .min_dimensions(220, 220)
                                    .dark_color(svg::Color("#000000ff"))
                                    .light_color(svg::Color("#ffffffff"))
                                    .build(); // String com o conteúdo SVG

                                qr_code_url.set(Some(image));
                                server_error.set("".to_string()); // Limpa qualquer erro anterior
                            }
                            Err(e) => server_error.set(format!("Erro ao gerar QR Code: {}", e)),
                        }
                    }
                    Err(e) => server_error.set(format!("Erro de conexão/servidor: {}", e)),
                }
                is_loading.set(false);
            });
            || () // Cleanup opcional
        });
    }

    // --- EFFECT 2: Injetar o SVG no DOM após a renderização ---
    // Este effect roda quando qr_code_url muda E o componente é renderizado.
    let svg_content_to_inject = qr_code_url.as_ref().cloned();
    let svg_ref_clone = svg_ref.clone();

    use_effect_with(svg_content_to_inject, move |svg_content| {
        if let Some(url) = svg_content {
            // Verifica se a tag <svg> existe no DOM
            if let Some(element) = svg_ref_clone.cast::<Element>() {
                // Injeta o conteúdo SVG (raw HTML)
                element.set_inner_html(url);
            }
        }
        || ()
    });

    // --- RENDERIZAÇÃO ---
    html! {
        <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 300px;">
            <h3>{"Configuração de Segundo Fator (2FA)"}</h3>

            {
                if *is_loading {
                    html! {
                        <div class="loader-container">
                            <p>{"Gerando seu código de segurança..."}</p>
                            <div class="spinner"></div>
                        </div>
                    }
                }
                else if qr_code_url.is_some() {
                    html! {
                        <div style="animation: fadeIn 0.5s; text-align: center;">
                            <svg
                                ref={svg_ref} // A referência está aqui!
                                style="border: 4px solid white; box-shadow: 0 0 10px rgba(0,0,0,0.1);"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 220 220"
                                width="200"
                                height="200"
                            >
                                // O conteúdo será injetado pelo Effect 2
                            </svg>
                            <p><small>{"Escaneie o código acima no seu app autenticador."}</small></p>
                        </div>
                    }
                } else {
                    html! { <p>{"Não foi possível carregar o QR Code."}</p> }
                }
            }
            {
                if !server_error.is_empty() {
                    html! { <ServerError message={(*server_error).clone()} /> }
                } else {
                    // Retorna um fragmento vazio válido
                    html! { <> </> }
                }
            }
        </div>
    }
}
