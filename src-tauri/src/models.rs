use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Jpg,
    Png,
    Webp,
    Heic,
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
            Self::Heic => "heic",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Jpg => "JPG",
            Self::Png => "PNG",
            Self::Webp => "WebP",
            Self::Heic => "HEIC",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub orientation: Option<u32>,
    pub has_alpha: bool,
    pub color_profile: Option<String>,
    pub metadata_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompressionPreset {
    BestQuality,
    #[default]
    Balanced,
    Smallest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    #[default]
    SameFormat,
    ConvertToJpg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressionSettings {
    pub preset: CompressionPreset,
    pub output_mode: OutputMode,
    pub preserve_metadata: bool,
    pub preserve_color_profile: bool,
    pub custom_output_dir: Option<String>,
    pub filename_suffix: Option<String>,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            preset: CompressionPreset::Balanced,
            output_mode: OutputMode::SameFormat,
            preserve_metadata: true,
            preserve_color_profile: true,
            custom_output_dir: None,
            filename_suffix: Some("-compressed".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompressionStatus {
    Queued,
    Processing,
    Completed,
    Skipped,
    NoSavings,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressionResult {
    pub id: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub saved_bytes: u64,
    pub saved_percent: f64,
    pub input_width: u32,
    pub input_height: u32,
    pub output_width: u32,
    pub output_height: u32,
    pub input_format: ImageFormat,
    pub output_format: ImageFormat,
    pub output_path: Option<String>,
    pub duration_ms: u64,
    pub status: CompressionStatus,
    pub error: Option<String>,
    pub color_profile_preserved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressUpdate {
    pub current_index: usize,
    pub total_count: usize,
    pub current_filename: String,
    pub stage: String,
}
