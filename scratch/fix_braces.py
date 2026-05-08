import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"
with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

if '\r\n' in content:
    le = '\r\n'
else:
    le = '\n'

# Fix the duplicate if
bad_block = f"""    for (path_str, metadata) in source_metas {{
        let source_path = PathBuf::from(&path_str);
        if metadata.is_none() {{
        if metadata.is_none() {{"""

good_block = f"""    for (path_str, metadata) in source_metas {{
        let source_path = PathBuf::from(&path_str);
        if metadata.is_none() {{"""

if bad_block.replace('\n', le) in content:
    content = content.replace(bad_block.replace('\n', le), good_block.replace('\n', le))
    with open(path, 'wb') as f:
        f.write(content.encode('utf-8'))
    print("Fixed duplication")
else:
    print("Could not find bad_block")
