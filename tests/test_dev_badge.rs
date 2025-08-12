use icon_gen::icon_gen::apply_dev_badge;
use image::{DynamicImage, ImageBuffer, Rgba};

#[test]
fn test_apply_dev_badge() {
    // Create a simple test image
    let width = 256;
    let height = 256;
    let img_buffer = ImageBuffer::from_fn(width, height, |_, _| {
        Rgba([100, 150, 200, 255]) // Blue color
    });
    
    let mut img = DynamicImage::ImageRgba8(img_buffer);
    
    // Apply the dev badge
    let result = apply_dev_badge(&mut img);
    
    // Check that the function succeeds
    assert!(result.is_ok());
    
    // Verify the image still has the same dimensions
    assert_eq!(img.width(), width);
    assert_eq!(img.height(), height);
    
    // Check that the bottom part of the image has been modified
    // (the ribbon should be at the bottom 1/4 of the image)
    let rgba_img = img.to_rgba8();
    let badge_height = height / 4;
    let ribbon_y_start = height - badge_height;
    
    // Sample a pixel from the ribbon area
    let ribbon_pixel = rgba_img.get_pixel(width / 2, ribbon_y_start + badge_height / 2);
    
    // The ribbon should have a red tint (red component should be significant)
    assert!(ribbon_pixel[0] > 100); // Red channel should be noticeable
}

#[test]
fn test_apply_dev_badge_small_image() {
    // Test with a small image
    let width = 32;
    let height = 32;
    let img_buffer = ImageBuffer::from_fn(width, height, |_, _| {
        Rgba([50, 100, 150, 255])
    });
    
    let mut img = DynamicImage::ImageRgba8(img_buffer);
    
    // Apply the dev badge
    let result = apply_dev_badge(&mut img);
    
    // Should work even with small images
    assert!(result.is_ok());
    assert_eq!(img.width(), width);
    assert_eq!(img.height(), height);
}
