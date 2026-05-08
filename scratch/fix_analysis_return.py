import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"
with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

if '\r\n' in content:
    le = '\r\n'
else:
    le = '\n'

old_block = f"""        results.push(AddonAnalysis {{
            source_path: path_str,
            name: meta.name,
            version: meta.version,
            status,
            existing_filename,
            old_version,
            size: meta.size,
            last_modified: meta.last_modified,
        }});"""

new_block = f"""        results.push(AddonAnalysis {{
            source_path: path_str,
            name: meta.name,
            version: meta.version,
            status,
            existing_filename,
            old_version,
            size: meta.size,
            last_modified: meta.last_modified,
            platform: meta.platform,
        }});"""

if old_block.replace('\n', le) in content:
    content = content.replace(old_block.replace('\n', le), new_block.replace('\n', le))
    with open(path, 'wb') as f:
        f.write(content.encode('utf-8'))
    print("Success")
else:
    print("Could not find old_block")
