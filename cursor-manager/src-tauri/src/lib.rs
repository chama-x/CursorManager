// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{Manager, Runtime, State};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::time::Duration;
use std::thread;
use std::error::Error;
use rand::distributions::{Alphanumeric, DistString};
use fs_extra;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    cursor_app: String,
    profile_base: String,
    archive_base: String,
    workspace_base: String,
    network_interface: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            cursor_app: "/Applications/Cursor.app/Contents/MacOS/Cursor".to_string(),
            profile_base: home.join("cursor-profiles").to_string_lossy().to_string(),
            archive_base: home.join("cursor-archives").to_string_lossy().to_string(),
            workspace_base: home.join("projects").to_string_lossy().to_string(),
            network_interface: "en0".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    name: String,
    path: String,
    created: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveInfo {
    name: String,
    path: String,
    created: String,
    original_session: String,
}

pub struct CursorManager {
    config: Config,
}

impl CursorManager {
    pub fn new(config: Option<Config>) -> Self {
        let config = config.unwrap_or_default();
        let profile_dir = Path::new(&config.profile_base);
        let archive_dir = Path::new(&config.archive_base);

        if !profile_dir.exists() {
            let _ = fs::create_dir_all(profile_dir);
        }
        if !archive_dir.exists() {
            let _ = fs::create_dir_all(archive_dir);
        }

        Self { config }
    }

    // Utility functions
    pub fn list_sessions(&self) -> Result<Vec<SessionInfo>, Box<dyn Error>> {
        let profile_dir = Path::new(&self.config.profile_base);
        let mut sessions = Vec::new();

        if !profile_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(profile_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let metadata = fs::metadata(&path)?;
                let created = metadata.created()
                    .map(|time| {
                        Local::now().date_naive().format("%Y-%m-%d").to_string()
                    })
                    .unwrap_or_else(|_| "Unknown".to_string());
                
                sessions.push(SessionInfo {
                    name: path.file_name().unwrap().to_string_lossy().to_string(),
                    path: path.to_string_lossy().to_string(),
                    created,
                });
            }
        }

