use image::{Rgba, RgbaImage};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Test that runs `icon-gen --ios-color "#fff" --ios` against a 1024×1024 dummy source
/// and asserts that `ios/Contents.json` exists and is valid JSON.
#[test]
fn test_ios_icon_generation_with_contents_json() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a dummy 1024x1024 source image
    let source_path = temp_path.join("dummy_source.png");
    create_dummy_1024x1024_image(&source_path);

    // Create output directory
    let output_dir = temp_path.join("test_output");

    // Get the path to the icon-gen binary
    let binary_path = get_icon_gen_binary_path();

    // Run icon-gen command: icon-gen --ios-color "#fff" --ios dummy_source.png -o test_output
    let output = Command::new(&binary_path)
        .arg(&source_path)
        .arg("--ios-color")
        .arg("#fff")
        .arg("--ios")
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to run icon-gen command");

    // Check that command succeeded
    if !output.status.success() {
        eprintln!("Command failed with status: {}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("icon-gen command failed");
    }

    // Verify that ios/Contents.json exists
    let contents_json_path = output_dir.join("ios").join("Contents.json");
    assert!(
        contents_json_path.exists(),
        "ios/Contents.json file should exist at: {}",
        contents_json_path.display()
    );

    // Read and verify that Contents.json is valid JSON
    let contents_json_content =
        std::fs::read_to_string(&contents_json_path).expect("Failed to read Contents.json");

    let parsed_json: serde_json::Value = serde_json::from_str(&contents_json_content)
        .expect("Contents.json should contain valid JSON");

    // Verify essential structure
    assert!(
        parsed_json["images"].is_array(),
        "Contents.json should have 'images' array"
    );
    assert!(
        parsed_json["info"].is_object(),
        "Contents.json should have 'info' object"
    );

    let images = parsed_json["images"].as_array().unwrap();
    assert!(
        !images.is_empty(),
        "Contents.json should have at least one image entry"
    );

    // Verify the info structure
    let info = &parsed_json["info"];
    assert_eq!(info["version"], 1, "Version should be 1");
    assert!(info["author"].is_string(), "Author should be a string");

    // Verify each image entry has required fields
    for (i, image) in images.iter().enumerate() {
        assert!(
            image["filename"].is_string(),
            "Image entry {} should have filename",
            i
        );
        assert!(
            image["idiom"].is_string(),
            "Image entry {} should have idiom",
            i
        );
        assert!(
            image["scale"].is_string(),
            "Image entry {} should have scale",
            i
        );
    }

    println!("✓ Integration test passed: iOS icons generated with valid Contents.json");
    println!("  - Generated {} image entries", images.len());
    println!("  - Contents.json is valid JSON");
    println!("  - All required fields present");
}

/// Creates a dummy 1024x1024 RGBA image and saves it as PNG
fn create_dummy_1024x1024_image(path: &Path) {
    let width = 1024;
    let height = 1024;

    // Create a simple gradient image
    let mut image = RgbaImage::new(width, height);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let red = (255.0 * x as f32 / width as f32) as u8;
        let green = (255.0 * y as f32 / height as f32) as u8;
        let blue = 128;
        let alpha = 255;

        *pixel = Rgba([red, green, blue, alpha]);
    }

    image.save(path).expect("Failed to save dummy image");
}

/// Gets the path to the icon-gen binary (either from cargo build or target directory)
fn get_icon_gen_binary_path() -> std::path::PathBuf {
    // First try to find in target/debug
    let debug_path = std::path::Path::new("target/debug/icon-gen");
    if debug_path.exists() {
        return debug_path.to_path_buf();
    }

    // If not found, build it first
    let build_output = Command::new("cargo")
        .args(&["build", "--bin", "icon-gen"])
        .output()
        .expect("Failed to run cargo build");

    if !build_output.status.success() {
        panic!(
            "Failed to build icon-gen binary: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    debug_path.to_path_buf()
}
