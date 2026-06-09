pub mod display;

use display::DisplayBackend;
use tauri::{Manager, Emitter};
use sysinfo::{System, Networks, Components, Disks};
use std::time::Duration;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone, Serialize)]
struct DiskUsagePayload {
    name: String,
    mount_point: String,
    total_space: u64,
    available_space: u64,
    used_space: u64,
}

#[derive(Clone, Serialize)]
struct DiskIoPayload {
    name: String,
    read_bytes_per_sec: u64,
    write_bytes_per_sec: u64,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AiKeys {
    pub deepseek: String,
    pub anyrouter: String,
}

#[derive(Serialize, Clone)]
pub struct AiQuotaPayload {
    pub deepseek_balance: Option<f64>,
    pub anyrouter_used: Option<f64>,
    pub anyrouter_total: Option<f64>,
    pub error: Option<String>,
}

#[tauri::command]
fn save_ai_keys(deepseek: String, anyrouter: String) -> Result<(), String> {
    let keys = AiKeys { deepseek, anyrouter };
    let path = dirs_next::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".omnidesk")
        .join("api_keys.json");
        
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string(&keys).unwrap();
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_ai_keys() -> Result<AiKeys, String> {
    let path = dirs_next::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".omnidesk")
        .join("api_keys.json");
        
    if path.exists() {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        if let Ok(keys) = serde_json::from_str::<AiKeys>(&content) {
            return Ok(keys);
        }
    }
    Ok(AiKeys::default())
}

#[tauri::command]
async fn fetch_ai_quota() -> Result<AiQuotaPayload, String> {
    let keys = get_ai_keys().unwrap_or_default();
    let mut payload = AiQuotaPayload {
        deepseek_balance: None,
        anyrouter_used: None,
        anyrouter_total: None,
        error: None,
    };
    
    let client = reqwest::Client::new();
    
    // Fetch DeepSeek
    if !keys.deepseek.is_empty() {
        if let Ok(resp) = client.get("https://api.deepseek.com/user/balance")
            .bearer_auth(&keys.deepseek)
            .send().await 
        {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(infos) = json.get("balance_infos").and_then(|i| i.as_array()) {
                    if let Some(info) = infos.first() {
                        if let Some(bal_str) = info.get("total_balance").and_then(|v| v.as_str()) {
                            payload.deepseek_balance = bal_str.parse::<f64>().ok();
                        } else if let Some(bal_num) = info.get("total_balance").and_then(|v| v.as_f64()) {
                            payload.deepseek_balance = Some(bal_num);
                        }
                    }
                }
            }
        }
    }

    // Fetch AnyRouter
    if !keys.anyrouter.is_empty() {
        if let Ok(resp) = client.get("https://anyrouter.top/api/user/self")
            .header("Authorization", format!("Bearer {}", keys.anyrouter))
            .send().await
        {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(data) = json.get("data") {
                    if let (Some(quota), Some(used)) = (data.get("quota").and_then(|v| v.as_f64()), data.get("used_quota").and_then(|v| v.as_f64())) {
                        payload.anyrouter_total = Some(quota / 500000.0);
                        payload.anyrouter_used = Some(used / 500000.0);
                    } else if let Some(quota) = data.get("quota").and_then(|v| v.as_f64()) {
                         payload.anyrouter_total = Some(quota / 500000.0);
                         payload.anyrouter_used = Some(0.0);
                    }
                }
            }
        }
    }
    
    Ok(payload)
}

