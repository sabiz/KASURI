# KASURI👘

The name "KASURI" is derived from the traditional Japanese "Kasuri" (絣) pattern used in textiles. 
It also carries the self-deprecating wordplay of "Kasu na Launcher" in Japanese, which humorously refers to itself as a "poor/useless launcher". 😅


![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows-brightgreen.svg)
![License](https://img.shields.io/badge/license-MIT-yellow.svg)
![Built with](https://img.shields.io/badge/built%20with-Rust%20%2B%20Tauri-orange.svg)

## :sparkles:Features

- **Fast Fuzzy Search**: Quickly find and launch applications with just a few keystrokes
- **System Tray Integration**: Always accessible from your system tray
- **Global Hotkey**: Open the launcher from anywhere with a configurable shortcut key
- **Windows Store App Support**: Search and launch both traditional and Windows Store applications
- **Application Alias Support**: Assign aliases to applications for easier searching
- **Automatic Startup Option**: Start KASURI with Windows
- **Lightweight**: Minimal resource usage when idle

## :egg:Requirement

- Windows 10 or newer
- 50MB of free disk space
- Administrative privileges (for installation)

## :hatching_chick:Installation

### From Releases

Currently not available. Release packages will be provided in future updates.

### Building from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) (v16 or newer)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- Tauri development dependencies (run `cargo install tauri-cli` for command-line tools)

#### Build Steps

```bash
# Clone the repository
git clone https://github.com/sabiz/kasuri.git
cd kasuri

# Install dependencies
npm install

# Development build
npm run tauri dev

# Production build
npm run tauri build
```

## :hatched_chick:Getting started


1. Press your configured hotkey (default is Alt+Space) to open the launcher
2. Type to search for applications
3. Use arrow keys to navigate between suggestions
4. Press Enter to launch the selected application

### Configuration

Settings can be configured in `settings.toml` located in the KASURI application data directory:

- `shortcut_key`: Global hotkey to open the launcher
- `width`: Width of the launcher window
- `auto_startup`: Whether to start KASURI when Windows starts
- `log_level`: Logging verbosity (error, warn, info, debug)
- `application_search_path_list`: Directories to search for applications
- `application_search_interval_on_startup_minute`: How often to refresh the application cache
- `application_name_aliases`: List of aliases for applications. Each alias entry should specify the application's path and the desired alias name. Example:

```toml
[[application_name_aliases]]
path = "C:/Program Files/ExampleApp/Example.exe"
alias = "exapp"
```

> **Note:**
> The `path` specified in each `application_name_aliases` entry must exactly match the path of an application discovered via `application_search_path_list` (such as `.lnk` or `.exe` files). If the path does not match, the alias will not be applied.

## :chicken:FAQ

### How do I change the hotkey?

Edit the `shortcut_key` setting in the `settings.toml` file. The format follows the [Tauri global shortcut](https://tauri.app/v1/api/js/globalShortcut/) convention.

### How do I set an alias for an application?

Add an entry to the `application_name_aliases` section in your `settings.toml` file. Specify the application's path and the alias you want to use. For example:

```toml
[[application_name_aliases]]
path = "C:/Program Files/ExampleApp/Example.exe"
alias = "exapp"
```
You can now search for the application using either its original name or the alias.

### Why doesn't KASURI find some of my applications?

KASURI searches for applications in the directories specified in `application_search_path_list`. Make sure all your application directories are included.

### How can I make KASURI launch on startup?

Set `auto_startup = true` in your `settings.toml` file.

### Where are my settings stored?

Settings are stored in `%APPDATA%\Local\KASURI\settings.toml`.

### Where are log files stored?

Log files are stored in the `logs` directory within the application installation folder. The application automatically creates this directory if it doesn't exist.

## License

[MIT License](LICENSE) :copyright: [sAbIz](https://github.com/sabiz):jp:
