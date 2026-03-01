# Pastry

<p align="center">
  <img src="screenshots/1.jpg" alt="Pastry screenshot 1" width="32%" />
  <img src="screenshots/2.jpg" alt="Pastry screenshot 2" width="32%" />
  <img src="screenshots/3.jpg" alt="Pastry screenshot 3" width="32%" />
</p>

Pastry is a clipboard management application. It helps you manage your clipboard history, run scripts, and build simple workflows.

<p align="center"><a href="README.zh-CN.md">简体中文文档</a></p>

## Core Features

- **Clipboard history**: Save text, rich text, images
- **Favorites**: Pin frequently used items
- **Scripts**: Transform clipboard content with JavaScript
- **JSON tools**: Format JSON and run JSONPath queries
- **Color picker**: Detect and pick common color formats
- **LAN sync**: Access and sync clipboard data over local network
- **Workflows**: Build automations with hotkey/script/clipboard/file nodes
- **System integration**: Global hotkey, tray mode, startup options, always-on-top
- **Customization**: Multi-language support and light/dark themes

## Basic Usage

- Copy content, then open Pastry with your global hotkey
- Click history items to copy again or pin favorites
- Use scripts with `input` / `output` to process clipboard content
- Build workflows by connecting nodes and assigning triggers

## Documentation

- [Script guide](docs/SCRIPT_GUIDE.md)
- [Workflow guide](docs/WORKFLOW_GUIDE.md)
- [Color picker guide](docs/COLOR_GUIDE.md)
- [LAN sync guide](docs/LAN_SYNC_GUIDE.md)
- [JSONPath guide](docs/JSONPATH_GUIDE.md)

## Troubleshooting (macOS)

If opening `Pastry.app` shows "is damaged and can't be opened. You should move it to the Bin.", run:

```bash
sudo xattr -r -d com.apple.quarantine /Applications/Pastry.app
```

Then reopen the app.

## License

GNU General Public License v3.0.
