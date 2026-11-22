# RustBlocks

**RustBlocks** is an original, single-player 3D voxel sandbox written in Rust. It is inspired by the broad block-building genre, but it does not use Minecraft's source code, name, textures, sounds, or other proprietary assets.

> **Build status:** early prototype source. The first Windows CI run will verify compilation and tests. It is not a complete Minecraft replacement, Ultra Edition, or a zero-bug release.

## Current gameplay

- First-person 3D camera with mouse look
- Procedurally generated block terrain and trees
- Grass, dirt, stone, wood, leaves, sand and coal-ore blocks
- WASD movement, jumping, gravity and basic block collision
- Mine blocks and collect them into the hotbar
- Place blocks from the selected hotbar slot
- Seven selectable block slots, keyboard and mouse-wheel selection
- Crosshair and target-block outline
- Chunk-based combined meshes with hidden internal faces omitted
- Automatic world/player/inventory saving in the per-user application-data folder
- Rust unit tests for core world and block rules

## Controls

| Input | Action |
|---|---|
| Click inside game | Capture mouse / play |
| WASD | Move |
| Mouse | Look around |
| Space | Jump |
| Left Shift | Move faster |
| Left mouse | Mine targeted block |
| Right mouse | Place selected block |
| 1–7 | Select hotbar slot |
| Mouse wheel | Cycle hotbar slot |
| Escape | Release mouse / pause overlay |

## Windows installer build

The workflow in `.github/workflows/windows-installer.yml` runs on pushes to `main`. It runs Rust tests, compiles a Windows x64 release executable, packages it as `RustBlocks-Setup.exe`, and uploads the installer as the `RustBlocks-Windows-Setup` workflow artifact. Pushing a version tag such as `v0.1.0` also publishes the installer as a GitHub Release asset.

Open the repository's **Actions** tab to inspect the build status and download a successful installer artifact.

## Build on Windows 10/11

1. Install **Rust stable** from [rustup.rs](https://rustup.rs/). If prompted, install the Visual Studio C++ build tools.
2. Double-click `build_windows.bat`, or open a terminal in this folder and run:

   ```powershell
   cargo build --release
   ```

3. Run `target\release\rustblocks.exe`.

The first build downloads Macroquad and its dependencies, so an internet connection is required. Subsequent builds can work offline while dependencies remain cached.

## Run tests

```powershell
cargo test
```

## Save data

The world is stored per user so saving works without administrator permissions. On Windows the save is `%LOCALAPPDATA%\RustBlocks\rustblocks_world.txt`. Back up this file before deleting or replacing it. To start a fresh world, close the game and delete the save file.

## Important limitations

This prototype does **not** yet include multiplayer, mobs, combat, crafting, furnaces, redstone, fluids, caves, multiple dimensions, farming, achievements, advanced lighting, textured block faces, or the broad item/recipe systems found in a full commercial sandbox game. Ultra graphics and Low/Balanced/High/Ultra profiles are planned, not implemented. Low-end compatibility is a goal, not a verified guarantee; actual hardware testing is required.

## Project structure

```text
rustblocks/
├── .github/workflows/windows-installer.yml
├── src/main.rs
├── Cargo.toml
├── README.md
├── ROADMAP.md
├── CHANGELOG.md
├── LOW_END_COMPATIBILITY_PLAN.md
├── rustblocks-installer.iss
├── build_windows.bat
├── run_windows.bat
├── LICENSE
└── .gitignore
```
