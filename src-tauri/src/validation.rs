use crate::errors::AppError;
use crate::models::OutputMode;

/// Verify that raster pixel dimensions did not change during processing.
/// This invariant is strictly required by the specification.
pub fn verify_dimensions(
    input_w: u32,
    input_h: u32,
    output_w: u32,
    output_h: u32,
) -> Result<(), AppError> {
    if input_w != output_w || input_h != output_h {
        return Err(AppError::DimensionMismatch {
            expected_w: input_w,
            expected_h: input_h,
            actual_w: output_w,
            actual_h: output_h,
        });
    }
    Ok(())
}

/// Calculate savings and determine if compression resulted in meaningful savings.
/// For SameFormat compression: if output_bytes >= input_bytes, returns (saved_bytes, saved_percent, false)
/// For ConvertToJpg: always allowed to keep output even if larger, because format changed.
pub fn calculate_savings(
    input_bytes: u64,
    output_bytes: u64,
    output_mode: OutputMode,
) -> (u64, f64, bool) {
    if output_bytes < input_bytes {
        let saved_bytes = input_bytes - output_bytes;
        let saved_percent = (saved_bytes as f64 / input_bytes as f64) * 100.0;
        (saved_bytes, saved_percent, true)
    } else {
        match output_mode {
            OutputMode::SameFormat => (0, 0.0, false),
            OutputMode::ConvertToJpg => {
                // Growth during format conversion is valid and not treated as NO_SAVINGS failure
                (0, 0.0, true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_verification_success() {
        assert!(verify_dimensions(1920, 1080, 1920, 1080).is_ok());
    }

    #[test]
    fn test_dimension_verification_failure() {
        let res = verify_dimensions(1920, 1080, 1280, 720);
        assert!(res.is_err());
        match res.unwrap_err() {
            AppError::DimensionMismatch {
                expected_w,
                expected_h,
                actual_w,
                actual_h,
            } => {
                assert_eq!(expected_w, 1920);
                assert_eq!(expected_h, 1080);
                assert_eq!(actual_w, 1280);
                assert_eq!(actual_h, 720);
            }
            _ => panic!("Expected DimensionMismatch error"),
        }
    }

    #[test]
    fn test_savings_calculation() {
        let (saved, pct, keep) = calculate_savings(1000, 400, OutputMode::SameFormat);
        assert_eq!(saved, 600);
        assert!((pct - 60.0).abs() < 0.001);
        assert!(keep);

        let (saved, pct, keep) = calculate_savings(1000, 1200, OutputMode::SameFormat);
        assert_eq!(saved, 0);
        assert_eq!(pct, 0.0);
        assert!(!keep);

        // Conversion allows size increase
        let (_, _, keep) = calculate_savings(1000, 1200, OutputMode::ConvertToJpg);
        assert!(keep);
    }
}
