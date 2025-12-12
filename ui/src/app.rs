use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::NodeRef;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::ui::darkmode_toggle::DarkmodeToggle;
use crate::context::theme::ThemeProvider;

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

#[component(App)]
pub fn app() -> Html {
    html! {
        <>
            <ThemeProvider>
                <header class="app-header">
                    <DarkmodeToggle />
                </header>
                <BrowserRouter>
                    <Switch<Route> render={switch} /> // <- must be child of <BrowserRouter>
                </BrowserRouter>
            </ThemeProvider>
        </>
    }
}
