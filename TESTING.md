# Testing Guide

This document describes the test suite for icon-gen and how to run the tests.

## Test Types

### Unit Tests
Located in `src/contents_json.rs`, these tests verify the core Contents.json serialization functionality:

- **`test_single_entry_json_serialization`**: Tests that serializing a single ImageEntry produces the expected JSON structure with all required fields
- **`test_contents_file_creation`**: Tests creating a new ContentsFile structure
- **`test_image_entry_creation`**: Tests creating basic ImageEntry objects
- **`test_app_icon_creation`**: Tests creating app icon entries with proper metadata
- **`test_serialization`**: Tests basic JSON serialization
- **`test_apple_contents_json_format`**: Tests realistic iOS Contents.json generation
- **`test_write_contents_json`**: Tests writing Contents.json to disk

### Integration Tests
Located in `tests/integration_test.rs`, these tests verify end-to-end CLI functionality:

- **`test_ios_icon_generation_with_contents_json`**: Runs `icon-gen --ios-color "#fff" --ios` against a 1024×1024 dummy source image and verifies:
  - `ios/Contents.json` file is created
  - Contents.json contains valid JSON
  - JSON has proper structure with images and info sections
  - All image entries have required fields (filename, idiom, scale)

## Running Tests

### All Tests
```bash
cargo test
```

### Unit Tests Only
```bash
cargo test --lib
```

### Integration Tests Only
```bash
cargo test --test integration_test
```

### Specific Test
```bash
cargo test test_single_entry_json_serialization
cargo test test_ios_icon_generation_with_contents_json
```

## CI/CD

The project includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that:
- Checks code formatting (`cargo fmt`)
- Runs clippy linter (`cargo clippy`)
- Runs all unit and integration tests
- Builds the project on multiple platforms (Ubuntu, Windows, macOS)

## Test Requirements

### Dependencies
The tests use these additional dependencies defined in `Cargo.toml`:
- `tempfile = "3.8"` - For creating temporary directories in integration tests

### Test Assets
Integration tests automatically create a 1024×1024 dummy PNG image for testing icon generation.

## Coverage

The test suite covers:
- ✅ JSON serialization of Contents.json structures
- ✅ Single entry serialization with expected field names
- ✅ End-to-end CLI execution with iOS icon generation
- ✅ File system operations (Contents.json creation)
- ✅ JSON validation and structure verification
