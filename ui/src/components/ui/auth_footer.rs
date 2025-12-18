use crate::app::Route;
use yew::prelude::*;
use yew_router::prelude::Link;

#[derive(Properties, PartialEq)]
pub struct AuthFooterProps {
    pub message: String,
    pub link_text: String,
    pub to: Route,
}

#[component]
pub fn AuthFooter(props: &AuthFooterProps) -> Html {
    html! {
        <div class="auth-footer">
            <span class="auth-footer__text">{ &props.message }</span>
            <Link<Route> to={props.to.clone()} classes="auth-footer__link">
                { &props.link_text }
            </Link<Route>>
        </div>
    }
}
