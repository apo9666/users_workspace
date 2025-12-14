use std::collections::HashMap;
use validator::ValidationErrors;

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
