# RustBlocks Windows compatibility and performance plan

## Current status
The current prototype is a Macroquad application. This installer workflow packages that prototype; it does not implement the planned Bevy renderer or the Ultra Edition feature set.

## Windows target
- Windows 10/11 64-bit (x86_64)
- MSVC toolchain
- Build artifact: `rustblocks.exe`
- Installer: `RustBlocks-Setup.exe` (Inno Setup)

## Performance profiles to implement and validate

### Low
- Short view distance and small chunk radius
- Low internal render scale
- Disable ambient occlusion, bloom, volumetric effects and expensive reflections
- Reduced shadow quality or shadows disabled
- Conservative vegetation and particle counts
- Frame-rate cap option

### Balanced
- Medium view distance and render scale
- Basic shadows and ambient occlusion where supported
- Moderate vegetation and particles

### Ultra
- Higher view distance and render scale
- Higher quality shadows, ambient occlusion, reflections and post-processing where supported

## Compatibility policy
Do not assume every GPU supports the same graphics APIs or shader features. Query adapter capabilities at startup, provide a safe fallback renderer/settings profile, and show a useful error if no supported graphics backend exists.

## Required test matrix
Test on integrated graphics and older dedicated GPUs, multiple driver versions, 4/8/16 GB RAM systems, windowed/fullscreen modes, and clean Windows installations. Record frame time, memory use, crashes and save/load integrity. Do not advertise a minimum spec until these tests have been run.
