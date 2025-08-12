use image::{DynamicImage, Rgba, RgbaImage};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Integration test for the --dev-mode feature
/// Generates a 128×128 icon with `--dev-mode` and asserts that:
/// 1. The command runs successfully
/// 2. The output icon exists
/// 3. A pixel in the ribbon area has red color with non-zero alpha
#[test]
fn test_dev_mode_badge_integration() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a dummy 128x128 source image
    let source_path = temp_path.join("test_source.png");
    create_test_image(&source_path, 128, 128);

    // Create output directory
    let output_dir = temp_path.join("test_output");

    // Get the path to the icon-gen binary
    let binary_path = get_icon_gen_binary_path();

    // Run icon-gen command with --dev-mode flag
    let output = Command::new(&binary_path)
        .arg(&source_path)
        .arg("--dev-mode")
        .arg("--png")
        .arg("128")  // Generate only 128x128 PNG
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to run icon-gen command");

    // Check that command succeeded
    if !output.status.success() {
        eprintln!("Command failed with status: {}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("icon-gen command with --dev-mode failed");
    }

    // Verify that the 128x128 icon was generated
    let output_icon_path = output_dir.join("128x128.png");
    assert!(
        output_icon_path.exists(),
        "Output icon should exist at: {}",
        output_icon_path.display()
    );

    // Load the generated icon
    let generated_icon = image::open(&output_icon_path)
        .expect("Failed to load generated icon");

    // Verify the image dimensions
    assert_eq!(generated_icon.width(), 128, "Generated icon width should be 128");
    assert_eq!(generated_icon.height(), 128, "Generated icon height should be 128");

    // Check that the dev badge was applied
    verify_dev_badge_applied(&generated_icon);

    println!("✓ Dev mode integration test passed:");
    println!("  - Icon generated successfully with --dev-mode");
    println!("  - Dev badge (red ribbon) detected in the image");
}

/// Verify that the dev badge (red ribbon) was applied to the image
fn verify_dev_badge_applied(img: &DynamicImage) {
    let rgba_img = img.to_rgba8();
    let width = img.width();
    let height = img.height();
    
    // The ribbon should be at the bottom 1/4 of the image
    let badge_height = height / 4;
    let ribbon_y_start = height - badge_height;
    
    // Sample multiple pixels from the ribbon area to ensure it's red-tinted
    let mut red_pixels_found = 0;
    let samples_to_check = 10;
    
    for i in 0..samples_to_check {
        // Sample pixels across the width of the ribbon
        let x = (width * i / samples_to_check) + (width / (samples_to_check * 2));
        let y = ribbon_y_start + badge_height / 2;
        
        let pixel = rgba_img.get_pixel(x, y);
        
        // Check if the pixel has a red tint
        // The ribbon color is blended with the original, so we check for red dominance
        if pixel[0] > 100 && pixel[3] > 0 {
            red_pixels_found += 1;
        }
    }
    
    // At least 70% of sampled pixels should show red tint
    let min_red_pixels = (samples_to_check * 7) / 10;
    assert!(
        red_pixels_found >= min_red_pixels,
        "Dev badge not properly applied. Only {} out of {} sampled pixels showed red tint (expected at least {})",
        red_pixels_found, samples_to_check, min_red_pixels
    );
    
    // Also check a specific pixel in the center of the ribbon
    let center_x = width / 2;
    let center_y = ribbon_y_start + badge_height / 2;
    let center_pixel = rgba_img.get_pixel(center_x, center_y);
    
    assert!(
        center_pixel[0] >= 100,
        "Center pixel red channel should be >= 100, but was {}",
        center_pixel[0]
    );
    assert!(
        center_pixel[3] > 0,
        "Center pixel alpha channel should be > 0, but was {}",
        center_pixel[3]
    );
}

/// Test that dev mode works with different icon sizes
#[test]
fn test_dev_mode_multiple_sizes() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a dummy 512x512 source image (larger source for better quality)
    let source_path = temp_path.join("test_source.png");
    create_test_image(&source_path, 512, 512);

    // Create output directory
    let output_dir = temp_path.join("test_output");

    // Get the path to the icon-gen binary
    let binary_path = get_icon_gen_binary_path();

    // Test with multiple sizes
    let sizes = vec![32, 64, 128, 256];
    let sizes_str = sizes.iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(",");

    // Run icon-gen command with --dev-mode flag for multiple sizes
    let output = Command::new(&binary_path)
        .arg(&source_path)
        .arg("--dev-mode")
        .arg("--png")
        .arg(&sizes_str)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to run icon-gen command");

    // Check that command succeeded
    assert!(
        output.status.success(),
        "icon-gen command with --dev-mode for multiple sizes failed"
    );

    // Verify each size was generated and has the dev badge
    for size in &sizes {
        let output_icon_path = output_dir.join(format!("{}x{}.png", size, size));
        assert!(
            output_icon_path.exists(),
            "Output icon {}x{} should exist",
            size, size
        );

        let generated_icon = image::open(&output_icon_path)
            .expect(&format!("Failed to load {}x{} icon", size, size));

        assert_eq!(generated_icon.width(), *size);
        assert_eq!(generated_icon.height(), *size);

        // For sizes 32 and above, the dev badge should be visible
        if *size >= 32 {
            verify_dev_badge_applied(&generated_icon);
        }
    }

    println!("✓ Dev mode multi-size test passed:");
    println!("  - Generated icons with sizes: {:?}", sizes);
    println!("  - All icons have dev badge applied");
}

/// Creates a test image with a blue gradient
fn create_test_image(path: &Path, width: u32, height: u32) {
    let mut image = RgbaImage::new(width, height);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        // Create a blue gradient that will contrast with the red ribbon
        let blue = (255.0 * x as f32 / width as f32) as u8;
        let green = (100.0 * y as f32 / height as f32) as u8;
        let red = 50;
        let alpha = 255;

        *pixel = Rgba([red, green, blue, alpha]);
    }

    image.save(path).expect("Failed to save test image");
}

/// Gets the path to the icon-gen binary
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
