use std::collections::HashMap;
use validator::ValidationErrors;
use yew::UseStateHandle;

use crate::components::ui::input_field::Field;

pub fn get_validation_errors(errs: ValidationErrors) -> HashMap<String, String> {
    errs.field_errors()
        .into_iter()
        .map(|(field, field_errs)| {
            let message = field_errs
                .first()
                .and_then(|e| e.message.as_ref())
                .map(|m| m.to_string())
                .unwrap_or_else(|| "Campo inválido".to_string());

            (field.to_string(), message)
        })
        .collect()
}

pub fn sync_field_error(
    field: &UseStateHandle<Field>,
    key: &str,
    error_map: &HashMap<String, String>,
) {
    let new_error = error_map.get(key).cloned();

    if (*field).error != new_error {
        let mut f = (**field).clone();
        f.error = new_error; // Aqui f.error vira None
        field.set(f);
    }
}
