import json
import os

base_dir = r"D:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\reports"
output_path = r"D:\Usuarios\Luis Macias\Documentos\Proyectos\AnvilCraft\src\lib\data\arguments.json"

arguments = {
    # Initialize with static selectors for entity parsers
    "minecraft:entity": ["@a", "@p", "@e", "@s", "@r"],
    "minecraft:game_profile": ["@a", "@p", "@e", "@s", "@r"]
}

# 1. Blocks
try:
    with open(os.path.join(base_dir, "blocks.json"), "r") as f:
        data = json.load(f)
        keys = list(data.keys())
        arguments["minecraft:block_state"] = keys
        arguments["minecraft:block_predicate"] = keys
        print(f"Extracted {len(keys)} blocks")
except Exception as e:
    print(f"Failed to extract blocks: {e}")

# 2. Items
try:
    with open(os.path.join(base_dir, "items.json"), "r") as f:
        data = json.load(f)
        keys = list(data.keys())
        arguments["minecraft:item_stack"] = keys
        arguments["minecraft:item_predicate"] = keys
        print(f"Extracted {len(keys)} items")
except Exception as e:
    print(f"Failed to extract items: {e}")

# 3. Registries
try:
    with open(os.path.join(base_dir, "registries.json"), "r") as f:
        data = json.load(f)
        
        # Mapping Registry Key -> Brigadier Parser
        # Some are direct matches, some need mapping
        
        # Biomes
        if "minecraft:biome" in data:
            biomes = list(data["minecraft:biome"]["entries"].keys())
            arguments["minecraft:resource_or_tag"] = biomes # Approximated
            # Store specifically for biome parser if present (often it's resource_location but with context)
            arguments["minecraft:biome"] = biomes # If parser is specifically minecraft:biome (it exists in newer versions)
            
        # Enchantments
        if "minecraft:enchantment" in data:
            enchants = list(data["minecraft:enchantment"]["entries"].keys())
            arguments["minecraft:enchantment"] = enchants # Only if parser specific
            # Often mapped to resource_location too, but let's see.
            
        # Effects
        if "minecraft:mob_effect" in data:
             effects = list(data["minecraft:mob_effect"]["entries"].keys())
             arguments["minecraft:mob_effect"] = effects
        
        # Entity Types (for summon)
        if "minecraft:entity_type" in data:
             entities = list(data["minecraft:entity_type"]["entries"].keys())
             arguments["minecraft:entity_summon"] = entities
             arguments["minecraft:resource"] = entities # Sometimes used for summon
             
        # Attributes
        if "minecraft:attribute" in data:
            attrs = list(data["minecraft:attribute"]["entries"].keys())
            arguments["minecraft:attribute"] = attrs
            
        # Potions
        if "minecraft:potion" in data:
            potions = list(data["minecraft:potion"]["entries"].keys())
            arguments["minecraft:potion"] = potions

        print("Extracted registries")
        
except Exception as e:
    print(f"Failed to extract registries: {e}")

with open(output_path, "w") as f:
    json.dump(arguments, f, separators=(',', ':'))

print(f"Saved arguments to {output_path}")
