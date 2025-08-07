//! Contents.json data model for Apple's Asset Catalog format
//!
//! This module defines the data structures that mirror Apple's official asset catalog
//! Contents.json schema as documented in the Asset Catalog Format Reference.
//! These structures are reusable for both iOS and macOS asset catalogs.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

/// Root structure of a Contents.json file
///
/// Represents the complete asset catalog metadata structure that includes
/// images, versioning information, and optional properties.
#[derive(Serialize, Debug, Clone)]
pub struct ContentsFile {
    /// Array of image entries for different scales, sizes, and device types
    pub images: Vec<ImageEntry>,

    /// Versioning and authorship information
    pub info: Info,

    /// Optional properties for the asset catalog
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Properties>,
}

/// Individual image entry within an asset catalog
///
/// Contains metadata and attributes for a specific image file, including
/// device targeting, scaling, sizing, and optional characteristics.
#[derive(Serialize, Debug, Clone)]
pub struct ImageEntry {
    /// The filename for the image file (.png, .jpg, .pdf, .heif, .heic, .avci)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// The device type for the image (e.g., "universal", "iphone", "ipad", "mac", "watch", "tv", "car")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idiom: Option<String>,

    /// The scale factor for the image (e.g., "1x", "2x", "3x")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,

    /// The size of the image in points (e.g., "29x29", "60x60")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    /// Expected size for app icons (used for validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_size: Option<String>,

    /// The role of the icon (e.g., "notificationCenter", "companionSettings", "appLauncher")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// The subtype for device-specific variants (e.g., "38mm", "42mm", "40mm", "44mm")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,

    /// Target folder for organized icon generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,

    /// Graphics feature set required (e.g., "metal1v2", "metal2v2")
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "graphics-feature-set"
    )]
    pub graphics_feature_set: Option<String>,

    /// Memory requirement for the asset (e.g., "1GB", "2GB")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,

    /// Color space for the image (e.g., "srgb", "display-p3")
    #[serde(skip_serializing_if = "Option::is_none", rename = "color-space")]
    pub color_space: Option<String>,

    /// Display gamut (e.g., "sRGB", "P3")
    #[serde(skip_serializing_if = "Option::is_none", rename = "display-gamut")]
    pub display_gamut: Option<String>,

    /// Compression type for the image
    #[serde(skip_serializing_if = "Option::is_none", rename = "compression-type")]
    pub compression_type: Option<String>,

    /// Horizontal language direction (e.g., "left-to-right", "right-to-left")
    #[serde(skip_serializing_if = "Option::is_none", rename = "language-direction")]
    pub language_direction: Option<String>,

    /// Screen width for Apple Watch (e.g., "<=145", ">145")
    #[serde(skip_serializing_if = "Option::is_none", rename = "screen-width")]
    pub screen_width: Option<String>,

    /// Template rendering intent (e.g., "original", "template")
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "template-rendering-intent"
    )]
    pub template_rendering_intent: Option<String>,

    /// Width class for size classes (e.g., "compact", "regular")
    #[serde(skip_serializing_if = "Option::is_none", rename = "width-class")]
    pub width_class: Option<String>,

    /// Height class for size classes (e.g., "compact", "regular")
    #[serde(skip_serializing_if = "Option::is_none", rename = "height-class")]
    pub height_class: Option<String>,
}

/// Versioning and authorship information for the asset catalog
///
/// Contains metadata about the tool that created the asset catalog
/// and the format version being used.
#[derive(Serialize, Debug, Clone)]
pub struct Info {
    /// The format version of the asset catalog (typically 1)
    pub version: u8,

    /// The application or tool that authored the asset catalog
    /// Apple recommends using your bundle ID (e.g., "com.company.app")
    pub author: String,
}

/// Optional properties for the asset catalog
///
/// Contains additional metadata such as on-demand resource tags
/// and vector preservation settings.
#[derive(Serialize, Debug, Clone)]
pub struct Properties {
    /// On-demand resource tags for the asset catalog
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "on-demand-resource-tags"
    )]
    pub on_demand_resource_tags: Option<Vec<String>>,

    /// Whether to preserve vector representation for PDF files
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "preserves-vector-representation"
    )]
    pub preserves_vector_representation: Option<bool>,
}

impl ContentsFile {
    /// Creates a new Contents.json structure with the specified author
    ///
    /// # Arguments
    /// * `author` - The author/bundle ID for the asset catalog
    ///
    /// # Returns
    /// A new ContentsFile with empty images array and basic info
    pub fn new(author: String) -> Self {
        Self {
            images: Vec::new(),
            info: Info { version: 1, author },
            properties: None,
        }
    }

    /// Adds an image entry to the contents file
    ///
    /// # Arguments
    /// * `image` - The ImageEntry to add
    pub fn add_image(&mut self, image: ImageEntry) {
        self.images.push(image);
    }

    /// Sets the properties for the contents file
    ///
    /// # Arguments
    /// * `properties` - The Properties to set
    #[allow(dead_code)]
    pub fn set_properties(&mut self, properties: Properties) {
        self.properties = Some(properties);
    }
}