#[derive(Clone, Serialize)]
struct SysInfoPayload {
    cpu_usage: f32,
    mem_usage: f32,
    swap_usage: f32,
    cpu_temp: f32,
    net_bytes_in: u64,
    net_bytes_out: u64,
    disks: Vec<DiskUsagePayload>,
    disk_io: Vec<DiskIoPayload>,
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
fn open_ssh(host: String, user: String, port: u16, password: Option<String>, terminal: Option<String>) -> Result<(), String> {
    // 构建 SSH 命令
    let ssh_args = if port == 22 {
        format!("ssh {}@{}", user, host)
    } else {
        format!("ssh -p {} {}@{}", port, user, host)
    };

    // 如果有密码，尝试用 sshpass
    let full_cmd = if let Some(ref pw) = password {
        if !pw.is_empty() {
            format!("sshpass -p '{}' {}", pw.replace('\'', "'\\''"), ssh_args)
        } else {
            ssh_args
        }
    } else {
        ssh_args
    };

    // 用户自定义终端
    if let Some(ref term) = terminal {
        if !term.is_empty() {
            let args = vec!["-e", "bash", "-c", &full_cmd];
            return std::process::Command::new(term)
                .args(&args)
                .spawn()
                .map(|_| ())
                .map_err(|e| format!("启动 {} 失败: {}", term, e));
        }
    }

    // 自动检测终端
    let terminals: Vec<(&str, Vec<&str>)> = vec![
        ("deepin-terminal", vec!["-e", "bash", "-c", &full_cmd]),
        ("wezterm", vec!["start", "--", "bash", "-c", &full_cmd]),
        ("gnome-terminal", vec!["--", "bash", "-c", &full_cmd]),
        ("konsole", vec!["-e", "bash", "-c", &full_cmd]),
        ("xfce4-terminal", vec!["-e", &full_cmd]),
        ("tilix", vec!["-e", &full_cmd]),
        ("mate-terminal", vec!["-e", &full_cmd]),
        ("alacritty", vec!["-e", "bash", "-c", &full_cmd]),
        ("kitty", vec!["-e", "bash", "-c", &full_cmd]),
        ("xterm", vec!["-e", &full_cmd]),
    ];

    for (term, args) in &terminals {
        if std::process::Command::new(term).args(args).spawn().is_ok() {
            return Ok(());
        }
    }

    Err("未找到可用的终端模拟器，请在设置中配置终端路径".to_string())
}

#[tauri::command]
async fn fetch_proxy(url: String, token: Option<String>, headers: Option<std::collections::HashMap<String, String>>) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut req = client.get(&url).header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
    if let Some(t) = token {
        if !t.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
    }
    if let Some(h) = headers {
        for (k, v) in h {
            req = req.header(&k, &v);
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
    // 废弃本地视频缓存机制：
    // Linux WebKitGTK 无法可靠地播放带有自定义协议或 asset:// 的本地视频。
    // 为了防止后台下载任务占用带宽导致前端在线流媒体卡顿/断流，我们在这里直接返回 None 并且不发起下载。
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

#[tauri::command]
fn send_notification(app_handle: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app_handle.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// --- XDG 桌面文件 / 应用启动器 ---

#[derive(Serialize)]
struct DesktopItem {
    name: String,         // 显示名（locale 优先）
    kind: String,         // "app" | "file" | "dir"
    path: String,         // 真实绝对路径（软链已解析）
    icon: Option<String>, // data URL（svg/png base64），无则前端用 emoji 兜底
}

/// 当前图标主题：优先 DDE，再 GNOME，兜底 hicolor
fn current_icon_theme() -> String {
    let try_gsettings = |schema: &str, key: &str| -> Option<String> {
        let out = std::process::Command::new("gsettings")
            .args(["get", schema, key])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&out.stdout);
        let s = s.trim().trim_matches('\'').trim_matches('"').trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    };
    try_gsettings("com.deepin.dde.appearance", "icon-theme")
        .or_else(|| try_gsettings("org.gnome.desktop.interface", "icon-theme"))
        .unwrap_or_else(|| "hicolor".to_string())
}

/// 从 LANG / LC_MESSAGES 推断 locale 键，如 "zh_CN.UTF-8" -> ["zh_CN", "zh"]
fn current_locale_keys() -> Vec<String> {
    let lang = std::env::var("LC_MESSAGES")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    let lang = lang.split('.').next().unwrap_or("").to_string();
    let mut keys = Vec::new();
    if !lang.is_empty() {
        keys.push(lang.clone());
        if let Some((base, _)) = lang.split_once('_') {
            keys.push(base.to_string());
        }
    }
    keys
}

/// 轻量解析 .desktop：只读 [Desktop Entry] 组，遇到下一个 [ 组即停（文件常有数百行翻译）。
/// 返回 (显示名, 图标名)。
fn parse_desktop_entry(path: &std::path::Path, locale_keys: &[String]) -> Option<(String, String)> {
    let file = std::fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);

    let mut in_entry = false;
    let mut name_default = String::new();
    let mut name_localized: HashMap<String, String> = HashMap::new();
    let mut icon = String::new();

    for line in reader.lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if trimmed == "[Desktop Entry]" {
                in_entry = true;
                continue;
            } else if in_entry {
                break; // 进入下一个组（如 [Desktop Action ...]），停止
            } else {
                continue;
            }
        }
        if !in_entry || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            if key == "Name" {
                name_default = val.to_string();
            } else if key.starts_with("Name[") && key.ends_with(']') {
                let loc = &key[5..key.len() - 1];
                name_localized.insert(loc.to_string(), val.to_string());
            } else if key == "Icon" {
                icon = val.to_string();
            }
        }
    }

    let name = locale_keys
        .iter()
        .find_map(|k| name_localized.get(k).cloned())
        .filter(|s| !s.is_empty())
        .unwrap_or(name_default);
    Some((name, icon))
}

