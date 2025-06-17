#![allow(non_snake_case, dead_code)]

mod structs;
mod commands;
mod util;
mod statics;
use structs::*;
use commands::*;
use statics::*;

use tauri::{
    image::Image,
    tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, Position,
};



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_wallpaper::init())
        .setup(|app| {
            // Config loading
            let config = AppConfig::load(app.handle()).unwrap_or_default();
            *VISUALISER_CONFIG.write().unwrap() = config.visualiserSettings;
            *EQUALISER_CONFIG.write().unwrap() = config.equaliserSettings;
            
            app.handle().emit("visualiserUpdate", serde_json::to_string(&config.visualiserSettings).unwrap()).unwrap();



            // Tray icon
            TrayIconBuilder::new()
                .icon(Image::from_path(app.path().resource_dir().unwrap().join("icons/128x128.png")).expect("Failed to load icon."))
                .on_tray_icon_event(move |tray, event| {
                    match event {
                        TrayIconEvent::Click {
                            id: _,
                            position: _,
                            rect,
                            button: _,
                            button_state,
                        } => match button_state {
                            MouseButtonState::Down => {
                                let app = tray.app_handle();

                                if let Some(window) = app.get_webview_window("settings") {
                                    if window.is_visible().unwrap() {
                                        window.hide().unwrap();
                                    } else {
                                        if let Ok(windowSize) = window.outer_size() {
                                            match rect.position {
                                                Position::Physical(pos) => {
                                                    window
                                                        .set_position(PhysicalPosition {
                                                            x: pos.x as f64
                                                                - windowSize.width as f64 / 2.,
                                                            y: pos.y as f64
                                                                - windowSize.height as f64,
                                                        })
                                                        .unwrap();
                                                }
                                                _ => {}
                                            }
                                        }

                                        window.show().unwrap();
                                        window.set_focus().unwrap();
                                    }
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            getWallpaper,
            startCapture,
            setupEqualiser,
            setEqualiserSettings,
            getConfigs,
            setVisualiserSettings,
            hideSettingsUi,
            close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running application...");
}
