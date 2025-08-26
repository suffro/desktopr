use tauri::{AppHandle, WebviewUrl};
use tauri::utils::config::Color;

fn parse_color(s: &str) -> Option<Color> {
    serde_json::from_str(&format!("\"{}\"", s)).ok()
}
fn url_or_fallback(u: &WebviewUrl, fallback: &str) -> String {
  match u {
      WebviewUrl::App(path) => {
          let path_str = path.to_string_lossy();
          if path_str.trim().is_empty() {
              fallback.to_string()
          } else {
              path_str.to_string()
          }
      }
      WebviewUrl::External(url) => url.as_str().to_string(),
      _ => fallback.to_string(), // copre varianti future/non documentate
  }
}

#[derive(Clone, Debug)]
pub struct WindowDefaults {
  pub title: String,
  pub url: String,
  pub background_color: Color,
  pub width: f64,
  pub height: f64,
  pub resizable: bool,
  pub decorations: bool,
  pub fullscreen: bool,
  pub always_on_top: bool,
}

pub fn read_main_defaults(app: &AppHandle) -> WindowDefaults {
  // Non esiste più WebviewWindowBuilder::transparent() in Tauri v2.
  // Se la main è trasparente, le nuove window devono essere configurate
  // allo stesso modo nel tauri.conf.json oppure via overlay di build.
  let prod = app.config()
    .product_name
    .clone()
    .unwrap_or_else(|| "Bubbledesk".to_string());

  let bg_color = parse_color("#ffffff").unwrap();

  let url_fallback = "http://blank.html";

  let main = app.config().app.windows.iter().find(|w| w.label == "main");

  if let Some(w) = main {
    let url_str = url_or_fallback(&w.url, url_fallback);
    WindowDefaults {
        title: if w.title.is_empty() {
            prod.clone()
        } else {
            w.title.clone()
        },
        url: url_str.clone(),
        background_color: w.background_color.clone().unwrap_or(bg_color.clone()),
        width: if w.width > 0.0 { w.width } else { 1200.0 },
        height: if w.height > 0.0 { w.height } else { 800.0 },
        resizable: w.resizable,
        decorations: w.decorations,
        fullscreen: w.fullscreen,
        always_on_top: w.always_on_top,
    }
  } else {
      WindowDefaults {
          title: prod.clone(),
          background_color: bg_color.clone(),
          url: url_fallback.clone().to_string(),
          width: 1200.0,
          height: 800.0,
          resizable: true,
          decorations: true,
          fullscreen: false,
          always_on_top: false,
      }
  }
}
