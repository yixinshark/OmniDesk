pub mod display;

use display::DisplayBackend;
use tauri::{Manager, Emitter};
use sysinfo::{System, Networks, Components};
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize)]
struct SysInfoPayload {
    cpu_usage: f32,
    mem_usage: f32,
    swap_usage: f32,
    cpu_temp: f32,
    net_bytes_in: u64,
    net_bytes_out: u64,
}

#[derive(Clone, Serialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    mem_usage: u64,
    status: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct WidgetManifest {
    id: String,
    name: String,
    description: String,
    width: u32,
    height: u32,
    icon: Option<String>,
}

#[derive(Clone, Serialize)]
struct WallpaperInfo {
    filename: String,
    path: String,
    wtype: String, // "static" or "video"
    size: u64,
}

#[tauri::command]
fn save_layout(app_handle: tauri::AppHandle, layout: serde_json::Value) -> Result<(), String> {
    let path = app_handle.path().app_data_dir().unwrap_or_default().join("layout.json");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, serde_json::to_string_pretty(&layout).unwrap())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn load_layout(app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let path = app_handle.path().app_data_dir().unwrap_or_default().join("layout.json");
    if let Ok(content) = std::fs::read_to_string(path) {
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(serde_json::json!(null))
    }
}

#[tauri::command]
fn widget_write_file(app_handle: tauri::AppHandle, filename: String, content: String) -> Result<(), String> {
    let path = app_handle.path().app_data_dir().unwrap_or_default().join("widget_data").join(&filename);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

use tauri_plugin_opener::OpenerExt;

#[tauri::command]
fn widget_read_file(app_handle: tauri::AppHandle, filename: String) -> Result<String, String> {
    let path = app_handle.path().app_data_dir().unwrap_or_default().join("widget_data").join(&filename);
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_url(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    app_handle.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
async fn fetch_proxy(url: String, token: Option<String>) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut req = client.get(&url).header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
    if let Some(t) = token {
        if !t.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
    }
    req.send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_and_cache_image(app_handle: tauri::AppHandle, url: String, filename: String) -> Result<Option<String>, String> {
    let path = app_handle.path().app_data_dir().unwrap_or_default().join("wallpapers").join(&filename);
    if path.exists() {
        return Ok(Some(path.to_string_lossy().to_string()));
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp_path = path.with_extension("tmp");
    if tmp_path.exists() {
        return Ok(None);
    }
    tauri::async_runtime::spawn(async move {
        let _ = std::fs::write(&tmp_path, b"");
        if let Ok(resp) = reqwest::get(&url).await {
            if let Ok(bytes) = resp.bytes().await {
                if std::fs::write(&tmp_path, bytes).is_ok() {
                    let _ = std::fs::rename(&tmp_path, &path);
                } else { let _ = std::fs::remove_file(&tmp_path); }
            } else { let _ = std::fs::remove_file(&tmp_path); }
        } else { let _ = std::fs::remove_file(&tmp_path); }
    });
    Ok(None)
}

#[tauri::command]
async fn check_and_cache_video(app_handle: tauri::AppHandle, url: String, filename: String) -> Result<Option<String>, String> {
    let path = app_handle.path().app_data_dir().unwrap_or_default().join("wallpapers").join(&filename);
    if path.exists() {
        return Ok(Some(path.to_string_lossy().to_string()));
    }

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    let tmp_path = path.with_extension("tmp");
    if tmp_path.exists() {
        return Ok(None);
    }

    tauri::async_runtime::spawn(async move {
        let _ = std::fs::write(&tmp_path, b""); 
        if let Ok(resp) = reqwest::get(&url).await {
            if let Ok(bytes) = resp.bytes().await {
                if std::fs::write(&tmp_path, bytes).is_ok() {
                    let _ = std::fs::rename(&tmp_path, &path);
                } else {
                    let _ = std::fs::remove_file(&tmp_path);
                }
            } else {
                let _ = std::fs::remove_file(&tmp_path);
            }
        } else {
            let _ = std::fs::remove_file(&tmp_path);
        }
    });

    Ok(None)
}

#[tauri::command]
fn list_available_widgets(_app_handle: tauri::AppHandle) -> Result<Vec<WidgetManifest>, String> {
    // Try multiple possible widget directories
    let mut widget_dirs = vec![
        std::path::PathBuf::from("public/widgets"),
        std::path::PathBuf::from("../public/widgets"),
    ];
    // Also check relative to the exe
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            widget_dirs.push(exe_dir.join("public/widgets"));
            widget_dirs.push(exe_dir.join("../public/widgets"));
        }
    }

    let mut widgets = Vec::new();
    for dir in &widget_dirs {
        if !dir.exists() { continue; }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let manifest_path = entry.path().join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<WidgetManifest>(&content) {
                            widgets.push(manifest);
                        }
                    }
                }
            }
        }
        if !widgets.is_empty() { break; }
    }
    Ok(widgets)
}

