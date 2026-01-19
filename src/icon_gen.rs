use crate::contents_json::{ContentsFile, ImageEntry};
use anyhow::{Context, Result};
use icns::{IconFamily, IconType};
use image::{
    codecs::{
        ico::{IcoEncoder, IcoFrame},
        png::{CompressionType, FilterType as PngFilterType, PngEncoder},
    },
    imageops::FilterType,
    ColorType, DynamicImage, ImageBuffer, ImageEncoder, Rgba,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    f32::consts::PI,
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

// Define Args struct for library compilation
#[derive(Debug)]
pub struct Args {
    pub input: PathBuf,
    pub output: PathBuf,
    pub png: Option<Vec<u32>>,
    pub desktop_only: bool,
    pub mobile_only: bool,
    pub tauri_desktop: bool,
    pub windows: bool,
    pub macos: bool,
    pub linux: bool,
    pub android: bool,
    pub android_round: bool,
    pub android_adaptive: bool,
    pub android_adaptive_bg: String,
    pub ios: bool,
    pub ios_color: String,
    pub dev_mode: bool,
    pub dev_bug: String,
}

#[derive(Debug, Deserialize)]
struct IcnsEntry {
    size: u32,
    ostype: String,
}

/// Apply a bug overlay to an image for development mode
/// Places a bug image in the center of the icon with optional rotation
pub fn apply_dev_badge_with_bug(
    img: &mut DynamicImage,
    bug_type: &str,
    angle_degrees: f32,
) -> Result<()> {
    let width = img.width();
    let height = img.height();
    let min_dim = width.min(height);

    // Load the embedded bug image
    let bug_data = get_embedded_bug_image(bug_type)?;
    let bug_img = image::load_from_memory(bug_data)
        .with_context(|| format!("Failed to load embedded bug image: {}", bug_type))?;

    // Calculate bug size - bug should be 1/4 of the minimum dimension
    let bug_size = min_dim / 4;

    // Resize the bug image while maintaining aspect ratio
    let resized_bug = resize_bug_with_aspect_ratio(&bug_img, bug_size);

    // Rotate the bug if angle is not 0
    let final_bug = if angle_degrees != 0.0 {
        rotate_image(&resized_bug, angle_degrees)
    } else {
        resized_bug
    };

    // Calculate position to center the bug
    let bug_width = final_bug.width();
    let bug_height = final_bug.height();
    let x = (width.saturating_sub(bug_width)) / 2;
    let y = (height.saturating_sub(bug_height)) / 2;

    // Overlay the bug onto the main image
    image::imageops::overlay(img, &final_bug, x.into(), y.into());

    Ok(())
}

/// Get embedded bug image data based on bug type
fn get_embedded_bug_image(bug_type: &str) -> Result<&'static [u8]> {
    match bug_type {
        "moth" => Ok(include_bytes!("bugs/moth.png")),
        "cockroach" => Ok(include_bytes!("bugs/cockroach.png")),
        "ladybug" => Ok(include_bytes!("bugs/ladybug.png")),
        "spider" => Ok(include_bytes!("bugs/spider.png")),
        "caterpillar" => Ok(include_bytes!("bugs/caterpillar.png")),
        _ => Err(anyhow::anyhow!(
            "Unknown bug type: {}. Available types: moth, cockroach, ladybug, spider, caterpillar",
            bug_type
        )),
    }
}

/// Rotate an image by the given angle in degrees
fn rotate_image(img: &DynamicImage, angle_degrees: f32) -> DynamicImage {
    let angle_radians = angle_degrees * PI / 180.0;

    // For simplicity, we'll implement a basic rotation
    // This could be enhanced with more sophisticated rotation algorithms
    let width = img.width();
    let height = img.height();

    // Create a new image with the same dimensions
    let mut rotated = ImageBuffer::from_fn(width, height, |_, _| Rgba([0, 0, 0, 0]));

    // Center of rotation
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    let rgba_img = img.to_rgba8();

    // Simple nearest-neighbor rotation
    for y in 0..height {
        for x in 0..width {
            // Translate to center
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;

            // Rotate
            let rotated_x = dx * angle_radians.cos() - dy * angle_radians.sin();
            let rotated_y = dx * angle_radians.sin() + dy * angle_radians.cos();

            // Translate back
            let source_x = (rotated_x + center_x) as i32;
            let source_y = (rotated_y + center_y) as i32;

            // Check bounds and copy pixel
            if source_x >= 0 && source_x < width as i32 && source_y >= 0 && source_y < height as i32
            {
                let pixel = rgba_img.get_pixel(source_x as u32, source_y as u32);
                rotated.put_pixel(x, y, *pixel);
            }
        }
    }

    DynamicImage::ImageRgba8(rotated)
}

