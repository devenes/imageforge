use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum AppError {
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("Output dimensions changed unexpectedly: expected {expected_w}x{expected_h}, got {actual_w}x{actual_h}")]
    DimensionMismatch {
        expected_w: u32,
        expected_h: u32,
        actual_w: u32,
        actual_h: u32,
    },

    #[error("Could not read image file: {user_message}")]
    DecodeFailed {
        user_message: String,
        technical_detail: String,
    },

    #[error("Could not compress image: {user_message}")]
    EncodeFailed {
        user_message: String,
        technical_detail: String,
    },

    #[error("File operation failed: {0}")]
    IoError(String),

    #[error("No meaningful savings found. Original file retained.")]
    NoSavings,

    #[error("Operation was cancelled")]
    Cancelled,

    #[error("Validation failed: {0}")]
    Validation(String),
}

impl AppError {
    pub fn decode(user_message: impl Into<String>, technical_detail: impl Into<String>) -> Self {
        Self::DecodeFailed {
            user_message: user_message.into(),
            technical_detail: technical_detail.into(),
        }
    }

    pub fn encode(user_message: impl Into<String>, technical_detail: impl Into<String>) -> Self {
        Self::EncodeFailed {
            user_message: user_message.into(),
            technical_detail: technical_detail.into(),
        }
    }

    pub fn user_friendly_message(&self) -> String {
        match self {
            Self::UnsupportedFormat(fmt) => format!("Unsupported file type: {fmt}"),
            Self::DimensionMismatch {
                expected_w,
                expected_h,
                actual_w,
                actual_h,
            } => {
                format!("Output dimensions changed unexpectedly ({actual_w}x{actual_h} instead of {expected_w}x{expected_h}).")
            }
            Self::DecodeFailed { user_message, .. } => user_message.clone(),
            Self::EncodeFailed { user_message, .. } => user_message.clone(),
            Self::IoError(msg) => format!("File access error: {msg}"),
            Self::NoSavings => "No meaningful savings found. Original file retained.".to_string(),
            Self::Cancelled => "Operation cancelled.".to_string(),
            Self::Validation(msg) => msg.clone(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}
