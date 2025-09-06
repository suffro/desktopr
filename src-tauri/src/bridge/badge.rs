// src/bridge/badge.rs
// Platform-specific implementation for app icon badges.
// macOS: Dock badge via NSDockTile.badgeLabel
// Windows: Taskbar overlay icon via ITaskbarList3::SetOverlayIcon
// Linux: Fallback by updating tray icon with generated badge image

use tauri::{AppHandle, Manager};
use std::path::PathBuf;

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

#[tauri::command]
pub fn bd_badge_set(app: AppHandle, count: Option<u32>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        set_badge_macos(count)?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        set_badge_windows(&app, count)?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        set_badge_linux(&app, count)?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Unsupported platform".into())
}

#[tauri::command]
pub fn bd_badge_clear(app: AppHandle) -> Result<(), String> {
    bd_badge_set(app, None)
}

#[cfg(target_os = "macos")]
fn set_badge_macos(count: Option<u32>) -> Result<(), String> {
    // Safety: Interacting with Cocoa APIs requires unsafe
    unsafe {
        use cocoa::appkit::NSApp;
        use cocoa::base::{id, nil};
        use cocoa::foundation::{NSString, NSAutoreleasePool};
        let _pool = NSAutoreleasePool::new(nil);

        let app: id = NSApp();
        let dock_tile: id = msg_send![app, dockTile];

        let label_str: id = match count {
            Some(0) | None => nil,
            Some(c) => NSString::alloc(nil).init_str(&format!("{}", c)),
        };

        // nil clears the badge.
        let _: () = msg_send![dock_tile, setBadgeLabel: label_str];
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_badge_windows(app: &AppHandle, count: Option<u32>) -> Result<(), String> {
    // Strategy:
    // - Build/select a tiny .ico overlay for digits 1..9 and "9+"
    // - Apply with ITaskbarList3::SetOverlayIcon on the *current* window HWND
    // Notes:
    // - This affects each window's taskbar button individually.
    // - You can keep prepared icons under resources/badges/*.ico.
    use windows::{
        Win32::UI::Shell::{ITaskbarList3, TaskbarList, THUMBBUTTON, THUMBBUTTONMASK, THUMBBUTTONFLAGS},
        Win32::UI::WindowsAndMessaging::{HICON, LoadImageW, IMAGE_ICON, LR_DEFAULTSIZE},
        core::PCWSTR,
    };

    // Get the "focused" or primary window. Adjust if you want per-window control.
    let win = app.get_window("main").ok_or("Window not found")?;
    #[allow(unused_imports)]
    use tauri::Manager as _;
    #[cfg(feature = "windows7-compat")]
    use tauri::platform::windows::WindowExtWindows;
    #[cfg(not(feature = "windows7-compat"))]
    use tauri::window::WindowExtWindows; // v2 path (adjust if needed)

    let hwnd = win.hwnd().map_err(|e| e.to_string())?;

    unsafe {
        let mut taskbar: ITaskbarList3 = TaskbarList::new()?;
        taskbar.HrInit()?;

        // Load overlay icon
        let icon: HICON = if let Some(c) = count.filter(|v| *v > 0) {
            let name = if c > 9 { "badge_9plus.ico" } else { &format!("badge_{}.ico", c) };
            let path = resource_badge_icon_path(name)?;
            let w: Vec<u16> = path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
            LoadImageW(None, PCWSTR(w.as_ptr()), IMAGE_ICON, 0, 0, LR_DEFAULTSIZE).ok().unwrap_or_default().0
        } else {
            // null handle clears the overlay
            HICON::default()
        };

        // Apply (null clears)
        taskbar.SetOverlayIcon(hwnd, icon, PCWSTR::null())?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn resource_badge_icon_path(file: &str) -> Result<PathBuf, String> {
    // Put your overlay icons under "<app_dir>/resources/badges/"
    // You can ship them via tauri.conf.json > bundle > resources.
    let base = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = base.parent().ok_or("exe dir not found")?;
    Ok(dir.join("resources").join("badges").join(file))
}

#[cfg(target_os = "linux")]
fn set_badge_linux(app: &AppHandle, count: Option<u32>) -> Result<(), String> {
    // No universal launcher badge API; use tray icon overlay + tooltip as fallback.
    // We'll generate a PNG with the count and set it on the tray.
    use image::{DynamicImage, GenericImage, ImageBuffer, Rgba};
    use tauri::tray::TrayIcon;

    let tray = app.tray_by_id("main").ok_or("Tray icon not found")?;
    if count.unwrap_or(0) == 0 {
        // Restore base icon shipped with the app
        let base = tray_icon_base_path(app)?;
        tray.set_icon(Some(&base)).map_err(|e| e.to_string())?;
        tray.set_tooltip(None).ok();
        return Ok(());
    }

    // Load base icon (PNG), draw red circle + number, save to cache and apply
    let base = tray_icon_base_path(app)?;
    let img = image::open(&base).map_err(|e| e.to_string())?;
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    // Draw a simple red badge circle in bottom-right
    let radius = (w.min(h) / 4).max(10);
    let cx = (w as i32 - radius as i32 - 4) as i32;
    let cy = (h as i32 - radius as i32 - 4) as i32;
    let red = Rgba([220, 38, 38, 255]); // Tailwind red-600-ish

    for y in -radius as i32..=radius as i32 {
        for x in -radius as i32..=radius as i32 {
            if x*x + y*y <= (radius as i32)*(radius as i32) {
                let px = (cx + x) as u32;
                let py = (cy + y) as u32;
                if px < w && py < h {
                    rgba.put_pixel(px, py, red);
                }
            }
        }
    }
    // (Optional) Render number as tiny bitmap: skipped for brevità; puoi sostituire il cerchio con dot

    let out = app
        .path()
        .app_cache_dir()
        .ok_or("no cache dir")?
        .join("tray_badge.png");
    rgba.save(&out).map_err(|e| e.to_string())?;

    tray.set_icon(Some(&out)).map_err(|e| e.to_string())?;
    tray.set_tooltip(Some(format!("{} unread", count.unwrap()))).ok();
    Ok(())
}

#[cfg(target_os = "linux")]
fn tray_icon_base_path(app: &AppHandle) -> Result<PathBuf, String> {
    // Assumi che l’icona base della tray sia in resources/tray.png
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("exe dir not found")?;
    Ok(dir.join("icons").join("icon.png"))
}