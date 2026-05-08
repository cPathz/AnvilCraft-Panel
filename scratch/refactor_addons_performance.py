import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"
with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

if '\r\n' in content:
    le = '\r\n'
else:
    le = '\n'

# 1. Add imports
if 'use rayon::prelude::*;' not in content:
    content = content.replace(f"use std::path::PathBuf;{le}", f"use std::path::{{PathBuf, Path}};{le}use rayon::prelude::*;{le}")

# 2. Define the internal helper function
helper_fn = f"""
fn get_addons_internal(
    target_dir: &Path,
    cache_path: &Path,
    force_scan: bool,
) -> Result<Vec<Addon>, String> {{
    if !target_dir.exists() {{
        return Ok(vec![]);
    }}

    // 1. Load Cache
    let mut cache = if !force_scan && cache_path.exists() {{
        let cache_content = fs::read_to_string(&cache_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<AddonCache>(&cache_content).unwrap_or(AddonCache {{ last_scan: 0, addons: vec![] }})
    }} else {{
        AddonCache {{ last_scan: 0, addons: vec![] }}
    }};

    // 2. Scan Directory
    let mut files_to_scan = Vec::new();
    let mut current_addons = Vec::new();
    let mut cache_modified = false;

    for entry in fs::read_dir(target_dir).map_err(|e| e.to_string())? {{
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {{
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if file_name.ends_with(".jar") || file_name.ends_with(".disabled") {{
                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                let size = metadata.len();
                let last_modified = metadata.modified().map_err(|e| e.to_string())?
                    .duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?
                    .as_secs() as i64;

                // Check Cache
                if let Some(cached) = cache.addons.iter().find(|a| a.file_name == file_name && a.size == size && a.last_modified == last_modified) {{
                    current_addons.push(cached.clone());
                }} else {{
                    files_to_scan.push(path.clone());
                    cache_modified = true;
                }}
            }}
        }}
    }}

    // 3. Parallel Scan for new/changed files
    if !files_to_scan.is_empty() {{
        let new_addons: Vec<Addon> = files_to_scan.par_iter()
            .filter_map(|path| extract_addon_metadata(path))
            .collect();
        current_addons.extend(new_addons);
        cache_modified = true;
    }}

    // 4. Check for deletions
    if current_addons.len() != cache.addons.len() {{
        cache_modified = true;
    }}

    // 5. Update Cache File
    if cache_modified || force_scan {{
        cache.addons = current_addons.clone();
        cache.last_scan = Utc::now().timestamp();
        let new_cache_json = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
        fs::write(cache_path, new_cache_json).map_err(|e| e.to_string())?;
    }}

    Ok(current_addons)
}}
"""

# Insert the helper function after extract_addon_metadata
# I'll just find the end of extract_addon_metadata
import re
pattern = r'fn extract_addon_metadata\(.*?\)\s*->\s*Option<Addon>\s*\{.*?\n\}'
match = re.search(pattern, content, re.DOTALL)
if match:
    insert_pos = match.end()
    content = content[:insert_pos] + le + helper_fn.replace('\n', le) + content[insert_pos:]
else:
    print("Could not find extract_addon_metadata to insert helper")
    exit(1)

# 3. Refactor get_instance_addons to use the helper
old_get_addons = r"""#[tauri::command]
pub async fn get_instance_addons(app: tauri::AppHandle, id: String, force_scan: bool) -> Result<Vec<Addon>, String> {
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
                    let cache_path = entry.path().join("addons_cache.json");

                    let target_dir = if mods_path.exists() {
                        mods_path
                    } else if plugins_path.exists() {
                        plugins_path
                    } else {
                        return Ok(vec![]);
                    };

                    // Load Cache
                    let mut cache = if !force_scan && cache_path.exists() {
                        let cache_content = fs::read_to_string(&cache_path).map_err(|e| e.to_string())?;
                        serde_json::from_str::<AddonCache>(&cache_content).unwrap_or(AddonCache { last_scan: 0, addons: vec![] })
                    } else {
                        AddonCache { last_scan: 0, addons: vec![] }
                    };

                    let mut updated_addons = vec![];
                    let mut cache_modified = false;

                    for file_entry in fs::read_dir(&target_dir).map_err(|e| e.to_string())? {
                        let file_entry = file_entry.map_err(|e| e.to_string())?;
                        let path = file_entry.path();
                        
                        if path.is_file() {
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            if file_name.ends_with(".jar") || file_name.ends_with(".disabled") {
                                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                                let size = metadata.len();
                                let last_modified = metadata.modified().map_err(|e| e.to_string())?
                                    .duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?
                                    .as_secs() as i64;

                                // Check Cache
                                if let Some(cached) = cache.addons.iter().find(|a| a.file_name == file_name && a.size == size && a.last_modified == last_modified) {
                                    updated_addons.push(cached.clone());
                                } else {
                                    // Rescan
                                    if let Some(addon) = extract_addon_metadata(&path) {
                                        updated_addons.push(addon);
                                        cache_modified = true;
                                    }
                                }
                            }
                        }
                    }

                    // Check for deletions
                    if updated_addons.len() != cache.addons.len() {
                        cache_modified = true;
                    }

                    if cache_modified || force_scan {
                        cache.addons = updated_addons.clone();
                        cache.last_scan = Utc::now().timestamp();
                        let new_cache_json = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
                        fs::write(cache_path, new_cache_json).map_err(|e| e.to_string())?;
                    }

                    return Ok(updated_addons);
                }
            }
        }
    }
    Err("Instance not found".to_string())
}"""

new_get_addons = r"""#[tauri::command]
pub async fn get_instance_addons(app: tauri::AppHandle, id: String, force_scan: bool) -> Result<Vec<Addon>, String> {
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
                    let cache_path = entry.path().join("addons_cache.json");

                    let target_dir = if mods_path.exists() {
                        mods_path
                    } else {
                        plugins_path
                    };

                    return get_addons_internal(&target_dir, &cache_path, force_scan);
                }
            }
        }
    }
    Err("Instance not found".to_string())
}"""

# 4. Refactor analyze_instance_addons to use the helper and parallel scan sources
# Find the start of analyze_instance_addons and replace its guts
old_analyze_body = r"""    // 2. Get current addons list for comparison
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

        let metadata = extract_addon_metadata(&source_path);"""

new_analyze_body = r"""    // 2. Get current addons list for comparison (USING CACHE)
    let cache_path = instance_folder.join("addons_cache.json");
    let existing_addons = get_addons_internal(&target_dir, &cache_path, false)?;

    // 3. Analyze each source path (IN PARALLEL)
    let mut batch_seen: Vec<Addon> = Vec::new();
    
    // First, extract metadata for all sources in parallel
    let source_metas: Vec<(String, Option<Addon>)> = source_paths.par_iter()
        .map(|p| (p.clone(), extract_addon_metadata(&PathBuf::from(p))))
        .collect();

    let mut results = Vec::new();
    for (path_str, metadata) in source_metas {
        let source_path = PathBuf::from(&path_str);
        if metadata.is_none() {"""

content = content.replace(old_get_addons.replace('\n', le), new_get_addons.replace('\n', le))
content = content.replace(old_analyze_body.replace('\n', le), new_analyze_body.replace('\n', le))

with open(path, 'wb') as f:
    f.write(content.encode('utf-8'))
print("Refactoring complete")
