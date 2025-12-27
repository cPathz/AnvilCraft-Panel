import json

with open(r"D:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\reports\registries.json", "r") as f:
    data = json.load(f)
    print(list(data.keys()))
