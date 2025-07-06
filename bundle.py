import os
import sys

# --- НАСТРОЙКА ---
# Имя выходного файла
OUTPUT_FILENAME = "bundle.md"

# Директории, которые нужно полностью проигнорировать
EXCLUDE_DIRS = {
    ".git", "__pycache__", "node_modules", ".vscode",
    ".idea", "build", "dist", "venv", ".env", ".zig-cache", "zig-out", ".github", "target"
}

# Файлы, которые нужно проигнорировать
EXCLUDE_FILES = {
    OUTPUT_FILENAME, ".DS_Store", "package-lock.json"
}

# Расширения файлов, которые считаются бинарными и не будут включены
BINARY_EXTENSIONS = {
    '.png', '.jpg', '.jpeg', '.gif', '.bmp', '.ico', '.pdf',
    '.zip', '.tar', '.gz', '.rar', '.exe', '.dll', '.so',
    '.o', '.a', '.lib', '.class', '.pyc', '.pyo', '.wasm'
}
# --- КОНЕЦ НАСТРОЙКИ ---

def bundle_files(root_dir, output_file):
    """
    Рекурсивно обходит директорию и собирает содержимое файлов в один Markdown файл.
    """
    with open(output_file, "w", encoding="utf-8") as bundle:
        print(f"Создан файл для записи: {output_file}")

        # os.walk идеально подходит для рекурсивного обхода
        for dirpath, dirnames, filenames in os.walk(root_dir):
            # Удаляем исключенные директории из списка для обхода
            # Это эффективный способ пропустить целые ветки
            dirnames[:] = [d for d in dirnames if d not in EXCLUDE_DIRS]

            for filename in filenames:
                if filename in EXCLUDE_FILES:
                    continue

                # Получаем расширение файла
                _, extension = os.path.splitext(filename)
                if extension.lower() in BINARY_EXTENSIONS:
                    continue

                file_path = os.path.join(dirpath, filename)
                relative_path = os.path.relpath(file_path, root_dir)

                print(f"Добавляю: {relative_path}")

                try:
                    with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
                        content = f.read()

                    # Записываем заголовок с путем к файлу
                    bundle.write(f"## File: `{relative_path}`\n\n")
                    # Записываем содержимое в блок кода Markdown
                    # Указываем расширение для возможной подсветки синтаксиса
                    lang = extension[1:] if extension else ""
                    bundle.write(f"```{lang}\n")
                    bundle.write(content)
                    bundle.write("\n```\n\n---\n\n")
                except Exception as e:
                    print(f"  [!] Не удалось прочитать файл {relative_path}: {e}")

def main():
    if len(sys.argv) < 2:
        print("Ошибка: не указан путь к директории.")
        print("Использование: python bundle.py <путь_к_папке>")
        sys.exit(1)

    input_path = sys.argv[1]

    if not os.path.isdir(input_path):
        print(f"Ошибка: '{input_path}' не является директорией.")
        sys.exit(1)

    bundle_files(input_path, OUTPUT_FILENAME)
    print(f"\nГотово! Все файлы собраны в '{OUTPUT_FILENAME}'")

if __name__ == "__main__":
    main()