/// 普通文件按扩展名映射到通用 MIME 图标名（freedesktop 命名）
fn mime_icon_for(path: &std::path::Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let by_ext = match ext.as_str() {
        "txt" | "md" | "log" | "csv" | "json" | "xml" | "yaml" | "yml" | "rs" | "c" | "h"
        | "cpp" | "py" | "js" | "ts" | "sh" | "toml" | "ini" | "conf" | "lua" => Some("text-x-generic"),
        "pdf" => Some("application-pdf"),
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" => Some("image-x-generic"),
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "flv" => Some("video-x-generic"),
        "mp3" | "wav" | "flac" | "ogg" | "aac" => Some("audio-x-generic"),
        "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" | "rar" | "deb" | "rpm" => Some("package-x-generic"),
        "doc" | "docx" | "odt" => Some("x-office-document"),
        "xls" | "xlsx" | "ods" => Some("x-office-spreadsheet"),
        "ppt" | "pptx" | "odp" => Some("x-office-presentation"),
        _ => None,
    };
    if let Some(n) = by_ext {
        return n.to_string();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.is_file() && meta.permissions().mode() & 0o111 != 0 {
                return "application-x-executable".to_string();
            }
        }
    }
    "text-x-generic".to_string()
}

/// 图标名 -> data URL。绝对路径直接读；否则按主题 + hicolor 回退解析。
fn resolve_icon(icon: &str, theme: &str) -> Option<String> {
    if icon.is_empty() {
        return None;
    }
    let icon_path = if icon.starts_with('/') {
        let p = std::path::PathBuf::from(icon);
        if p.exists() { Some(p) } else { None }
    } else {
        freedesktop_icons::lookup(icon)
            .with_size(48)
            .with_theme(theme)
            .find()
            .or_else(|| freedesktop_icons::lookup(icon).with_size(48).find())
    };
    let p = icon_path?;
    let bytes = std::fs::read(&p).ok()?;
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let mime = match ext.as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "xpm" => "image/x-xpixmap",
        _ => "image/png",
    };
    Some(format!("data:{};base64,{}", mime, base64_encode(&bytes)))
}

