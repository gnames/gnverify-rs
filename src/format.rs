use super::GNVerifyError;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum Format {
    CSV,
    Compact,
    Pretty,
}

impl Default for Format {
    fn default() -> Self {
        Format::CSV
    }
}
impl Format {
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
