use std::{fs, process::Command, sync::atomic::Ordering, thread};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};

use crate::{structs::{AppConfig, EqualiserSettings, VisualiserSettings}};
use super::util::audioCapture;
use tauri_plugin_wallpaper::{AttachRequest, WallpaperExt};



#[tauri::command]
pub fn close(appHandle: AppHandle, restart: bool) {
    if restart {
        appHandle.restart();
    } else {
        appHandle.exit(0);
    }
}

#[tauri::command]
pub fn getConfigs() -> Result<(EqualiserSettings, VisualiserSettings), String> {
    Ok((*crate::EQUALISER_CONFIG.read().unwrap(), crate::VISUALISER_CONFIG.read().unwrap().clone()))
}

#[tauri::command]
pub fn getMonitors(appHandle: AppHandle) -> Result<String, String> {
    let monitors = appHandle.available_monitors().expect("Failed to retrieve available monitors.");
    Ok(serde_json::to_string(&monitors).expect("Failed to serialise available monitors."))
}

#[tauri::command]
pub async fn getWallpaper() -> Result<Vec<u8>, String> {
    match wallpaper::get() {
        Ok(w) => {
            match fs::read(&w) {
                Ok(buffer) => {
                    Ok(buffer)
                },
                Err(e) => {
                    Err(e.to_string())
                }
            }
        },
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn hideSettingsUi(appHandle: AppHandle) -> Result<(), ()> {
    if let Some(window) = appHandle.get_webview_window("settings") {
        if window.is_visible().unwrap() {
            window.hide().unwrap();
        }
    }

    Ok(())
}

#[tauri::command]
pub fn setEqualiserSettings(appHandle: AppHandle, newSettings: String) -> Result<(), ()> {
    let settings: EqualiserSettings = serde_json::from_str(&newSettings).unwrap();
    let config = settings.toConfig();

    if fs::exists("C:/Program Files/EqualizerAPO/config/config.txt").unwrap() {
        fs::write("C:/Program Files/EqualizerAPO/config/config.txt", config).unwrap();
    }
    
    *crate::EQUALISER_CONFIG.write().unwrap() = settings;
    AppConfig {
        visualiserSettings: crate::VISUALISER_CONFIG.read().unwrap().clone(),
        equaliserSettings: settings,
    }.save(&appHandle).unwrap();

    Ok(())
}

#[tauri::command]
pub fn setMonitor(appHandle: AppHandle, monitorName: String) -> Result<(), String> {
    let monitors = appHandle.available_monitors().expect("Failed to retrieve available monitors.");
    
    if let Some(monitor) = monitors.iter().find(|m| *m.name().unwrap_or(&"".to_string()) == monitorName) {
        let mainWindow = appHandle.get_webview_window("main").unwrap();

        if !crate::IS_ATTACHED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            appHandle.wallpaper().attach(AttachRequest { window_label: "main".to_string() }).unwrap();
        }

        appHandle.emit("setCanvasPosition", (
            serde_json::to_string(&PhysicalPosition {
                x: monitor.position().x - mainWindow.outer_position().unwrap().x,
                y: monitor.position().y - mainWindow.outer_position().unwrap().y,
            }).unwrap(), 
            serde_json::to_string(&monitor.size()).unwrap())
        ).unwrap();
    } else {
        return Err("Monitor not found".into());
    }

    Ok(())
}

#[tauri::command]
pub fn setupEqualiser(appHandle: AppHandle) -> Result<(), ()> {
    let apoInstaller = appHandle.path().resource_dir().unwrap().join("resources/EqualizerAPO-x64-1.4.2.exe");

    Command::new("powershell")
        .arg("-Command")
        .arg(format!(
            "Start-Process -FilePath '{}' -Verb runAs",
            apoInstaller.to_str().unwrap()
        ))
        .spawn()
        .map_err(|e| format!("Failed to launch APO installer: {}", e))
        .unwrap();

    Ok(())
}

#[tauri::command]
pub fn setVisualiserSettings(appHandle: AppHandle, newSettings: String) -> Result<(), ()> {
    let settings: VisualiserSettings = serde_json::from_str(&newSettings).unwrap();
    let lastMonitor = crate::VISUALISER_CONFIG.read().unwrap().screen.clone();

    *crate::VISUALISER_CONFIG.write().unwrap() = settings.clone();

    appHandle.emit("visualiserUpdate", newSettings).unwrap();
    AppConfig {
        visualiserSettings: settings.clone(),
        equaliserSettings: *crate::EQUALISER_CONFIG.read().unwrap(),
    }.save(&appHandle).unwrap();

    if lastMonitor != settings.screen {
        appHandle.emit("startScreenChange", settings.screen).unwrap();
    }

    Ok(())
}

#[tauri::command]
pub fn startCapture(appHandle: AppHandle) -> Result<(), String> {
    if crate::IS_CAPTURE_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        ()
    }

    thread::spawn(move || {
        struct CapturingGuard;
        impl Drop for CapturingGuard {
            fn drop(&mut self) {
                crate::IS_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
            }
        }

        let _guard = CapturingGuard;

        if let Err(e) = audioCapture(appHandle) {
            eprintln!("Audio capture thread panicked: {:?}", e);
        }
    });

    Ok(())
}
