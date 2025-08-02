# Icon Generator CLI

A cross-platform CLI tool for generating icons in various formats from a single source image. This tool creates icons for Windows (ICO), macOS (ICNS), Linux (PNG), Android, and iOS platforms.

## Features

- **Multi-platform icon generation**: Generates icons for Windows, macOS, Linux, Android, and iOS
- **Multiple formats**: ICO, ICNS, and PNG formats with appropriate sizes
- **Customizable sizes**: Generate custom PNG sizes as needed
- **Mobile platform support**: Automatically generates Android and iOS app icons
- **Background color support**: Add background colors for iOS icons
- **Cross-platform**: Runs on Windows, macOS, and Linux

## Installation

### From Source

1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Clone this repository:
   ```bash
   git clone <repository-url>
   cd icon-generator
   ```
3. Build and install:
   ```bash
   cargo install --path .
   ```

### Pre-built Binaries

Check the [Releases](https://github.com/your-repo/icon-generator/releases) page for pre-built binaries.

## Usage

### Basic Usage

```bash
icon-gen input-image.png
```

This will generate all icon formats in the `./icons` directory.

### Advanced Usage

```bash
# Specify output directory
icon-gen input-image.png -o /path/to/output

# Generate only Windows ICO format
icon-gen input-image.png --ico-only

# Generate only macOS ICNS format
icon-gen input-image.png --icns-only

# Generate custom PNG sizes only
icon-gen input-image.png -p 16,32,64,128,256

# Generate only desktop platform icons (Windows, macOS, Linux)
icon-gen input-image.png --desktop-only

# Generate only mobile platform icons (Android, iOS)
icon-gen input-image.png --mobile-only

# Set iOS background color
icon-gen input-image.png --ios-color "#FF5733"

# Generate icons for specific platforms
icon-gen input-image.png --windows --macos
icon-gen input-image.png --android --ios
icon-gen input-image.png --linux -p 16,32,64,128,256
```

### Command Line Options

```
Generate various icons for all major platforms

Usage: icon-gen [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to the source icon (squared PNG file with transparency)

Options:
  -o, --output <DIR>           Output directory [default: ./icons]
  -p, --png <SIZES>            Custom PNG icon sizes to generate. When set, only these sizes are generated
      --ico-only               Generate only ICO format (Windows icons)
      --icns-only              Generate only ICNS format (macOS icons)
      --desktop-only           Generate only desktop platform icons (Windows, macOS, Linux)
      --mobile-only            Generate only mobile platform icons (Android, iOS)
      --windows                Generate icons for Windows platform
      --macos                  Generate icons for macOS platform
      --linux                  Generate icons for Linux/Desktop platform
      --android                Generate icons for Android platform
      --ios                    Generate icons for iOS platform
      --ios-color <IOS_COLOR>  The background color for iOS icons (CSS color format) [default: #ffffff]
  -h, --help                   Print help
```

## Generated Icon Formats

### Windows (ICO)
- **File**: `icon.ico`
- **Sizes**: 16×16, 24×24, 32×32, 48×48, 64×64, 256×256
- **Format**: Multi-layer ICO file with PNG compression for 256×256

### macOS (ICNS)
- **File**: `icon.icns`
- **Sizes**: 16×16, 32×32, 128×128, 256×256, 512×512, 1024×1024 (including @2x variants)
- **Format**: Apple ICNS format

### Linux/Desktop (PNG)
- **Files**: `32x32.png`, `64x64.png`, `128x128.png`, `256x256.png`, `icon.png` (512×512)
- **Format**: PNG with transparency

### Android
- **Directory**: `android/mipmap-*/`
- **Files**: `ic_launcher.png` in each density folder
- **Densities**: mdpi (48×48), hdpi (72×72), xhdpi (96×96), xxhdpi (144×144), xxxhdpi (192×192)

### iOS
- **Directory**: `ios/`
- **Files**: Various `AppIcon-*` files for different iOS icon requirements
- **Sizes**: 20×20 to 1024×1024 with @1x, @2x, @3x variants
- **Background**: Applies specified background color (iOS requires opaque icons)

## Input Requirements

- **Format**: PNG (other formats supported by the `image` crate may work)
- **Size**: Must be square (width = height)
- **Recommended**: 1024×1024 or larger for best quality
- **Transparency**: Supported (except iOS icons which get background color applied)

## Examples

### Generate all icons from a 1024×1024 PNG
```bash
icon-gen app-icon-1024.png
```

### Generate only desktop icons (Windows, macOS, Linux)
```bash
icon-gen app-icon.png --desktop-only
```

### Generate custom sizes for a favicon
```bash
icon-gen favicon-source.png -p 16,32,48 -o favicon-output
```

### Generate iOS icons with custom background
```bash
icon-gen transparent-icon.png --ios-color "#2196F3"
```

### Generate only mobile app icons
```bash
icon-gen app-icon.png --mobile-only
```

### Generate icons for specific platforms only
```bash
# Generate only Windows icons
icon-gen app-icon.png --windows

# Generate only mobile app icons (alternative approach)
icon-gen app-icon.png --android --ios

# Generate desktop icons only (alternative approach)
icon-gen app-icon.png --windows --macos --linux

# Generate Linux icons with custom sizes
icon-gen app-icon.png --linux -p 16,24,32,48,64,128,256
```

## Output Structure

When generating all formats, the output directory structure will be:

```
icons/
├── icon.ico              # Windows ICO
├── icon.icns             # macOS ICNS
├── 32x32.png             # Linux desktop
├── 64x64.png
├── 128x128.png
├── 256x256.png
├── icon.png              # 512x512
├── android/              # Android icons
│   ├── mipmap-mdpi/
│   │   └── ic_launcher.png
│   ├── mipmap-hdpi/
│   │   └── ic_launcher.png
│   └── ...
└── ios/                  # iOS icons
    ├── AppIcon-20x20@1x.png
    ├── AppIcon-20x20@2x.png
    └── ...
```

## Building from Source

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- System dependencies for image processing (usually included)

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the Tauri CLI icon generation functionality
- Built with the Rust ecosystem's excellent image processing libraries
