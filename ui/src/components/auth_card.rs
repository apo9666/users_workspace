use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub title: String,
    pub children: Children,
}

#[component]
pub fn AuthCard(props: &Props) -> Html {
    html! {
        <div class="auth-container">
            <div class="auth-box">
                <h1>{ &props.title }</h1>
                { props.children.clone() }
            </div>
        </div>
    }
}
