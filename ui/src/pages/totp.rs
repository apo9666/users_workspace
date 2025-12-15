use yew::{platform::spawn_local, prelude::*};

use crate::{components::ui::server_error::ServerError, services::auth::totp_setup};

#[component]
fn TotpPage() -> Html {
    let qr_code_url = use_state(|| None::<String>);
    let is_loading = use_state(|| true); // Começa como true
    let server_error = use_state(|| String::new());

    {
        let qr_code_url = qr_code_url.clone();
        let is_loading = is_loading.clone();
        let server_error = server_error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match totp_setup("aut").await {
                    Ok(res) if res.status().is_success() => {
                        server_error.set(format!("Conta criada com sucesso"));
                    }
                    Ok(res) => server_error.set(format!("Erro no servidor: {}", res.status())),
                    Err(e) => server_error.set(format!("Erro de conexão: {}", e)),
                }
                is_loading.set(false);
            });
            || () // Cleanup opcional
        });
    }

    html! {
        <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 300px;">
            <h3>{"Configuração de Segundo Fator (2FA)"}</h3>

            {
                if *is_loading {
                    html! {
                        <div class="loader-container">
                            <p>{"Gerando seu código de segurança..."}</p>
                            <div class="spinner"></div> // Você pode estilizar isso no seu CSS
                        </div>
                    }
                }
                else if let Some(url) = &*qr_code_url {
                    html! {
                        <div style="animation: fadeIn 0.5s;">
                            <img src={url.clone()} alt="QR Code" style="border: 4px solid white; box-shadow: 0 0 10px rgba(0,0,0,0.1);" />
                            <p><small>{"Escaneie o código acima no seu app autenticador."}</small></p>
                        </div>
                    }
                } else {
                    html! { <p>{"Não foi possível carregar o QR Code."}</p> }
                }
            }
            <ServerError message={(*server_error).clone()} />

        </div>
    }
}
