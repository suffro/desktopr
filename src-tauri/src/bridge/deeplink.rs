// src/bridge/deeplink.rs
use serde::Serialize;
use std::str::FromStr;
use tauri::Manager;
use tauri::Url;

#[derive(Debug, Serialize)]
pub struct DeepLinkPayload {
  pub scheme: String,      // es: "bubbledesk" (o "bdesk-myapp")
  pub app: String,         // appSlug (vuoto con schema fisso)
  pub action: String,      // primo segmento del path
  pub path: Vec<String>,   // altri segmenti
  pub query: Vec<(String, String)>,
  pub raw: String,
  pub error: Option<String>,
}

impl FromStr for DeepLinkPayload {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let url = url::Url::parse(s).map_err(|_| ())?;
    let scheme = url.scheme().to_string();

    // Se stai usando schema fisso "bubbledesk", usa host come appSlug (consigliato)
    // Esempio URL: bubbledesk://my-app/auth/callback?code=...
    let app = url.host_str().unwrap_or_default().to_string();

    let mut segs: Vec<String> = url
      .path_segments()
      .map(|c| c.map(|p| p.to_string()).collect())
      .unwrap_or_else(Vec::new);
    let action = if !segs.is_empty() { segs.remove(0) } else { String::new() };

    let query = url.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();

    Ok(Self {
      scheme,
      app,
      action,
      path: segs,
      query,
      raw: s.to_string(),
      error: None,
    })
  }
}

pub fn emit_parsed_deeplink<H: tauri::Manager>(h: H, url: &str) {
  match DeepLinkPayload::from_str(url) {
    Ok(payload) => { let _ = h.emit_all("deeplink", &payload); }
    Err(_) => {
      let _ = h.emit_all("deeplink", &DeepLinkPayload {
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
