use thiserror::Error;

/// List of error types used in the library.
#[derive(Error, Debug)]
pub enum GNVerifyError {
    /// Indicates that a user entered a string that cannot be
    /// converted to a Format type. In such case the default format
    /// (Format::CSV) wil be used.
    #[error("cannot convert {format:?} to an output format value")]
    InvalidFormatString { format: String },
}
