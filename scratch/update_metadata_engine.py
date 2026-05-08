import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"

full_function = """fn extract_addon_metadata(path: &PathBuf) -> Option<Addon> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let metadata = fs::metadata(path).ok()?;
    let size = metadata.len();
    let last_modified = metadata.modified().ok()?
        .duration_since(std::time::UNIX_EPOCH).ok()?
        .as_secs() as i64;
    let enabled = !file_name.ends_with(".disabled");

    let mut name = file_name.clone();
    let mut version = "Unknown".to_string();
    let mut author: Option<String> = None;
    let mut description: Option<String> = None;
    let mut platform = "Unknown".to_string();

    // 1. Check for fabric.mod.json (Fabric)
    if let Ok(mut fabric_file) = archive.by_name("fabric.mod.json") {
        let mut content = String::new();
        if fabric_file.read_to_string(&mut content).is_ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = json.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = json.get("authors").and_then(|v| v.as_array()).and_then(|a| a.get(0)).and_then(|v| {
                    if v.is_string() { v.as_str() } else { v.get("name").and_then(|n| n.as_str()) }
                }).map(|s| s.to_string());
                description = json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                platform = "Fabric".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 2. Check for quilt.mod.json (Quilt)
    if let Ok(mut quilt_file) = archive.by_name("quilt.mod.json") {
        let mut content = String::new();
        if quilt_file.read_to_string(&mut content).is_ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let quat = json.get("quilt_loader").or(json.get("metadata"));
                if let Some(m) = quat {
                    if let Some(n) = m.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                    if let Some(v) = m.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                    author = m.get("contributors").and_then(|v| v.as_object()).and_then(|o| o.keys().next()).map(|s| s.to_string());
                    description = m.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                }
                platform = "Quilt".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 3. Check for mods.toml (Forge 1.13+)
    if let Ok(mut forge_file) = archive.by_name("META-INF/mods.toml") {
        let mut content = String::new();
        if forge_file.read_to_string(&mut content).is_ok() {
            if let Ok(toml_val) = toml::from_str::<toml::Value>(&content) {
                if let Some(mods_array) = toml_val.get("mods").and_then(|v| v.as_array()) {
                    if let Some(mods) = mods_array.get(0) {
                        if let Some(n) = mods.get("displayName").and_then(|v| v.as_str()) { name = n.to_string(); }
                        if let Some(v) = mods.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                        author = mods.get("authors").and_then(|v| v.as_str()).map(|s| s.to_string());
                        description = mods.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                        platform = "Forge".to_string();
                        return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
                    }
                }
            }
        }
    }

    // 4. Check for mcmod.info (Legacy Forge)
    if let Ok(mut mcmod_file) = archive.by_name("mcmod.info") {
        let mut content = String::new();
        if mcmod_file.read_to_string(&mut content).is_ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let mod_obj = if json.is_array() { json.get(0) } else { json.get("modList").and_then(|l| l.get(0)).or(Some(&json)) };
                if let Some(m) = mod_obj {
                    if let Some(n) = m.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                    if let Some(v) = m.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                    author = m.get("authorList").and_then(|v| v.as_array()).and_then(|a| a.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
                    description = m.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                }
                platform = "Forge (Legacy)".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 5. Check for paper-plugin.yml (Modern Paper)
    if let Ok(mut paper_file) = archive.by_name("paper-plugin.yml") {
        let mut content = String::new();
        if paper_file.read_to_string(&mut content).is_ok() {
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(n) = yaml.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = yaml.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = yaml.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
                description = yaml.get("description").and_then(|v| v.as_str()).map(|s: &str| s.to_string());
                platform = "Paper".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 6. Check for plugin.yml (Spigot/Bukkit)
    if let Ok(mut plugin_file) = archive.by_name("plugin.yml") {
        let mut content = String::new();
        if plugin_file.read_to_string(&mut content).is_ok() {
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(n) = yaml.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = yaml.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = yaml.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
                description = yaml.get("description").and_then(|v| v.as_str()).map(|s: &str| s.to_string());
                platform = "Spigot".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 7. Check for bungee.yml (BungeeCord)
    if let Ok(mut bungee_file) = archive.by_name("bungee.yml") {
        let mut content = String::new();
        if bungee_file.read_to_string(&mut content).is_ok() {
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(n) = yaml.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = yaml.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = yaml.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
                platform = "Bungee".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 8. Check for velocity-plugin.json (Velocity)
    if let Ok(mut velocity_file) = archive.by_name("velocity-plugin.json") {
        let mut content = String::new();
        if velocity_file.read_to_string(&mut content).is_ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = json.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = json.get("authors").and_then(|v| v.as_array()).and_then(|a| a.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
                description = json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                platform = "Velocity".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }
    
    None
}"""

with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

# Determine line ending
if '\r\n' in content:
    le = '\r\n'
else:
    le = '\n'

# Find the function and replace it
import re
# We need to find the old extract_addon_metadata function and replace it
# It starts at 'fn extract_addon_metadata' and ends at the first 'None\r\n}' or similar

start_index = content.find("fn extract_addon_metadata")
if start_index == -1:
    print("Could not find function start")
    exit(1)

# Find the end of this function (it ends with None and then a closing brace)
end_marker = "None\r\n}"
if end_marker not in content:
    end_marker = "None\n}"

end_index = content.find(end_marker, start_index)
if end_index == -1:
    print("Could not find function end")
    exit(1)

end_index += len(end_marker)

new_content = content[:start_index] + full_function.replace('\n', le) + content[end_index:]

with open(path, 'wb') as f:
    f.write(new_content.encode('utf-8'))
