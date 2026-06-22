# Tauri Template

A minimal Tauri desktop application template with a polished UI, extracted from a production app.

## Features

- **Borderless window** — Transparent background with rounded corners
- **macOS overlay title bar** — Hidden title with custom traffic light position
- **Drag & resize** — Drag region with double-click to maximize
- **Sidebar navigation** — Compact sidebar with nav buttons and theme toggle
- **Status bar** — Bottom bar with status indicator and clock
- **Light / Dark theme** — CSS variable-based theming with localStorage persistence
- **Cross-monitor support** — Automatic resize and centering when dragging between monitors

## Project Structure

```
├── frontend/                    # React + Vite + Tailwind
│   ├── src/
│   │   ├── App.tsx              # Main layout (sidebar + content + status bar)
│   │   ├── main.tsx             # Entry point with ThemeProvider
│   │   ├── index.css            # Global styles, CSS variables, themes
│   │   ├── components/
│   │   │   ├── StatusBar.tsx    # Bottom status bar
│   │   │   └── Layout/
│   │   │       └── PageHeader.tsx
│   │   └── stores/
│   │       └── themeStore.ts    # Light/dark theme state
├── src-tauri/                   # Tauri backend (Rust)
│   ├── src/
│   │   ├── lib.rs               # Window management, shadow, focus, cross-monitor
│   │   ├── main.rs              # Entry point
│   │   └── commands/
│   │       └── window.rs        # minimize/maximize/close/drag IPC commands
│   ├── tauri.conf.json          # Window config (transparent, overlay, traffic lights)
│   └── capabilities/
│       └── default.json         # Permissions
└── package.json
```

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.75
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Install Dependencies

```bash
cd frontend
npm install
```

### Development

```bash
npm run tauri:dev
```

### Build

```bash
npm run tauri:build
```

## Customization

### Window Configuration

Edit `src-tauri/tauri.conf.json`:

```json
{
  "app": {
    "windows": [
      {
        "transparent": true,
        "decorations": true,
        "titleBarStyle": "Overlay",
        "hiddenTitle": true,
        "trafficLightPosition": { "x": 16, "y": 22 }
      }
    ]
  }
}
```

### Themes

CSS variables in `frontend/src/index.css` control the entire color scheme. Toggle via `data-theme` attribute on `<html>`.

### Sidebar

The sidebar width is controlled by the `--sidebar-width` CSS variable (default: `180px`).

### Status Bar

The `StatusBar` component in `frontend/src/components/StatusBar.tsx` can be customized or replaced.
