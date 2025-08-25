// src/bridge/deeplink.rs
use serde::Serialize;
use std::str::FromStr;
use tauri::{AppHandle, Emitter, Url}; // <-- importa Emitter (non serve Manager)

#[derive(Debug, Serialize)]
pub struct DeepLinkPayload {
  pub scheme: String,
  pub app: String,
  pub action: String,
  pub path: Vec<String>,
  pub query: Vec<(String, String)>,
  pub raw: String,
  pub error: Option<String>,
}

impl FromStr for DeepLinkPayload {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let u = url::Url::parse(s).map_err(|_| ())?;
    let scheme = u.scheme().to_string();
    let app = u.host_str().unwrap_or_default().to_string();

    let mut segs: Vec<String> = u
      .path_segments()
      .map(|c| c.map(|p| p.to_string()).collect())
      .unwrap_or_default();

    let action = if !segs.is_empty() { segs.remove(0) } else { String::new() };
    let query = u.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();

    Ok(Self {
      scheme, app, action, path: segs, query, raw: s.to_string(), error: None
    })
  }
}

pub fn emit_parsed_deeplink(handle: &AppHandle, url: &str) {
  match DeepLinkPayload::from_str(url) {
    Ok(payload) => {
      // In Tauri v2, AppHandle::emit invia a TUTTE le finestre
      let _ = handle.emit("deeplink", &payload);
    }
    Err(_) => {
      let _ = handle.emit("deeplink", &DeepLinkPayload {
        scheme: String::new(),
        app: String::new(),
        action: String::new(),
        path: vec![],
        query: vec![],
        raw: url.to_string(),
        error: Some("invalid_url".into()),
      });
    }
  }
}
