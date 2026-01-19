# Icon Generator CLI

A cross-platform CLI tool for generating icons in various formats from a single source image. This tool creates icons for Windows (ICO), macOS (ICNS), Linux (PNG), Android, iOS, and Tauri platforms.

## Features

- **Multi-platform icon generation**: Generates icons for Windows, macOS, Linux, Android, iOS, and Tauri
- **Platform-specific directories**: Each platform gets its own dedicated subdirectory for easy drag-and-drop
- **Multiple formats**: ICO, ICNS, and PNG formats with appropriate sizes
- **Customizable sizes**: Generate custom PNG sizes as needed
- **Mobile platform support**: Automatically generates Android and iOS app icons
- **Tauri support**: Dedicated `tauri-desktop` folder for Tauri projects
- **Background color support**: Add background colors for iOS icons
- **Development badge support**: Add a "DEV" badge overlay to icons for development builds
- **Apple Asset Catalog support**: Automatic generation of `Contents.json` for iOS and macOS
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

This will generate all icon formats in a directory named `icon-generator-input-image/` (based on the source filename).

### Advanced Usage

```bash
# Specify output directory
icon-gen input-image.png -o /path/to/output

# Generate only Tauri desktop icons (recommended for Tauri projects)
icon-gen input-image.png --tauri-desktop

# Generate custom PNG sizes only
icon-gen input-image.png -p 16,32,64,128,256

# Generate only desktop platform icons (Windows, macOS, Linux)
icon-gen input-image.png --desktop-only

# Generate only mobile platform icons (Android, iOS)
icon-gen input-image.png --mobile-only

# Set iOS background color
icon-gen input-image.png --ios-color "#FF5733"

# Add development/debug badge to all generated icons
icon-gen input-image.png --dev-mode

# Add development badge with specific bug type
icon-gen input-image.png --dev-mode --dev-bug spider

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
  -o, --output <DIR>           Output directory (default: icon-generator-{source-name})
  -p, --png <SIZES>            Custom PNG icon sizes to generate. When set, only these sizes are generated
      --desktop-only           Generate only desktop platform icons (Windows, macOS, Linux)
      --mobile-only            Generate only mobile platform icons (Android, iOS)
      --tauri-desktop          Generate Tauri desktop icons (tauri-desktop folder with icons for src-tauri/icons)
      --windows                Generate icons for Windows platform
      --macos                  Generate icons for macOS platform
      --linux                  Generate icons for Linux/Desktop platform
      --android                Generate icons for Android platform
      --ios                    Generate icons for iOS platform
      --ios-color <IOS_COLOR>  The background color for iOS icons (CSS color format) [default: #ffffff]
      --dev-mode               Add a development/debug badge to all generated icons
      --dev-bug <BUG>          Bug type to use for dev badge (cockroach, ladybug, moth, spider, caterpillar) - only effective with --dev-mode [default: moth]
  -h, --help                   Print help
```

## Generated Icon Formats

### Windows (ICO)

- **Directory**: `windows/`
- **File**: `icon.ico`
- **Sizes**: 16×16, 24×24, 32×32, 48×48, 64×64, 256×256
- **Format**: Multi-layer ICO file with PNG compression for 256×256

### macOS (ICNS)

- **Directory**: `macos/`
- **Files**: `icon.icns`, `Contents.json`
- **Sizes**: 16×16, 32×32, 128×128, 256×256, 512×512, 1024×1024 (including @2x variants)
- **Format**: Apple ICNS format

### Linux/Desktop (PNG)

- **Directory**: `linux/`
- **Files**: `32x32.png`, `64x64.png`, `128x128.png`, `256x256.png`, `icon.png` (512×512)
- **Format**: PNG with transparency

### Tauri Desktop

- **Directory**: `tauri-desktop/`
- **Files**: `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.ico`, `icon.icns`
- **Purpose**: Ready-to-use icons for Tauri's `src-tauri/icons` folder

### Android

- **Directory**: `android/mipmap-*/`
- **Files**: `ic_launcher.png` in each density folder
- **Densities**: mdpi (48×48), hdpi (72×72), xhdpi (96×96), xxhdpi (144×144), xxxhdpi (192×192)

### iOS

- **Directory**: `ios/`
- **Files**: Various `AppIcon-*` files for different iOS icon requirements
- **Sizes**: 20×20 to 1024×1024 with @1x, @2x, @3x variants
- **Background**: Applies specified background color (iOS requires opaque icons)
- **Asset Catalog**: Automatic generation of `Contents.json` for Xcode compatibility

## Development Badge Feature

The `--dev-mode` flag (alias: `--debug`) adds a visual development badge overlay to all generated icons. This feature is useful for:

- **Development builds**: Easily distinguish development icons from production icons
- **Testing environments**: Visual indicator for QA and staging builds
- **Beta releases**: Mark pre-release versions with a clear visual indicator

