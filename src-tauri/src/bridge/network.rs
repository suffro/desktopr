// src/bridge/network.rs
use std::net::ToSocketAddrs;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::time::Instant;

// Network probes run natively, outside the webview's CORS rules, against URLs the
// web app chooses. Local and LAN hosts stay reachable by design; requests are
// limited to HTTP(S) and bounded in time, download size and polling rate.
const MAX_TIMEOUT_MS: u64 = 30_000;
const MAX_BANDWIDTH_BYTES: usize = 10 * 1024 * 1024;
const MIN_MONITOR_INTERVAL_MS: u64 = 1_000;
const MAX_MONITOR_TARGETS: usize = 10;

fn validate_url(raw: &str) -> Result<String, String> {
    let url = url::Url::parse(raw).map_err(|e| format!("invalid URL: {e}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(format!("unsupported URL scheme: {}", url.scheme()));
    }
    if url.host_str().is_none() {
        return Err("URL has no host".into());
    }
    Ok(url.into())
}

fn clamp_timeout(timeout_ms: u64) -> u64 {
    timeout_ms.clamp(1, MAX_TIMEOUT_MS)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkStatus {
    pub online: bool,
    pub reason: String,      // "ok" | "dns" | "timeout" | "tls" | "unknown"
    pub latency_ms: Option<u128>,
}

#[tauri::command]
pub async fn dtr_network_get_status(app: AppHandle) -> Result<NetworkStatus, String> {
    // NOTE: Cheap probe to a fast, highly available endpoint.
    // Prefer a HEAD to a CDN endpoint you control; fallback to public.
    probe(&app, Some("https://www.cloudflare.com/cdn-cgi/trace".to_string()), 2500).await
}

#[tauri::command]
pub async fn dtr_network_ping(
    app: AppHandle,
    url: Option<String>,
    timeout_ms: Option<u64>,
) -> Result<serde_json::Value, String> {
    let timeout = clamp_timeout(timeout_ms.unwrap_or(2000));
    let start = Instant::now();
    let status = probe(&app, url, timeout).await?;
    let elapsed = start.elapsed().as_millis() as u64;
    Ok(serde_json::json!({
        "ok": status.online,
        "latencyMs": status.latency_ms.unwrap_or(elapsed as u128)
    }))
}

#[tauri::command]
pub async fn dtr_network_resolve(host: String) -> Result<serde_json::Value, String> {
    // NOTE: Use system resolver via ToSocketAddrs; for more control use trust-dns-resolver.
    let addrs: Vec<String> = (host.as_str(), 443)
        .to_socket_addrs()
        .map_err(|e| format!("DNS error: {e}"))?
        .map(|a| a.ip().to_string())
        .collect();
    Ok(serde_json::json!({ "addresses": addrs }))
}

#[tauri::command]
pub async fn dtr_network_bandwidth_estimate(
    app: AppHandle,
    url: Option<String>,
    size_hint_bytes: Option<u64>,
    timeout_ms: Option<u64>,
) -> Result<serde_json::Value, String> {
    // NOTE: Download a small static file to estimate throughput.
    let url = validate_url(&url.unwrap_or_else(|| "https://speed.cloudflare.com/__down?bytes=200000".to_string()))?;
    let timeout = clamp_timeout(timeout_ms.unwrap_or(4000));
    let start = Instant::now();
    let bytes_len = download_sample(&url, timeout, MAX_BANDWIDTH_BYTES).await? as u64;
    let ms = start.elapsed().as_millis().max(1) as u64;
    // kbps = (bytes * 8) / ms
    let kbps = ((bytes_len * 8) as f64) / (ms as f64);
    Ok(serde_json::json!({ "kbps": kbps, "bytes": bytes_len, "ms": ms }))
}

// Downloads at most `max_bytes` (plus the final chunk) and returns the bytes read.
async fn download_sample(url: &str, timeout_ms: u64, max_bytes: usize) -> Result<usize, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| e.to_string())?;
    let mut resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    let mut received: usize = 0;
    while received < max_bytes {
        match resp.chunk().await.map_err(|e| e.to_string())? {
            Some(chunk) => received += chunk.len(),
            None => break,
        }
    }
    Ok(received)
}

