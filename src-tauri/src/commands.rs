use crate::registry::{scan_installed_programs, InstalledProgram};
use std::process::Command;
use serde_json::Value;
use walkdir::WalkDir;
use std::path::Path;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[tauri::command]
pub async fn list_programs() -> Vec<InstalledProgram> {
    let mut programs = scan_installed_programs();
    
    let (pip, npm, choco, scoop) = std::thread::spawn(|| {
        (scan_pip_packages(), scan_npm_packages(), scan_choco_packages(), scan_scoop_packages())
    }).join().unwrap_or_default();

    programs.extend(pip);
    programs.extend(npm);
    programs.extend(choco);
    programs.extend(scoop);
    
    programs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    programs
}

#[tauri::command]
pub fn open_url(url: String) {
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "start", "", &url]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let _ = cmd.spawn();
}

fn get_dir_size(path: &str) -> f64 {
    let mut total_size = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() { total_size += metadata.len(); }
        }
    }
    total_size as f64 / 1_048_576.0
}

fn run_silent_cmd(args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new("cmd");
    cmd.args(["/C"]).args(args);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output()
}

// --- UNIFIED PYTHON FINDER ---
fn find_python_executable() -> Option<String> {
    // 1. Try standard commands
    for exe in &["py", "python", "python3"] {
        let mut cmd = Command::new(exe);
        cmd.arg("--version");
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                return Some(exe.to_string());
            }
        }
    }

    // 2. Try standard installation paths (AppData & Program Files)
    let mut search_paths = Vec::new();
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        for ver in &["313", "312", "311", "310", "39"] {
            search_paths.push(format!("{}\\Programs\\Python\\Python{}\\python.exe", local_app_data, ver));
        }
        search_paths.push(format!("{}\\Microsoft\\WindowsApps\\python.exe", local_app_data));
    }
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        for ver in &["313", "312", "311", "310", "39"] {
            search_paths.push(format!("{}\\Python{}\\python.exe", program_files, ver));
        }
    }

    for path in search_paths {
        if Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

// --- PIP SCANNER ---
fn scan_pip_packages() -> Vec<InstalledProgram> {
    let mut packages = Vec::new();
    let python_exe = match find_python_executable() {
        Some(exe) => exe,
        None => return packages, // Python not found, return empty list
    };

    let script = r#"
import importlib.metadata, json
res = []
for dist in importlib.metadata.distributions():
    try:
        size = 0
        for f in dist.files or []:
            try:
                p = f.locate()
                if p and p.is_file(): size += p.stat().st_size
            except: pass
        res.append({"name": dist.metadata["Name"], "version": dist.version, "size": size})
    except: pass
print(json.dumps(res))
"#;

    let mut cmd = Command::new(&python_exe);
    cmd.args(["-c", script]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let json_str = String::from_utf8_lossy(&out.stdout);
            if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(&json_str) {
                for item in arr {
                    if let (Some(name), Some(version), Some(size)) = (item["name"].as_str(), item["version"].as_str(), item["size"].as_f64()) {
                        packages.push(InstalledProgram {
                            name: name.to_string(),
                            version: version.to_string(),
                            publisher: "PyPI (Python)".into(),
                            install_location: python_exe.clone(), // Save python path so we can uninstall later!
                            uninstall_string: format!("{} -m pip uninstall -y {}", python_exe, name),
                            registry_view: "pip".into(),
                            icon_path: "".into(),
                            source: "pip".into(),
                            size_mb: size / 1_048_576.0,
                        });
                    }
                }
            }
        }
    }
    packages
}

// --- NPM SCANNER ---
fn scan_npm_packages() -> Vec<InstalledProgram> {
    let mut packages = Vec::new();
    let mut npm_root = String::new();
    
    if let Ok(out) = run_silent_cmd(&["npm", "root", "-g"]) {
        npm_root = String::from_utf8_lossy(&out.stdout).trim().to_string();
    }

    if let Ok(out) = run_silent_cmd(&["npm", "list", "-g", "--depth=0", "--json"]) {
        let json_str = String::from_utf8_lossy(&out.stdout);
        if let Ok(parsed) = serde_json::from_str::<Value>(&json_str) {
            if let Some(deps) = parsed["dependencies"].as_object() {
                for (name, info) in deps {
                    let version = info["version"].as_str().unwrap_or("unknown");
                    let mut size_mb = 0.0;
                    let mut path = String::new();
                    if !npm_root.is_empty() {
                        path = format!("{}\\{}", npm_root, name);
                        if Path::new(&path).exists() { size_mb = get_dir_size(&path); }
                    }
                    packages.push(InstalledProgram {
                        name: name.to_string(),
                        version: version.to_string(),
                        publisher: "npm (Node.js)".into(),
                        install_location: path,
                        uninstall_string: format!("npm uninstall -g {}", name),
                        registry_view: "npm".into(),
                        icon_path: "".into(),
                        source: "npm".into(),
                        size_mb,
                    });
                }
            }
        }
    }
    packages
}

