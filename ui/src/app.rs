use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::layout::auth::AuthLayout;
use crate::components::layout::main::MainLayout;
use crate::context::theme::ThemeProvider;
use crate::pages::home::HomePage;
use crate::pages::login::LoginPage;
use crate::pages::not_found::NotFoundPage;
use crate::pages::signup::SignupPage;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/signup")]
    Signup,
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
        Route::Home => html! {
            <MainLayout>
                <HomePage />
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
                <BrowserRouter>
                    <Switch<Route> render={switch} />
                </BrowserRouter>
            </ThemeProvider>
        </>
    }
}