async fn probe(app: &AppHandle, url: Option<String>, timeout_ms: u64) -> Result<NetworkStatus, String> {
    // NOTE: First do a fast DNS check to discriminate reasons.
    let dns_ok = ("one.one.one.one", 443).to_socket_addrs().is_ok();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(clamp_timeout(timeout_ms)))
        .build()
        .map_err(|e| e.to_string())?;
    let target = validate_url(&url.unwrap_or_else(|| "https://www.google.com/generate_204".to_string()))?;

    let start = Instant::now();
    let res = client.head(&target).send().await;
    let status = match res {
        Ok(r) => {
            if r.status().is_success() || r.status().as_u16() == 204 {
                let latency = start.elapsed().as_millis();
                NetworkStatus { online: true, reason: "ok".into(), latency_ms: Some(latency) }
            } else {
                NetworkStatus { online: false, reason: "unknown".into(), latency_ms: None }
            }
        }
        Err(e) if e.is_timeout() => NetworkStatus { online: false, reason: "timeout".into(), latency_ms: None },
        Err(_e) => {
            let reason = if !dns_ok { "dns" } else { "unknown" };
            NetworkStatus { online: false, reason: reason.into(), latency_ms: None }
        }
    };

    // Emit a status event for listeners
    let _ = app.emit("network:status", &status);
    Ok(status)
}

// --- Optional: simple monitor (interval polling) ---
use std::sync::{Mutex, OnceLock};
use tokio::task::JoinHandle;

struct MonitorState {
    handle: Option<JoinHandle<()>>,
}

static MONITOR: OnceLock<Mutex<MonitorState>> = OnceLock::new();

#[tauri::command]
pub async fn dtr_network_set_monitor(app: AppHandle, interval_ms: u64, targets: Option<Vec<String>>) -> Result<(), String> {
    // NOTE: Simple polling monitor; for OS-level callbacks use platform-specific crates.
    let app_handle = app.clone();
    let tgts = targets.unwrap_or_else(|| vec![
        "https://www.google.com/generate_204".to_string(),
        "https://www.cloudflare.com/cdn-cgi/trace".to_string()
    ]);
    if tgts.len() > MAX_MONITOR_TARGETS {
        return Err(format!("at most {MAX_MONITOR_TARGETS} monitor targets are allowed"));
    }
    let tgts = tgts.iter().map(|t| validate_url(t)).collect::<Result<Vec<_>, _>>()?;
    let interval_ms = interval_ms.max(MIN_MONITOR_INTERVAL_MS);

    let handle = tokio::spawn(async move {
        loop {
            for t in &tgts {
                let _ = probe(&app_handle, Some(t.clone()), 2500).await;
            }
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
        }
    });

    let state = MONITOR.get_or_init(|| Mutex::new(MonitorState { handle: None }));
    let mut s = state.lock().unwrap();
    if let Some(old) = s.handle.take() { old.abort(); }
    s.handle = Some(handle);
    Ok(())
}

#[tauri::command]
pub async fn dtr_network_stop_monitor() -> Result<(), String> {
    if let Some(state) = MONITOR.get() {
        let mut s = state.lock().unwrap();
        if let Some(h) = s.handle.take() { h.abort(); }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_https_including_local_hosts() {
        assert!(validate_url("https://example.com/health").is_ok());
        assert!(validate_url("http://localhost:8080/").is_ok());
        assert!(validate_url("http://192.168.1.20/status").is_ok());
    }

    #[test]
    fn rejects_other_schemes_and_malformed_urls() {
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("not a url").is_err());
    }

    #[test]
    fn download_sample_stops_at_the_size_cap() {
        use std::io::{Read, Write};

        // Serve an endless body without Content-Length until the client disconnects.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n");
            let block = vec![0u8; 64 * 1024];
            for _ in 0..(64 * 1024 * 1024 / block.len()) {
                if stream.write_all(&block).is_err() {
                    break;
                }
            }
        });

        let cap = 1024 * 1024;
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let received = runtime
            .block_on(download_sample(&format!("http://{addr}/"), 10_000, cap))
            .unwrap();

        assert!(received >= cap, "read {received} bytes");
        assert!(received < cap + 1024 * 1024, "did not stop near the cap: {received} bytes");
    }

    #[test]
    fn clamps_timeouts() {
        assert_eq!(clamp_timeout(0), 1);
        assert_eq!(clamp_timeout(2_000), 2_000);
        assert_eq!(clamp_timeout(600_000), MAX_TIMEOUT_MS);
    }
}
