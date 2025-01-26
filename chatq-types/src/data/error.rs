use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ModelConversionError {
    #[error("Inner value was not provided")]
    ValueNotProvided(Cow<'static, str>),
    #[error("uuid conversion failure")]
    #[cfg(feature = "proto")]
    UuidConversion(#[from] sqlx::types::uuid::Error),
}
