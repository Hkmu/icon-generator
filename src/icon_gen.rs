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
use rand::Rng;
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
    pub ico_only: bool,
    pub icns_only: bool,
    pub desktop_only: bool,
    pub mobile_only: bool,
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
        _ => Err(anyhow::anyhow!(
            "Unknown bug type: {}. Available types: moth, cockroach, ladybug, spider",
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
    if args.icns_only {
        // Only macOS icons
        if should_generate_macos {
            generate_icns(&source, &args.output, args.dev_mode, &args.dev_bug)?;
        }
    } else if args.ico_only {
        generate_ico(&source, &args.output, args.dev_mode, &args.dev_bug)?;
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
    if !has_platform_flags
        && !args.icns_only
        && !args.ico_only
        && !args.desktop_only
        && !args.mobile_only
    {
        return true;
    }

    // If mobile_only is set
    if args.mobile_only {
        return true;
    }

    false
}

/// Determine when the macOS writer should be invoked
/// Only invoke when macOS icons are produced (icns_only, macos, desktop_only, default)
fn should_invoke_macos_writer(args: &Args, has_platform_flags: bool) -> bool {
    // If specific macOS flag is set
    if args.macos {
        return true;
    }

    // If icns_only is set
    if args.icns_only {
        return true;
    }

    // If desktop_only is set
    if args.desktop_only {
        return true;
    }

    // If no platform flags are set (default case - generate_all)
    if !has_platform_flags
        && !args.icns_only
        && !args.ico_only
        && !args.desktop_only
        && !args.mobile_only
    {
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
    println!("Generating icon.ico...");
    let mut frames = Vec::new();

    // Common ICO sizes
    for size in [16, 24, 32, 48, 64, 256] {
        let mut resized = source.resize_exact(size, size, FilterType::Lanczos3);

        // Apply dev badge before encoding
        if dev_mode {
            let angle = if dev_bug == "moth" {
                rand::thread_rng().gen_range(0.0..360.0)
            } else {
                0.0
            };
            apply_dev_badge_with_bug(&mut resized, dev_bug, angle)?;
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

    let mut out_file = BufWriter::new(File::create(out_dir.join("icon.ico"))?);
    let encoder = IcoEncoder::new(&mut out_file);
    encoder.encode_images(&frames)?;
    out_file.flush()?;

    println!("✓ Generated icon.ico");
    Ok(())
}

fn generate_icns(
    source: &DynamicImage,
    out_dir: &Path,
    dev_mode: bool,
    dev_bug: &str,
) -> Result<()> {
    println!("Generating icon.icns...");
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
            let angle = if dev_bug == "moth" {
                rand::thread_rng().gen_range(0.0..360.0)
            } else {
                0.0
            };
            apply_dev_badge_with_bug(&mut image, dev_bug, angle)?;
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

    let mut out_file = BufWriter::new(File::create(out_dir.join("icon.icns"))?);
    family.write(&mut out_file)?;
    out_file.flush()?;

    println!("✓ Generated icon.icns");

    // Step 3: Generate Contents.json for macOS
    let macos_images = build_macos_contents_json(&entries)?;
    write_macos_contents_json(out_dir, macos_images)?;

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
    println!("Generating Linux desktop icons...");
    let desktop_sizes = [32, 64, 128, 256, 512];
    for size in desktop_sizes {
        let filename = if size == 512 {
            "icon.png".to_string()
        } else {
            format!("{size}x{size}.png")
        };

        let resized = source.resize_exact(size, size, FilterType::Lanczos3);
        let output_path = out_dir.join(&filename);
        save_png(&resized, &output_path, dev_mode, dev_bug)?;
        println!("  ✓ Generated {filename}");
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

    let sizes = [
        (20, vec![1, 2, 3]),
        (29, vec![1, 2, 3]),
        (40, vec![1, 2, 3]),
        (60, vec![2, 3]),
        (76, vec![1, 2]),
        (83, vec![2]), // 83.5 -> 83
        (1024, vec![1]),
    ];

    for (base_size, multipliers) in sizes {
        for multiplier in multipliers {
            let actual_size = base_size * multiplier;
            let filename = if base_size == 1024 {
                "AppIcon-1024x1024.png".to_string()
            } else {
                format!("AppIcon-{base_size}x{base_size}@{multiplier}x.png")
            };

            let mut resized = source.resize_exact(actual_size, actual_size, FilterType::Lanczos3);

            // Add background color for iOS icons
            let mut bg_img = ImageBuffer::from_fn(actual_size, actual_size, |_, _| bg_color);
            image::imageops::overlay(&mut bg_img, &resized, 0, 0);
            resized = DynamicImage::ImageRgba8(bg_img);

            let output_path = ios_dir.join(&filename);
            save_png(&resized, &output_path, dev_mode, dev_bug)?;
            println!("  ✓ Generated ios/{filename}");

            // Immediately after PNG is written, create ImageEntry
            let expected_size = if base_size == 1024 {
                1024
            } else {
                base_size * multiplier
            };
            let idiom = determine_ios_idiom(base_size, multiplier);
            let size_str = if base_size == 83 {
                "83.5x83.5".to_string() // Special case for 83.5
            } else {
                format!("{base_size}x{base_size}")
            };

            let mut image_entry = ImageEntry::new_app_icon(
                filename,
                idiom,
                size_str,
                format!("{multiplier}x"),
                determine_ios_role(base_size),
            );

            image_entry.expected_size = Some(expected_size.to_string());

            // Add watch subtypes if needed
            if let Some(subtype) = determine_watch_subtype(base_size, multiplier) {
                image_entry = image_entry.with_subtype(subtype);
            }

            images.push(image_entry);
        }
    }

    // Write Contents.json
    write_contents_json(&ios_dir, images)?;

    Ok(())
}

fn save_png(image: &DynamicImage, path: &Path, dev_mode: bool, dev_bug: &str) -> Result<()> {
    let mut img = image.clone();

    // Apply dev badge if in dev mode
    if dev_mode {
        let angle = if dev_bug == "moth" {
            rand::thread_rng().gen_range(0.0..360.0)
        } else {
            0.0
        };
        apply_dev_badge_with_bug(&mut img, dev_bug, angle)?;
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

/// Determine the appropriate iOS idiom based on size and multiplier
fn determine_ios_idiom(base_size: u32, _multiplier: u32) -> String {
    match base_size {
        1024 => "ios-marketing".to_string(),
        20 | 29 | 40 | 60 => "iphone".to_string(), // iPhone sizes
        76 | 83 => "ipad".to_string(),             // iPad sizes
        _ => "universal".to_string(),
    }
}

/// Determine the role for an iOS icon based on the base size
fn determine_ios_role(base_size: u32) -> Option<String> {
    match base_size {
        20 => Some("notificationCenter".to_string()),
        29 => Some("companionSettings".to_string()),
        40 => Some("spotlight".to_string()),
        60 | 76 => Some("appLauncher".to_string()),
        83 => Some("appLauncher".to_string()), // iPad Pro app launcher
        _ => None,
    }
}

/// Determine watch subtype (not applicable for our current simple sizes)
fn determine_watch_subtype(_base_size: u32, _multiplier: u32) -> Option<String> {
    // For now, we don't generate watch-specific subtypes in our simplified generation
    // This would be expanded based on the comprehensive Contents.json example
    None
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
