```
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
```

## Project Overview
This is a Rust rewrite of the Decrypt tool, which is designed for educational purposes to demonstrate transparent encryption bypass techniques. It uses FLTK-rs for the UI and is Windows-only.

**Important Disclaimer**: This software is for learning and communication only. It has not been tested in production encryption environments, and must not be used for commercial production,涉密 environments, or any illegal activities.

## Architecture
The codebase is split into two core parts:
1. **Thin binary wrappers** (`src/`):
   - `decrypt.rs`: Main application entry point, loads the dynamic library and calls `main_app()`
   - `launcher.rs`: Launcher entry point, loads the dynamic library and calls `launcher_app()`
   - Both binaries only handle library loading, all actual logic lives in the dynamic library

2. **Dynamic library** (`lib/`):
   - `lib.rs`: Exposes the public C ABI entry points `main_app()` and `launcher_app()`
   - `window.rs`: Main application UI and business logic
   - `decrypt.rs`: Decryption core functionality
   - `launcher.rs`: Launcher UI logic
   - `widget.rs`: Generated FLTK UI code (built from `ui/widget.fl` at compile time)
   - `theme.rs`: FLTK theme/styling configuration
   - `path.rs`: File path handling utilities
   - `loadicon.rs`: Icon loading functionality

## Build Commands
### Standard builds
```bash
# Build release version (optimized, no console window)
cargo build --release

# Build debug version (with console window for development)
cargo build
```

### Build details
- The build script (`build.rs`) automatically generates Rust UI code from `ui/widget.fl` using `fl2rust`
- On Windows, it embeds the application icon, manifest, and version resources from `resource/decrypt.rc`
- Release builds are optimized for size: stripped, LTO enabled, panic=abort, opt-level="z"

## Key Files
- `Cargo.toml`: Project configuration, dependencies, and build profiles
- `ui/widget.fl`: FLTK UI designer file, edit this to modify the UI layout
- `resource/decrypt.rc`: Windows resource file for icons, version info, and manifest