pub fn generate_icons(args: Args) -> Result<()> {
    // Load source image
    let source = load_image(&args.input)?;

    // Ensure the output directory exists
    std::fs::create_dir_all(&args.output).context("Can't create output directory")?;

    // Check if any platform-specific flags are set
    let has_platform_flags = args.windows || args.macos || args.linux || args.android || args.ios;

    // Determine which platforms should generate icons
    let should_generate_ios = should_invoke_ios_writer(&args, has_platform_flags);
    let should_generate_macos = should_invoke_macos_writer(&args, has_platform_flags);

    // Generate icons based on options
    if args.tauri_desktop {
        // Generate Tauri desktop icons (requires Windows, macOS, Linux icons first)
        generate_ico(&source, &args.output, args.dev_mode, &args.dev_bug)?;
        if should_generate_macos {
            generate_icns(&source, &args.output, args.dev_mode, &args.dev_bug)?;
        }
        generate_linux_icons(&source, &args.output, args.dev_mode, &args.dev_bug)?;
        generate_tauri_desktop_icons(&source, &args.output, args.dev_mode, &args.dev_bug)?;
    } else if args.desktop_only {
        generate_desktop_only(&source, &args, should_generate_macos)?;
    } else if args.mobile_only {
        generate_mobile_only(&source, &args, should_generate_ios)?;
    } else if has_platform_flags {
        generate_platforms(&source, &args, should_generate_ios, should_generate_macos)?;
    } else {
        generate_all(&source, &args, should_generate_ios, should_generate_macos)?;
    }

    Ok(())
}

fn load_image(path: &Path) -> Result<DynamicImage> {
    let source = image::open(path).context("Failed to load image")?;

    // Ensure the image is square
    if source.width() != source.height() {
        anyhow::bail!("Source image must be square (width == height)");
    }

    Ok(source)
}

/// Determine when the iOS writer should be invoked
/// Only invoke when args.ios flag is on OR iOS icons are produced via generate_all/mobile_only
fn should_invoke_ios_writer(args: &Args, has_platform_flags: bool) -> bool {
    // If specific iOS flag is set
    if args.ios {
        return true;
    }

    // If no platform flags are set (default case - generate_all)
    if !has_platform_flags && !args.desktop_only && !args.mobile_only && !args.tauri_desktop {
        return true;
    }

    // If mobile_only is set
    if args.mobile_only {
        return true;
    }

    false
}

/// Determine when the macOS writer should be invoked
/// Only invoke when macOS icons are produced (macos, desktop_only, tauri_desktop, default)
fn should_invoke_macos_writer(args: &Args, has_platform_flags: bool) -> bool {
    // If specific macOS flag is set
    if args.macos {
        return true;
    }

    // If desktop_only is set
    if args.desktop_only {
        return true;
    }

    // If tauri_desktop is set
    if args.tauri_desktop {
        return true;
    }

    // If no platform flags are set (default case - generate_all)
    if !has_platform_flags && !args.desktop_only && !args.mobile_only && !args.tauri_desktop {
        return true;
    }

    false
}

fn generate_all(
    source: &DynamicImage,
    args: &Args,
    should_generate_ios: bool,
    should_generate_macos: bool,
) -> Result<()> {
    if let Some(sizes) = &args.png {
        generate_custom_sizes(source, sizes, &args.output, args.dev_mode, &args.dev_bug)?;
    } else {
        // Generate default formats when no specific platform flags are set
        generate_ico(source, &args.output, args.dev_mode, &args.dev_bug)?;

        if should_generate_macos {
            generate_icns(&source, &args.output, args.dev_mode, &args.dev_bug)?;
        }

        generate_linux_icons(source, &args.output, args.dev_mode, &args.dev_bug)?;
        generate_tauri_desktop_icons(source, &args.output, args.dev_mode, &args.dev_bug)?;
        generate_mobile(source, args, should_generate_ios)?;
    }

    Ok(())
}

