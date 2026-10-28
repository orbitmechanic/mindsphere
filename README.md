# Mindsphere

A local-first, open-source sphere mind mapper built with Godot. Every thought gets a place on a globe; turn the sphere, group related ideas, and draw paths between them. A flat board view shows the same map as columns.

Mindsphere is an early but usable prototype. It has no account system, no cloud dependency, and no telemetry. Spheres are plain JSON files on your machine.

**Try it in the browser:** <https://orbitmechanic.github.io/mindsphere/>

## What it does

- **3D sphere view** — orbit, zoom, drag thoughts across the surface, and follow connection paths.
- **2D board view** — groups become columns, with notes and links visible at a glance.
- **Thought inspector** — edit titles and notes, move thoughts between groups, and create connections.
- **Local library** — create, rename, duplicate, delete, import, and export spheres.
- **Autosave** — changes are saved locally after a short idle delay.
- **No lock-in** — exported JSON is readable, portable, and easy to back up.

## Run it

Requires **Godot 4.7 or newer** (standard build, no .NET required).

```sh
godot --headless --path . --import
godot --path .
```

On first launch, Mindsphere creates a small demo sphere called “Thinking it through.”

### Browser build

The GitHub Pages build runs the same project through Godot's Web export. Browser spheres are stored in the site's `localStorage`; nothing is uploaded. Use **Export** to download a sphere as JSON. **Import** opens a browser dialog where you can choose a `.json` file or paste exported JSON; you can also drag a `.json` sphere file onto the window. The web build does not use the desktop file dialogs.

To build and serve it locally:

```sh
godot --headless --path . --export-release "Web" build/web/index.html
python3 -m http.server 8765 --directory build/web
```

## Controls

| Action | Control |
| --- | --- |
| Orbit the sphere | Drag the background |
| Zoom | Scroll |
| Move a thought | Drag it across the sphere |
| Edit a thought | Select it, then use the inspector |
| New thought | `N` or the button in the top bar |
| Switch sphere / board | `1` / `2` |
| Search spheres | `Ctrl+F` |
| Export sphere | `Ctrl+E` |
| Save now | `Ctrl+S` |
| Delete selected thought | `Delete` or `Backspace` |
| Clear selection / close modal | `Esc` |

## Your data

Spheres live in the Godot user data directory:

- **Linux:** `~/.local/share/godot/app_userdata/Mindsphere/spheres/`
- **macOS:** `~/Library/Application Support/Godot/app_userdata/Mindsphere/spheres/`
- **Windows:** `%APPDATA%\Godot\app_userdata\Mindsphere\spheres\`

Settings are stored beside the `spheres/` folder in `settings.json`. Use **Export as JSON** for a portable copy, or point your own backup/sync tool at the folder. Mindsphere itself never talks to a cloud service.

In the browser build, the library and settings live in `localStorage` under `mindsphere.library.v1` and `mindsphere.settings.v1`. Clearing site data removes them, so export anything you want to keep.

## Project layout

```text
main.tscn                 Main scene
scripts/main.gd           App shell, UI, persistence, shortcuts
scripts/sphere_view.gd    3D globe, picking, dragging, labels, links
scripts/board_view.gd     2D grouped board
scripts/sphere_store.gd   Local JSON model and storage
scripts/ui_factory.gd     Theme and shared controls
shaders/                  Background, globe, and atmosphere shaders
tests/                    Headless probes and screenshot helpers
```

## Tests

```sh
godot --headless --path . -s res://tests/probe.gd
godot --headless --path . -s res://tests/scene_probe.gd
```

Both probes print `PROBE_RESULT=PASS` on success. Screenshot helpers live in `tests/capture.gd` and `tests/capture_states.gd`.

## License

MIT. See [LICENSE](LICENSE).
