use serde_json::Value;

use crate::{function_examples, functions};

pub fn dispatch(function: &str, args: &Value) -> Result<Value, String> {
    match function {
        "ping" => functions::ping(args),
        "dividePos" => function_examples::divide_positional(args),
        "divide" => function_examples::divide(args),
        "greet" => function_examples::greet(args),
        "write" => function_examples::storage_write(args),
        "read" => function_examples::storage_read(args),
        _ => Err(format!("unknown function: {function}")),
    }
}
