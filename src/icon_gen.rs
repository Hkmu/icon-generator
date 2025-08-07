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
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
    path::Path,
    str::FromStr,
};

#[derive(Debug, Deserialize)]
struct IcnsEntry {
    size: u32,
    ostype: String,
}

pub fn generate_icons(args: super::Args) -> Result<()> {
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
            generate_icns(&source, &args.output)?;
        }
    } else if args.ico_only {
        generate_ico(&source, &args.output)?;
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
fn should_invoke_ios_writer(args: &super::Args, has_platform_flags: bool) -> bool {
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
fn should_invoke_macos_writer(args: &super::Args, has_platform_flags: bool) -> bool {
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
    args: &super::Args,
    should_generate_ios: bool,
    should_generate_macos: bool,
) -> Result<()> {
    if let Some(sizes) = &args.png {
        generate_custom_sizes(source, sizes, &args.output)?;
    } else {
        // Generate default formats when no specific platform flags are set
        generate_ico(source, &args.output)?;

        if should_generate_macos {
            generate_icns(source, &args.output)?;
        }

        generate_linux_icons(source, &args.output)?;
        generate_mobile(source, args, should_generate_ios)?;
    }

    Ok(())
}

fn generate_desktop_only(
    source: &DynamicImage,
    args: &super::Args,
    should_generate_macos: bool,
) -> Result<()> {
    if let Some(sizes) = &args.png {
        generate_custom_sizes(source, sizes, &args.output)?;
    } else {
        generate_ico(source, &args.output)?;

        if should_generate_macos {
            generate_icns(source, &args.output)?;
        }

        generate_linux_icons(source, &args.output)?;
    }
    Ok(())
}

fn generate_mobile_only(
    source: &DynamicImage,
    args: &super::Args,
    should_generate_ios: bool,
) -> Result<()> {
    generate_mobile(source, args, should_generate_ios)?;
    Ok(())
}

fn generate_platforms(
    source: &DynamicImage,
    args: &super::Args,
    should_generate_ios: bool,
    should_generate_macos: bool,
) -> Result<()> {
    if args.windows {
        generate_ico(source, &args.output)?;
    }

    if args.macos && should_generate_macos {
        generate_icns(source, &args.output)?;
    }

    if args.linux {
        if let Some(sizes) = &args.png {
            generate_custom_sizes(source, sizes, &args.output)?;
        } else {
            generate_linux_icons(source, &args.output)?;
        }
    }

    if args.android {
        generate_android_icons(source, &args.output)?;
    }

    if args.ios && should_generate_ios {
        generate_ios_icons(source, &args.output, &args.ios_color)?;
    }

    Ok(())
}

fn generate_ico(source: &DynamicImage, out_dir: &Path) -> Result<()> {
    println!("Generating icon.ico...");
    let mut frames = Vec::new();

    // Common ICO sizes
    for size in [16, 24, 32, 48, 64, 256] {
        let resized = source.resize_exact(size, size, FilterType::Lanczos3);
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

fn generate_icns(source: &DynamicImage, out_dir: &Path) -> Result<()> {
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
        let image = source.resize_exact(entry.size, entry.size, FilterType::Lanczos3);
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

fn generate_custom_sizes(source: &DynamicImage, sizes: &[u32], out_dir: &Path) -> Result<()> {
    println!("Generating custom PNG sizes...");
    for &size in sizes {
        let resized = source.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
        let output_path = out_dir.join(format!("{}x{}.png", size, size));
        save_png(&resized, &output_path)?;
        println!("  ✓ Generated {}x{}.png", size, size);
    }
    Ok(())
}

fn generate_linux_icons(source: &DynamicImage, out_dir: &Path) -> Result<()> {
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
        save_png(&resized, &output_path)?;
        println!("  ✓ Generated {filename}");
    }
    Ok(())
}

fn generate_mobile(
    source: &DynamicImage,
    args: &super::Args,
    should_generate_ios: bool,
) -> Result<()> {
    println!("Generating mobile platform icons...");

    // Android icons
    generate_android_icons(source, &args.output)?;

    // iOS icons with background color - only generate when appropriate flags are set
    if should_generate_ios {
        generate_ios_icons(source, &args.output, &args.ios_color)?;
    }

    Ok(())
}

fn generate_android_icons(source: &DynamicImage, out_dir: &Path) -> Result<()> {
    let android_dir = out_dir.join("android");
    create_dir_all(&android_dir)?;

    let densities = [
        ("mdpi", 48),
        ("hdpi", 72),
        ("xhdpi", 96),
        ("xxhdpi", 144),
        ("xxxhdpi", 192),
    ];

    for (density, size) in densities {
        let mipmap_dir = android_dir.join(format!("mipmap-{density}"));
        create_dir_all(&mipmap_dir)?;

        let resized = source.resize_exact(size, size, FilterType::Lanczos3);
        let output_path = mipmap_dir.join("ic_launcher.png");
        save_png(&resized, &output_path)?;
        println!("  ✓ Generated android/mipmap-{density}/ic_launcher.png");
    }

    Ok(())
}

fn generate_ios_icons(source: &DynamicImage, out_dir: &Path, color: &str) -> Result<()> {
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
            save_png(&resized, &output_path)?;
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

fn save_png(image: &DynamicImage, path: &Path) -> Result<()> {
    let mut file = std::fs::File::create(path).context("Failed to create PNG file")?;
    image
        .write_to(&mut file, image::ImageOutputFormat::Png)
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
