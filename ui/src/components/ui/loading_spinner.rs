use yew::prelude::*;

#[component]
pub fn LoadingSpinner() -> Html {
    html! {
        <div class="loader-container">
            <p>{"Gerando seu código de segurança..."}</p>
            <div class="loader-container-spinner"></div>
        </div>
    }
}
