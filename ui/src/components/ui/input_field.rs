use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InputFieldProps {
    pub label: &'static str,
    pub input_type: &'static str,
    pub placeholder: &'static str,
    pub name: &'static str,
    pub input_ref: NodeRef,
    pub error: Option<String>,
    // Adicionamos um callback opcional para limpar o erro ao digitar
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
}

#[function_component(InputField)]
pub fn input_field(props: &InputFieldProps) -> Html {
    let has_error = props.error.is_some();

    let input_class = classes!("form-input", has_error.then(|| "input-error"));

    html! {
        <div class="form-group">
            <label>{props.label}</label>
            <input
                ref={props.input_ref.clone()}
                type={props.input_type}
                placeholder={props.placeholder}
                class={input_class}
                oninput={props.oninput.clone()}
            />
            if let Some(msg) = &props.error {
                <span class="error-message">{msg}</span>
            }
        </div>
    }
}
