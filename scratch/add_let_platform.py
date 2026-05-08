import os

path = r"d:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src-tauri\src\commands\instance.rs"
with open(path, 'rb') as f:
    content = f.read().decode('utf-8')

platforms = ["Fabric", "Quilt", "Forge", "Forge (Legacy)", "Paper", "Spigot", "Bungee", "Velocity"]

for p in platforms:
    old = f'platform = "{p}".to_string();'
    new = f'let platform = "{p}".to_string();'
    content = content.replace(old, new)

with open(path, 'wb') as f:
    f.write(content.encode('utf-8'))
