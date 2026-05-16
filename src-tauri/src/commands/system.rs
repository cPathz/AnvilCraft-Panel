use sysinfo::System;

#[tauri::command]
pub async fn get_system_memory() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.total_memory()
}

#[tauri::command]
pub fn get_java_version() -> String {
    // Placeholder or simple implementation
    "Java 21 (Detected)".to_string()
}

#[tauri::command]
pub fn get_distribution_channel() -> String {
    // Windows sets this env var for MSIX/AppX packaged apps
    if std::env::var("PACKAGE_FULL_NAME").is_ok() {
        "msix".to_string()
    } else {
        "standalone".to_string()
    }
}
