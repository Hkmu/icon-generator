use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod contents_json;
mod icon_gen;

#[derive(Debug, Parser)]
#[clap(
    name = "icon-gen",
    about = "Generate various icons for all major platforms"
)]
struct Args {
    /// Path to the source icon (squared PNG or SVG file with transparency).
    #[clap(value_name = "INPUT")]
    input: PathBuf,

    /// Output directory.
    #[clap(short, long, value_name = "DIR", default_value = "./icons")]
    output: PathBuf,

    /// Custom PNG icon sizes to generate. When set, only these sizes are generated.
    #[clap(short, long, value_delimiter = ',', value_name = "SIZES")]
    png: Option<Vec<u32>>,

    /// Generate only ICO format (Windows icons)
    #[clap(long)]
    ico_only: bool,

    /// Generate only ICNS format (macOS icons)
    #[clap(long)]
    icns_only: bool,

    /// Generate only desktop platform icons (Windows, macOS, Linux)
    #[clap(long)]
    desktop_only: bool,

    /// Generate only mobile platform icons (Android, iOS)
    #[clap(long)]
    mobile_only: bool,

    /// Generate icons for Windows platform
    #[clap(long)]
    windows: bool,

    /// Generate icons for macOS platform
    #[clap(long)]
    macos: bool,

    /// Generate icons for Linux/Desktop platform
    #[clap(long)]
    linux: bool,

    /// Generate icons for Android platform
    #[clap(long)]
    android: bool,

    /// Generate round icons for Android (ic_launcher_round) - enabled by default with --android
    #[clap(long)]
    android_round: bool,

    /// Generate adaptive icons for Android (with foreground/background layers)
    #[clap(long)]
    android_adaptive: bool,

    /// Background color for Android adaptive icons (CSS color format)
    #[clap(long, default_value = "#ffffff")]
    android_adaptive_bg: String,

    /// Generate icons for iOS platform
    #[clap(long)]
    ios: bool,

    /// The background color for iOS icons (CSS color format)
    #[clap(long, default_value = "#ffffff")]
    ios_color: String,

    /// Add a development/debug badge to all generated icons
    #[clap(long, alias = "debug")]
    dev_mode: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Convert to icon_gen::Args
    let icon_args = icon_gen::Args {
        input: args.input,
        output: args.output,
        png: args.png,
        ico_only: args.ico_only,
        icns_only: args.icns_only,
        desktop_only: args.desktop_only,
        mobile_only: args.mobile_only,
        windows: args.windows,
        macos: args.macos,
        linux: args.linux,
        android: args.android,
        android_round: args.android_round || args.android, // Enable round by default with android
        android_adaptive: args.android_adaptive,
        android_adaptive_bg: args.android_adaptive_bg,
        ios: args.ios,
        ios_color: args.ios_color,
        dev_mode: args.dev_mode,
    };

    icon_gen::generate_icons(icon_args)
}
