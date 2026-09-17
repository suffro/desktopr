use serde_json::Value;
use tauri::{ipc::CapabilityBuilder, AppHandle, Manager};

// The static `remote` capability only covers the `main` window. Windows created
// at runtime get their own copy of it, bound to their label, so a window can
// never inherit permissions meant for another kind of window. The generated
// file already contains the developer's remote origins and optional permissions.
const REMOTE_CAPABILITY: &str = include_str!("../../capabilities/remote.json");

const BRIDGE_PERMISSION: &str = "desktopr-bridge";
const COMPANION_BRIDGE_PERMISSION: &str = "desktopr-bridge-companion";

// Plugin permission prefixes companion windows do not receive: they change
// app-wide state rather than the companion's own sandbox.
const COMPANION_DENIED_PERMISSION_PREFIXES: &[&str] = &["autostart:", "updater:"];

// Debug-only diagnostics test commands (one of them panics the app on purpose).
const DEBUG_BRIDGE_PERMISSION: &str = "desktopr-bridge-debug";

/// Grants the full `remote` capability to a window opened at runtime.
pub fn grant_window_capability(app: &AppHandle, label: &str) -> Result<(), String> {
    let permissions = remote_permissions()?;
    add_capability(app, &format!("remote-window-{label}"), label, permissions)
}

/// Grants the reduced companion capability to a companion window.
pub fn grant_companion_capability(app: &AppHandle, label: &str) -> Result<(), String> {
    let permissions = companion_permissions(remote_permissions()?);
    add_capability(app, &format!("companion-{label}"), label, permissions)
}

fn remote_capability() -> Result<Value, String> {
    serde_json::from_str(REMOTE_CAPABILITY).map_err(|e| format!("invalid remote capability: {e}"))
}

fn remote_permissions() -> Result<Vec<String>, String> {
    let capability = remote_capability()?;
    let permissions = capability
        .get("permissions")
        .and_then(Value::as_array)
        .ok_or("remote capability has no permissions")?;

    permissions
        .iter()
        .map(|p| {
            p.as_str()
                .map(str::to_string)
                .ok_or_else(|| "scoped permissions are not supported in runtime capabilities".to_string())
        })
        .collect()
}

fn companion_permissions(permissions: Vec<String>) -> Vec<String> {
    permissions
        .into_iter()
        .filter(|p| p != DEBUG_BRIDGE_PERMISSION)
        .filter(|p| !COMPANION_DENIED_PERMISSION_PREFIXES.iter().any(|prefix| p.starts_with(prefix)))
        .map(|p| if p == BRIDGE_PERMISSION { COMPANION_BRIDGE_PERMISSION.to_string() } else { p })
        .collect()
}

fn add_capability(app: &AppHandle, identifier: &str, label: &str, permissions: Vec<String>) -> Result<(), String> {
    let capability = remote_capability()?;
    let mut builder = CapabilityBuilder::new(identifier).window(label).webview(label);

    if let Some(urls) = capability.pointer("/remote/urls").and_then(Value::as_array) {
        for url in urls.iter().filter_map(Value::as_str) {
            builder = builder.remote(url.to_string());
        }
    }

    for permission in permissions {
        builder = builder.permission(permission);
    }

    app.add_capability(builder).map_err(|e| format!("failed to grant capability to '{label}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn companion_permissions_drop_app_wide_access() {
        let permissions = vec![
            "core:default".to_string(),
            "desktopr-bridge".to_string(),
            "autostart:allow-enable".to_string(),
            "updater:default".to_string(),
            "desktopr-bridge-debug".to_string(),
            "clipboard-manager:allow-read-text".to_string(),
        ];

        assert_eq!(
            companion_permissions(permissions),
            vec!["core:default", "desktopr-bridge-companion", "clipboard-manager:allow-read-text"]
        );
    }

    #[test]
    fn generated_remote_capability_is_readable() {
        let permissions = remote_permissions().expect("remote capability must parse");
        assert!(permissions.iter().any(|p| p == BRIDGE_PERMISSION));
    }
}
