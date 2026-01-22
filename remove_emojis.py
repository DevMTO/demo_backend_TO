#!/usr/bin/env python3
"""
Script para eliminar emojis de archivos Rust preservando el formato exacto.
"""
import os
import re

# Lista de emojis a eliminar (con el espacio que les sigue)
EMOJIS_TO_REMOVE = [
    '⚠️ ', '✅ ', '❌ ', '🔐 ', '🔄 ', '🚀 ', '📝 ', '📦 ',
    '🔒 ', '💾 ', '📊 ', '🌐 ', '🔍 ', '👤 ', '🎫 ', '📋 ',
    '🔔 ', '💰 ', '🚌 ', '🍽️ ', '📁 ', '🏢 ', '🎯 ',
    # Sin espacio (por si hay casos sin espacio después)
    '⚠️', '✅', '❌', '🔐', '🔄', '🚀', '📝', '📦',
    '🔒', '💾', '📊', '🌐', '🔍', '👤', '🎫', '📋',
    '🔔', '💰', '🚌', '🍽️', '📁', '🏢', '🎯'
]

def remove_emojis_from_file(filepath):
    """Elimina emojis de un archivo preservando el formato."""
    try:
        # Leer con encoding UTF-8
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Reemplazar cada emoji
        for emoji in EMOJIS_TO_REMOVE:
            content = content.replace(emoji, '')
        
        # Solo escribir si hubo cambios
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8', newline='') as f:
                f.write(content)
            return True
        return False
    except Exception as e:
        print(f"Error procesando {filepath}: {e}")
        return False

def main():
    src_dir = os.path.join(os.path.dirname(__file__), 'src')
    
    modified_count = 0
    total_count = 0
    
    for root, dirs, files in os.walk(src_dir):
        for file in files:
            if file.endswith('.rs'):
                total_count += 1
                filepath = os.path.join(root, file)
                if remove_emojis_from_file(filepath):
                    relative_path = os.path.relpath(filepath, src_dir)
                    print(f"Modificado: {relative_path}")
                    modified_count += 1
    
    print(f"\nTotal archivos .rs: {total_count}")
    print(f"Archivos modificados: {modified_count}")

if __name__ == '__main__':
    main()
