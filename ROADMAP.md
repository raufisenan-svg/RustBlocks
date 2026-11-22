# RustBlocks roadmap

This roadmap distinguishes the existing prototype from planned work. A milestone is not complete until its implementation and tests are committed.

## Prototype baseline (current source)

- First-person movement, basic collision, jumping and mouse look
- Small procedural voxel terrain, trees and seven block types
- Mining, placing, hotbar selection and local world persistence
- Basic combined chunk meshes with hidden faces omitted
- Windows x64 build and installer workflow (build must pass in GitHub Actions before calling it verified)

## Next milestones

1. **Stability and portability:** build on clean Windows runners, fix compiler/test findings, verify installation and saves for standard user accounts, and test on integrated graphics.
2. **World systems:** seed selection, larger chunk streaming, deterministic terrain generation, caves, and safer versioned save files.
3. **Gameplay:** inventory screen, crafting, tools, health, day/night cycle, sound, and basic creatures.
4. **Graphics profiles:** implement Low, Balanced, High and Ultra settings with adjustable view distance, shadows, texture filtering and effects; profile each setting on representative hardware.
5. **Content and polish:** original textures/audio, menus, settings persistence, accessibility, localization and in-game help.

## Compatibility policy

The target is Windows 10/11 x64. Low-end compatibility is a design goal, not a verified guarantee. Do not advertise a minimum GPU, frame rate, or universal compatibility until measurements are available from real hardware. The current prototype has no advanced PBR, SSAO, dynamic shadow system, volumetric clouds, multiplayer, or Ultra rendering pipeline.
