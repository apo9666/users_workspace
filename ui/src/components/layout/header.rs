use crate::components::ui::darkmode_toggle::DarkmodeToggle;
use yew::prelude::*;

#[component]
pub fn HeaderLayout() -> Html {
    html! {
        <header class="app-header">
            <div class="header-content">
                <div class="logo">
                    <img src="/logo.svg" alt="Logo" class="logo-img" />
                    <span>{"RustApp"}</span>
                </div>
                <nav class="header-nav">
                    // Futuros links de navegação aqui
                    <DarkmodeToggle />
                </nav>
            </div>
        </header>
    }
}
