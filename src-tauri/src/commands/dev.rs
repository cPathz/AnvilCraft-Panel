use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImportReport {
    pub added: Vec<String>,
    pub skipped: Vec<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionManifest {
    #[serde(default)]
    pub versions: HashMap<String, VersionFiles>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionFiles {
    pub commands: String,
    pub blocks: String,
    pub items: String,
    pub registries: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImportProgress {
    pub message: String,
    pub step: u32,
    pub total_steps: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CommandNode {
    #[serde(rename = "type")]
    node_type: String,
    children: Option<HashMap<String, CommandNode>>,
    parser: Option<String>,
    executable: Option<bool>,
    redirect: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SimplifiedCommandNode {
    #[serde(rename = "type")]
    node_type: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    children: HashMap<String, SimplifiedCommandNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parser: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    versions: Vec<String>,
}

impl SimplifiedCommandNode {
    fn new(node_type: String, parser: Option<String>) -> Self {
        Self {
            node_type,
            children: HashMap::new(),
            parser,
            versions: Vec::new(),
        }
    }
}

fn simplify_command_node(node: CommandNode) -> SimplifiedCommandNode {
    let mut children = HashMap::new();
    if let Some(node_children) = node.children {
        for (name, child) in node_children {
            children.insert(name, simplify_command_node(child));
        }
    }

    SimplifiedCommandNode {
        node_type: node.node_type,
        children,
        parser: node.parser,
        versions: Vec::new(),
    }
}

fn merge_command_nodes(
    base: &mut SimplifiedCommandNode,
    other: SimplifiedCommandNode,
    version: &str,
) {
    // Add version to current base node
    if !base.versions.contains(&version.to_string()) {
        base.versions.push(version.to_string());
    }

    // Merge children
    for (name, child) in other.children {
        base.children
            .entry(name)
            .and_modify(|existing| merge_command_nodes(existing, child.clone(), version))
            .or_insert_with(|| {
                let mut new_node = child;
                add_version_recursive(&mut new_node, version);
                new_node
            });
    }
}

fn add_version_recursive(node: &mut SimplifiedCommandNode, version: &str) {
    if !node.versions.contains(&version.to_string()) {
        node.versions.push(version.to_string());
    }
    for child in node.children.values_mut() {
        add_version_recursive(child, version);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VersionedItem {
    id: String,
    versions: Vec<String>,
}

#[tauri::command]
pub async fn import_minecraft_data(
    source_path: String,
    app_handle: tauri::AppHandle,
) -> Result<ImportReport, String> {
    use std::collections::HashSet;
    use tauri::Emitter;

    let source_dir = Path::new(&source_path);
    if !source_dir.exists() || !source_dir.is_dir() {
        return Err(format!(
            "Source path does not exist or is not a directory: {}",
            source_path
        ));
    }

    let target_base_path = Path::new("../src/lib/data/minecraft");
    if !target_base_path.exists() {
        fs::create_dir_all(target_base_path)
            .map_err(|e| format!("Failed to create target base path: {}", e))?;
    }

    // Load existing manifest or create new
    let manifest_path = target_base_path.join("manifest.json");
    let mut manifest: VersionManifest = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or(VersionManifest {
            versions: HashMap::new(),
        })
    } else {
        VersionManifest {
            versions: HashMap::new(),
        }
    };

    let mut report = ImportReport {
        added: Vec::new(),
        skipped: Vec::new(),
        error: None,
    };

    let _ = app_handle.emit(
        "import-progress",
        ImportProgress {
            message: "Scanning and Aggregating Data...".into(),
            step: 0,
            total_steps: 100,
        },
    );

    let entries = fs::read_dir(source_dir).map_err(|e| e.to_string())?;

    // Aggregator: Type -> ItemId -> Set of Versions
    let mut master_aggregator: HashMap<String, HashMap<String, HashSet<String>>> = HashMap::new();

    // Standard static arguments (version agnostic for now, or assume all)
    let static_entities = vec!["@a", "@p", "@e", "@s", "@r"];
    for entity in static_entities {
        master_aggregator
            .entry("minecraft:entity".to_string())
            .or_default()
            .entry(entity.to_string())
            .or_default()
            .insert("all".to_string()); // Special "all" marker or logic
        master_aggregator
            .entry("minecraft:game_profile".to_string())
            .or_default()
            .entry(entity.to_string())
            .or_default()
            .insert("all".to_string());
    }

    let mut best_version_name: String = String::new();

    // Master Tree for Command Merging
    let mut master_command_tree = SimplifiedCommandNode::new("root".to_string(), None);

    // Collect entries first to count them? No, stream them.
    // Actually, we need to iterate all to find best version AND aggregate.

    let entry_list: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    let total_count = entry_list.len();

    for (idx, entry) in entry_list.into_iter().enumerate() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = entry.file_name().into_string().unwrap_or_default();
            let is_vanilla_space = dir_name.starts_with("vanilla ");
            let is_vanilla_underscore = dir_name.starts_with("vanilla_");

            if is_vanilla_space || is_vanilla_underscore {
                let prefix = if is_vanilla_space {
                    "vanilla "
                } else {
                    "vanilla_"
                };
                let version_str = dir_name.trim_start_matches(prefix).to_string();
                let safe_version_name = version_str.replace(".", "_");

                // Update Progress
                let _ = app_handle.emit(
                    "import-progress",
                    ImportProgress {
                        message: format!("Processing {}...", version_str),
                        step: ((idx as f32 / total_count as f32) * 80.0) as u32,
                        total_steps: 100,
                    },
                );

                // 1. Determine Best Version (Existing Logic)
                let current_parts: Vec<u32> = version_str
                    .split('.')
                    .map(|s| s.parse().unwrap_or(0))
                    .collect();
                let best_parts: Vec<u32> = best_version_name
                    .split('.')
                    .map(|s| s.parse().unwrap_or(0))
                    .collect();

                let mut is_newer = false;
                let max_len = std::cmp::max(current_parts.len(), best_parts.len());
                for i in 0..max_len {
                    let curr = *current_parts.get(i).unwrap_or(&0);
                    let best = *best_parts.get(i).unwrap_or(&0);
                    if curr > best {
                        is_newer = true;
                        break;
                    }
                    if curr < best {
                        is_newer = false;
                        break;
                    }
                }
                if is_newer || best_version_name.is_empty() {
                    best_version_name = version_str.clone();
                    // Removed best_version_path as we now merge all trees
                }

                // 2. File Copying (Only if not in manifest)
                let target_subfolder_name = format!("v{}", safe_version_name);
                let target_version_dir = target_base_path.join(&target_subfolder_name);

                // Always update manifest logic if needed, but only copy if missing or forced?
                // For safety/repair, we try to ensure files exist.
                if !target_version_dir.exists() {
                    let _ = fs::create_dir_all(&target_version_dir);
                }

                let files_to_copy = vec![
                    "commands.json",
                    "blocks.json",
                    "items.json",
                    "registries.json",
                ];
                let mut version_files = VersionFiles {
                    commands: "".to_string(),
                    blocks: "".to_string(),
                    items: "".to_string(),
                    registries: "".to_string(),
                };

                for filename in &files_to_copy {
                    let src_file = path.join(filename);
                    if src_file.exists() {
                        let dest_file = target_version_dir.join(filename);
                        if !dest_file.exists() {
                            let _ = fs::copy(&src_file, &dest_file);
                        }

                        let relative_path =
                            format!("minecraft/{}/{}", target_subfolder_name, filename);
                        match *filename {
                            "commands.json" => version_files.commands = relative_path,
                            "blocks.json" => version_files.blocks = relative_path,
                            "items.json" => version_files.items = relative_path,
                            "registries.json" => version_files.registries = relative_path,
                            _ => {}
                        }
                    }
                }

                if !manifest.versions.contains_key(&version_str) {
                    manifest.versions.insert(version_str.clone(), version_files);
                    report.added.push(version_str.clone());
                } else {
                    report.skipped.push(version_str.clone());
                }

                // 3. AGGREGATION LOGIC (Read from SOURCE to be sure)
                // Blocks
                if let Ok(content) = fs::read_to_string(path.join("blocks.json")) {
                    if let Ok(data) =
                        serde_json::from_str::<HashMap<String, serde_json::Value>>(&content)
                    {
                        for key in data.keys() {
                            master_aggregator
                                .entry("minecraft:block_state".into())
                                .or_default()
                                .entry(key.clone())
                                .or_default()
                                .insert(version_str.clone());
                            master_aggregator
                                .entry("minecraft:block_predicate".into())
                                .or_default()
                                .entry(key.clone())
                                .or_default()
                                .insert(version_str.clone());
                        }
                    }
                }

                // Items
                if let Ok(content) = fs::read_to_string(path.join("items.json")) {
                    if let Ok(data) =
                        serde_json::from_str::<HashMap<String, serde_json::Value>>(&content)
                    {
                        for key in data.keys() {
                            master_aggregator
                                .entry("minecraft:item_stack".into())
                                .or_default()
                                .entry(key.clone())
                                .or_default()
                                .insert(version_str.clone());
                            master_aggregator
                                .entry("minecraft:item_predicate".into())
                                .or_default()
                                .entry(key.clone())
                                .or_default()
                                .insert(version_str.clone());
                        }
                    }
                }

                // Registries logic...
                if let Ok(content) = fs::read_to_string(path.join("registries.json")) {
                    if let Ok(data) =
                        serde_json::from_str::<HashMap<String, serde_json::Value>>(&content)
                    {
                        // ... (keep existing registry logic, implied same) ...
                        let registry_map: HashMap<&str, Vec<&str>> = HashMap::from([
                            (
                                "minecraft:biome",
                                vec!["minecraft:biome", "minecraft:resource_or_tag"],
                            ),
                            ("minecraft:enchantment", vec!["minecraft:enchantment"]),
                            ("minecraft:mob_effect", vec!["minecraft:mob_effect"]),
                            (
                                "minecraft:entity_type",
                                vec!["minecraft:entity_summon", "minecraft:resource"],
                            ),
                            ("minecraft:attribute", vec!["minecraft:attribute"]),
                            ("minecraft:potion", vec!["minecraft:potion"]),
                            (
                                "minecraft:sound_event",
                                vec!["minecraft:resource_location", "minecraft:sound"],
                            ),
                            ("minecraft:particle_type", vec!["minecraft:particle"]),
                            ("minecraft:dimension", vec!["minecraft:dimension"]),
                        ]);

                        for (reg_key, arg_types) in registry_map {
                            if let Some(entry) = data.get(reg_key) {
                                if let Some(entries) =
                                    entry.get("entries").and_then(|e| e.as_object())
                                {
                                    for key in entries.keys() {
                                        for arg_type in &arg_types {
                                            master_aggregator
                                                .entry(arg_type.to_string())
                                                .or_default()
                                                .entry(key.clone())
                                                .or_default()
                                                .insert(version_str.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 4. COMMAND TREE MERGING
                if let Ok(content) = fs::read_to_string(path.join("commands.json")) {
                    if let Ok(root_node) = serde_json::from_str::<CommandNode>(&content) {
                        let simple_tree = simplify_command_node(root_node);
                        merge_command_nodes(&mut master_command_tree, simple_tree, &version_str);
                    }
                }
            }
        }
    }

    // Save manifest
    let json_output = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, json_output).map_err(|e| e.to_string())?;

    // --- Generate Final Data ---
    let data_dir = Path::new("../src/lib/data");
    if !data_dir.exists() {
        let _ = fs::create_dir_all(data_dir);
    }

    // 1. Write Smart Arguments
    let _ = app_handle.emit(
        "import-progress",
        ImportProgress {
            message: "Generating smart argument index...".into(),
            step: 90,
            total_steps: 100,
        },
    );

    // Convert HashMap<Type, HashMap<Id, Set<Ver>>> to HashMap<Type, Vec<VersionedItem>>
    let mut final_arguments: HashMap<String, Vec<VersionedItem>> = HashMap::new();

    for (arg_type, item_map) in master_aggregator {
        let mut items_list: Vec<VersionedItem> = item_map
            .into_iter()
            .map(|(id, ver_set)| {
                let mut versions: Vec<String> = ver_set.into_iter().collect();
                // Sort versions for cleanliness
                // versions.sort_by(...) // Basic sort is fine for now, or use the semver logic
                versions.sort();
                VersionedItem { id, versions }
            })
            .collect();

        // Sort items by ID for binary search potential or just tidiness
        items_list.sort_by(|a, b| a.id.cmp(&b.id));

        final_arguments.insert(arg_type, items_list);
    }

    fs::write(
        data_dir.join("arguments.json"),
        serde_json::to_string(&final_arguments).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    // 2. Write Merged Command Tree
    let _ = app_handle.emit(
        "import-progress",
        ImportProgress {
            message: "Saving command tree...".into(),
            step: 95,
            total_steps: 100,
        },
    );

    // Sort versions in the tree for consistency (optional but good)
    // We could traverse master_command_tree and sort versions, but skipping for perf now.

    fs::write(
        data_dir.join("command_tree.json"),
        serde_json::to_string(&master_command_tree).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let _ = app_handle.emit(
        "import-progress",
        ImportProgress {
            message: "Cleaning up raw data...".into(),
            step: 99,
            total_steps: 100,
        },
    );

    // 3. Cleanup Raw Data
    // We strictly delete the version subdirectories we created/used, keeping manifest.json
    // target_base_path is "../src/lib/data/minecraft"
    if let Ok(entries) = fs::read_dir(target_base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // It's a version folder (or other dir), safe to remove as we only want JSONs in root
                let _ = fs::remove_dir_all(path);
            }
        }
    }

    let _ = app_handle.emit(
        "import-progress",
        ImportProgress {
            message: "Import Complete!".into(),
            step: 100,
            total_steps: 100,
        },
    );
    Ok(report)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataStats {
    pub version_count: usize,
    pub latest_version: String,
    pub last_updated: String,
    pub versions: Vec<String>,
}

#[tauri::command]
pub async fn get_data_stats() -> Result<DataStats, String> {
    let target_base_path = Path::new("../src/lib/data/minecraft/manifest.json");
    if !target_base_path.exists() {
        return Err("Manifest not found".to_string());
    }

    let content = fs::read_to_string(&target_base_path).map_err(|e| e.to_string())?;
    let manifest: VersionManifest = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let mut versions: Vec<String> = manifest.versions.keys().cloned().collect();
    // Sort logic (simple)
    versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.split('.').map(|s| s.parse().unwrap_or(0)).collect();
        let b_parts: Vec<u32> = b.split('.').map(|s| s.parse().unwrap_or(0)).collect();

        let max_len = std::cmp::max(a_parts.len(), b_parts.len());
        for i in 0..max_len {
            let curr_a = *a_parts.get(i).unwrap_or(&0);
            let curr_b = *b_parts.get(i).unwrap_or(&0);
            if curr_a > curr_b {
                return std::cmp::Ordering::Greater;
            }
            if curr_a < curr_b {
                return std::cmp::Ordering::Less;
            }
        }
        std::cmp::Ordering::Equal
    });

    let latest = versions
        .last()
        .cloned()
        .unwrap_or_else(|| "None".to_string());

    // Get last modified time of arguments.json as a proxy for "last updated"
    let args_path = Path::new("../src/lib/data/arguments.json");
    let last_updated = if args_path.exists() {
        "Just now".to_string() // Simplified for now, or use actual metadata if needed
    } else {
        "Never".to_string()
    };

    Ok(DataStats {
        version_count: versions.len(),
        latest_version: latest,
        last_updated,
        versions, // Return all sorted versions
    })
}
