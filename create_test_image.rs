use image::{Rgba, RgbaImage};

fn main() {
    let mut image = RgbaImage::new(128, 128);
    
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let blue = (255.0 * x as f32 / 128.0) as u8;
        let green = (100.0 * y as f32 / 128.0) as u8;
        let red = 50;
        let alpha = 255;
        *pixel = Rgba([red, green, blue, alpha]);
    }
    
    image.save("/tmp/test_icon.png").expect("Failed to save test image");
    println!("Created test image at /tmp/test_icon.png");
}
