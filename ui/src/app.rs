use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::layout::auth::AuthLayout;
use crate::components::layout::main::MainLayout;
use crate::components::protected_route::ProtectedRoute;
use crate::context::theme::ThemeProvider;
use crate::context::user::UserProvider;
use crate::pages::home::HomePage;
use crate::pages::login::LoginPage;
use crate::pages::not_found::NotFoundPage;
use crate::pages::signup::SignupPage;
use crate::pages::totp::TotpPage;
use crate::pages::webauthn_register::WebAuthnRegisterPage;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/signup")]
    Signup,
    #[at("/totp")]
    Totp,
    #[at("/webauthn/register")]
    WebAuthnRegister,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Login => html! {
            <AuthLayout>
                <LoginPage />
            </AuthLayout>
        },
        Route::Signup => html! {
            <AuthLayout>
                <SignupPage />
            </AuthLayout>
        },
        Route::Totp => html! {
            <AuthLayout>
                <ProtectedRoute>
                    <TotpPage />
                </ProtectedRoute>
            </AuthLayout>
        },
        Route::WebAuthnRegister => html! {
            <AuthLayout>
                <ProtectedRoute>
                    <WebAuthnRegisterPage />
                </ProtectedRoute>
            </AuthLayout>
        },
        Route::Home => html! {
            <MainLayout>
                <ProtectedRoute>
                    <HomePage />
                </ProtectedRoute>
            </MainLayout>
        },
        Route::NotFound => html! { <NotFoundPage /> },
    }
}

#[component]
pub fn App() -> Html {
    html! {
        <>
            <ThemeProvider>
                <UserProvider>
                    <BrowserRouter>
                        <Switch<Route> render={switch} />
                    </BrowserRouter>
                </UserProvider>
            </ThemeProvider>
        </>
    }
}