fn generate_desktop_only(
    source: &DynamicImage,
    args: &Args,
    should_generate_macos: bool,
) -> Result<()> {
    if let Some(sizes) = &args.png {
        generate_custom_sizes(source, sizes, &args.output, args.dev_mode, &args.dev_bug)?;
    } else {
        generate_ico(source, &args.output, args.dev_mode, &args.dev_bug)?;

        if should_generate_macos {
            generate_icns(source, &args.output, args.dev_mode, &args.dev_bug)?;
        }

        generate_linux_icons(source, &args.output, args.dev_mode, &args.dev_bug)?;
        generate_tauri_desktop_icons(source, &args.output, args.dev_mode, &args.dev_bug)?;
    }
    Ok(())
}

fn generate_mobile_only(
    source: &DynamicImage,
    args: &Args,
    should_generate_ios: bool,
) -> Result<()> {
    generate_mobile(source, args, should_generate_ios)?;
    Ok(())
}

fn generate_platforms(
    source: &DynamicImage,
    args: &Args,
    should_generate_ios: bool,
    should_generate_macos: bool,
) -> Result<()> {
    let has_desktop_platform = args.windows || args.macos || args.linux;

    if args.windows {
        generate_ico(source, &args.output, args.dev_mode, &args.dev_bug)?;
    }

    if args.macos && should_generate_macos {
        generate_icns(source, &args.output, args.dev_mode, &args.dev_bug)?;
    }

    if args.linux {
        if let Some(sizes) = &args.png {
            generate_custom_sizes(source, sizes, &args.output, args.dev_mode, &args.dev_bug)?;
        } else {
            generate_linux_icons(source, &args.output, args.dev_mode, &args.dev_bug)?;
        }
    }

    // Generate tauri-desktop icons when any desktop platform is enabled
    if has_desktop_platform {
        generate_tauri_desktop_icons(source, &args.output, args.dev_mode, &args.dev_bug)?;
    }

    if args.android {
        generate_android_icons_extended(source, args)?;
    }

    if args.ios && should_generate_ios {
        generate_ios_icons(
            source,
            &args.output,
            &args.ios_color,
            args.dev_mode,
            &args.dev_bug,
        )?;
    }

    Ok(())
}

fn generate_ico(
    source: &DynamicImage,
    out_dir: &Path,
    dev_mode: bool,
    dev_bug: &str,
) -> Result<()> {
    let windows_dir = out_dir.join("windows");
    create_dir_all(&windows_dir)?;

    println!("Generating windows/icon.ico...");
    let mut frames = Vec::new();

    // Common ICO sizes
    for size in [16, 24, 32, 48, 64, 256] {
        let mut resized = source.resize_exact(size, size, FilterType::Lanczos3);

        // Apply dev badge before encoding
        if dev_mode {
            apply_dev_badge_with_bug(&mut resized, dev_bug, 0.0)?;
        }

        let rgba_image = resized.to_rgba8();

        // Only the 256px layer can be compressed according to the ico specs
        if size == 256 {
            let mut buf = Vec::new();
            write_png(rgba_image.as_raw(), &mut buf, size)?;
            frames.push(IcoFrame::with_encoded(buf, size, size, ColorType::Rgba8)?);
        } else {
            frames.push(IcoFrame::as_png(
                rgba_image.as_raw(),
                size,
                size,
                ColorType::Rgba8,
            )?);
        }
    }

    let mut out_file = BufWriter::new(File::create(windows_dir.join("icon.ico"))?);
    let encoder = IcoEncoder::new(&mut out_file);
    encoder.encode_images(&frames)?;
    out_file.flush()?;

    println!("✓ Generated windows/icon.ico");
    Ok(())
}

