use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::chatq;
use crate::data::filter::MessageFilterPattern;

use super::error::ModelConversionError;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Limit {
    All,
    Amount(i32),
}

impl Display for Limit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Limit::All => write!(f, ""),
            Limit::Amount(amount) => write!(f, "LIMIT {}", amount),
        }
    }
}

impl From<Limit> for chatq::message_query_pattern::Limit {
    fn from(value: Limit) -> Self {
        match value {
            Limit::All => chatq::message_query_pattern::Limit::All(true),
            Limit::Amount(amount) => chatq::message_query_pattern::Limit::Amount(amount),
        }
    }
}

impl TryFrom<chatq::message_query_pattern::Limit> for Limit {
    type Error = ModelConversionError;

    fn try_from(value: chatq::message_query_pattern::Limit) -> Result<Self, Self::Error> {
        match value {
            chatq::message_query_pattern::Limit::All(_) => Ok(Limit::All),
            chatq::message_query_pattern::Limit::Amount(amount) => Ok(Limit::Amount(amount)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MessageQueryPattern {
    pub limit: Limit,
    pub filter: MessageFilterPattern,
}

impl From<MessageQueryPattern> for chatq::MessageQueryPattern {
    fn from(value: MessageQueryPattern) -> Self {
        Self {
            limit: Some(value.limit.into()),
            filter: Some(value.filter.into()),
        }
    }
}

impl TryFrom<chatq::MessageQueryPattern> for MessageQueryPattern {
    type Error = ModelConversionError;

    fn try_from(value: chatq::MessageQueryPattern) -> Result<Self, Self::Error> {
        Ok(Self {
            limit: value
                .limit
                .ok_or(ModelConversionError::ValueNotProvided("limit"))?
                .try_into()?,
            filter: value
                .filter
                .ok_or(ModelConversionError::ValueNotProvided("limit"))?
                .try_into()?,
        })
    }
}
