use std::rc::Rc;

use log::error;
use serde::{Deserialize, Serialize};
use web_sys::window;
use yew::{html::ChildrenProps, prelude::*};

const USER_KEY: &str = "user-context";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct User {
    pub name: String,
    pub email: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub mfa_token: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct UserState {
    pub user: Option<User>,
}

#[derive(Clone, PartialEq)]
pub struct UserContext {
    pub state: UseReducerHandle<UserState>,
}

pub enum UserAction {
    Set(User),
    Clear,
}

impl Reducible for UserState {
    type Action = UserAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            UserAction::Set(user) => {
                save_user(&user);
                Rc::new(Self { user: Some(user) })
            }
            UserAction::Clear => {
                remove_user();
                Rc::new(Self { user: None })
            }
        }
    }
}

pub fn get_user() -> Option<User> {
    let window = window()?;
    let storage = window.local_storage().ok().flatten()?;
    let saved = storage.get_item(USER_KEY).ok().flatten()?;

    serde_json::from_str(&saved)
        .map_err(|e| {
            error!("Failed to deserialize User: {}", e);
            let _ = storage.remove_item(USER_KEY);
            e
        })
        .ok()
}

pub fn save_user(user: &User) -> Option<()> {
    let window = window()?;
    let storage = window.local_storage().ok().flatten()?;

    let json = serde_json::to_string(user)
        .map_err(|e| {
            error!("Failed to serialize User: {}", e);
            e
        })
        .ok()?;

    storage.set_item(USER_KEY, &json).ok()?;
    Some(())
}

pub fn remove_user() -> Option<()> {
    let window = window()?;
    let storage = window.local_storage().ok().flatten()?;

    storage.remove_item(USER_KEY).ok()?;
    Some(())
}

#[component]
pub fn UserProvider(props: &ChildrenProps) -> Html {
    let state = use_reducer(|| UserState { user: get_user() });

    html! {
        <ContextProvider<UserContext> context={UserContext { state }}>
            { props.children.clone() }
        </ContextProvider<UserContext>>
    }
}