fn generate_icns(
    source: &DynamicImage,
    out_dir: &Path,
    dev_mode: bool,
    dev_bug: &str,
) -> Result<()> {
    let macos_dir = out_dir.join("macos");
    create_dir_all(&macos_dir)?;

    println!("Generating macos/icon.icns...");
    let icns_json = r#"
    {
      "16x16": { "size": 16, "ostype": "is32" },
      "16x16@2x": { "size": 32, "ostype": "ic11" },
      "32x32": { "size": 32, "ostype": "il32" },
      "32x32@2x": { "size": 64, "ostype": "ic12" },
      "128x128": { "size": 128, "ostype": "ic07" },
      "128x128@2x": { "size": 256, "ostype": "ic13" },
      "256x256": { "size": 256, "ostype": "ic08" },
      "256x256@2x": { "size": 512, "ostype": "ic14" },
      "512x512": { "size": 512, "ostype": "ic09" },
      "512x512@2x": { "size": 1024, "ostype": "ic10" }
    }
    "#;

    let entries: HashMap<String, IcnsEntry> = serde_json::from_str(icns_json).unwrap();
    let mut family = IconFamily::new();

    for (name, entry) in &entries {
        let mut image = source.resize_exact(entry.size, entry.size, FilterType::Lanczos3);

        // Apply dev badge before encoding
        if dev_mode {
            apply_dev_badge_with_bug(&mut image, dev_bug, 0.0)?;
        }

        let mut buf = Vec::new();
        let rgba_image = image.to_rgba8();
        write_png(rgba_image.as_raw(), &mut buf, entry.size)?;
        let image = icns::Image::read_png(&buf[..])?;

        family
            .add_icon_with_type(
                &image,
                IconType::from_ostype(entry.ostype.parse().unwrap()).unwrap(),
            )
            .with_context(|| format!("Can't add {name} to Icns Family"))?;
    }

    let mut out_file = BufWriter::new(File::create(macos_dir.join("icon.icns"))?);
    family.write(&mut out_file)?;
    out_file.flush()?;

    println!("✓ Generated macos/icon.icns");

    // Step 3: Generate Contents.json for macOS
    let macos_images = build_macos_contents_json(&entries)?;
    write_macos_contents_json(&macos_dir, macos_images)?;

    Ok(())
}

fn generate_custom_sizes(
    source: &DynamicImage,
    sizes: &[u32],
    out_dir: &Path,
    dev_mode: bool,
    dev_bug: &str,
) -> Result<()> {
    println!("Generating custom PNG sizes...");
    for &size in sizes {
        let resized = source.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
        let output_path = out_dir.join(format!("{}x{}.png", size, size));
        save_png(&resized, &output_path, dev_mode, dev_bug)?;
        println!("  ✓ Generated {}x{}.png", size, size);
    }
    Ok(())
}

fn generate_linux_icons(
    source: &DynamicImage,
    out_dir: &Path,
    dev_mode: bool,
    dev_bug: &str,
) -> Result<()> {
    let linux_dir = out_dir.join("linux");
    create_dir_all(&linux_dir)?;

    println!("Generating linux desktop icons...");
    let desktop_sizes = [32, 64, 128, 256, 512];
    for size in desktop_sizes {
        let filename = if size == 512 {
            "icon.png".to_string()
        } else {
            format!("{size}x{size}.png")
        };

        let resized = source.resize_exact(size, size, FilterType::Lanczos3);
        let output_path = linux_dir.join(&filename);
        save_png(&resized, &output_path, dev_mode, dev_bug)?;
        println!("  ✓ Generated linux/{filename}");
    }
    Ok(())
}

/// Generate Tauri desktop icons in a tauri-desktop/ directory
/// Contains the specific files needed for Tauri's src-tauri/icons folder
fn generate_tauri_desktop_icons(
    source: &DynamicImage,
    out_dir: &Path,
    dev_mode: bool,
    dev_bug: &str,
) -> Result<()> {
    let tauri_dir = out_dir.join("tauri-desktop");
    create_dir_all(&tauri_dir)?;

    println!("Generating tauri-desktop icons...");

    // Generate 32x32.png
    let resized_32 = source.resize_exact(32, 32, FilterType::Lanczos3);
    let output_path = tauri_dir.join("32x32.png");
    save_png(&resized_32, &output_path, dev_mode, dev_bug)?;
    println!("  ✓ Generated tauri-desktop/32x32.png");

    // Generate 128x128.png
    let resized_128 = source.resize_exact(128, 128, FilterType::Lanczos3);
    let output_path = tauri_dir.join("128x128.png");
    save_png(&resized_128, &output_path, dev_mode, dev_bug)?;
    println!("  ✓ Generated tauri-desktop/128x128.png");

    // Generate 128x128@2x.png (256x256)
    let resized_256 = source.resize_exact(256, 256, FilterType::Lanczos3);
    let output_path = tauri_dir.join("128x128@2x.png");
    save_png(&resized_256, &output_path, dev_mode, dev_bug)?;
    println!("  ✓ Generated tauri-desktop/128x128@2x.png");

    // Generate icon.ico (copy from windows directory)
    let windows_ico = out_dir.join("windows").join("icon.ico");
    if windows_ico.exists() {
        std::fs::copy(&windows_ico, tauri_dir.join("icon.ico"))?;
        println!("  ✓ Generated tauri-desktop/icon.ico");
    }

    // Generate icon.icns (copy from macos directory)
    let macos_icns = out_dir.join("macos").join("icon.icns");
    if macos_icns.exists() {
        std::fs::copy(&macos_icns, tauri_dir.join("icon.icns"))?;
        println!("  ✓ Generated tauri-desktop/icon.icns");
    }

    Ok(())
}

