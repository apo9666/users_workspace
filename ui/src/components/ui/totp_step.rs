use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TotpStepProps {
    pub number: String,
    pub title: String,
    pub description: String,
}

#[component]
pub fn TotpStep(props: &TotpStepProps) -> Html {
    html! {
        <div class="step-header">
            <span class="step-number">{ &props.number }</span>
            <span class="step-title">{ &props.title }</span>
            <span class="step-desc">{ &props.description }</span>
        </div>
    }
}
