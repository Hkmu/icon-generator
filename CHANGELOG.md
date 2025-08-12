# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