fn generate_mobile(source: &DynamicImage, args: &Args, should_generate_ios: bool) -> Result<()> {
    println!("Generating mobile platform icons...");

    // Android icons with round and adaptive support
    generate_android_icons_extended(source, args)?;

    // iOS icons with background color - only generate when appropriate flags are set
    if should_generate_ios {
        generate_ios_icons(
            source,
            &args.output,
            &args.ios_color,
            args.dev_mode,
            &args.dev_bug,
        )?;
    }

    Ok(())
}

fn generate_ios_icons(
    source: &DynamicImage,
    out_dir: &Path,
    color: &str,
    dev_mode: bool,
    dev_bug: &str,
) -> Result<()> {
    let ios_dir = out_dir.join("ios");
    create_dir_all(&ios_dir)?;

    // Parse background color
    let bg_color = css_color::Srgb::from_str(color)
        .map(|color| {
            Rgba([
                (color.red * 255.) as u8,
                (color.green * 255.) as u8,
                (color.blue * 255.) as u8,
                255,
            ])
        })
        .unwrap_or(Rgba([255, 255, 255, 255]));

    // Track produced files for Contents.json
    let mut images: Vec<ImageEntry> = Vec::new();

    // Xcode AppIcon.appiconset slots - includes all optional slots
    // Each entry: (base_size, multipliers, idiom, optional size_override, pixel_size_override)
    let sizes = [
        // iPhone App icons
        (29, vec![2, 3], "iphone", None, None), // iPhone: 29x29@2x, @3x (Settings, etc.)
        (40, vec![2, 3], "iphone", None, None), // iPhone: 40x40@2x, @3x (Spotlight)
        (60, vec![2, 3], "iphone", None, None), // iPhone: 60x60@2x, @3x (App icon)
        // iPhone Notification icons (optional slot)
        (20, vec![2, 3], "iphone", None, None), // iPhone: 20x20@2x, @3x (Notifications)
        // iPad App icons
        (29, vec![1, 2], "ipad", None, None),   // iPad: 29x29@1x, @2x (Settings, etc.)
        (40, vec![1, 2], "ipad", None, None),   // iPad: 40x40@1x, @2x (Spotlight)
        (76, vec![1, 2], "ipad", None, None),   // iPad: 76x76@1x, @2x (App icon)
        // iPad Notification icons (optional slot)
        (20, vec![2], "ipad", None, None),      // iPad: 20x20@2x (Notifications)
        // iPad Pro 12.9" App icon (optional slot) - 83.5pt @2x = 167px
        (83, vec![2], "ipad", Some("83.5x83.5"), Some(167)),
    ];

    for (base_size, multipliers, idiom, size_override, pixel_size_override) in sizes {
        for multiplier in multipliers {
            // Use pixel_size_override if provided (for 83.5pt -> 167px case), otherwise calculate
            let actual_size = pixel_size_override.unwrap_or(base_size * multiplier);
            let filename = format!("AppIcon-{base_size}x{base_size}@{multiplier}x.png");

            let mut resized = source.resize_exact(actual_size, actual_size, FilterType::Lanczos3);

            // Add background color for iOS icons
            // We need to blend the source image with background color and force opacity
            let resized_rgba = resized.to_rgba8();
            let final_img = ImageBuffer::from_fn(actual_size, actual_size, |x, y| {
                let src_pixel = resized_rgba.get_pixel(x, y);
                let src_alpha = src_pixel[3] as f32 / 255.0;

                if src_alpha == 0.0 {
                    // Fully transparent - use background color
                    bg_color
                } else {
                    // Blend source with background and force opacity
                    let inv_alpha = 1.0 - src_alpha;
                    Rgba([
                        ((src_alpha * src_pixel[0] as f32 + inv_alpha * bg_color[0] as f32)) as u8,
                        ((src_alpha * src_pixel[1] as f32 + inv_alpha * bg_color[1] as f32)) as u8,
                        ((src_alpha * src_pixel[2] as f32 + inv_alpha * bg_color[2] as f32)) as u8,
                        255, // Force full opacity for iOS
                    ])
                }
            });
            resized = DynamicImage::ImageRgba8(final_img);

            let output_path = ios_dir.join(&filename);
            save_png(&resized, &output_path, dev_mode, dev_bug)?;
            println!("  ✓ Generated ios/{filename}");

            // Create ImageEntry for Contents.json (no role field for standard AppIcon)
            let size_str = size_override.unwrap_or(&format!("{base_size}x{base_size}")).to_string();

            let mut image_entry = ImageEntry::new_app_icon(
                filename,
                idiom.to_string(),
                size_str,
                format!("{multiplier}x"),
                None, // No role for standard Xcode AppIcon
            );

            image_entry.expected_size = Some(actual_size.to_string());

            images.push(image_entry);
        }
    }

    // Generate 1024pt App Store marketing icon and add to Contents.json
    let marketing_filename = "AppIcon-1024x1024.png";
    let marketing_size = 1024;
    let marketing_icon = source.resize_exact(marketing_size, marketing_size, FilterType::Lanczos3);

    // Apply background color using same blend logic
    let marketing_rgba = marketing_icon.to_rgba8();
    let marketing_final = ImageBuffer::from_fn(marketing_size, marketing_size, |x, y| {
        let src_pixel = marketing_rgba.get_pixel(x, y);
        let src_alpha = src_pixel[3] as f32 / 255.0;

        if src_alpha == 0.0 {
            // Fully transparent - use background color
            bg_color
        } else {
            // Blend source with background and force opacity
            let inv_alpha = 1.0 - src_alpha;
            Rgba([
                ((src_alpha * src_pixel[0] as f32 + inv_alpha * bg_color[0] as f32)) as u8,
                ((src_alpha * src_pixel[1] as f32 + inv_alpha * bg_color[1] as f32)) as u8,
                ((src_alpha * src_pixel[2] as f32 + inv_alpha * bg_color[2] as f32)) as u8,
                255, // Force full opacity for iOS
            ])
        }
    });

    let marketing_icon_final = DynamicImage::ImageRgba8(marketing_final);
    let marketing_path = ios_dir.join(marketing_filename);
    save_png(&marketing_icon_final, &marketing_path, dev_mode, dev_bug)?;
    println!("  ✓ Generated ios/{} (for App Store)", marketing_filename);

    // Add marketing icon entry to Contents.json
    let marketing_entry = ImageEntry::new_app_icon(
        marketing_filename.to_string(),
        "ios-marketing".to_string(),
        "1024x1024".to_string(),
        "1x".to_string(),
        None,
    );
    images.push(marketing_entry);

    // Write Contents.json (now includes all 17 icons)
    write_contents_json(&ios_dir, images)?;

    Ok(())
}

