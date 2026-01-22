// Helper functions for MetaAgent tools

use serde_json::{json, Value};

/// Create a success result with message
pub fn success(message: impl Into<String>) -> Value {
    json!({
        "success": true,
        "message": message.into()
    })
}

/// Create a success result with data
pub fn success_with_data(data: Value) -> Value {
    let mut result = json!({"success": true});
    if let (Some(obj), Some(data_obj)) = (result.as_object_mut(), data.as_object()) {
        for (k, v) in data_obj {
            obj.insert(k.clone(), v.clone());
        }
    }
    result
}

/// Create an error result
pub fn error(message: impl Into<String>) -> Value {
    json!({
        "success": false,
        "error": message.into()
    })
}

/// Shorten an agent ID for display (first 8 chars)
pub fn shorten_id(id: &str) -> &str {
    &id[..8.min(id.len())]
}

/// Shorten a path for display
pub fn shorten_path(path: &str, max_len: usize) -> String {
    if path.len() > max_len {
        format!("...{}", &path[path.len() - (max_len - 3)..])
    } else {
        path.to_string()
    }
}

/// Shorten a message for display
pub fn shorten_message(msg: &str, max_len: usize) -> String {
    if msg.len() > max_len {
        format!("{}...", &msg[..max_len - 3])
    } else {
        msg.to_string()
    }
}

/// Validate a required string field from input
pub fn validate_required(input: &Value, field: &str) -> Result<String, Value> {
    let value = input[field].as_str().unwrap_or("");
    if value.is_empty() {
        Err(error(format!("{} is required", field)))
    } else {
        Ok(value.to_string())
    }
}

/// Validate multiple required fields
pub fn validate_required_fields(input: &Value, fields: &[&str]) -> Result<(), Value> {
    for field in fields {
        let value = input[*field].as_str().unwrap_or("");
        if value.is_empty() {
            return Err(error(format!("{} is required", field)));
        }
    }
    Ok(())
}

/// Get optional string field with default
pub fn get_optional_str<'a>(input: &'a Value, field: &str, default: &'a str) -> &'a str {
    input[field].as_str().unwrap_or(default)
}

/// Get optional bool field with default
pub fn get_optional_bool(input: &Value, field: &str, default: bool) -> bool {
    input[field].as_bool().unwrap_or(default)
}

/// Get optional u64 field with default
pub fn get_optional_u64(input: &Value, field: &str, default: u64) -> u64 {
    input[field].as_u64().unwrap_or(default)
}
