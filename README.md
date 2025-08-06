# KASURI👘

![icon](./res/kasuri.ico)

The name "KASURI" is derived from the traditional Japanese "Kasuri" (絣) pattern used in textiles. 
It also carries the self-deprecating wordplay of "Kasu na Launcher" in Japanese, which humorously refers to itself as a "poor/useless launcher". 😅


![Version](https://img.shields.io/badge/version-0.2.1-blue.svg)
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

You can download the latest release from the [GitHub Releases page](https://github.com/sabiz/KASURI/releases).

1. Download the Windows installer (e.g. `KASURI_<version>_x64-setup.exe`) from the latest release.
2. Run the installer and follow the on-screen instructions.
3. After installation, you can launch KASURI from the Start Menu or the installation directory.

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

KASURI settings can be easily configured from the in-app **Settings Screen**.

To open the Settings Screen, right-click the KASURI icon in the system tray and select "Settings" from the menu.

- You can intuitively change major settings such as hotkey, window width, auto startup, log level, application search paths, and application aliases via the GUI.
- Changes are applied immediately (some items may require a restart).
- There is no need to edit the `settings.toml` file directly.

### System Tray Menu Items

- **Settings**: Opens the settings window where you can configure hotkey, window width, auto startup, log level, application search paths, and application aliases.
- **Reload**: Reloads the application cache (e.g., re-scans for applications). Use this if you have installed or removed applications and want to update the list.
- **Open Log Directory**: Opens the folder where log files are stored. Useful for troubleshooting or checking logs.
- **Exit**: Exits KASURI completely.

## :chicken:FAQ



### How do I change the hotkey?

You can change the hotkey from the Settings Screen. The format follows the [Tauri global shortcut](https://tauri.app/v1/api/js/globalShortcut/) convention.



### How do I set an alias for an application?

You can add application aliases from the "Application Aliases" section in the Settings Screen. Once set, you can search for the application using either its original name or the alias.



### Why doesn't KASURI find some of my applications?

KASURI searches for applications in the directories specified in the Settings Screen. Please make sure all necessary directories are included in your configuration.



### How can I make KASURI launch on startup?

Enable the "Auto Startup" option in the Settings Screen.



### Where are my settings stored?

Settings are stored internally in `%APPDATA%\Local\KASURI\settings.toml`, but you should normally use the Settings Screen for all configuration.

### Where are log files stored?

Log files are stored in the `logs` directory within the application installation folder. The application automatically creates this directory if it doesn't exist.

## License

[MIT License](LICENSE) :copyright: [sAbIz](https://github.com/sabiz):jp:
