use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::NodeRef;
use yew::prelude::*;

use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/secure")]
    Secure,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[component(Secure)]
fn secure() -> Html {
    let navigator = use_navigator().unwrap();

    let onclick = Callback::from(move |_| navigator.push(&Route::Home));
    html! {
        <div>
            <h1>{ "Secure" }</h1>
            <button {onclick}>{ "Go Home" }</button>
        </div>
    }
}

#[component]
fn LoginPage() -> Html {
    let email_ref = NodeRef::default();
    let password_ref = NodeRef::default();
    let login_status = use_state(|| String::new());

    let handle_login = {
        let email_ref = email_ref.clone();
        let password_ref = password_ref.clone();
        let login_status = login_status.clone();

        move |_: MouseEvent| {
            let email = email_ref
                .cast::<HtmlInputElement>()
                .and_then(|el| Some(el.value()))
                .unwrap_or_default();
            let password = password_ref
                .cast::<HtmlInputElement>()
                .and_then(|el| Some(el.value()))
                .unwrap_or_default();

            if email.is_empty() || password.is_empty() {
                login_status.set("Por favor, preencha todos os campos".to_string());
                return;
            }

            let login_status = login_status.clone();

            spawn_local(async move {
                let client = reqwest::Client::new();
                let body = serde_json::json!({
                    "email": email,
                    "password": password
                });

                match client
                    .post("http://localhost:8080/login")
                    .json(&body)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            login_status.set(format!("Login bem-sucedido! Bem-vindo, {}", email));
                        } else {
                            login_status.set(format!("Erro no login: {}", response.status()));
                        }
                    }
                    Err(e) => {
                        login_status.set(format!("Erro na conexão: {}", e));
                    }
                }
            });
        }
    };

    html! {
        <div class="login-container">
            <div class="login-box">
                <h1>{"Login"}</h1>
                <div class="form-group">
                    <label>{"Email:"}</label>
                    <input
                        ref={email_ref.clone()}
                        type="email"
                        placeholder="seu@email.com"
                        class="form-input"
                    />
                </div>
                <div class="form-group">
                    <label>{"Senha:"}</label>
                    <input
                        ref={password_ref.clone()}
                        type="password"
                        placeholder="Sua senha"
                        class="form-input"
                    />
                </div>
                <button onclick={handle_login} class="login-btn">
                    {"Entrar"}
                </button>
                if !login_status.is_empty() {
                    <p class="status-message">{ (*login_status).clone() }</p>
                }
            </div>
        </div>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Login => html! {
            <LoginPage />
        },
        Route::Home => html! { <h1>{ "Home" }</h1> },
        Route::Secure => html! {
            <Secure />
        },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[component]
fn App() -> Html {
    // Detect system preference or use localStorage
    let detect_system_preference = || {
        if let Some(window) = web_sys::window() {
            // Check localStorage first
            if let Some(storage) = window.local_storage().ok().flatten() {
                if let Ok(Some(saved)) = storage.get_item("theme-preference") {
                    return saved == "dark";
                }
            }
        }
        // Default to light theme; user can toggle
        false
    };

    let is_dark = use_state(detect_system_preference);

    // Apply theme on mount and whenever is_dark changes
    {
        let is_dark = *is_dark;
        use_effect(move || {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(root) = document.document_element() {
                        if is_dark {
                            let _ = root.set_attribute("data-theme", "dark");
                        } else {
                            let _ = root.remove_attribute("data-theme");
                        }
                    }
                }
            }
            || ()
        });
    }

    let toggle_theme = {
        let is_dark = is_dark.clone();
        Callback::from(move |_| {
            let new = !*is_dark;
            is_dark.set(new);

            // Save preference to localStorage
            if let Some(window) = web_sys::window() {
                if let Some(storage) = window.local_storage().ok().flatten() {
                    let _ =
                        storage.set_item("theme-preference", if new { "dark" } else { "light" });
                }
                if let Some(document) = window.document() {
                    if let Some(root) = document.document_element() {
                        if new {
                            let _ = root.set_attribute("data-theme", "dark");
                        } else {
                            let _ = root.remove_attribute("data-theme");
                        }
                    }
                }
            }
        })
    };

    html! {
        <>
            <header class="app-header">
                <div class="darkmode-toggle">
                    <input
                        type="checkbox"
                        id="darkmode-toggle"
                        checked={*is_dark}
                        onchange={toggle_theme.clone()}
                    />
                    <label for="darkmode-toggle">
                        <svg class="sun" viewBox="0 0 496 496">
                            <rect x="152.994" y="58.921" transform="matrix(0.3827 0.9239 -0.9239 0.3827 168.6176 -118.5145)" width="40.001" height="16" />
                            <rect x="46.9" y="164.979" transform="matrix(0.9239 0.3827 -0.3827 0.9239 71.29 -12.4346)" width="40.001" height="16" />
                            <rect x="46.947" y="315.048" transform="matrix(0.9239 -0.3827 0.3827 0.9239 -118.531 50.2116)" width="40.001" height="16" />
                            <rect x="164.966" y="409.112" transform="matrix(-0.9238 -0.3828 0.3828 -0.9238 168.4872 891.7491)" width="16" height="39.999" />
                            <rect x="303.031" y="421.036" transform="matrix(-0.3827 -0.9239 0.9239 -0.3827 50.2758 891.6655)" width="40.001" height="16" />
                            <rect x="409.088" y="315.018" transform="matrix(-0.9239 -0.3827 0.3827 -0.9239 701.898 785.6559)" width="40.001" height="16" />
                            <rect x="409.054" y="165.011" transform="matrix(-0.9239 0.3827 -0.3827 -0.9239 891.6585 168.6574)" width="40.001" height="16" />
                            <rect x="315.001" y="46.895" transform="matrix(0.9238 0.3828 -0.3828 0.9238 50.212 -118.5529)" width="16" height="39.999" />
                            <path d="M248,88c-88.224,0-160,71.776-160,160s71.776,160,160,160s160-71.776,160-160S336.224,88,248,88z M248,392 c-79.4,0-144-64.6-144-144s64.6-144,144-144s144,64.6,144,144S327.4,392,248,392z" />
                            <rect x="240" width="16" height="72" />
                            <rect x="62.097" y="90.096" transform="matrix(0.7071 0.7071 -0.7071 0.7071 98.0963 -40.6334)" width="71.999" height="16" />
                            <rect y="240" width="72" height="16" />
                            <rect x="90.091" y="361.915" transform="matrix(-0.7071 -0.7071 0.7071 -0.7071 -113.9157 748.643)" width="16" height="71.999" />
                            <rect x="240" y="424" width="16" height="72" />
                            <rect x="361.881" y="389.915" transform="matrix(-0.7071 -0.7071 0.7071 -0.7071 397.8562 960.6281)" width="71.999" height="16" />
                            <rect x="424" y="240" width="72" height="16" />
                            <rect x="389.911" y="62.091" transform="matrix(0.7071 0.7071 -0.7071 0.7071 185.9067 -252.6357)" width="16" height="71.999" />
                        </svg>
                        <svg class="moon" viewBox="0 0 49.739 49.739">
                            <path d="M25.068,48.889c-9.173,0-18.017-5.06-22.396-13.804C-3.373,23.008,1.164,8.467,13.003,1.979l2.061-1.129l-0.615,2.268
                                            c-1.479,5.459-0.899,11.25,1.633,16.306c2.75,5.493,7.476,9.587,13.305,11.526c5.831,1.939,12.065,1.492,17.559-1.258v0
                                            c0.25-0.125,0.492-0.258,0.734-0.391l2.061-1.13l-0.585,2.252c-1.863,6.873-6.577,12.639-12.933,15.822
                                            C32.639,48.039,28.825,48.888,25.068,48.889z M12.002,4.936c-9.413,6.428-12.756,18.837-7.54,29.253
                                            c5.678,11.34,19.522,15.945,30.864,10.268c5.154-2.582,9.136-7.012,11.181-12.357c-5.632,2.427-11.882,2.702-17.752,0.748
                                            c-6.337-2.108-11.473-6.557-14.463-12.528C11.899,15.541,11.11,10.16,12.002,4.936z" />
                        </svg>
                    </label>
                </div>
            </header>
            <BrowserRouter>
                <Switch<Route> render={switch} /> // <- must be child of <BrowserRouter>
            </BrowserRouter>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
