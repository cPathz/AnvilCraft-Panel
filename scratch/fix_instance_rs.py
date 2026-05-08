import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"
with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

# Determine line ending
if '\r\n' in content:
    le = '\r\n'
else:
    le = '\n'

old_1 = f"    // 3. Analyze each source path{le}    let mut results = Vec::new();{le}    for path_str in source_paths {{"
new_1 = f"    // 3. Analyze each source path{le}    let mut results = Vec::new();{le}    let mut batch_seen: Vec<Addon> = Vec::new();{le}    for path_str in source_paths {{"

old_2 = f"        // Check for duplicates/updates{le}        for existing in &existing_addons {{{le}            if existing.name == meta.name {{{le}                existing_filename = Some(existing.file_name.clone());{le}                old_version = Some(existing.version.clone());{le}{le}                if existing.version == meta.version{le}                    && existing.size == meta.size{le}                    && existing.last_modified == meta.last_modified{le}                {{{le}                    status = \"duplicate\".into();{le}                }} else {{{le}                    status = \"update\".into();{le}                }}{le}                break;{le}            }}{le}        }}"

new_2 = f"        // Check for duplicates/updates within current selection{le}        let mut found_in_batch = false;{le}        for seen in &batch_seen {{{le}            if seen.name == meta.name {{{le}                existing_filename = Some(seen.file_name.clone());{le}                old_version = Some(seen.version.clone());{le}                status = if seen.version == meta.version && seen.size == meta.size {{ \"duplicate\".into() }} else {{ \"update\".into() }};{le}                found_in_batch = true;{le}                break;{le}            }}{le}        }}{le}{le}        if !found_in_batch {{{le}            // Check for duplicates/updates against existing files on disk{le}            for existing in &existing_addons {{{le}                if existing.name == meta.name {{{le}                    existing_filename = Some(existing.file_name.clone());{le}                    old_version = Some(existing.version.clone());{le}{le}                    if existing.version == meta.version{le}                        && existing.size == meta.size{le}                        && existing.last_modified == meta.last_modified{le}                    {{{le}                        status = \"duplicate\".into();{le}                    }} else {{{le}                        status = \"update\".into();{le}                    }}{le}                    break;{le}                }}{le}            }}{le}        }}{le}{le}        batch_seen.push(meta.clone());"

if old_1 in content:
    content = content.replace(old_1, new_1)
else:
    print("Could not find old_1")

if old_2 in content:
    content = content.replace(old_2, new_2)
else:
    print("Could not find old_2")

with open(path, 'wb') as f:
    f.write(content.encode('utf-8'))
