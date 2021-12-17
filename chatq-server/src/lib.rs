pub mod data;
pub mod entity;

use sea_orm::{ActiveModelTrait, EntityTrait};
use sqlx::PgPool;

use std::{time::Duration};

use entity::prelude::*;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, Set};
use data::models::*;

pub mod chatq {
    tonic::include_proto!("chatq");
}

#[tonic::async_trait]
pub trait Db {
    async fn insert_message(&self, stub: MessageStub) -> Result<Message, Box<dyn std::error::Error>>;
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

        {
            let sqlx_pool = PgPool::connect(addr).await?;
            sqlx::migrate!().run(&sqlx_pool).await?
        }

        let db = Database::connect(opt).await?;

        Ok(ChatQDb { db })
    }
}

#[tonic::async_trait]
impl Db for ChatQDb {
    async fn insert_message(&self, stub: MessageStub) -> Result<Message, Box<dyn std::error::Error>> {

        let stub_c = stub.clone();

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
                        ..Default::default()
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
                        ..Default::default()
                    };

                    auds.push(aud);
                }

                AudiencesServer::insert_many(auds).exec(&self.db).await?;
            },
        }

        match stub.source {
            MessageSource::Players(players) => {

                let mut srcs: Vec<entity::sources_player::ActiveModel> = Vec::new();

                for uuid in players {
                    let src = entity::sources_player::ActiveModel {
                        player: Set(uuid),
                        message_id: Set(message_id),
                        ..Default::default()
                    };

                    srcs.push(src);
                }

                SourcesPlayer::insert_many(srcs).exec(&self.db).await?;
            },
            MessageSource::Plugins(plugins) => {

                let mut srcs: Vec<entity::sources_plugin::ActiveModel> = Vec::new();

                for plugin in plugins {
                    let src = entity::sources_plugin::ActiveModel {
                        plugin: Set(plugin.value),
                        message_id: Set(message_id),
                        ..Default::default()
                    };

                    srcs.push(src);
                }

                SourcesPlugin::insert_many(srcs).exec(&self.db).await?;
            },
        }

        let mesg = Message::from_stub(message_id, stub_c);

        Ok(mesg)
    }
}
