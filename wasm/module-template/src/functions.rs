use serde_json::{json, Value};

pub fn ping(_args: &Value) -> Result<Value, String> {
    Ok(json!({ "pong": true }))
}
