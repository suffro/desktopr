mod dispatcher;
mod functions;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Read};

const MAX_STDIN_BYTES: usize = 1024 * 1024;

#[derive(Deserialize)]
struct Request {
    #[serde(rename = "fn")]
    function: String,
    #[serde(default)]
    args: Value,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Response {
    Success { ok: bool, value: Value },
    Error { ok: bool, error: String },
}

fn read_request() -> Result<Request, String> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 8192];
    let mut stdin = io::stdin().lock();

    loop {
        let read = stdin
            .read(&mut chunk)
            .map_err(|error| format!("failed to read stdin: {error}"))?;
        if read == 0 {
            break;
        }
        if bytes.len() + read > MAX_STDIN_BYTES {
            return Err(format!("stdin exceeds {MAX_STDIN_BYTES} bytes"));
        }
        bytes.extend_from_slice(&chunk[..read]);
    }

    serde_json::from_slice(&bytes).map_err(|error| format!("invalid request JSON: {error}"))
}

fn main() {
    let response = match read_request() {
        Ok(request) => match dispatcher::dispatch(&request.function, &request.args) {
            Ok(value) => Response::Success { ok: true, value },
            Err(error) => Response::Error { ok: false, error },
        },
        Err(error) => Response::Error { ok: false, error },
    };

    match serde_json::to_string(&response) {
        Ok(json) => println!("{json}"),
        Err(error) => println!(r#"{{"ok":false,"error":"serialization failed: {error}"}}"#),
    }
}