        Ok(sessions)
    }

    pub fn list_archives(&self) -> Result<Vec<ArchiveInfo>, Box<dyn Error>> {
        let archive_dir = Path::new(&self.config.archive_base);
        let mut archives = Vec::new();

        if !archive_dir.exists() {
            return Ok(archives);
        }

        for entry in fs::read_dir(archive_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let metadata = fs::metadata(&path)?;
                let created = metadata.created()
                    .map(|time| {
                        Local::now().date_naive().format("%Y-%m-%d").to_string()
                    })
                    .unwrap_or_else(|_| "Unknown".to_string());
                
                // Try to determine original session name
                let name = path.file_name().unwrap().to_string_lossy();
                let original_session = if let Some(idx) = name.rfind('-') {
                    name[0..idx].to_string()
                } else {
                    "Unknown".to_string()
                };
                
                archives.push(ArchiveInfo {
                    name: name.to_string(),
                    path: path.to_string_lossy().to_string(),
                    created,
                    original_session,
                });
            }
        }

        Ok(archives)
    }

    pub fn random_mac(&self) -> String {
        // Generate a random MAC address with a locally administered address
        let random_part = Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
        format!("02:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", 
            random_part.as_bytes()[0] % 256, 
            random_part.as_bytes()[1] % 256,
            random_part.as_bytes()[2] % 256,
            random_part.as_bytes()[3] % 256,
            random_part.as_bytes()[4] % 256)
    }

    pub async fn spoof_mac<R: Runtime>(&self, app: &tauri::AppHandle<R>, interface: Option<String>) -> Result<String, Box<dyn Error>> {
        let iface = interface.unwrap_or(self.config.network_interface.clone());
        let new_mac = self.random_mac();
        
        // Note: These commands require sudo privileges
        // Turn off Wi-Fi
        let _ = app.shell()
            .command("sudo")
            .args(["networksetup", "-setairportpower", &iface, "off"])
            .output()
            .await?;

        // Change MAC address
        let output = app.shell()
            .command("sudo")
            .args(["/sbin/ifconfig", &iface, "ether", &new_mac])
            .output()
            .await?;

        let success = output.status.success();
        
        // Turn Wi-Fi back on
        let _ = app.shell()
            .command("sudo")
            .args(["networksetup", "-setairportpower", &iface, "on"])
            .output()
            .await?;

        // Get current MAC address for verification
        let verify_output = app.shell()
            .command("ifconfig")
            .args([&iface])
            .output()
            .await?;
        
        let verify_text = String::from_utf8_lossy(&verify_output.stdout);

        if success {
            Ok(format!("MAC address for {} changed to {}", iface, new_mac))
        } else {
            Ok(format!("Failed to change MAC address. Error: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    // Session management functions
    pub fn create_session(&self, name: &str) -> Result<String, Box<dyn Error>> {
        if name.is_empty() {
            return Err("Session name cannot be empty".into());
        }
        
        let session_dir = Path::new(&self.config.profile_base).join(name);
        if session_dir.exists() {
            return Err(format!("Session '{}' already exists", name).into());
        }
        
        fs::create_dir_all(&session_dir)?;
        Ok(format!("Session '{}' created at {}", name, session_dir.to_string_lossy()))
    }

    pub async fn launch_session<R: Runtime>(&self, app: &tauri::AppHandle<R>, session: &str, spoof_mac: bool, connect_vpn: bool) -> Result<String, Box<dyn Error>> {
        let session_dir = Path::new(&self.config.profile_base).join(session);
        if !session_dir.exists() {
            return Err(format!("Session directory not found: {}", session_dir.to_string_lossy()).into());
        }
        
        let mut result = String::new();
        
        if spoof_mac {
            let mac_result = self.spoof_mac(app, None).await?;
            result.push_str(&format!("{}\n", mac_result));
        }
        
        if connect_vpn {
            result.push_str("Connect to your VPN now for a new IP before continuing.\n");
            // In a real app, we'd wait for user confirmation here
        }
        
        // Launch Cursor with the profile directory
        let args = vec![
            "--user-data-dir=".to_string() + &session_dir.to_string_lossy(),
            "--new-window".to_string(),
        ];
        
        // Use shell plugin for the actual launch
        let cursor_result = app.shell()
            .command(&self.config.cursor_app)
            .args(args)
            .spawn()?;
        
        result.push_str(&format!("Launched Cursor with session '{}'\n", session));
        result.push_str("PID: ");
        result.push_str(&format!("{:?}", cursor_result.pid())); // Include PID info if available
        
        Ok(result)
    }

    pub fn archive_session(&self, session: &str) -> Result<String, Box<dyn Error>> {
        let session_dir = Path::new(&self.config.profile_base).join(session);
        if !session_dir.exists() {
            return Err(format!("Session '{}' not found", session).into());
        }
        
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let archive_name = format!("{}-{}", session, timestamp);
        let archive_dir = Path::new(&self.config.archive_base).join(&archive_name);
        
        if archive_dir.exists() {
            return Err(format!("Archive destination already exists: {}", archive_dir.to_string_lossy()).into());
        }
        
        // Create the archive directory
        fs::create_dir_all(&archive_dir)?;
        
        // Copy all files from session to archive
        let mut copy_options = fs_extra::dir::CopyOptions::new();
        copy_options.copy_inside = true;
        fs_extra::dir::copy(&session_dir, &archive_dir, &copy_options)?;
        
        Ok(format!("Session '{}' archived to '{}'", session, archive_name))
    }

    pub fn restore_archive(&self, archive: &str, new_session_name: Option<&str>) -> Result<String, Box<dyn Error>> {
        let archive_dir = Path::new(&self.config.archive_base).join(archive);
        if !archive_dir.exists() {
            return Err(format!("Archive '{}' not found", archive).into());
        }
        
        // Determine the target session name
        let session_name = if let Some(name) = new_session_name {
            name.to_string()
        } else {
            // Extract the original session name from the archive name
            if let Some(idx) = archive.rfind('-') {
                archive[0..idx].to_string()
            } else {
                return Err(format!("Cannot determine original session name from archive '{}'", archive).into());
            }
        };
        
        let session_dir = Path::new(&self.config.profile_base).join(&session_name);
        if session_dir.exists() {
            return Err(format!("Session '{}' already exists, cannot restore to this name", session_name).into());
        }
        
        // Create the session directory
        fs::create_dir_all(&session_dir)?;
        
        // Copy all files from archive to session
        let mut copy_options = fs_extra::dir::CopyOptions::new();
        copy_options.copy_inside = true;
        fs_extra::dir::copy(&archive_dir, &session_dir, &copy_options)?;
        
        Ok(format!("Archive '{}' restored to session '{}'", archive, session_name))
    }

    pub fn delete_session(&self, session: &str) -> Result<String, Box<dyn Error>> {
        let session_dir = Path::new(&self.config.profile_base).join(session);
        if !session_dir.exists() {
            return Err(format!("Session '{}' not found", session).into());
        }
        
        fs::remove_dir_all(&session_dir)?;
        Ok(format!("Session '{}' deleted", session))
    }

    pub fn delete_archive(&self, archive: &str) -> Result<String, Box<dyn Error>> {
        let archive_dir = Path::new(&self.config.archive_base).join(archive);
        if !archive_dir.exists() {
            return Err(format!("Archive '{}' not found", archive).into());
        }
        
        fs::remove_dir_all(&archive_dir)?;
        Ok(format!("Archive '{}' deleted", archive))
    }
}

// Tauri command functions - refactored to use State

#[tauri::command]
pub fn get_config(config_state: State<Config>) -> Config {
    config_state.inner().clone()
}

#[tauri::command]
pub fn update_config(new_config: Config, config_state: State<Mutex<Config>>) -> Result<Config, String> {
    let mut config = config_state.lock().map_err(|e| e.to_string())?;
    *config = new_config.clone();
    Ok(new_config)
}

#[tauri::command]
pub fn list_sessions(config_state: State<Config>) -> Result<Vec<SessionInfo>, String> {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_archives(config_state: State<Config>) -> Result<Vec<ArchiveInfo>, String> {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.list_archives().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_session(config_state: State<Config>, name: &str) -> Result<String, String> {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.create_session(name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn launch_session_cmd<R: Runtime>(app: tauri::AppHandle<R>, config_state: State<Config>, session: &str, spoof_mac: bool, connect_vpn: bool) -> Result<String, String> {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.launch_session(&app, session, spoof_mac, connect_vpn).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn spoof_mac_cmd<R: Runtime>(app: tauri::AppHandle<R>, config_state: State<Config>, interface: Option<String>) -> Result<String, String> {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.spoof_mac(&app, interface).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn archive_session(config_state: State<Config>, session: &str) -> Result<String, String> {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.archive_session(session).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_archive(config_state: State<Config>, archive: &str, new_session_name: Option<&str>) -> Result<String, String> {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.restore_archive(archive, new_session_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_session(config_state: State<Config>, session: &str) -> Result<String, String> {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.delete_session(session).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_archive(config_state: State<Config>, archive: &str) -> Result<String, String> {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.delete_archive(archive).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn random_mac(config_state: State<Config>) -> String {
    let manager = CursorManager::new(Some(config_state.inner().clone()));
    manager.random_mac()
}

pub fn run<R: Runtime>() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Config::default())
        .manage(Mutex::new(Config::default())) // For mutable state
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            list_sessions,
            list_archives,
            create_session,
            launch_session_cmd,
            spoof_mac_cmd,
            archive_session,
            restore_archive,
            delete_session,
            delete_archive,
            random_mac,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
