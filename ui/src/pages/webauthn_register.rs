use crate::components::ui::button::Button;
use crate::components::ui::server_error::ServerError;
use crate::context::user::UserContext;
use crate::{components::auth_card::AuthCard, services::auth::webauthn_registration_start};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::js_sys::{Reflect, Uint8Array};
use web_sys::{
    CredentialCreationOptions, PublicKeyCredential, PublicKeyCredentialCreationOptions, js_sys,
    window,
};
use yew::prelude::*;

pub fn prepare_pk_options(
    json_str: &str,
) -> Result<web_sys::PublicKeyCredentialCreationOptions, String> {
    // 1. Parse do JSON para um objeto JS genérico direto (mais seguro para o navegador)
    let js_value = js_sys::JSON::parse(json_str).map_err(|_| "JSON inválido vindo do servidor")?;

    // 2. Extrair o conteúdo interno se houver o envelope "publicKey"
    let pk_options_js = if let Ok(inner) = Reflect::get(&js_value, &JsValue::from_str("publicKey"))
    {
        if inner.is_object() {
            inner
        } else {
            js_value.clone()
        }
    } else {
        js_value.clone()
    };

    // 3. Corrigir o Challenge (de String para Uint8Array)
    if let Ok(challenge_b64) = Reflect::get(&pk_options_js, &JsValue::from_str("challenge")) {
        if let Some(s) = challenge_b64.as_string() {
            let bytes = URL_SAFE_NO_PAD
                .decode(s)
                .map_err(|_| "Erro no challenge b64")?;
            let array = Uint8Array::from(&bytes[..]);
            Reflect::set(&pk_options_js, &JsValue::from_str("challenge"), &array).unwrap();
        }
    }

    // 4. Corrigir o User ID (de String para Uint8Array)
    if let Ok(user_obj) = Reflect::get(&pk_options_js, &JsValue::from_str("user")) {
        if let Ok(id_b64) = Reflect::get(&user_obj, &JsValue::from_str("id")) {
            if let Some(s) = id_b64.as_string() {
                let bytes = URL_SAFE_NO_PAD
                    .decode(s)
                    .map_err(|_| "Erro no user id b64")?;
                let array = Uint8Array::from(&bytes[..]);
                Reflect::set(&user_obj, &JsValue::from_str("id"), &array).unwrap();
            }
        }
    }

    // 5. Validar se pubKeyCredParams existe (o erro que você recebeu)
    let params = Reflect::get(&pk_options_js, &JsValue::from_str("pubKeyCredParams"))
        .map_err(|_| "Campo pubKeyCredParams ausente")?;

    if params.is_undefined() || params.is_null() {
        return Err("O servidor não enviou pubKeyCredParams".to_string());
    }

    // Retorna o objeto pronto para o web-sys
    Ok(pk_options_js.unchecked_into::<web_sys::PublicKeyCredentialCreationOptions>())
}

fn js_value_to_string(val: JsValue) -> String {
    // Tenta pegar a propriedade "message" (comum em erros de JS/DOM)
    if let Ok(msg) = Reflect::get(&val, &JsValue::from_str("message")) {
        if let Some(s) = msg.as_string() {
            return s;
        }
    }
    // Caso contrário, tenta converter o objeto inteiro para string
    val.as_string()
        .unwrap_or_else(|| "Erro desconhecido na chave de segurança".to_string())
}

#[component]
pub fn WebAuthnRegisterPage() -> Html {
    let user_context = use_context::<UserContext>().expect("no user ctx found");
    let is_loading = use_state(|| false);
    let status_msg = use_state(|| String::new());
    let error_msg = use_state(|| String::new());

    let mfa_token = user_context
        .state
        .user
        .as_ref()
        .and_then(|user| user.mfa_registration_token.as_ref().cloned())
        .expect("mfa registration token");

    let handle_register = {
        let mfa_token = mfa_token.clone();
        let is_loading = is_loading.clone();
        let status_msg = status_msg.clone();
        let error_msg = error_msg.clone();

        move |_| {
            let mfa_token = mfa_token.clone();
            let is_loading = is_loading.clone();
            let status_msg = status_msg.clone();
            let error_msg = error_msg.clone();

            spawn_local(async move {
                is_loading.set(true);
                error_msg.set("".into());
                status_msg.set("Iniciando registro...".into());

                // 1. POST /webauthn/register_start
                // Aqui você recebe as opções (challenge, user id, etc) do servidor
                // Nota: O backend deve retornar os campos em base64 ou bytes
                let start_resp = match webauthn_registration_start(&mfa_token).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        error_msg.set(e);
                        is_loading.set(false);
                        return;
                    }
                };

                status_msg.set("Toque na sua chave de segurança...".into());

                // 2. Chamar a API do Navegador (WebAuthn)
                let window = window().unwrap();
                let nav = window.navigator();
                let credentials = nav.credentials();
                let options = CredentialCreationOptions::new();
                // let js_value = js_sys::JSON::parse(&start_resp).expect("JSON inválido");
                // fix_webauthn_types(&js_value);

                let pk_options = prepare_pk_options(&start_resp).unwrap();

                options.set_public_key(&pk_options);

                // Converte as opções do JS para o formato que o navegador entende
                let promise = credentials.create_with_options(&options).unwrap();
                let result = wasm_bindgen_futures::JsFuture::from(promise).await;

                match result {
                    Ok(cred) => {
                        let cred = cred.dyn_into::<PublicKeyCredential>().unwrap();

                        // 3. POST /webauthn/register_finish
                        // Envia a resposta da chave de volta para validação
                        status_msg.set("Finalizando no servidor...".into());
                        match call_register_finish(cred).await {
                            Ok(_) => status_msg.set("Dispositivo registrado com sucesso!".into()),
                            Err(e) => error_msg.set(e),
                        }
                    }
                    Err(js_err) => {
                        let err_string = js_value_to_string(js_err);
                        web_sys::console::log_1(
                            &format!("Erro detalhado do WebAuthn: {}", err_string).into(),
                        );
                        error_msg.set(err_string); // Isso vai mostrar o nome real do erro (ex: SecurityError)
                        is_loading.set(false);
                    }
                }

                is_loading.set(false);
            });
        }
    };

    html! {
        <AuthCard title="Segurança">
            <div class="webauthn-container">
                <div class="webauthn-icon">{"🔑"}</div>
                <h3>{"Chave de Segurança"}</h3>
                <p>{"Adicione uma Yubikey ou Passkey para proteger sua conta."}</p>

                <Button
                    label="Registrar Novo Dispositivo"
                    onclick={handle_register}
                    is_loading={*is_loading}
                />

                if !status_msg.is_empty() && error_msg.is_empty() {
                    <p class="status-info">{ &*status_msg }</p>
                }

                <ServerError message={(*error_msg).clone()} />
            </div>
        </AuthCard>
    }
}

// Funções de mock para as chamadas de API (implementar com seu service de auth)
async fn call_register_start() -> Result<PublicKeyCredentialCreationOptions, String> {
    /* api_client.post("/webauthn/register_start").await */
    unimplemented!()
}

async fn call_register_finish(_cred: PublicKeyCredential) -> Result<(), String> {
    /* api_client.post("/webauthn/register_finish", _cred).await */
    unimplemented!()
}
