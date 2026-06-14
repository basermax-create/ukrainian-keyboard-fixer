use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey: String,
    pub direction: String,      // "auto" | "en_to_ua" | "ua_to_en"
    pub autostart: bool,
    pub notifications: bool,
    #[serde(default = "default_true")]
    pub first_run: bool,
}

fn default_true() -> bool { true }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: "CommandOrControl+Shift+U".into(),
            direction: "auto".into(),
            autostart: false,
            notifications: true,
            first_run: true,
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
    inner: Mutex<AppSettings>,
}

impl SettingsStore {
    pub fn load(app: &AppHandle) -> Self {
        let dir = app
            .path()
            .app_config_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("settings.json");

        let settings = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
            .unwrap_or_default();

        Self {
            path,
            inner: Mutex::new(settings),
        }
    }

    pub fn get(&self) -> AppSettings {
        self.inner.lock().unwrap().clone()
    }

    pub fn set(&self, s: AppSettings) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&s).map_err(std::io::Error::other)?;
        fs::write(&self.path, json)?;
        *self.inner.lock().unwrap() = s;
        Ok(())
    }
}