fn save_png(image: &DynamicImage, path: &Path, dev_mode: bool, dev_bug: &str) -> Result<()> {
    let mut img = image.clone();

    // Apply dev badge if in dev mode
    if dev_mode {
        apply_dev_badge_with_bug(&mut img, dev_bug, 0.0)?;
    }

    let mut file = std::fs::File::create(path).context("Failed to create PNG file")?;
    img.write_to(&mut file, image::ImageOutputFormat::Png)
        .context("Failed to write PNG")?;
    Ok(())
}

// Encode image data as PNG with compression
fn write_png<W: Write>(image_data: &[u8], w: W, size: u32) -> Result<()> {
    let encoder = PngEncoder::new_with_quality(w, CompressionType::Best, PngFilterType::Adaptive);
    encoder.write_image(image_data, size, size, ColorType::Rgba8)?;
    Ok(())
}

/// Write Contents.json file with the provided image entries
fn write_contents_json(ios_dir: &Path, images: Vec<ImageEntry>) -> Result<()> {
    let mut contents = ContentsFile::new("icon-generator".to_string());

    for image in images {
        contents.add_image(image);
    }

    let contents_path = ios_dir.join("Contents.json");
    let contents_json =
        serde_json::to_string_pretty(&contents).context("Failed to serialize Contents.json")?;

    std::fs::write(&contents_path, contents_json).context("Failed to write Contents.json file")?;

    println!("  ✓ Generated ios/Contents.json");
    Ok(())
}

