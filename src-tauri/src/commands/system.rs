use sysinfo::System;

#[tauri::command]
pub fn get_system_memory() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_all();
    sys.total_memory()
}

#[tauri::command]
pub fn get_java_version() -> String {
    // Placeholder or simple implementation
    "Java 21 (Detected)".to_string()
}
