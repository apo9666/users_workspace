use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::ui::darkmode_toggle::DarkmodeToggle;
use crate::context::theme::ThemeProvider;
use crate::pages::home::HomePage;
use crate::pages::login::LoginPage;
use crate::pages::not_found::NotFoundPage;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Login => html! {
            <LoginPage />
        },
        Route::Home => html! {
           <HomePage />
        },
        Route::NotFound => html! {
            <NotFoundPage />
        },
    }
}

#[component]
pub fn App() -> Html {
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
