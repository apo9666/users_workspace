use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub message: String,
}

#[component]
pub fn ServerError(props: &Props) -> Html {
    if props.message.is_empty() {
        return html! {};
    }

    html! {
        <p class="server-error">
            { &props.message }
        </p>
    }
}