// --- CHOCOLATEY & SCOOP SCANNERS ---
fn scan_choco_packages() -> Vec<InstalledProgram> {
    let mut packages = Vec::new();
    if let Ok(out) = run_silent_cmd(&["choco", "list", "--limit-output"]) {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 2 {
                    packages.push(InstalledProgram {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        publisher: "Chocolatey".into(),
                        install_location: "".into(),
                        uninstall_string: format!("choco uninstall -y {}", parts[0]),
                        registry_view: "choco".into(),
                        icon_path: "".into(),
                        source: "choco".into(),
                        size_mb: 0.0,
                    });
                }
            }
        }
    }
    packages
}

fn scan_scoop_packages() -> Vec<InstalledProgram> {
    let mut packages = Vec::new();
    if let Ok(out) = run_silent_cmd(&["scoop", "list"]) {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines().skip(2) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && !line.trim().is_empty() {
                    packages.push(InstalledProgram {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        publisher: "Scoop".into(),
                        install_location: "".into(),
                        uninstall_string: format!("scoop uninstall {}", parts[0]),
                        registry_view: "scoop".into(),
                        icon_path: "".into(),
                        source: "scoop".into(),
                        size_mb: 0.0,
                    });
                }
            }
        }
    }
    packages
}

// --- UNINSTALL LOGIC ---
#[tauri::command]
pub fn uninstall_program(program: InstalledProgram) -> Result<String, String> {
    if program.source == "pip" {
        // We saved the exact python path in install_location during the scan!
        let python_exe = if program.install_location.is_empty() { "python" } else { &program.install_location };
        
        let mut cmd = Command::new(python_exe);
        cmd.args(["-m", "pip", "uninstall", "-y", &program.name]);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let status = cmd.status().map_err(|e| e.to_string())?;
        return if status.success() { 
            Ok(format!("Uninstalled {}", program.name)) 
        } else { 
            Err("Pip uninstall failed.".into()) 
        };
    }
    
    let mut cmd = Command::new("cmd");
    if program.source == "npm" {
        cmd.args(["/C", "npm", "uninstall", "-g", &program.name]);
        let status = cmd.status().map_err(|e| e.to_string())?;
        return if status.success() { Ok(format!("Uninstalled {}", program.name)) } else { Err("NPM uninstall failed".into()) };
    }
    if program.source == "choco" {
        cmd.args(["/C", "choco", "uninstall", "-y", &program.name]);
        let status = cmd.status().map_err(|e| e.to_string())?;
        return if status.success() { Ok(format!("Uninstalled {}", program.name)) } else { Err("Choco uninstall failed".into()) };
    }
    if program.source == "scoop" {
        cmd.args(["/C", "scoop", "uninstall", &program.name]);
        let status = cmd.status().map_err(|e| e.to_string())?;
        return if status.success() { Ok(format!("Uninstalled {}", program.name)) } else { Err("Scoop uninstall failed".into()) };
    }

    if is_winget_installed(&program.name) { return run_winget_uninstall(&program.name); }
    if program.uninstall_string.to_lowercase().contains("msiexec") {
        if let Some(product_code) = extract_msi_product_code(&program.uninstall_string) {
            return run_msiexec_uninstall(&product_code);
        }
    }
    launch_interactive_uninstaller(&program.uninstall_string)
}

fn is_winget_installed(name: &str) -> bool {
    let output = run_silent_cmd(&["winget", "list", "--name", name, "--accept-source-agreements"]);
    match output { Ok(o) => o.status.success(), Err(_) => false }
}

fn run_winget_uninstall(name: &str) -> Result<String, String> {
    let output = run_silent_cmd(&["winget", "uninstall", "--name", name, "--silent", "--accept-source-agreements", "--disable-interactivity"]).map_err(|e| format!("Failed to launch winget: {e}"))?;
    if output.status.success() { Ok(format!("Uninstalled {name} via winget.")) } else { Err(String::from_utf8_lossy(&output.stderr).to_string()) }
}

