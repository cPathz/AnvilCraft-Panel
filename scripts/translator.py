import json
import os
import argparse
from typing import Dict, Any

try:
    from deep_translator import GoogleTranslator
except ImportError:
    print("Error: El módulo 'deep-translator' no está instalado.")
    print("Por favor instala las dependencias usando: pip install deep-translator")
    exit(1)

def translate_dict(data: Dict[str, Any], translator: GoogleTranslator) -> Dict[str, Any]:
    """Recursively translates the values of a dictionary."""
    translated_data = {}
    for key, value in data.items():
        if isinstance(value, dict):
            # Recursion for nested objects
            translated_data[key] = translate_dict(value, translator)
        elif isinstance(value, str):
            try:
                # Perform translation
                print(f"Traduciendo key: '{key}'...")
                translated = translator.translate(value)
                translated_data[key] = translated
            except Exception as e:
                print(f"Error traduciendo la key '{key}': {e}")
                translated_data[key] = value
        else:
            translated_data[key] = value
    return translated_data

def main():
    parser = argparse.ArgumentParser(description="Traduce diccionarios JSON de AnvilCraft usando Google Translate.")
    parser.add_argument("--source", type=str, default="../src/lib/locales/es.json", help="Ruta al JSON de origen (español por defecto)")
    parser.add_argument("--target_lang", type=str, default="en", help="Código de idioma objetivo (ej. 'en', 'fr', 'pt')")
    parser.add_argument("--output", type=str, default="../src/lib/locales/en.json", help="Ruta de guardado del JSON traducido")
    args = parser.parse_args()

    source_path = os.path.abspath(args.source)
    output_path = os.path.abspath(args.output)
    target_lang = args.target_lang

    if not os.path.exists(source_path):
        print(f"Error: El archivo fuente no existe en la ruta {source_path}")
        return

    print(f"Leyendo archivo origen: {source_path}")
    with open(source_path, 'r', encoding='utf-8') as f:
        try:
            source_data = json.load(f)
        except json.JSONDecodeError:
            print("Error: El archivo fuente no es un JSON válido.")
            return

    print(f"Iniciando traducción al idioma '{target_lang}'...")
    translator = GoogleTranslator(source='auto', target=target_lang)
    
    translated_data = translate_dict(source_data, translator)

    print(f"Escribiendo resultado en: {output_path}")
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(translated_data, f, ensure_ascii=False, indent=2)

    print("\n¡Traducción completada con éxito! 🎉")
    print(f"Tus traducciones se han guardado en {output_path}")

if __name__ == "__main__":
    main()
