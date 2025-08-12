use image::io::Reader as ImageReader;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "dev_test_output/512x512.png".to_string());
    
    let img = ImageReader::open(&path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image");
    
    let rgba_img = img.to_rgba8();
    let width = img.width();
    let height = img.height();
    
    // The ribbon should be at the bottom 1/4 of the image
    let badge_height = height / 4;
    let ribbon_y_start = height - badge_height;
    
    println!("Checking dev badge in: {}", path);
    println!("Image dimensions: {}x{}", width, height);
    println!("Expected ribbon area: y={} to y={}", ribbon_y_start, height);
    
    // Sample the center of the ribbon
    let center_x = width / 2;
    let center_y = ribbon_y_start + badge_height / 2;
    let pixel = rgba_img.get_pixel(center_x, center_y);
    
    println!("\nCenter pixel of ribbon area (x={}, y={}):", center_x, center_y);
    println!("  RGBA: [{}, {}, {}, {}]", pixel[0], pixel[1], pixel[2], pixel[3]);
    
    // Check multiple samples across the ribbon
    let mut red_dominant_count = 0;
    let samples = 10;
    
    for i in 0..samples {
        let x = (width * i / samples) + (width / (samples * 2));
        let y = ribbon_y_start + badge_height / 2;
        let p = rgba_img.get_pixel(x, y);
        
        if p[0] > 100 && p[0] > p[1] && p[0] > p[2] {
            red_dominant_count += 1;
        }
    }
    
    println!("\nRibbon analysis:");
    println!("  {} out of {} samples show red dominance", red_dominant_count, samples);
    
    if red_dominant_count >= samples * 7 / 10 {
        println!("✓ Dev badge detected!");
    } else {
        println!("⚠ Dev badge may not be properly applied");
    }
}
