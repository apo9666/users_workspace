use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    pub label: &'static str,
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub is_loading: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or("button")]
    pub btn_type: &'static str, // button, submit, etc.
}

#[component]
pub fn Button(props: &ButtonProps) -> Html {
    html! {
        <button
            type={props.btn_type}
            onclick={props.onclick.clone()}
            disabled={props.is_loading}
            class={classes!(
                "ui-button",
                props.is_loading.then(|| "btn-loading"),
                props.class.clone()
            )}
        >
            if props.is_loading {
                <span class="spinner"></span>
                { "Aguarde..." }
            } else {
                { props.label }
            }
        </button>
    }
}