impl ImageEntry {
    /// Creates a new image entry with minimal required fields
    ///
    /// # Arguments
    /// * `filename` - The filename for the image
    /// * `idiom` - The device idiom (e.g., "universal", "iphone")
    /// * `scale` - The scale factor (e.g., "1x", "2x", "3x")
    ///
    /// # Returns
    /// A new ImageEntry with the specified basic properties
    pub fn new(filename: String, idiom: String, scale: String) -> Self {
        Self {
            filename: Some(filename),
            idiom: Some(idiom),
            scale: Some(scale),
            size: None,
            expected_size: None,
            role: None,
            subtype: None,
            folder: None,
            graphics_feature_set: None,
            memory: None,
            color_space: None,
            display_gamut: None,
            compression_type: None,
            language_direction: None,
            screen_width: None,
            template_rendering_intent: None,
            width_class: None,
            height_class: None,
        }
    }

    /// Creates a new app icon entry with size and role
    ///
    /// # Arguments
    /// * `filename` - The filename for the icon
    /// * `idiom` - The device idiom
    /// * `size` - The size in points (e.g., "60x60")
    /// * `scale` - The scale factor
    /// * `role` - Optional role for the icon
    ///
    /// # Returns
    /// A new ImageEntry configured for app icons
    pub fn new_app_icon(
        filename: String,
        idiom: String,
        size: String,
        scale: String,
        role: Option<String>,
    ) -> Self {
        Self {
            filename: Some(filename),
            idiom: Some(idiom),
            scale: Some(scale),
            size: Some(size.clone()),
            expected_size: Some(size),
            role,
            subtype: None,
            folder: None,
            graphics_feature_set: None,
            memory: None,
            color_space: None,
            display_gamut: None,
            compression_type: None,
            language_direction: None,
            screen_width: None,
            template_rendering_intent: None,
            width_class: None,
            height_class: None,
        }
    }

    /// Sets the subtype for device-specific variants
    ///
    /// # Arguments
    /// * `subtype` - The device subtype (e.g., "38mm", "42mm")
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_subtype(mut self, subtype: String) -> Self {
        self.subtype = Some(subtype);
        self
    }

    /// Sets the folder for organized output
    ///
    /// # Arguments
    /// * `folder` - The target folder path
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_folder(mut self, folder: String) -> Self {
        self.folder = Some(folder);
        self
    }
}

impl Default for Info {
    /// Creates default Info with generic author
    fn default() -> Self {
        Self {
            version: 1,
            author: "icon-generator".to_string(),
        }
    }
}

