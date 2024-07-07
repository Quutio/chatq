use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::filter::MessageFilterPattern;
use crate::data::message::Message;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Limit {
    All,
    Amount(i32),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MessageQueryPattern {
    pub limit: Limit,
    pub filter: MessageFilterPattern,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Sessionless {
    pub page_size: i32,
    pub pattern: MessageQueryPattern,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct WithSession {
    pub session_id: Uuid,
    pub page_number: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum MessageQueryRequestKind {
    Sessionless(Sessionless),
    WithSession(WithSession),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MessageQueryRequest {
    pub kind: MessageQueryRequestKind,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct QueryMessageResponse {
    pub session_key: Uuid,
    pub total_count: i32,
    pub current_page: i32,
    pub total_pages: i32,
    pub messages: Vec<Message>,
}

#[cfg(feature = "proto")]
pub mod from_proto {
    use crate::chatq;
    use crate::chatq::message_query_request::Kind;
    use crate::data::error::ModelConversionError;
    use crate::data::message::Message;
    use crate::data::query::{
        Limit, MessageQueryPattern, MessageQueryRequest, MessageQueryRequestKind,
        QueryMessageResponse, Sessionless, WithSession,
    };
    use std::fmt::Display;

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

    impl TryFrom<chatq::MessageQueryRequest> for MessageQueryRequest {
        type Error = ModelConversionError;
        fn try_from(value: chatq::MessageQueryRequest) -> Result<Self, Self::Error> {
            match value
                .kind
                .ok_or(ModelConversionError::ValueNotProvided("kind"))?
            {
                Kind::Sessionless(sessionless) => Ok(Self {
                    kind: MessageQueryRequestKind::Sessionless(Sessionless {
                        page_size: sessionless.page_size,
                        pattern: sessionless
                            .pattern
                            .ok_or(ModelConversionError::ValueNotProvided("pattern"))?
                            .try_into()?,
                    }),
                }),
                Kind::WithSession(with_session) => Ok(Self {
                    kind: MessageQueryRequestKind::WithSession(WithSession {
                        session_id: with_session
                            .session_id
                            .ok_or(ModelConversionError::ValueNotProvided("session_id"))?
                            .try_into()?,
                        page_number: with_session.page_number,
                    }),
                }),
            }
        }
    }

    impl From<MessageQueryRequest> for chatq::MessageQueryRequest {
        fn from(value: MessageQueryRequest) -> Self {
            match value.kind {
                MessageQueryRequestKind::Sessionless(sessionless) => Self {
                    kind: Some(Kind::Sessionless(
                        chatq::message_query_request::Sessionless {
                            page_size: sessionless.page_size,
                            pattern: Some(sessionless.pattern.into()),
                        },
                    )),
                },
                MessageQueryRequestKind::WithSession(with_session) => Self {
                    kind: Some(Kind::WithSession(
                        chatq::message_query_request::WithSession {
                            session_id: Some(with_session.session_id.into()),
                            page_number: with_session.page_number, 
                        },
                    )),
                },
            }
        }
    }

    impl TryFrom<chatq::QueryMessageResponse> for QueryMessageResponse {
        type Error = ModelConversionError;

        fn try_from(value: chatq::QueryMessageResponse) -> Result<Self, Self::Error> {
            Ok(Self {
                session_key: value.session_key.ok_or(ModelConversionError::ValueNotProvided("session_key"))?.try_into()
                    .map_err(|err| ModelConversionError::ValueNotProvided("invalid uuid"))?,
                total_count: value.total_count,
                current_page: value.current_page,
                total_pages: value.total_pages,
                messages: value
                    .messages
                    .into_iter()
                    .map(|x| x.try_into())
                    .collect::<Result<Vec<Message>, Self::Error>>()?,
            })
        }
    }

    impl From<QueryMessageResponse> for chatq::QueryMessageResponse {
        fn from(value: QueryMessageResponse) -> Self {
            Self {
                session_key: Some(value.session_key.into()),
                total_count: value.total_count,
                current_page: value.current_page,
                total_pages: value.total_pages,
                messages: value
                    .messages
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<_>>(),
            }
        }
    }
}
