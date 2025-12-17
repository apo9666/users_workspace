use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TotpManualKeyProps {
    pub secret: String,
}

#[component]
pub fn TotpManualKey(props: &TotpManualKeyProps) -> Html {
    html! {
        <div class="manual-key-info">
            <span>{"Chave Manual"}</span>
            <code>{ &props.secret }</code>
        </div>
    }
}