#[tauri::command]
async fn get_top_processes() -> Result<Vec<ProcessInfo>, String> {
    // 移到后台线程执行，避免阻塞主线程导致 UI 卡顿
    tauri::async_runtime::spawn_blocking(|| {
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        // CPU usage 需要两次采样才有差值，短暂等待
        std::thread::sleep(Duration::from_millis(100));
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let mut processes: Vec<ProcessInfo> = sys.processes()
            .values()
            .map(|p| ProcessInfo {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu_usage: p.cpu_usage(),
                mem_usage: p.memory(),
                status: format!("{:?}", p.status()),
            })
            .collect();

        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        processes.truncate(15);
        Ok(processes)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn kill_process(pid: u32) -> Result<String, String> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
        // Safety: only allow killing user processes (not system-critical)
        let name = process.name().to_string_lossy().to_string();
        let lower = name.to_lowercase();
        if lower == "systemd" || lower == "init" || pid <= 2 {
            return Err("拒绝杀死系统关键进程".to_string());
        }
        if process.kill() {
            Ok(format!("已杀死进程 {} (PID: {})", name, pid))
        } else {
            Err(format!("无法杀死进程 {} (PID: {})", name, pid))
        }
    } else {
        Err(format!("找不到 PID 为 {} 的进程", pid))
    }
}

#[tauri::command]
fn list_wallpapers(app_handle: tauri::AppHandle) -> Result<Vec<WallpaperInfo>, String> {
    let dir = app_handle.path().app_data_dir().unwrap_or_default().join("wallpapers");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut wallpapers = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                let wtype = if ext_str == "mp4" || ext_str == "mov" || ext_str == "webm" {
                    "video"
                } else if ext_str == "jpg" || ext_str == "jpeg" || ext_str == "png" || ext_str == "webp" {
                    "static"
                } else {
                    continue;
                };
                // Skip temp files
                if path.extension().map_or(false, |e| e == "tmp") {
                    continue;
                }
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                wallpapers.push(WallpaperInfo {
                    filename,
                    path: path.to_string_lossy().to_string(),
                    wtype: wtype.to_string(),
                    size,
                });
            }
        }
    }
    // Sort by filename descending (newer dates first)
    wallpapers.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(wallpapers)
}

#[tauri::command]
fn delete_wallpaper(app_handle: tauri::AppHandle, filename: String) -> Result<(), String> {
    let path = app_handle.path().app_data_dir().unwrap_or_default().join("wallpapers").join(&filename);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    // Also delete thumbnail if exists
    let thumb = path.with_extension("jpg");
    if thumb.exists() {
        let _ = std::fs::remove_file(&thumb);
    }
    Ok(())
}

#[tauri::command]
fn get_image_data_url(path: String) -> Result<Option<String>, String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(p).map_err(|e| e.to_string())?;
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("jpg").to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/jpeg",
    };
    let b64 = base64_encode(&bytes);
    Ok(Some(format!("data:{};base64,{}", mime, b64)))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

#[tauri::command]
fn generate_video_thumbnail(app_handle: tauri::AppHandle, filename: String) -> Result<Option<String>, String> {
    let dir = app_handle.path().app_data_dir().unwrap_or_default().join("wallpapers");
    let video_path = dir.join(&filename);
    let thumb_path = dir.join(format!("{}_thumb.jpg", filename.rsplit_once('.').map(|(n, _)| n).unwrap_or(&filename)));

    if thumb_path.exists() {
        return Ok(Some(thumb_path.to_string_lossy().to_string()));
    }

    if !video_path.exists() {
        return Ok(None);
    }

    // Use ffmpeg to extract first frame
    let output = std::process::Command::new("ffmpeg")
        .args(["-i", &video_path.to_string_lossy(), "-vframes", "1", "-q:v", "3", "-y", &thumb_path.to_string_lossy()])
        .output();

    match output {
        Ok(o) if o.status.success() && thumb_path.exists() => {
            Ok(Some(thumb_path.to_string_lossy().to_string()))
        }
        _ => Ok(None),
    }
}

#[tauri::command]
fn read_config(app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let path = app_handle.path().app_data_dir().unwrap_or_default().join("config.json");
    if let Ok(content) = std::fs::read_to_string(path) {
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(serde_json::json!({}))
    }
}

