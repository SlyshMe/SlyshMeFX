use std::{fs, process::Command, sync::atomic::Ordering, thread};
use tauri::{AppHandle, Emitter, Manager};

use crate::structs::{AppConfig, EqualiserSettings, VisualiserSettings};
use super::util::audioCapture;



#[tauri::command]
pub fn close(appHandle: AppHandle, restart: bool) {
    if restart {
        appHandle.restart();
    } else {
        appHandle.exit(0);
    }
}

#[tauri::command]
pub fn getConfigs() -> Result<(EqualiserSettings, VisualiserSettings), ()> {
    Ok((*crate::EQUALISER_CONFIG.read().unwrap(), *crate::VISUALISER_CONFIG.read().unwrap()))
}

#[tauri::command]
pub async fn getWallpaper() -> Result<Vec<u8>, ()> {
    match wallpaper::get() {
        Ok(path) => {
            let buffer = fs::read(&path).map_err(|_| ())?;
            Ok(buffer)
        }
        Err(_) => Err(()),
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
        visualiserSettings: *crate::VISUALISER_CONFIG.read().unwrap(),
        equaliserSettings: settings,
    }.save(appHandle).unwrap();

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
    *crate::VISUALISER_CONFIG.write().unwrap() = settings;

    appHandle.emit("visualiserUpdate", newSettings).unwrap();
    AppConfig {
        visualiserSettings: settings,
        equaliserSettings: *crate::EQUALISER_CONFIG.read().unwrap(),
    }.save(appHandle).unwrap();

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
