import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"
with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

if '\r\n' in content:
    le = '\r\n'
else:
    le = '\n'

# 1. Add safety check to toggle_instance_addon
old_toggle_start = r"""pub async fn toggle_instance_addon(
    app: tauri::AppHandle,
    id: String,
    file_name: String,
    enabled: bool,
) -> Result<(), String> {"""

new_toggle_start = r"""pub async fn toggle_instance_addon(
    app: tauri::AppHandle,
    id: String,
    file_name: String,
    enabled: bool,
) -> Result<(), String> {
    // Safety check: Is server running?
    let state = app.state::<ChildProcessMap>();
    let is_running = state.0.lock().map(|m| m.contains_key(&id)).unwrap_or(false);
    if is_running {
        return Err("No se puede gestionar complementos mientras el servidor está encendido".to_string());
    }
"""

# 2. Add safety check to delete_instance_addon
old_delete_start = r"""pub async fn delete_instance_addon(
    app: tauri::AppHandle,
    id: String,
    file_name: String,
    delete_folder: bool,
) -> Result<(), String> {"""

new_delete_start = r"""pub async fn delete_instance_addon(
    app: tauri::AppHandle,
    id: String,
    file_name: String,
    delete_folder: bool,
) -> Result<(), String> {
    // Safety check: Is server running?
    let state = app.state::<ChildProcessMap>();
    let is_running = state.0.lock().map(|m| m.contains_key(&id)).unwrap_or(false);
    if is_running {
        return Err("No se puede eliminar complementos mientras el servidor está encendido".to_string());
    }
"""

content = content.replace(old_toggle_start.replace('\n', le), new_toggle_start.replace('\n', le))
content = content.replace(old_delete_start.replace('\n', le), new_delete_start.replace('\n', le))

with open(path, 'wb') as f:
    f.write(content.encode('utf-8'))
print("Safety checks added to instance.rs")
