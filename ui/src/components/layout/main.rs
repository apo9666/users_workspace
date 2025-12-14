use yew::prelude::*;

use crate::components::layout::header::HeaderLayout;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}

#[component]
pub fn MainLayout(props: &Props) -> Html {
    html! {
        <div class="app-container">
            <HeaderLayout />
            <main class="app-main">
                { props.children.clone() }
            </main>
        </div>
    }
}
