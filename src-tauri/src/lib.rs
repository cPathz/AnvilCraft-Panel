// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

pub mod commands;
pub mod models;

use models::ChildProcessMap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(ChildProcessMap(Arc::new(Mutex::new(HashMap::new()))));
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
            commands::instance::save_instance_settings,
            commands::instance::open_instances_folder,
            commands::instance::detect_minecraft_version,
            commands::instance::create_instance_from_path,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
