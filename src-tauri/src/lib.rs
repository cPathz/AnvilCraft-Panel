// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

pub mod commands;
mod version;

pub mod models;

use notify::{Watcher, RecursiveMode, Event};
use models::{ChildProcessMap, AddonWatcherState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            app.manage(ChildProcessMap(Arc::new(Mutex::new(HashMap::new()))));
            
            // Setup Addon Watcher
            let app_handle = app.app_handle().clone();
            let app_data = app_handle.path().app_data_dir().unwrap();
            let instances_dir = app_data.join("instances");
            
            if instances_dir.exists() {
                let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
                    match res {
                        Ok(event) => {
                            for path in event.paths {
                                let p_str = path.to_string_lossy();
                                if (p_str.contains("mods") || p_str.contains("plugins")) && 
                                   (p_str.ends_with(".jar") || p_str.ends_with(".disabled") || p_str.contains("addons_cache.json")) {
                                    let _ = app_handle.emit("addons-changed", ());
                                    break;
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }).unwrap();
                
                let _ = watcher.watch(&instances_dir, RecursiveMode::Recursive);
                app.manage(AddonWatcherState(Mutex::new(Some(watcher))));
            } else {
                app.manage(AddonWatcherState(Mutex::new(None)));
            }
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let state = app.state::<ChildProcessMap>();
                let has_running_tasks = state.0.lock().map(|m| !m.is_empty()).unwrap_or(false);

                if has_running_tasks {
                    // Prevent close
                    api.prevent_close();
                    // Notify frontend
                    let _ = app.emit("app-close-forbidden", ());
                    // Optional: Bring window to front
                    let _ = window.set_focus();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Instance
            commands::instance::create_instance,
            commands::instance::read_instances,
            commands::instance::delete_instance,
            commands::instance::update_instance_icon,
            commands::instance::update_instance_name,
            commands::instance::update_instance_version,
            commands::instance::save_instance_settings,
            commands::instance::open_instances_folder,
            commands::instance::detect_minecraft_version,
            commands::instance::create_instance_from_path,
            commands::instance::get_instance_max_players,
            commands::instance::get_instance_addons_type,
            commands::instance::get_instance_addons,
            commands::instance::open_instance_addons_folder,
            commands::instance::analyze_instance_addons,
            commands::instance::install_instance_addons,
            commands::instance::toggle_instance_addon,
            commands::instance::delete_instance_addon,
            // Server
            commands::server::start_instance,
            commands::server::stop_instance,
            commands::server::kill_instance,
            commands::server::send_command,
            // Versions
            commands::versions::get_minecraft_versions,
            commands::versions::get_project_versions,
            // System
            commands::system::get_system_memory,
            commands::system::get_java_version,
            // Dev
            commands::dev::import_minecraft_data,
            commands::dev::get_data_stats,
            // Java
            commands::java::get_available_java_versions,
            commands::java::download_java_runtime,
            // Version
            commands::version::get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
