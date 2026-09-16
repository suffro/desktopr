use serde_json::{json, Value};

fn number(args: &Value, key: &str, index: usize, default: f64) -> f64 {
    match args {
        Value::Array(values) => values.get(index).and_then(Value::as_f64).unwrap_or(default),
        Value::Object(values) => values.get(key).and_then(Value::as_f64).unwrap_or(default),
        _ => default,
    }
}

pub fn add(args: &Value) -> Result<Value, String> {
    Ok(json!(number(args, "a", 0, 0.0) + number(args, "b", 1, 0.0)))
}

pub fn sub(args: &Value) -> Result<Value, String> {
    Ok(json!(number(args, "a", 0, 0.0) - number(args, "b", 1, 0.0)))
}

pub fn mul(args: &Value) -> Result<Value, String> {
    Ok(json!(number(args, "a", 0, 0.0) * number(args, "b", 1, 0.0)))
}

pub fn div(args: &Value) -> Result<Value, String> {
    let left = number(args, "a", 0, 0.0);
    let right = number(args, "b", 1, 0.0);
    if right == 0.0 {
        return Err("division by zero".to_string());
    }
    Ok(json!(left / right))
}

pub fn modulo(args: &Value) -> Result<Value, String> {
    let left = number(args, "a", 0, 0.0);
    let right = number(args, "b", 1, 0.0);
    if right == 0.0 {
        return Err("modulo by zero".to_string());
    }
    Ok(json!(left % right))
}

pub fn pow(args: &Value) -> Result<Value, String> {
    let base = number(args, "a", 0, 0.0);
    let exponent = number(args, "b", 1, 0.0);
    Ok(json!(base.powf(exponent)))
}

pub fn sqrt(args: &Value) -> Result<Value, String> {
    let value = number(args, "a", 0, 0.0);
    if value < 0.0 {
        return Err("sqrt of negative number".to_string());
    }
    Ok(json!(value.sqrt()))
}
