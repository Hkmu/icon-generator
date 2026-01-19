# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-01-19

### Added
- Platform-specific directories for each platform (windows/, macos/, linux/, android/, ios/)
- Tauri desktop support with `--tauri-desktop` flag
- Dedicated `tauri-desktop/` folder with icons ready for Tauri's `src-tauri/icons`
- `--version`/`-V` option to display version information
- Caterpillar as new bug type for development badges
- Long description in CLI help text

### Changed
- **BREAKING**: Output directory structure now uses platform-specific subdirectories
  - Windows icons → `windows/icon.ico`
  - macOS icons → `macos/icon.icns` and `macos/Contents.json`
  - Linux icons → `linux/*.png`
  - Tauri icons → `tauri-desktop/*` (includes 32x32.png, 128x128.png, 128x128@2x.png, icon.ico, icon.icns)
- **BREAKING**: Removed `--ico-only` flag (use `--windows` instead)
- **BREAKING**: Removed `--icns-only` flag (use `--macos` instead)
- Default output folder renamed to `icon-generator-{source-name}/`
- iOS background color handling improved with forced opacity (alpha=255) for Apple compliance
- Tauri desktop icons automatically generated when desktop platforms are enabled

### Fixed
- iOS icon generation now forces all pixels to alpha=255 (required by Apple)
- Improved blending algorithm for semi-transparent pixels in iOS icons
- Proper alpha blending when applying background colors to iOS icons

## [0.2.0] - 2024-12-19

### Added
- Development mode badge feature (`--dev-mode`) that adds a visual indicator to generated icons
- Semi-transparent red ribbon overlay at the bottom of icons for development builds
- Diagonal stripe pattern on the development badge for clear visual distinction
- Text rendering capability support with rusttype dependency for future enhancements
- Comprehensive test suite for development badge functionality
- Manual verification tools for development badge (`verify_dev_badge`)

### Changed
- Icons generated with `--dev-mode` flag now include a clear development indicator

## [0.1.0] - Initial Release

### Added
- Cross-platform icon generation from a single source image
- Support for multiple icon formats:
  - Windows ICO format
  - macOS ICNS format
  - PNG icons in various sizes
  - Android drawable resources
  - iOS app icons with Contents.json
- Platform-specific generation flags (--windows, --macos, --linux, --android, --ios)
- Desktop-only and mobile-only generation modes
- Custom PNG size specifications
- iOS background color customization
- Comprehensive error handling and validation