/// Build a Vec<ImageEntry> for macOS using the same icns_json map (sizes 16–1024)
/// Idiom = "mac", scale is "1x" or "2x" depending on whether name contains @2x
/// Folder always "." (same directory)
fn build_macos_contents_json(entries: &HashMap<String, IcnsEntry>) -> Result<Vec<ImageEntry>> {
    let mut images = Vec::new();

    for (name, entry) in entries {
        // Determine scale based on whether name contains @2x
        let scale = if name.contains("@2x") {
            "2x".to_string()
        } else {
            "1x".to_string()
        };

        // Extract base size from name (e.g., "16x16" from "16x16@2x" or "16x16")
        let base_name = if name.contains("@2x") {
            name.replace("@2x", "")
        } else {
            name.clone()
        };

        // Create filename based on the entry size and scale
        let filename = format!("icon_{}.png", entry.size);

        let mut image_entry = ImageEntry::new(filename, "mac".to_string(), scale);

        // Set size to the base name (e.g., "16x16", "32x32", etc.)
        image_entry.size = Some(base_name);

        // Set folder to "." (same directory)
        image_entry = image_entry.with_folder(".".to_string());

        images.push(image_entry);
    }

    Ok(images)
}

/// Write Contents.json file for macOS next to icon.icns
fn write_macos_contents_json(out_dir: &Path, images: Vec<ImageEntry>) -> Result<()> {
    let mut contents = ContentsFile::new("icon-generator".to_string());

    for image in images {
        contents.add_image(image);
    }

    let contents_path = out_dir.join("Contents.json");
    let contents_json = serde_json::to_string_pretty(&contents)
        .context("Failed to serialize macOS Contents.json")?;

    std::fs::write(&contents_path, contents_json)
        .context("Failed to write macOS Contents.json file")?;

    println!("  ✓ Generated Contents.json");
    Ok(())
}

/// Generate Android icons with support for round and adaptive icons
fn generate_android_icons_extended(source: &DynamicImage, args: &Args) -> Result<()> {
    let android_dir = args.output.join("android");
    create_dir_all(&android_dir)?;

    println!("Generating Android icons...");

    let densities = [
        ("mdpi", 48),
        ("hdpi", 72),
        ("xhdpi", 96),
        ("xxhdpi", 144),
        ("xxxhdpi", 192),
    ];

    // Generate standard square icons (ic_launcher.png)
    for (density, size) in densities {
        let mipmap_dir = android_dir.join(format!("mipmap-{density}"));
        create_dir_all(&mipmap_dir)?;

        let resized = source.resize_exact(size, size, FilterType::Lanczos3);
        let output_path = mipmap_dir.join("ic_launcher.png");
        save_png(&resized, &output_path, args.dev_mode, &args.dev_bug)?;
        println!("  ✓ Generated android/mipmap-{density}/ic_launcher.png");
    }

    // Generate round icons if requested (enabled by default with --android)
    if args.android_round {
        println!("Generating Android round icons...");
        for (density, size) in densities {
            let mipmap_dir = android_dir.join(format!("mipmap-{density}"));

            // Create a round version by applying a circular mask
            let resized = source.resize_exact(size, size, FilterType::Lanczos3);
            let round_icon = apply_circular_mask(&resized)?;

            let output_path = mipmap_dir.join("ic_launcher_round.png");
            save_png(&round_icon, &output_path, args.dev_mode, &args.dev_bug)?;
            println!("  ✓ Generated android/mipmap-{density}/ic_launcher_round.png");
        }
    }

    // Generate adaptive icons if requested
    if args.android_adaptive {
        println!("Generating Android adaptive icons...");
        generate_adaptive_icons(
            source,
            &android_dir,
            &args.android_adaptive_bg,
            args.dev_mode,
            &args.dev_bug,
        )?;
    }

    Ok(())
}

/// Apply a circular mask to an image to create a round icon
fn apply_circular_mask(img: &DynamicImage) -> Result<DynamicImage> {
    let width = img.width();
    let height = img.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let radius = width.min(height) as f32 / 2.0;

    let mut rgba_img = img.to_rgba8();

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Apply anti-aliasing at the edge
            if distance > radius {
                rgba_img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            } else if distance > radius - 1.0 {
                // Anti-aliasing edge
                let alpha_factor = radius - distance;
                let pixel = rgba_img.get_pixel_mut(x, y);
                pixel[3] = (pixel[3] as f32 * alpha_factor) as u8;
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(rgba_img))
}

