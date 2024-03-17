use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ModelConversionError {
    #[error("Inner value was not provided")]
    ValueNotProvided(&'static str),
    #[error("uuid conversion failure")]
    #[cfg(feature = "proto")]
    UuidConversion(#[from] sqlx::types::uuid::Error),
}
