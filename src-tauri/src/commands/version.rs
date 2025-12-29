#[tauri::command]
pub fn get_app_version() -> &'static str {
    crate::version::APP_VERSION
}
