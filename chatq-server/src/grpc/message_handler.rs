use crate::message_handler::MessageHandler;
use anyhow::Context;
use chatq_types::chatq::{
    FetchSnapshotResponse, GenerateSnapshotResponse, SnapshotFetchRequest, SnapshotGenerateRequest,
};
use chatq_types::data::query::{MessageQueryPattern, MessageQueryRequest};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use chatq_types::chatq;
use chatq_types::chatq::{
    MessageBroadcast, MessageInsertRequest, MessageInsertResponse, MessageListenRequest,
    QueryMessageResponse,
};
use chatq_types::data::filter::{EvaluableFilter, MessageFilterPattern};
use chatq_types::data::message::Message;
use crate::{logic, ChatQDao};
use crate::event_channel::BroadcastMessageEventChannel;
use crate::logic::Ctx;

pub struct GrpcMessageHandler {
    pub logic: Ctx<ChatQDao, BroadcastMessageEventChannel>,
}

impl GrpcMessageHandler {
    pub fn new(logic: Ctx<ChatQDao, BroadcastMessageEventChannel>) -> Self {
        Self { logic }
    }
}

#[async_trait]
impl chatq::message_handler_server::MessageHandler for GrpcMessageHandler {
    async fn insert_messages(
        &self,
        request: Request<MessageInsertRequest>,
    ) -> Result<Response<MessageInsertResponse>, Status> {
        let req = request.into_inner();
        let stub = req
            .stub
            .ok_or_else(|| Status::invalid_argument("MessageStub not present"))?;

        let insert = logic::insert_message(
            &self.logic,
            stub.clone().try_into().map_err(|err| {
                Status::invalid_argument(format!("Invalid message stub :: {}", err))
            })?
        )
            .await
            .map_err(|err| Status::aborted(err.to_string()))?;

        Ok(Response::new(MessageInsertResponse {
            message: Some(insert.into()),
        }))
    }

    type ListenMessagesStream = ReceiverStream<Result<MessageBroadcast, Status>>;

    async fn listen_messages(
        &self,
        request: Request<MessageListenRequest>,
    ) -> Result<Response<Self::ListenMessagesStream>, Status> {

        unimplemented!();

        // let (tx, rx) = mpsc::channel(4);
        //
        // let req = request.into_inner().clone();
        //
        // let filter = req
        //     .pattern
        //     .ok_or(Status::invalid_argument("filter not present"))?;
        //
        // let filter: MessageFilterPattern = filter
        //     .try_into()
        //     .map_err(|err| Status::invalid_argument(format!("invalid filter :: {}", err)))?;
        //
        // let mut subscribe_rx = self.subscribe_tx.subscribe();
        // tokio::spawn(async move {
        //     while let Ok(message) = subscribe_rx.recv().await {
        //         let internal: chatq::Message = match message
        //             .clone()
        //             .msg
        //             .ok_or(Status::internal("message not present"))
        //         {
        //             Ok(val) => val,
        //             Err(_) => {
        //                 continue;
        //             }
        //         };
        //
        //         let internal: Message = match internal.try_into() {
        //             Ok(val) => val,
        //             Err(_) => {
        //                 continue;
        //             }
        //         };
        //
        //         if filter.evaluate(&internal) {
        //             match tx.send(Ok(message)).await {
        //                 Ok(_) => {}
        //                 Err(_) => {
        //                     break;
        //                 }
        //             }
        //         }
        //     }
        // });
        //
        // Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn query_messages(
        &self,
        request: tonic::Request<chatq_types::chatq::MessageQueryRequest>,
    ) -> Result<Response<QueryMessageResponse>, Status> {
        let req = request.into_inner();

        println!("{:#?}", req);

        let query: MessageQueryRequest = req
            .try_into()
            .map_err(|err| Status::invalid_argument(format!("invalid request :: {}", err)))?;

        let resp = logic::query_messages(&self.logic, query).await
            .map_err(|err| Status::aborted(err.to_string()))?;

        Ok(Response::new(resp.into()))
    }

    async fn generate_snapshot(
        &self,
        request: Request<SnapshotGenerateRequest>,
    ) -> Result<Response<GenerateSnapshotResponse>, Status> {

        unimplemented!();

        // let req = request.into_inner();
        // let query = req
        //     .query
        //     .ok_or(Status::invalid_argument("pattern not present"))?;
        //
        // let query: MessageQueryPattern = query
        //     .try_into()
        //     .map_err(|err| Status::invalid_argument(format!("invalid filter :: {}", err)))?;
        //
        // let target: Uuid = req
        //     .target
        //     .ok_or(Status::invalid_argument("uuid not present"))?
        //     .try_into()
        //     .map_err(|err| Status::invalid_argument(format!("invalid uuid :: {}", err)))?;
        //
        // let res = self
        //     .handler
        //     .db
        //     .generate_snapshot(target, &query)
        //     .await
        //     .map_err(|err| Status::internal(format!("database failure {}", err)))?;
        //
        // let resp = GenerateSnapshotResponse {
        //     id: Some(res.id.into()),
        //     snapshot: Some(res.into()),
        // };
        //
        // Ok(Response::new(resp))
    }

    async fn fetch_snapshot(
        &self,
        request: Request<SnapshotFetchRequest>,
    ) -> Result<Response<FetchSnapshotResponse>, Status> {

        unimplemented!();
        //
        // let req = request
        //     .into_inner()
        //     .id
        //     .ok_or(Status::invalid_argument("id not present."))?
        //     .try_into()
        //     .map_err(|err| Status::internal(format!("invalid id :: {}", err)))?;
        //
        // let res = self
        //     .handler
        //     .db
        //     .fetch_snapshot(req)
        //     .await
        //     .map_err(|err| Status::internal(format!("database failure {}", err)))?;
        //
        // return match res {
        //     None => Ok(Response::new(FetchSnapshotResponse {
        //         snapshot: None,
        //         map: None,
        //     })),
        //     Some(snapshot) => Ok(Response::new(FetchSnapshotResponse {
        //         snapshot: Some(snapshot.into()),
        //         map: None,
        //     })),
        // };
    }
}
