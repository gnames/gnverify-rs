use thiserror::Error;

#[derive(Error, Debug)]
pub enum GNVerifyError {
    #[error("cannot convert {format:?} to an output format value")]
    InvalidFormatString { format: String },
}
