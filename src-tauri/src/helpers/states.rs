use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct CloseGuard {
    pub closing: AtomicBool,
}

pub struct LatestWindowLabel {
    pub label: Mutex<String>,
}
impl LatestWindowLabel {
    pub fn set(&self, v: impl Into<String>) {
        *self.label.lock().unwrap() = v.into();
    }
}

pub fn get_latest_window_label(app: &AppHandle) -> String {
    let state = app.state::<LatestWindowLabel>();
    let label = state.label.lock().unwrap().clone();
    label
}

// Companion sandbox registry keeps a mapping between a window label and its sandbox root.
// This is used to isolate filesystem operations for companion windows.
pub struct CompanionSandboxRegistry {
    pub map: Mutex<HashMap<String, PathBuf>>,
}

impl CompanionSandboxRegistry {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    // Register or update the sandbox root for a given window label.
    pub fn insert(&self, key: impl Into<String>, path: PathBuf) {
        if let Ok(mut m) = self.map.lock() {
            m.insert(key.into(), path);
        }
    }

    // Remove sandbox information for a given window label.
    pub fn remove(&self, key: &str) {
        if let Ok(mut m) = self.map.lock() {
            m.remove(key);
        }
    }
}

// Convenience helpers to interact with the registry through the Tauri AppHandle.
// These helpers assume that CompanionSandboxRegistry is managed in Tauri state.

// Register sandbox root for a specific window label.
pub fn register_companion_sandbox(app: &AppHandle, window_label: impl Into<String>, path: PathBuf) {
    let registry = app.state::<CompanionSandboxRegistry>();
    registry.insert(window_label, path);
}

// Unregister sandbox for a window label, typically when the window is destroyed.
pub fn unregister_companion_sandbox(app: &AppHandle, window_label: &str) {
    let registry = app.state::<CompanionSandboxRegistry>();
    registry.remove(window_label);
}
