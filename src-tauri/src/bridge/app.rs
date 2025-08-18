use tauri::AppHandle;
use serde::Serialize;

#[derive(Serialize)]
pub struct AppInfo { pub name: String, pub version: String, pub platform: String, pub arch: String }

#[tauri::command]
pub fn bd_app_info(app: AppHandle) -> Result<AppInfo, String> {
  let pkg = app.package_info();
  Ok(AppInfo {
    name: pkg.name.clone(),
    version: pkg.version.to_string(),
    platform: std::env::consts::OS.into(),
    arch: std::env::consts::ARCH.into()
  })
}
