# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based CLI tool called `icon-gen` that generates cross-platform icons from a single source image. It creates icons for Windows (ICO), macOS (ICNS), Linux (PNG), Android, and iOS platforms with various customization options.

## Common Development Commands

### Building and Testing
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .

# Run the CLI
cargo run -- input-image.png
```

### Testing Development Features
```bash
# Generate icons with development badge
cargo run -- input-image.png --dev-mode

# Test specific platform generation
cargo run -- input-image.png --android --ios
```

## Architecture

### Core Modules
- **src/main.rs**: CLI argument parsing and entry point
- **src/icon_gen.rs**: Main icon generation logic (1400+ lines)
- **src/contents_json.rs**: Apple Asset Catalog JSON structure definitions
- **src/bin/**: Utility binaries for testing and development

### Key Dependencies
- `image`: Image processing and format conversion
- `icns`: macOS ICNS format generation
- `clap`: Command-line argument parsing
- `serde`/`serde_json`: JSON serialization for Apple Asset Catalogs
- `css-color`: CSS color parsing for iOS/Android background colors
- `rusttype`: Text rendering for development badges

### Platform Support Architecture

The tool uses a flexible flag-based system to determine which platforms to generate icons for:

1. **Flag Logic**: `should_invoke_ios_writer()` and `should_invoke_macos_writer()` functions determine when to generate platform-specific icons
2. **Generation Modes**: 
   - `generate_all()`: Default mode when no specific flags are set
   - `generate_platforms()`: When specific platform flags are used
   - `generate_desktop_only()` / `generate_mobile_only()`: For platform category restrictions

### Icon Generation Flow

1. **Image Loading**: Validates input is square and loads with `image` crate
2. **Platform Detection**: Determines which platforms to generate based on CLI flags
3. **Size Generation**: Resizes images using Lanczos3 filtering for quality
4. **Format Encoding**: Creates platform-specific formats (ICO, ICNS, PNG)
5. **Asset Catalogs**: Generates Apple Contents.json files for iOS/macOS
6. **Development Badge**: Applies visual DEV overlay if `--dev-mode` is enabled

### Android Icon Support

- **Standard Icons**: Square icons in density-specific folders (mipmap-*)
- **Round Icons**: Circular masked versions (ic_launcher_round.png)
- **Adaptive Icons**: Foreground/background layers with XML configuration
- **Densities**: mdpi (48px), hdpi (72px), xhdpi (96px), xxhdpi (144px), xxxhdpi (192px)

### iOS Integration

- **Background Colors**: iOS icons get opaque backgrounds (configurable via `--ios-color`)
- **Asset Catalog**: Automatically generates Contents.json with proper metadata
- **Icon Roles**: Supports notification center, spotlight, app launcher, and companion settings
- **Size Variants**: Handles @1x, @2x, @3x scaling for all iOS icon sizes

### Development Badge Feature

The `--dev-mode` flag adds a visual "DEV" badge to all generated icons:
- Red semi-transparent ribbon at bottom
- Diagonal stripe pattern for visual distinction
- Automatically scales with icon size
- Applied before encoding to all formats

### Apple Asset Catalog Integration

The `contents_json.rs` module provides comprehensive Apple Asset Catalog support:
- **ContentsFile**: Root structure with images, info, and properties
- **ImageEntry**: Individual icon metadata with idiom, scale, size, role
- **Platform Support**: Works for both iOS and macOS with proper folder structure
- **Xcode Ready**: Generated files work immediately in Xcode projects

## Input Requirements

- **Format**: PNG files (other formats supported by `image` crate may work)
- **Dimensions**: Must be square (width == height)
- **Recommended Size**: 1024×1024 or larger for best quality
- **Transparency**: Supported (except iOS which gets background color applied)

## Output Structure

```
icons/
├── icon.ico              # Windows multi-layer ICO
├── icon.icns             # macOS ICNS with all sizes
├── Contents.json         # macOS Asset Catalog metadata
├── 32x32.png             # Linux desktop icons
├── 64x64.png
├── 128x128.png
├── 256x256.png
├── icon.png              # 512x512 Linux icon
├── android/              # Android icons by density
│   ├── mipmap-mdpi/
│   ├── mipmap-hdpi/
│   └── ...
└── ios/                  # iOS icons with asset catalog
    ├── Contents.json     # iOS Asset Catalog metadata
    ├── AppIcon-*.png     # Various iOS icon sizes
    └── ...
```

## Error Handling

The application uses `anyhow::Result` for comprehensive error handling with context messages. Common error points include:
- Image loading and validation
- File I/O operations
- Directory creation
- Platform-specific encoding operations

## Testing

The project includes integration tests in the `tests/` directory:
- `integration_test.rs`: General functionality testing
- `dev_badge.rs`: Development badge feature testing
- Test utilities in `src/bin/` for creating test images and verifying badge functionality