fn extract_msi_product_code(uninstall_string: &str) -> Option<String> {
    let upper = uninstall_string.to_uppercase();
    let start = upper.find('{')?;
    let end = upper.find('}')?;
    Some(uninstall_string[start..=end].to_string())
}

fn run_msiexec_uninstall(product_code: &str) -> Result<String, String> {
    let status = Command::new("msiexec").args(["/x", product_code, "/qn", "/norestart"]).status().map_err(|e| format!("Failed to launch msiexec: {e}"))?;
    if status.success() { Ok("Uninstalled MSI package silently.".into()) } else { Err(format!("msiexec exited with code {:?}", status.code())) }
}

fn launch_interactive_uninstaller(uninstall_string: &str) -> Result<String, String> {
    let mut parts = uninstall_string.split_whitespace();
    let program = parts.next().ok_or("Empty uninstall string")?;
    let args: Vec<&str> = parts.collect();
    Command::new(program).args(&args).spawn().map_err(|e| format!("Failed to launch uninstaller: {e}"))?;
    Ok("Interactive uninstaller launched.".into())
}


// --- SMART JUNK SCANNER ---
#[derive(serde::Serialize)]
pub struct JunkCategory {
    pub id: String,
    pub name: String,
    pub size_mb: f64,
    pub path: String,
}

#[tauri::command]
pub fn scan_junk() -> Vec<JunkCategory> {
    let mut categories = Vec::new();
    
    // 1. User Temp Folder
    let user_temp = std::env::temp_dir();
    let temp_size = get_dir_size(&user_temp.to_string_lossy());
    categories.push(JunkCategory {
        id: "user_temp".into(),
        name: "User Temp Files".into(),
        size_mb: temp_size,
        path: user_temp.to_string_lossy().to_string(),
    });

    // 2. Windows Temp Folder
    let win_temp = std::path::Path::new("C:\\Windows\\Temp");
    if win_temp.exists() {
        let win_temp_size = get_dir_size("C:\\Windows\\Temp");
        categories.push(JunkCategory {
            id: "win_temp".into(),
            name: "Windows Temp Files".into(),
            size_mb: win_temp_size,
            path: "C:\\Windows\\Temp".into(),
        });
    }

    // 3. NPM Cache
    let mut npm_cache_size = 0.0;
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let npm_cache = std::path::Path::new(&local_app_data).join("npm-cache");
        if npm_cache.exists() {
            npm_cache_size = get_dir_size(&npm_cache.to_string_lossy());
        }
    }
    categories.push(JunkCategory {
        id: "npm_cache".into(),
        name: "NPM Cache".into(),
        size_mb: npm_cache_size,
        path: "npm-cache".into(),
    });

    // 4. Pip Cache
    let mut pip_cache_size = 0.0;
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let pip_cache = std::path::Path::new(&local_app_data).join("pip\\cache");
        if pip_cache.exists() {
            pip_cache_size = get_dir_size(&pip_cache.to_string_lossy());
        }
    }
    categories.push(JunkCategory {
        id: "pip_cache".into(),
        name: "Pip Cache".into(),
        size_mb: pip_cache_size,
        path: "pip-cache".into(),
    });

    categories
}

#[tauri::command]
pub fn clean_junk(id: String) -> Result<String, String> {
    let mut deleted_count = 0;

    let target_path = match id.as_str() {
        "user_temp" => Some(std::env::temp_dir().to_string_lossy().to_string()),
        "win_temp" => Some("C:\\Windows\\Temp".to_string()),
        "npm_cache" => {
            let _ = Command::new("cmd").args(["/C", "npm", "cache", "clean", "--force"]).output();
            return Ok("NPM Cache cleaned.".into());
        }
        "pip_cache" => {
            let _ = Command::new("cmd").args(["/C", "pip", "cache", "purge"]).output();
            return Ok("Pip Cache cleaned.".into());
        }
        _ => None,
    };

    if let Some(path) = target_path {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    // Try to remove the directory. If it fails (in use), ignore it.
                    if std::fs::remove_dir_all(&p).is_ok() { deleted_count += 1; }
                } else {
                    // Try to remove the file. If it fails (in use), ignore it.
                    if std::fs::remove_file(&p).is_ok() { deleted_count += 1; }
                }
            }
        }
        return Ok(format!("Cleaned {} items.", deleted_count));
    }

    Err("Unknown junk category".into())
}