use serde::{Deserialize, Serialize};
use winreg::enums::*;
use winreg::RegKey;
use walkdir::WalkDir;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledProgram {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_location: String,
    pub uninstall_string: String,
    pub registry_view: String,
    pub icon_path: String,
    pub source: String,
    pub size_mb: f64,
}

pub fn scan_installed_programs() -> Vec<InstalledProgram> {
    let mut programs = Vec::new();

    scan_hive(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall", KEY_READ | KEY_WOW64_64KEY, "64-bit", &mut programs);
    scan_hive(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall", KEY_READ | KEY_WOW64_32KEY, "32-bit", &mut programs);
    scan_hive(HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall", KEY_READ, "per-user", &mut programs);

    programs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    programs.dedup_by(|a, b| a.name == b.name && a.uninstall_string == b.uninstall_string);

    programs
}

fn scan_hive(hive: winreg::HKEY, path: &str, flags: u32, view_label: &str, out: &mut Vec<InstalledProgram>) {
    let hkey = RegKey::predef(hive);
    let key = match hkey.open_subkey_with_flags(path, flags) {
        Ok(k) => k,
        Err(_) => return,
    };

    for subkey_name in key.enum_keys().flatten() {
        if let Ok(subkey) = key.open_subkey_with_flags(&subkey_name, flags) {
            let name: String = subkey.get_value("DisplayName").unwrap_or_default();
            if name.trim().is_empty() { continue; }

            let system_component: u32 = subkey.get_value("SystemComponent").unwrap_or(0);
            if system_component == 1 { continue; }

            let uninstall_string: String = subkey.get_value("UninstallString").unwrap_or_default();
            if uninstall_string.trim().is_empty() { continue; }

            let mut icon_path: String = subkey.get_value("DisplayIcon").unwrap_or_default();
            if icon_path.contains(',') { icon_path = icon_path.split(',').next().unwrap_or("").to_string(); }
            icon_path = icon_path.replace("\"", "");

            let mut install_location: String = subkey.get_value("InstallLocation").unwrap_or_default();
            
            // --- SMART FALLBACK LOGIC ---
            // If InstallLocation is empty, try to get the folder from the Icon or Uninstaller
            if install_location.trim().is_empty() || !Path::new(&install_location).exists() {
                
                // 1. Try from Icon Path
                if !icon_path.is_empty() {
                    let p = Path::new(&icon_path);
                    if let Some(parent) = p.parent() {
                        install_location = parent.to_string_lossy().to_string();
                    }
                }

                // 2. If still empty, try from Uninstall String
                if install_location.trim().is_empty() || !Path::new(&install_location).exists() {
                    let clean_u = uninstall_string.trim_matches('"');
                    let u_path = clean_u.split(' ').next().unwrap_or("");
                    let p = Path::new(u_path);
                    if let Some(parent) = p.parent() {
                        install_location = parent.to_string_lossy().to_string();
                    }
                }
            }

            // Calculate size
            let mut size_mb = 0.0;
            if !install_location.trim().is_empty() && Path::new(&install_location).exists() {
                // Check if it's a directory before walking
                if Path::new(&install_location).is_dir() {
                    let mut total_size = 0;
                    for entry in WalkDir::new(&install_location).into_iter().filter_map(|e| e.ok()) {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_file() { total_size += metadata.len(); }
                        }
                    }
                    size_mb = total_size as f64 / 1_048_576.0;
                }
            }

            out.push(InstalledProgram {
                name: name.trim().to_string(),
                version: subkey.get_value("DisplayVersion").unwrap_or_default(),
                publisher: subkey.get_value("Publisher").unwrap_or_default(),
                install_location,
                uninstall_string,
                registry_view: view_label.to_string(),
                icon_path,
                source: "windows".to_string(),
                size_mb,
            });
        }
    }
}