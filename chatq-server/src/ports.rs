use thiserror::Error;
use chatq_types::data::message::{Message, MessageStub};
use chatq_types::data::query::{MessageQueryPattern, MessageQueryRequest, QueryMessageResponse};
use tonic::async_trait;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("arbitrary error : {0}")]
    Arbitrary(String),
}

pub type MessageRepoResult<T> = Result<T, RepoError>;

#[async_trait]
pub trait MessageRepo: Send + Sync {
    async fn insert(&self, message: MessageStub) -> MessageRepoResult<Message>;
    async fn query(&self, query: MessageQueryRequest) -> MessageRepoResult<QueryMessageResponse>;
}

#[derive(Debug, Clone)]
pub enum MessageEvent {
    Insert(Message),
    Query(MessageQueryPattern),
}

#[derive(Error, Debug)]
pub enum MessageEventChannelError {
    #[error("arbitrary error : {0}")]
    Arbitrary(String),
}
pub type MessageEventChannelResult<T> = Result<T, MessageEventChannelError>;

#[async_trait]
pub trait MessageEventChannel: Send + Sync {
    fn publish(&self, event: MessageEvent);
    async fn subscribe(&self) -> MessageEventChannelResult<tokio::sync::broadcast::Receiver<MessageEvent>>;
}
