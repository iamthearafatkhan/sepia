use crate::registry::{scan_installed_programs, InstalledProgram};
use std::process::Command;
use serde_json::Value;
use walkdir::WalkDir;
use std::path::Path;
use std::collections::HashMap;

#[tauri::command]
pub async fn list_programs() -> Vec<InstalledProgram> {
    let mut programs = scan_installed_programs();
    programs.extend(scan_pip_packages());
    programs.extend(scan_npm_packages());
    programs.extend(scan_choco_packages());
    programs.extend(scan_scoop_packages());
    programs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    programs
}

#[tauri::command]
pub fn open_url(url: String) {
    let _ = Command::new("cmd").args(["/C", "start", "", &url]).spawn();
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

// --- PIP SCANNER (FIXED) ---
fn scan_pip_packages() -> Vec<InstalledProgram> {
    let mut packages = Vec::new();
    let mut locations = HashMap::new();

    // Force python -m pip to avoid PATH issues on Windows
    let show_cmd = Command::new("cmd").args(["/C", "python", "-m", "pip", "show"]).output();
    
    if let Ok(out) = show_cmd {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut current_name = String::new();
        
        for line in stdout.lines() {
            // Aggressively trim to remove hidden \r characters
            let trimmed = line.trim(); 
            
            if trimmed.starts_with("Name: ") {
                current_name = trimmed.replacen("Name: ", "", 1).trim().to_string();
            } else if trimmed.starts_with("Location: ") {
                let loc = trimmed.replacen("Location: ", "", 1).trim().to_string();
                if !current_name.is_empty() {
                    locations.insert(current_name.clone(), loc);
                    current_name.clear();
                }
            }
        }
    }

    let list_cmd = Command::new("cmd").args(["/C", "python", "-m", "pip", "list", "--format=json"]).output();
    
    if let Ok(out) = list_cmd {
        let json_str = String::from_utf8_lossy(&out.stdout);
        if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(&json_str) {
            for item in arr {
                if let (Some(name), Some(version)) = (item["name"].as_str(), item["version"].as_str()) {
                    let mut size_mb = 0.0;
                    let mut path = String::new();
                    
                    if let Some(loc) = locations.get(name) {
                        // Python package folders usually replace dashes with underscores
                        let pkg_dir1 = format!("{}\\{}", loc, name.replace("-", "_"));
                        let pkg_dir2 = format!("{}\\{}", loc, name);
                        
                        if Path::new(&pkg_dir1).exists() {
                            path = pkg_dir1;
                        } else if Path::new(&pkg_dir2).exists() {
                            path = pkg_dir2;
                        }

                        if !path.is_empty() {
                            size_mb = get_dir_size(&path);
                        }
                    }

                    packages.push(InstalledProgram {
                        name: name.to_string(),
                        version: version.to_string(),
                        publisher: "PyPI (Python)".into(),
                        install_location: path,
                        uninstall_string: format!("python -m pip uninstall -y {}", name),
                        registry_view: "pip".into(),
                        icon_path: "".into(),
                        source: "pip".into(),
                        size_mb,
                    });
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
    
    if let Ok(out) = Command::new("cmd").args(["/C", "npm", "root", "-g"]).output() {
        npm_root = String::from_utf8_lossy(&out.stdout).trim().to_string();
    }

    let output = Command::new("cmd").args(["/C", "npm", "list", "-g", "--depth=0", "--json"]).output();
    if let Ok(out) = output {
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
    if let Ok(out) = Command::new("cmd").args(["/C", "choco", "list", "--limit-output"]).output() {
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
    if let Ok(out) = Command::new("cmd").args(["/C", "scoop", "list"]).output() {
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
        let status = Command::new("cmd").args(["/C", "python", "-m", "pip", "uninstall", "-y", &program.name]).status()
            .map_err(|e| e.to_string())?;
        return if status.success() { Ok(format!("Uninstalled {}", program.name)) } else { Err("Pip uninstall failed".into()) };
    }
    if program.source == "npm" {
        let status = Command::new("cmd").args(["/C", "npm", "uninstall", "-g", &program.name]).status().map_err(|e| e.to_string())?;
        return if status.success() { Ok(format!("Uninstalled {}", program.name)) } else { Err("NPM uninstall failed".into()) };
    }
    if program.source == "choco" {
        let status = Command::new("cmd").args(["/C", "choco", "uninstall", "-y", &program.name]).status().map_err(|e| e.to_string())?;
        return if status.success() { Ok(format!("Uninstalled {}", program.name)) } else { Err("Choco uninstall failed".into()) };
    }
    if program.source == "scoop" {
        let status = Command::new("cmd").args(["/C", "scoop", "uninstall", &program.name]).status().map_err(|e| e.to_string())?;
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
    let output = Command::new("cmd").args(["/C", "winget", "list", "--name", name, "--accept-source-agreements"]).output();
    match output { Ok(o) => o.status.success(), Err(_) => false }
}

fn run_winget_uninstall(name: &str) -> Result<String, String> {
    let output = Command::new("cmd").args(["/C", "winget", "uninstall", "--name", name, "--silent", "--accept-source-agreements", "--disable-interactivity"]).output().map_err(|e| format!("Failed to launch winget: {e}"))?;
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