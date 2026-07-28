use std::sync::Arc;
use chatq_types::data::message::{Message, MessageStub};
use chatq_types::data::query::{MessageQueryPattern, MessageQueryRequest, MessageQueryRequestKind, QueryMessageResponse};
use crate::ports::{MessageEvent, MessageEventChannel, MessageEventChannelError, MessageRepo, RepoError};

pub struct Ctx<R, E>
where
    R: MessageRepo,
    E: MessageEventChannel,
{
    pub repo: Arc<R>,
    pub channel: Arc<E>,
}

#[derive(thiserror::Error, Debug)]
pub enum LogicError {
    #[error(transparent)]
    Repo(#[from] RepoError),
    #[error(transparent)]
    MessageEventChannel(#[from] MessageEventChannelError),
}

pub type LogicResult<T> = Result<T, LogicError>;

pub async fn insert_message<R, E>(ctx: &Ctx<R, E>, stub: MessageStub) -> LogicResult<Message>
where
    R: MessageRepo,
    E: MessageEventChannel,
{
    let message = ctx.repo.insert(stub).await?;

    ctx.channel
        .publish(MessageEvent::Insert(message.clone()))?;

    Ok(message)
}

pub async fn query_messages<R, E>(ctx: &Ctx<R, E>, query: MessageQueryRequest) -> LogicResult<QueryMessageResponse>
where
    R: MessageRepo,
    E: MessageEventChannel,
{
    let queried = ctx.repo.query(query.clone()).await?;

    match query.kind {
        MessageQueryRequestKind::Sessionless(inner) => {
            ctx.channel
                .publish(MessageEvent::Query(inner.pattern.clone()))?;
        }
        MessageQueryRequestKind::WithSession(_unused) => {}
    }

    Ok(queried)
}