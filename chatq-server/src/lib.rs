pub mod data;
pub mod entity;

use chatq::Servers;
use sea_orm::{ActiveModelTrait, EntityTrait};

use std::{time::Duration};

use entity::prelude::*;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, Set};
use data::models::*;

pub mod chatq {
    tonic::include_proto!("chatq");
}

#[tonic::async_trait]
pub trait Db {
    async fn insert_message(&self, stub: MessageStub) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct ChatQDb {
    db: DatabaseConnection,
}

impl ChatQDb {
    pub async fn new(addr: &str) -> anyhow::Result<Self> {
        let mut opt = ConnectOptions::new(addr.to_owned());
        opt.max_connections(100)
            .min_connections(5)
            .idle_timeout(Duration::from_secs(8))
            .sqlx_logging(true);

        let db = Database::connect(opt).await?;

        Ok(ChatQDb { db })
    }
}

#[tonic::async_trait]
impl Db for ChatQDb {
    async fn insert_message(&self, stub: MessageStub) -> Result<(), Box<dyn std::error::Error>> {
        
        let mesg = entity::messages::ActiveModel {
            issued: Set(stub.timestamp),
            content: Set(stub.content),
            ..Default::default()
        };

        let mesg = mesg.insert(&self.db).await?;

        let message_id = mesg.message_id.unwrap();

        match stub.audience {
            MessageAudience::Players(players) => {

                let mut auds: Vec<entity::audiences_player::ActiveModel> = Vec::new();

                for uuid in players {
                    let aud = entity::audiences_player::ActiveModel {
                        player: Set(uuid),
                        message_id: Set(message_id),
                    };

                    auds.push(aud);
                }

                AudiencesPlayer::insert_many(auds).exec(&self.db).await?;
            },
            MessageAudience::Servers(servers) => {

                let mut auds: Vec<entity::audiences_server::ActiveModel> = Vec::new();

                for server in servers {
                    let aud = entity::audiences_server::ActiveModel {
                        server: Set(server.value),
                        message_id: Set(message_id),
                    };

                    auds.push(aud);
                }

                AudiencesServer::insert_many(auds).exec(&self.db).await?;
            },
        }

        Ok(())
    }
}
