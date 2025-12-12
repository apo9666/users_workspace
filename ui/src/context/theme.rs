use yew::{html::ChildrenProps, prelude::*};

#[derive(Clone, PartialEq)]
pub struct ThemeState {
    pub toggle_theme: Callback<Event>,
    pub is_dark: UseStateHandle<bool>,
}

fn detect_system_preference() -> bool {
    if let Some(window) = web_sys::window() {
        if let Some(storage) = window.local_storage().ok().flatten() {
            if let Ok(Some(saved)) = storage.get_item("theme-preference") {
                return saved == "dark";
            }
        }
    }
    false
}

fn set_theme(is_dark: bool) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(root) = document.document_element() {
                if is_dark {
                    let _ = root.set_attribute("data-theme", "dark");
                } else {
                    let _ = root.remove_attribute("data-theme");
                }
            }
        }
    }
}

#[component(ThemeProvider)]
pub fn theme_provider(props: &ChildrenProps) -> Html {
    let is_dark = use_state(|| detect_system_preference());

    set_theme(*is_dark);

    let toggle_theme = {
        let is_dark = is_dark.clone();
        Callback::from(move |_| {
            let new = !*is_dark;
            is_dark.set(new);

            // Save preference to localStorage
            if let Some(window) = web_sys::window() {
                if let Some(storage) = window.local_storage().ok().flatten() {
                    let _ =
                        storage.set_item("theme-preference", if new { "dark" } else { "light" });
                }
                set_theme(new);
            }
        })
    };

    let theme_state = ThemeState {
        toggle_theme,
        is_dark,
    };

    html! {
        <ContextProvider<ThemeState> context={theme_state}>
            { props.children.clone() }
        </ContextProvider<ThemeState>>
    }
}