#[tauri::command]
fn list_desktop_entries() -> Result<Vec<DesktopItem>, String> {
    let desktop = dirs_next::home_dir()
        .ok_or("无法获取 home 目录")?
        .join("Desktop");
    if !desktop.exists() {
        return Ok(Vec::new());
    }

    let theme = current_icon_theme();
    let locale_keys = current_locale_keys();
    let mut items = Vec::new();

    for entry in std::fs::read_dir(&desktop).map_err(|e| e.to_string())? {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        let link_path = entry.path();
        let file_name = link_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if file_name.starts_with('.') {
            continue; // 跳过隐藏文件
        }

        // 解析软链到真实路径（~/Desktop 多为指向 /usr/share/applications 的软链）
        let real = std::fs::canonicalize(&link_path).unwrap_or_else(|_| link_path.clone());
        let is_dir = real.is_dir();
        let is_desktop = real.extension().map(|e| e == "desktop").unwrap_or(false);

        let (name, kind, icon_name): (String, &str, Option<String>) = if is_desktop {
            let parsed = parse_desktop_entry(&real, &locale_keys);
            let nm = parsed
                .as_ref()
                .map(|(n, _)| n.clone())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    real.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| file_name.clone())
                });
            let ic = parsed
                .and_then(|(_, i)| if i.is_empty() { None } else { Some(i) });
            (nm, "app", ic)
        } else if is_dir {
            (file_name.clone(), "dir", Some("folder".to_string()))
        } else {
            (file_name.clone(), "file", Some(mime_icon_for(&real)))
        };

        let icon = icon_name.and_then(|n| resolve_icon(&n, &theme));
        items.push(DesktopItem {
            name,
            kind: kind.to_string(),
            path: real.to_string_lossy().to_string(),
            icon,
        });
    }

    // 排序：文件夹 > 应用 > 文件，组内按名称
    items.sort_by(|a, b| {
        let rank = |k: &str| match k { "dir" => 0, "app" => 1, _ => 2 };
        rank(a.kind.as_str())
            .cmp(&rank(b.kind.as_str()))
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(items)
}

#[tauri::command]
fn launch_desktop_entry(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    let is_desktop = p.extension().map(|e| e == "desktop").unwrap_or(false);

    if is_desktop {
        // gio launch 会正确处理 Exec 字段码(%F/%U)、Terminal=、DBus 激活、URI(computer:///)
        if std::process::Command::new("gio")
            .args(["launch", path.as_str()])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        // 回退：gtk-launch <app-id>
        if let Some(id) = p.file_stem().map(|s| s.to_string_lossy().to_string()) {
            if !id.is_empty()
                && std::process::Command::new("gtk-launch").arg(&id).spawn().is_ok()
            {
                return Ok(());
            }
        }
        return Err(format!("无法启动应用：{}", path));
    }

    // 普通文件 / 文件夹：交给默认程序
    std::process::Command::new("xdg-open")
        .arg(&path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("打开失败：{}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Fix GBM EGL display initialization error on Linux
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_layout, load_layout, widget_read_file, widget_write_file, open_url, open_ssh, fetch_proxy, check_and_cache_video,
            list_available_widgets, get_top_processes, kill_process, read_config, write_config, check_and_cache_image,
            list_wallpapers, delete_wallpaper, generate_video_thumbnail, get_image_data_url,
            save_ai_keys, get_ai_keys, fetch_ai_quota, send_notification,
            list_desktop_entries, launch_desktop_entry
        ])
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .register_uri_scheme_protocol("local-video", |ctx, request| {
            // Extract filename from URL: local-video://localhost/filename
            let url = request.uri().to_string();
            let filename = url.split('/').last().unwrap_or("");
            let app_handle = ctx.app_handle();
            let wallpapers_dir = app_handle.path().app_data_dir().unwrap_or_default().join("wallpapers");
            let file_path = wallpapers_dir.join(filename);

            if !file_path.exists() {
                return http::Response::builder()
                    .status(404)
                    .body(Vec::new())
                    .unwrap();
            }

            let file_size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
            let mime = if filename.ends_with(".mp4") || filename.ends_with(".mov") {
                "video/mp4"
            } else if filename.ends_with(".webm") {
                "video/webm"
            } else {
                "application/octet-stream"
            };

            // 解析 Range 请求头（WebKitGTK 播放视频必须支持 Range）
            let range_header = request.headers().get("Range").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();

            if range_header.starts_with("bytes=") {
                let range_spec = &range_header[6..];
                let parts: Vec<&str> = range_spec.split('-').collect();
                let start: u64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                let end: u64 = if parts.len() > 1 && !parts[1].is_empty() {
                    parts[1].parse().unwrap_or(file_size - 1)
                } else {
                    // 不要一次性读太大的块，限制为 2MB
                    std::cmp::min(start + 2 * 1024 * 1024 - 1, file_size - 1)
                };
                let end = std::cmp::min(end, file_size - 1);
                let length = end - start + 1;

                use std::io::{Seek, Read};
                let mut file = match std::fs::File::open(&file_path) {
                    Ok(f) => f,
                    Err(_) => return http::Response::builder().status(500).body(Vec::new()).unwrap(),
                };
                let _ = file.seek(std::io::SeekFrom::Start(start));
                let mut buf = vec![0u8; length as usize];
                let _ = file.read_exact(&mut buf);

                return http::Response::builder()
                    .status(206)
                    .header("Content-Type", mime)
                    .header("Content-Length", length.to_string())
                    .header("Content-Range", format!("bytes {}-{}/{}", start, end, file_size))
                    .header("Accept-Ranges", "bytes")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(buf)
                    .unwrap();
            }

            // 非 Range 请求：返回完整文件（但也声明支持 Range）
            match std::fs::read(&file_path) {
                Ok(bytes) => {
                    http::Response::builder()
                        .status(200)
                        .header("Content-Type", mime)
                        .header("Content-Length", file_size.to_string())
                        .header("Accept-Ranges", "bytes")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(bytes)
                        .unwrap()
                }
                Err(_) => {
                    http::Response::builder()
                        .status(500)
                        .body(Vec::new())
                        .unwrap()
                }
            }
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
                let mut disks = Disks::new_with_refreshed_list();
                sys.refresh_all();
                // 启动时稍作延迟，确保 CPU 测算有基准数据
                std::thread::sleep(Duration::from_millis(500));

                let mut prev_bytes_in: u64 = 0;
                let mut prev_bytes_out: u64 = 0;
                let mut prev_disk_io: HashMap<String, (u64, u64)> = HashMap::new(); // sectors read, written
                let mut first_run = true;

                loop {
                    sys.refresh_cpu_usage();
                    sys.refresh_memory();
                    networks.refresh(true);
                    components.refresh(true);
                    disks.refresh(true);

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
                    let mut disk_usage = Vec::new();
                    for disk in &disks {
                        let mount_point = disk.mount_point().to_string_lossy().to_string();
                        // 过滤掉 snap, loop, boot 等非用户数据盘
                        if mount_point.starts_with("/snap/") || mount_point.starts_with("/run/") || mount_point.starts_with("/sys/") || mount_point.starts_with("/dev/") || mount_point.starts_with("/boot") || disk.is_removable() {
                            continue;
                        }
                        
                        let mut total_space = disk.total_space();
                        let mut available_space = disk.available_space();
                        let mut used_space = total_space.saturating_sub(available_space);
                        
                        // 使用 libc::statvfs 获取最精确的文件系统块数据（排除 Linux ext4 预留给 root 的 5% 空间的影响）
                        let c_mount_point = std::ffi::CString::new(mount_point.clone()).unwrap_or_default();
                        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
                        if unsafe { libc::statvfs(c_mount_point.as_ptr(), &mut stat) } == 0 {
                            total_space = (stat.f_blocks as u64) * (stat.f_frsize as u64);
                            let free_space = (stat.f_bfree as u64) * (stat.f_frsize as u64);
                            available_space = (stat.f_bavail as u64) * (stat.f_frsize as u64);
                            used_space = total_space.saturating_sub(free_space);
                        }

                        disk_usage.push(DiskUsagePayload {
                            name: disk.name().to_string_lossy().to_string(),
                            mount_point,
                            total_space,
                            available_space,
                            used_space,
                        });
                    }

                    let mut disk_io = Vec::new();
                    if let Ok(file) = File::open("/proc/diskstats") {
                        let reader = BufReader::new(file);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 14 {
                                    let name = parts[2].to_string();
                                    // 仅统计物理设备如 nvme, sd 等，忽略 loop 和 ram
                                    if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("sr") {
                                        continue;
                                    }
                                    if let (Ok(sectors_read), Ok(sectors_written)) = (parts[5].parse::<u64>(), parts[9].parse::<u64>()) {
                                        let prev = prev_disk_io.entry(name.clone()).or_insert((sectors_read, sectors_written));
                                        
                                        let read_bytes = if first_run { 0 } else { sectors_read.saturating_sub(prev.0) * 512 };
                                        let write_bytes = if first_run { 0 } else { sectors_written.saturating_sub(prev.1) * 512 };
                                        
                                        *prev = (sectors_read, sectors_written);

                                        // 过滤掉子分区 (如 nvme0n1p1), 只记录主磁盘设备的 IO 以防止重复计算
                                        if !name.chars().last().unwrap_or('a').is_digit(10) || name.contains("nvme") && !name.contains("p") {
                                            disk_io.push(DiskIoPayload {
                                                name: name.clone(),
                                                read_bytes_per_sec: read_bytes / 2, // 2秒的间隔
                                                write_bytes_per_sec: write_bytes / 2,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }

                    first_run = false;

                    let payload = SysInfoPayload {
                        cpu_usage,
                        mem_usage,
                        swap_usage,
                        cpu_temp,
                        net_bytes_in,
                        net_bytes_out,
                        disks: disk_usage,
                        disk_io,
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
