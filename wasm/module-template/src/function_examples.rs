use serde_json::{json, Value};
use std::fs;

pub fn storage_write(args: &Value) -> Result<Value, String> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or("test-storage.txt");
    let contents = args
        .get("contents")
        .and_then(Value::as_str)
        .unwrap_or("Hello from Desktopr plugin storage");

    fs::write(path, contents).map_err(|error| error.to_string())?;
    Ok(json!({
        "written": true,
        "path": path,
        "bytes": contents.len()
    }))
}

pub fn storage_read(args: &Value) -> Result<Value, String> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or("test-storage.txt");
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    Ok(json!({
        "read": true,
        "path": path,
        "contents": contents
    }))
}

pub fn divide(args: &Value) -> Result<Value, String> {
    let left = args.get("a").and_then(Value::as_f64).unwrap_or(0.0);
    let right = args.get("b").and_then(Value::as_f64).unwrap_or(0.0);
    if right == 0.0 {
        return Err("division by zero".to_string());
    }
    Ok(json!(left / right))
}

pub fn greet(args: &Value) -> Result<Value, String> {
    let name = args.get("name").and_then(Value::as_str).unwrap_or("world");
    Ok(json!(format!("Hello {name}")))
}

pub fn divide_positional(args: &Value) -> Result<Value, String> {
    let left = args.get(0).and_then(Value::as_f64).unwrap_or(0.0);
    let right = args.get(1).and_then(Value::as_f64).unwrap_or(0.0);
    if right == 0.0 {
        return Err("division by zero".to_string());
    }
    Ok(json!(left / right))
}
