use super::GNVerifyError;
use strum_macros::Display;

/// Indicates desired format for the output;
#[derive(Debug, Display, Clone)]
pub enum Format {
    /// Outputs verification results in CSV format.
    CSV,
    /// Outputs verification results in as a compact JSON format.
    Compact,
    /// Outputs verification results in a prettyfied JSON format.
    Pretty,
}

impl Default for Format {
    fn default() -> Self {
        Format::CSV
    }
}
impl Format {
    /// Creates a new format entity out of a string.
    pub fn new(f: &str) -> Result<Self, GNVerifyError> {
        match f {
            "pretty" => Ok(Format::Pretty),
            "compact" => Ok(Format::Compact),
            "csv" => Ok(Format::CSV),
            _ => Err(GNVerifyError::InvalidFormatString {
                format: f.to_owned(),
            }),
        }
    }
}

#[test]
fn format_as_str() {
    assert_eq!(Format::CSV.to_string(), "CSV")
}