#[tauri::command]
fn write_config(app_handle: tauri::AppHandle, config: serde_json::Value) -> Result<(), String> {
    let path = app_handle.path().app_data_dir().unwrap_or_default().join("config.json");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Fix GBM EGL display initialization error on Linux
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_layout, load_layout, widget_read_file, widget_write_file, open_url, fetch_proxy, check_and_cache_video,
            list_available_widgets, get_top_processes, kill_process, read_config, write_config, check_and_cache_image,
            list_wallpapers, delete_wallpaper, generate_video_thumbnail, get_image_data_url
        ])
        .plugin(tauri_plugin_opener::init())
        .register_uri_scheme_protocol("local-video", |ctx, request| {
            // Extract filename from URL: local-video://localhost/filename
            let url = request.uri().to_string();
            let filename = url.split('/').last().unwrap_or("");
            // Get wallpapers dir from app data
            let app_handle = ctx.app_handle();
            let wallpapers_dir = app_handle.path().app_data_dir().unwrap_or_default().join("wallpapers");
            let file_path = wallpapers_dir.join(filename);
            if file_path.exists() {
                if let Ok(bytes) = std::fs::read(&file_path) {
                    let mime = if filename.ends_with(".mp4") || filename.ends_with(".mov") {
                        "video/mp4"
                    } else if filename.ends_with(".webm") {
                        "video/webm"
                    } else {
                        "application/octet-stream"
                    };
                    return http::Response::builder()
                        .header("Content-Type", mime)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(bytes)
                        .unwrap();
                }
            }
            http::Response::builder()
                .status(404)
                .body(Vec::new())
                .unwrap()
        })
        .setup(|app| {
            if let Some(_window) = app.get_webview_window("main") {
                #[cfg(target_os = "linux")]
                {
                    // 1. 强制铺满全屏，防止 KWin 的 DESKTOP type 忽略 maximize
                    if let Ok(Some(monitor)) = _window.primary_monitor() {
                        let size = monitor.size();
                        let _ = _window.set_size(*size);
                        let _ = _window.set_position(tauri::PhysicalPosition::new(0, 0));
                    }
                    
                    let backend = display::x11::X11Backend;
                    if let Err(e) = backend.mount_to_desktop(&_window) {
                        eprintln!("Failed to mount to desktop: {}", e);
                    }
                }
            }

            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut sys = System::new_all();
                let mut networks = Networks::new_with_refreshed_list();
                let mut components = Components::new_with_refreshed_list();
                sys.refresh_all();
                // 启动时稍作延迟，确保 CPU 测算有基准数据
                std::thread::sleep(Duration::from_millis(500));

                let mut prev_bytes_in: u64 = 0;
                let mut prev_bytes_out: u64 = 0;
                let mut first_run = true;

                loop {
                    sys.refresh_cpu_usage();
                    sys.refresh_memory();
                    networks.refresh(true);
                    components.refresh(true);

                    let cpu_usage = sys.global_cpu_usage();
                    let total_mem = sys.total_memory();
                    let used_mem = sys.used_memory();
                    let mem_usage = if total_mem > 0 {
                        (used_mem as f32 / total_mem as f32) * 100.0
                    } else {
                        0.0
                    };

                    // Swap 使用率
                    let total_swap = sys.total_swap();
                    let used_swap = sys.used_swap();
                    let swap_usage = if total_swap > 0 {
                        (used_swap as f32 / total_swap as f32) * 100.0
                    } else {
                        0.0
                    };

                    // CPU 温度（只取 coretemp/x86_pkg_temp 等 CPU 相关传感器，排除主板杂项传感器）
                    let cpu_temp = components
                        .iter()
                        .filter(|c| {
                            let label = c.label().to_lowercase();
                            let name = label.clone();
                            label.contains("core")
                                || label.contains("cpu")
                                || label.contains("package")
                                || label.contains("tctl")
                                || label.contains("tccd")
                                || name.contains("coretemp")
                                || name.contains("k10temp")
                                || name.contains("x86_pkg_temp")
                        })
                        .filter_map(|c| c.temperature())
                        .filter(|&t| t > 0.0)
                        .fold(0.0_f32, f32::max);

                    // 计算网络速率（差值）
                    let mut total_in: u64 = 0;
                    let mut total_out: u64 = 0;
                    for (_name, data) in &networks {
                        total_in += data.total_received();
                        total_out += data.total_transmitted();
                    }
                    let net_bytes_in = if first_run { 0 } else { total_in.saturating_sub(prev_bytes_in) };
                    let net_bytes_out = if first_run { 0 } else { total_out.saturating_sub(prev_bytes_out) };
                    prev_bytes_in = total_in;
                    prev_bytes_out = total_out;
                    first_run = false;

                    let payload = SysInfoPayload {
                        cpu_usage,
                        mem_usage,
                        swap_usage,
                        cpu_temp,
                        net_bytes_in,
                        net_bytes_out,
                    };

                    // 广播给前端（2秒间隔，避免频繁 emit 导致 UI 卡顿）
                    let _ = app_handle.emit("sysinfo_update", payload);
                    std::thread::sleep(Duration::from_secs(2));
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
