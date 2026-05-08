import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"
with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

if '\r\n' in content:
    le = '\r\n'
else:
    le = '\n'

# 1. Update extensions list
content = content.replace('.ends_with(".disabled")', '.ends_with(".disabled") || file_name.ends_with(".bkp") || file_name.ends_with(".bak") || file_name.ends_with(".old") || file_name.ends_with(".off")')
# Avoid double replace if already done partial
if 'file_name.ends_with(".jar") || file_name.ends_with(".disabled") || file_name.ends_with(".bkp")' not in content:
    content = content.replace('file_name.ends_with(".jar") || file_name.ends_with(".disabled")', 
                             'file_name.ends_with(".jar") || file_name.ends_with(".disabled") || file_name.ends_with(".bkp") || file_name.ends_with(".bak") || file_name.ends_with(".old") || file_name.ends_with(".off")')

# 2. Add toggle_instance_addon and delete_instance_addon commands
commands = r"""
#[tauri::command]
pub async fn toggle_instance_addon(
    app: tauri::AppHandle,
    id: String,
    file_name: String,
    enabled: bool,
) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    let dot_minecraft = entry.path().join(".minecraft");
                    let mods_path = dot_minecraft.join("mods");
                    let plugins_path = dot_minecraft.join("plugins");

                    let target_dir = if mods_path.exists() {
                        mods_path
                    } else {
                        plugins_path
                    };

                    let source_path = target_dir.join(&file_name);
                    if !source_path.exists() {
                        return Err("File not found".to_string());
                    }

                    let mut new_name = file_name.clone();
                    if enabled {
                        // Remove any "off" suffix and ensure it ends with .jar
                        for suffix in &[".disabled", ".bkp", ".bak", ".old", ".off"] {
                            if new_name.ends_with(suffix) {
                                new_name = new_name.replace(suffix, "");
                            }
                        }
                        if !new_name.ends_with(".jar") {
                            new_name.push_str(".jar");
                        }
                    } else {
                        // Add .disabled suffix
                        if !new_name.ends_with(".disabled") {
                            new_name.push_str(".disabled");
                        }
                    }

                    if new_name != file_name {
                        fs::rename(source_path, target_dir.join(new_name)).map_err(|e| e.to_string())?;
                    }
                    return Ok(());
                }
            }
        }
    }
    Err("Instance not found".to_string())
}

#[tauri::command]
pub async fn delete_instance_addon(
    app: tauri::AppHandle,
    id: String,
    file_name: String,
    delete_folder: bool,
) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    let dot_minecraft = entry.path().join(".minecraft");
                    let mods_path = dot_minecraft.join("mods");
                    let plugins_path = dot_minecraft.join("plugins");

                    let target_dir = if mods_path.exists() {
                        mods_path
                    } else {
                        plugins_path
                    };

                    let file_path = target_dir.join(&file_name);
                    if file_path.exists() {
                        // 1. Detect possible folder before deleting the file (to have metadata if needed)
                        if delete_folder {
                            let addon_meta = extract_addon_metadata(&file_path);
                            let folder_names = vec![
                                file_name.replace(".jar", "").replace(".disabled", "").replace(".bkp", "").replace(".bak", "").replace(".old", "").replace(".off", ""),
                                addon_meta.map(|m| m.name).unwrap_or_default(),
                            ];

                            for f_name in folder_names {
                                if f_name.is_empty() { continue; }
                                let possible_dir = target_dir.join(&f_name);
                                if possible_dir.exists() && possible_dir.is_dir() {
                                    let _ = fs::remove_dir_all(possible_dir);
                                    break;
                                }
                            }
                        }

                        // 2. Delete the file
                        fs::remove_file(file_path).map_err(|e| e.to_string())?;
                    }
                    return Ok(());
                }
            }
        }
    }
    Err("Instance not found".to_string())
}
"""

# Append to the end of the file
content += commands.replace('\n', le)

with open(path, 'wb') as f:
    f.write(content.encode('utf-8'))
print("Commands added to instance.rs")
