use crate::components::ui::darkmode_toggle::DarkmodeToggle;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}

#[component]
pub fn AuthLayout(props: &Props) -> Html {
    html! {
        <div class="auth-wrapper">
            <header class="auth-header">
                <DarkmodeToggle />
            </header>

            <main class="auth-content">
                <div class="auth-container">
                    { for props.children.iter() }
                </div>
            </main>

            <footer class="auth-footer">
                <p>{ "© 2025 - Seu App" }</p>
            </footer>
        </div>
    }
}
