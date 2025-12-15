use yew::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub struct Field {
    pub value: String,
    pub error: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct InputFieldProps {
    pub label: &'static str,
    pub input_type: &'static str,
    pub placeholder: &'static str,
    pub field: UseStateHandle<Field>,
}

#[component]
pub fn InputField(props: &InputFieldProps) -> Html {
    let oninput = {
        let field = props.field.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            field.set(Field {
                value: input.value(),
                error: None,
            });
        })
    };

    let has_error = props.field.error.is_some();

    html! {
        <div class="form-group">
            <label>{props.label}</label>
            <input
                type={props.input_type}
                value={props.field.value.clone()}
                oninput={oninput}
                placeholder={props.placeholder}
                class={classes!("form-input", has_error.then(|| "input-error"))}
            />
            if let Some(msg) = &props.field.error {
                <span class="error-message">{msg}</span>
            }
        </div>
    }
}