The badge uses charming bug images (moth, cockroach, ladybug, spider, or caterpillar) that are automatically scaled and centered on each icon. The default bug is a moth with random rotation for visual variety.

### Bug Options

- **`moth`** (default): Classic moth with random rotation (0-360°)
- **`cockroach`**: Cockroach with fixed orientation
- **`ladybug`**: Ladybug with fixed orientation
- **`spider`**: Spider with fixed orientation
- **`caterpillar`**: Caterpillar with fixed orientation

![](images/bugs.png)

### Usage Examples

```bash
# Use default moth with random rotation
icon-gen source.png --dev-mode

# Specify different bug types
icon-gen source.png --dev-mode --dev-bug spider
icon-gen source.png --dev-mode --dev-bug ladybug
icon-gen source.png --dev-mode --dev-bug cockroach
icon-gen source.png --dev-mode --dev-bug caterpillar

# Combine with other options
icon-gen source.png --dev-mode --dev-bug moth --mobile-only
```

### Features

- **Smart Scaling**: Bugs automatically scale to 1/4 of the icon size
- **Perfect Centering**: Bugs are precisely centered on each icon
- **Random Rotation**: Moth gets random rotation for visual variety
- **Cross-Platform**: Works across all supported formats (ICO, ICNS, PNG, Android, iOS)
- **Alpha Blending**: Seamless overlay with proper transparency handling

## Apple Asset Catalog Support

This tool automatically generates Apple's `Contents.json` files for both iOS and macOS platforms, making the generated icons ready for use in Xcode projects without any additional configuration.

### iOS Asset Catalog (`{output}/ios/Contents.json`)

- **Purpose**: Defines metadata for iOS app icons including sizes, scales, idioms, and roles
- **Compatibility**: Works with all iOS devices (iPhone, iPad, and iOS Marketing)
- **Roles**: Includes proper roles for notification center, spotlight, app launcher, and companion settings
- **No Xcode Post-processing**: Icons are immediately ready for use in iOS projects

### macOS Asset Catalog (`{output}/Contents.json`)

- **Purpose**: Defines metadata for macOS app icons with proper scaling and size information
- **Compatibility**: Supports all macOS icon sizes from 16×16 to 1024×1024 with @1x and @2x variants
- **Integration**: Works seamlessly with macOS app bundles and Xcode projects
- **No Xcode Post-processing**: Icons are immediately ready for use in macOS projects

The generated `Contents.json` files follow Apple's official Asset Catalog Format Reference, ensuring full compatibility with Xcode and the Apple development ecosystem.

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

### Generate icons for Tauri projects

```bash
# Generate only Tauri desktop icons (recommended for Tauri)
icon-gen app-icon.png --tauri-desktop

# The tauri-desktop folder can be directly copied to src-tauri/icons
# Simply drag and drop the contents into your Tauri project
```

### Generate icons with development badge

```bash
# Add a bug overlay to all generated icons for development builds
icon-gen app-icon.png --dev-mode

# Specify which bug to use
icon-gen app-icon.png --dev-mode --dev-bug spider
icon-gen app-icon.png --dev-mode --dev-bug caterpillar

# Combine with other options
icon-gen app-icon.png --dev-mode --mobile-only
icon-gen app-icon.png --dev-mode --ios-color "#2196F3"
icon-gen app-icon.png --dev-mode --dev-bug ladybug --android --ios
```

## Output Structure

When generating all formats, the output directory structure will be:

```
{output}/                  # Default: icon-generator-{source-name}/
├── windows/              # Windows icons
│   └── icon.ico
├── macos/                # macOS icons
│   ├── icon.icns
│   └── Contents.json     # macOS Asset Catalog metadata
├── linux/                # Linux desktop icons
│   ├── 32x32.png
│   ├── 64x64.png
│   ├── 128x128.png
│   ├── 256x256.png
│   └── icon.png          # 512x512
├── tauri-desktop/        # Tauri desktop icons
│   ├── 32x32.png
│   ├── 128x128.png
│   ├── 128x128@2x.png
│   ├── icon.ico
│   └── icon.icns
├── android/              # Android icons
│   ├── mipmap-mdpi/
│   │   └── ic_launcher.png
│   ├── mipmap-hdpi/
│   │   └── ic_launcher.png
│   └── ...
└── ios/                  # iOS icons
    ├── Contents.json     # iOS Asset Catalog metadata
    ├── AppIcon-20x20@1x.png
    ├── AppIcon-20x20@2x.png
    ├── AppIcon-29x29@1x.png
    ├── AppIcon-29x29@2x.png
    ├── AppIcon-40x40@1x.png
    ├── AppIcon-40x40@2x.png
    ├── AppIcon-60x60@2x.png
    ├── AppIcon-60x60@3x.png
    ├── AppIcon-76x76@1x.png
    ├── AppIcon-76x76@2x.png
    ├── AppIcon-83x83@2x.png
    └── AppIcon-1024x1024.png
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