/// Writes a Contents.json file to the specified directory
///
/// Creates a Contents.json file in the given directory with the provided images
/// and standard metadata (version 1, author "icon-gen").
///
/// # Arguments
/// * `dir` - The directory path where Contents.json will be written
/// * `images` - Vector of ImageEntry structs representing the icon metadata
///
/// # Returns
/// * `Result<()>` - Ok if successful, error if file writing fails
///
/// # Errors
/// Returns an error if:
/// - JSON serialization fails
/// - File writing fails
/// - Directory is not accessible
#[allow(dead_code)]
pub fn write_contents_json(dir: &Path, images: Vec<ImageEntry>) -> Result<()> {
    let cf = ContentsFile {
        images,
        info: Info {
            version: 1,
            author: "icon-gen".to_string(),
        },
        properties: None,
    };
    let json = serde_json::to_string_pretty(&cf)?;
    std::fs::write(dir.join("Contents.json"), json).context("write Contents.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contents_file_creation() {
        let contents = ContentsFile::new("com.example.app".to_string());
        assert_eq!(contents.info.author, "com.example.app");
        assert_eq!(contents.info.version, 1);
        assert!(contents.images.is_empty());
    }

    #[test]
    fn test_image_entry_creation() {
        let image = ImageEntry::new(
            "icon-60@2x.png".to_string(),
            "iphone".to_string(),
            "2x".to_string(),
        );
        assert_eq!(image.filename.unwrap(), "icon-60@2x.png");
        assert_eq!(image.idiom.unwrap(), "iphone");
        assert_eq!(image.scale.unwrap(), "2x");
    }

    #[test]
    fn test_app_icon_creation() {
        let icon = ImageEntry::new_app_icon(
            "AppIcon60x60@2x.png".to_string(),
            "iphone".to_string(),
            "60x60".to_string(),
            "2x".to_string(),
            Some("appLauncher".to_string()),
        );

        assert_eq!(icon.filename.unwrap(), "AppIcon60x60@2x.png");
        assert_eq!(icon.size.unwrap(), "60x60");
        assert_eq!(icon.expected_size.unwrap(), "60x60");
        assert_eq!(icon.role.unwrap(), "appLauncher");
    }

    #[test]
    fn test_serialization() {
        let mut contents = ContentsFile::new("com.test.app".to_string());
        contents.add_image(ImageEntry::new(
            "test.png".to_string(),
            "universal".to_string(),
            "1x".to_string(),
        ));

        let json = serde_json::to_string_pretty(&contents).unwrap();
        assert!(json.contains("test.png"));
        assert!(json.contains("com.test.app"));
    }

    #[test]
    fn test_apple_contents_json_format() {
        // Create a realistic iOS app icon Contents.json structure
        let mut contents = ContentsFile::new("com.example.iconapp".to_string());

        // Add iPhone notification icon
        contents.add_image(ImageEntry::new_app_icon(
            "AppIcon29x29.png".to_string(),
            "iphone".to_string(),
            "29x29".to_string(),
            "1x".to_string(),
            Some("notificationCenter".to_string()),
        ));

        // Add iPhone app launcher icon
        contents.add_image(ImageEntry::new_app_icon(
            "AppIcon60x60@2x.png".to_string(),
            "iphone".to_string(),
            "60x60".to_string(),
            "2x".to_string(),
            Some("appLauncher".to_string()),
        ));

        let json = serde_json::to_string_pretty(&contents).unwrap();

        // Print for demonstration
        println!("\n=== Generated Apple Contents.json ===");
        println!("{}", json);
        println!("=== End Contents.json ===");

        // Verify structure
        assert!(json.contains("images"));
        assert!(json.contains("info"));
        assert!(json.contains("AppIcon29x29.png"));
        assert!(json.contains("AppIcon60x60@2x.png"));
        assert!(json.contains("notificationCenter"));
        assert!(json.contains("appLauncher"));
        assert!(json.contains("com.example.iconapp"));
        assert!(json.contains("\"version\": 1"));
    }

    #[test]
    fn test_single_entry_json_serialization() {
        // Create a single image entry
        let image_entry = ImageEntry::new_app_icon(
            "AppIcon60x60@2x.png".to_string(),
            "iphone".to_string(),
            "60x60".to_string(),
            "2x".to_string(),
            Some("appLauncher".to_string()),
        );

        // Create Contents.json with single entry
        let mut contents = ContentsFile::new("com.test.app".to_string());
        contents.add_image(image_entry);

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&contents).unwrap();

        // Verify the expected JSON structure
        let expected_fields = [
            "\"images\":",
            "\"filename\": \"AppIcon60x60@2x.png\"",
            "\"idiom\": \"iphone\"",
            "\"size\": \"60x60\"",
            "\"scale\": \"2x\"",
            "\"expected_size\": \"60x60\"",
            "\"role\": \"appLauncher\"",
            "\"info\":",
            "\"version\": 1",
            "\"author\": \"com.test.app\"",
        ];

        for field in expected_fields {
            assert!(
                json.contains(field),
                "JSON missing expected field: {}\nActual JSON:\n{}",
                field,
                json
            );
        }

        // Verify it's valid JSON by parsing it back
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("Generated JSON should be valid");

        // Verify structure
        assert!(parsed["images"].is_array());
        assert_eq!(parsed["images"].as_array().unwrap().len(), 1);
        assert!(parsed["info"].is_object());

        // Verify the image entry structure
        let image = &parsed["images"][0];
        assert_eq!(image["filename"], "AppIcon60x60@2x.png");
        assert_eq!(image["idiom"], "iphone");
        assert_eq!(image["size"], "60x60");
        assert_eq!(image["scale"], "2x");
        assert_eq!(image["expected_size"], "60x60");
        assert_eq!(image["role"], "appLauncher");

        // Verify the info structure
        assert_eq!(parsed["info"]["version"], 1);
        assert_eq!(parsed["info"]["author"], "com.test.app");

        println!("Single entry Contents.json serialization test passed");
    }

    #[test]
    fn test_write_contents_json() {
        use std::env;
        use std::fs;

        // Create a temporary directory for testing
        let temp_dir = env::temp_dir().join("icon_gen_test");
        fs::create_dir_all(&temp_dir).unwrap();

        // Create test images
        let images = vec![
            ImageEntry::new_app_icon(
                "AppIcon29x29.png".to_string(),
                "iphone".to_string(),
                "29x29".to_string(),
                "1x".to_string(),
                Some("notificationCenter".to_string()),
            ),
            ImageEntry::new_app_icon(
                "AppIcon60x60@2x.png".to_string(),
                "iphone".to_string(),
                "60x60".to_string(),
                "2x".to_string(),
                Some("appLauncher".to_string()),
            ),
        ];

        // Write Contents.json file
        write_contents_json(&temp_dir, images).unwrap();

        // Verify file was created and has correct content
        let contents_path = temp_dir.join("Contents.json");
        assert!(contents_path.exists());

        let file_content = fs::read_to_string(&contents_path).unwrap();
        assert!(file_content.contains("images"));
        assert!(file_content.contains("info"));
        assert!(file_content.contains("AppIcon29x29.png"));
        assert!(file_content.contains("AppIcon60x60@2x.png"));
        assert!(file_content.contains("icon-gen"));
        assert!(file_content.contains("\"version\": 1"));

        // Clean up
        fs::remove_dir_all(&temp_dir).ok();
    }
}
