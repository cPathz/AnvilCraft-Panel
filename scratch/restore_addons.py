import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"

full_function = """#[tauri::command]
pub async fn analyze_instance_addons(
    app: tauri::AppHandle,
    id: String,
    source_paths: Vec<String>,
) -> Result<Vec<AddonAnalysis>, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    // 1. Find Instance
    let mut instance_folder = PathBuf::new();
    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    instance_folder = entry.path();
                    break;
                }
            }
        }
    }

    if instance_folder.as_os_str().is_empty() {
        return Err("Instance not found".to_string());
    }

    let dot_minecraft = instance_folder.join(".minecraft");
    let mods_path = dot_minecraft.join("mods");
    let plugins_path = dot_minecraft.join("plugins");

    let target_dir = if mods_path.exists() {
        mods_path
    } else {
        plugins_path
    };

    // 2. Get current addons list for comparison
    let existing_addons = if target_dir.exists() {
        let mut list = Vec::new();
        for entry in fs::read_dir(&target_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                if let Some(addon) = extract_addon_metadata(&path) {
                    list.push(addon);
                }
            }
        }
        list
    } else {
        Vec::new()
    };

    // 3. Analyze each source path
    let mut results = Vec::new();
    let mut batch_seen: Vec<Addon> = Vec::new();

    for path_str in source_paths {
        let source_path = PathBuf::from(&path_str);
        if !source_path.exists() {
            continue;
        }

        let metadata = extract_addon_metadata(&source_path);
        if metadata.is_none() {
            results.push(AddonAnalysis {
                source_path: path_str.clone(),
                name: source_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                version: "N/A".into(),
                status: "invalid".into(),
                existing_filename: None,
                old_version: None,
                size: 0,
                last_modified: 0,
            });
            continue;
        }

        let meta = metadata.unwrap();
        let mut status = "valid".to_string();
        let mut existing_filename = None;
        let mut old_version = None;

        // A. Check for duplicates/updates within current selection (intra-batch)
        let mut found_in_batch = false;
        for seen in &batch_seen {
            if seen.name == meta.name {
                existing_filename = Some(seen.file_name.clone());
                old_version = Some(seen.version.clone());
                status = if seen.version == meta.version && seen.size == meta.size { "duplicate_selection".into() } else { "update_selection".into() };
                found_in_batch = true;
                break;
            }
        }

        if !found_in_batch {
            // B. Check for duplicates/updates against existing files on disk
            for existing in &existing_addons {
                if existing.name == meta.name {
                    existing_filename = Some(existing.file_name.clone());
                    old_version = Some(existing.version.clone());

                    if existing.version == meta.version
                        && existing.size == meta.size
                        && existing.last_modified == meta.last_modified
                    {
                        status = "duplicate".into();
                    } else {
                        status = "update".into();
                    }
                    break;
                }
            }
        }

        batch_seen.push(meta.clone());

        results.push(AddonAnalysis {
            source_path: path_str,
            name: meta.name,
            version: meta.version,
            status,
            existing_filename,
            old_version,
            size: meta.size,
            last_modified: meta.last_modified,
        });
    }

    Ok(results)
}"""

with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

# Determine line ending
if '\r\n' in content:
    le = '\r\n'
else:
    le = '\n'

# Find the start of the function and the end of it
import re
pattern = r'#\[tauri::command\]\s+pub async fn analyze_instance_addons\(.*?\)\s*->\s*Result<Vec<AddonAnalysis>,\s*String>\s*\{.*?\n\}'
# Actually, it might be safer to find the command and then the next command

start_marker = '#[tauri::command]\r\npub async fn analyze_instance_addons'
if start_marker not in content:
    start_marker = '#[tauri::command]\npub async fn analyze_instance_addons'

start_index = content.find(start_marker)
if start_index == -1:
    print("Could not find start marker")
    sys.exit(1)

# Find end of function (the next #[tauri::command] or end of file)
next_cmd = content.find('#[tauri::command]', start_index + 1)
if next_cmd == -1:
    end_index = len(content)
else:
    end_index = next_cmd

new_content = content[:start_index] + full_function.replace('\n', le) + le + le + content[end_index:]

with open(path, 'wb') as f:
    f.write(new_content.encode('utf-8'))
