# Инструкция по установке и запуску

## Установка Rust

Если у вас не установлен Rust, выполните следующие шаги:

### Windows

1. **Установите Visual Studio Build Tools (ОБЯЗАТЕЛЬНО!):**
   
   **Вариант A: Visual Studio Build Tools (рекомендуется):**
   - Скачайте "Build Tools for Visual Studio" с https://visualstudio.microsoft.com/downloads/
   - Выберите "Build Tools for Visual Studio 2022"
   - При установке выберите компонент "Desktop development with C++"
   - Это установит компилятор MSVC, необходимый для Rust на Windows
   
   **Вариант B: Полная Visual Studio:**
   - Скачайте Visual Studio Community (бесплатная) с https://visualstudio.microsoft.com/
   - При установке выберите рабочую нагрузку "Desktop development with C++"
   
   **Вариант C: Альтернатива - GNU toolchain (если не хотите устанавливать Visual Studio):**
   ```powershell
   # Установите MSYS2 с https://www.msys2.org/
   # Затем установите Rust с GNU toolchain:
   rustup toolchain install stable-x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   ```

2. **Скачайте установщик Rust:**
   - Перейдите на https://rustup.rs/
   - Скачайте и запустите `rustup-init.exe`

3. **Установите Rust:**
   - Запустите установщик
   - Нажмите Enter для установки по умолчанию
   - Rust установится в `%USERPROFILE%\.cargo\bin`

4. **Перезапустите PowerShell/терминал:**
   - Закройте и откройте заново PowerShell или командную строку
   - Это необходимо для обновления переменных окружения PATH

5. **Проверьте установку:**
   ```powershell
   rustc --version
   cargo --version
   ```

### Альтернативный способ (через Chocolatey)

Если у вас установлен Chocolatey:
```powershell
choco install rust
```

## Сборка проекта

После установки Rust выполните:

```powershell
# Перейдите в директорию проекта
cd "E:\Cursor projects\Suminagasi"

# Соберите проект в режиме release
cargo build --release

# Или запустите сразу
cargo run --release
```

## Требования

- **Windows 10/11** (64-bit)
- **Visual Studio Build Tools** или **Visual Studio Community** с компонентом "Desktop development with C++"
- **GPU** с поддержкой DirectX 11 или Vulkan
- **Rust** (последняя стабильная версия)

## Возможные проблемы

### Ошибка "linker `link.exe` not found"

**Проблема:** Rust не может найти компилятор C++ для Windows.

**Решение:**
1. Установите Visual Studio Build Tools (см. шаг 1 выше)
2. Или используйте GNU toolchain:
   ```powershell
   rustup toolchain install stable-x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   ```
3. Перезапустите терминал после установки

### Cargo не найден после установки

1. Убедитесь, что вы перезапустили терминал/PowerShell
2. Проверьте переменную PATH:
   ```powershell
   $env:PATH -split ';' | Select-String cargo
   ```
3. Если cargo не в PATH, добавьте вручную:
   ```powershell
   $env:PATH += ";$env:USERPROFILE\.cargo\bin"
   ```

### Ошибки компиляции

Если возникают ошибки компиляции, убедитесь что:
- Используется последняя версия Rust: `rustup update`
- Все зависимости загружены: `cargo fetch`

### Проблемы с GPU

Если приложение не запускается из-за проблем с GPU:
- Убедитесь, что установлены последние драйверы видеокарты
- Проверьте поддержку DirectX 11 или Vulkan

## Дополнительная информация

- Документация Rust: https://doc.rust-lang.org/
- Документация wgpu: https://docs.rs/wgpu/
