use serde_json::Value;

use crate::functions;

pub fn dispatch(function: &str, args: &Value) -> Result<Value, String> {
    match function {
        "add" => functions::add(args),
        "sub" => functions::sub(args),
        "mul" => functions::mul(args),
        "div" => functions::div(args),
        "mod" => functions::modulo(args),
        "pow" => functions::pow(args),
        "sqrt" => functions::sqrt(args),
        _ => Err(format!("unknown function: {function}")),
    }
}
