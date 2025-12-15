use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::Route;
use crate::context::user::UserContext;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}

#[derive(serde::Serialize)]
struct ReturnToQuery {
    return_to: String,
}

#[component]
pub fn ProtectedRoute(props: &Props) -> Html {
    let user_ctx = use_context::<UserContext>().expect("no user ctx found");
    let navigator = use_navigator().expect("Navigator not found");
    let location = use_location().expect("Location not found");

    use_effect_with(
        (user_ctx.state.user.is_some(), location.path().to_string()),
        move |(is_auth, path)| {
            if !is_auth {
                let query = ReturnToQuery {
                    return_to: path.clone(),
                };

                let _ = navigator.push_with_query(&Route::Login, &query);
            }
        },
    );

    if user_ctx.state.user.is_some() {
        html! {
            { props.children.clone() }
        }
    } else {
        html! {}
    }
}
