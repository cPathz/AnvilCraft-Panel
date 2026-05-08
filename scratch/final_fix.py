import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"
with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

if '\r\n' in content:
    le = '\r\n'
else:
    le = '\n'

# Fix 1: Missing platform in invalid case
old_1 = f"""                size: 0,
                last_modified: 0,
            }});"""
new_1 = f"""                size: 0,
                last_modified: 0,
                platform: "Unknown".into(),
            }});"""

if old_1.replace('\n', le) in content:
    content = content.replace(old_1.replace('\n', le), new_1.replace('\n', le))
else:
    print("Could not find old_1")

# Fix 2: Unused platform variable
old_2 = f"""    let mut description: Option<String> = None;
    let mut platform = "Unknown".to_string();"""
new_2 = f"""    let mut description: Option<String> = None;"""

if old_2.replace('\n', le) in content:
    content = content.replace(old_2.replace('\n', le), new_2.replace('\n', le))
else:
    print("Could not find old_2")

with open(path, 'wb') as f:
    f.write(content.encode('utf-8'))