/// Generate Android adaptive icons with foreground and background layers
fn generate_adaptive_icons(
    source: &DynamicImage,
    android_dir: &Path,
    bg_color_str: &str,
    dev_mode: bool,
    dev_bug: &str,
) -> Result<()> {
    // Parse background color
    let bg_color = css_color::Srgb::from_str(bg_color_str)
        .map(|color| {
            Rgba([
                (color.red * 255.) as u8,
                (color.green * 255.) as u8,
                (color.blue * 255.) as u8,
                255,
            ])
        })
        .unwrap_or(Rgba([255, 255, 255, 255]));

    // Adaptive icon sizes (108dp with 72dp visible area)
    // The extra 36dp (18dp on each side) is for visual effects
    let adaptive_densities = [
        ("mdpi", 108),
        ("hdpi", 162),
        ("xhdpi", 216),
        ("xxhdpi", 324),
        ("xxxhdpi", 432),
    ];

    // Generate foreground layers (the actual icon, scaled to fit in safe zone)
    for (density, size) in adaptive_densities {
        let mipmap_dir = android_dir.join(format!("mipmap-{density}"));
        create_dir_all(&mipmap_dir)?;

        // Scale the icon to 66% of the adaptive icon size to fit in the safe zone
        // This ensures the icon is fully visible in all shapes (circle, square, rounded square, etc.)
        let icon_size = (size as f32 * 0.66) as u32;
        let padding = (size - icon_size) / 2;

        let resized = source.resize_exact(icon_size, icon_size, FilterType::Lanczos3);

        // Create a transparent canvas of the full adaptive size
        let mut foreground = ImageBuffer::from_fn(size, size, |_, _| Rgba([0, 0, 0, 0]));

        // Center the icon on the canvas
        image::imageops::overlay(&mut foreground, &resized, padding.into(), padding.into());

        let foreground_img = DynamicImage::ImageRgba8(foreground);
        let output_path = mipmap_dir.join("ic_launcher_foreground.png");
        save_png(&foreground_img, &output_path, dev_mode, dev_bug)?;
        println!("  ✓ Generated android/mipmap-{density}/ic_launcher_foreground.png");

        // Generate background layer (solid color)
        let background = ImageBuffer::from_fn(size, size, |_, _| bg_color);
        let background_img = DynamicImage::ImageRgba8(background);
        let bg_output_path = mipmap_dir.join("ic_launcher_background.png");
        save_png(&background_img, &bg_output_path, false, "")?; // Don't apply dev badge to background
        println!("  ✓ Generated android/mipmap-{density}/ic_launcher_background.png");
    }

    // Generate XML configuration files for adaptive icons
    generate_adaptive_icon_xml(&android_dir)?;

    Ok(())
}

/// Generate XML configuration files for Android adaptive icons
fn generate_adaptive_icon_xml(android_dir: &Path) -> Result<()> {
    // Create mipmap-anydpi-v26 directory for adaptive icon XML
    let anydpi_dir = android_dir.join("mipmap-anydpi-v26");
    create_dir_all(&anydpi_dir)?;

    // ic_launcher.xml for adaptive square icon
    let ic_launcher_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@mipmap/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_foreground" />
</adaptive-icon>"#;

    std::fs::write(anydpi_dir.join("ic_launcher.xml"), ic_launcher_xml)
        .context("Failed to write ic_launcher.xml")?;
    println!("  ✓ Generated android/mipmap-anydpi-v26/ic_launcher.xml");

    // ic_launcher_round.xml for adaptive round icon (same layers, system handles the shape)
    let ic_launcher_round_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@mipmap/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_foreground" />
</adaptive-icon>"#;

    std::fs::write(
        anydpi_dir.join("ic_launcher_round.xml"),
        ic_launcher_round_xml,
    )
    .context("Failed to write ic_launcher_round.xml")?;
    println!("  ✓ Generated android/mipmap-anydpi-v26/ic_launcher_round.xml");

    Ok(())
}

/// Resize the bug image to the given size, maintaining the aspect ratio
fn resize_bug_with_aspect_ratio(bug_img: &DynamicImage, target_size: u32) -> DynamicImage {
    let original_width = bug_img.width() as f32;
    let original_height = bug_img.height() as f32;
    let target_size_f32 = target_size as f32;

    // Calculate aspect ratio
    let aspect_ratio = original_width / original_height;

    // Calculate new dimensions to fit within target_size while maintaining aspect ratio
    let (new_width, new_height) = if aspect_ratio > 1.0 {
        // Image is wider than tall
        let width = target_size_f32;
        let height = target_size_f32 / aspect_ratio;
        (width as u32, height as u32)
    } else {
        // Image is taller than wide (or square)
        let height = target_size_f32;
        let width = target_size_f32 * aspect_ratio;
        (width as u32, height as u32)
    };

    // Resize the image with the calculated dimensions
    bug_img.resize_exact(new_width, new_height, FilterType::Lanczos3)
}
