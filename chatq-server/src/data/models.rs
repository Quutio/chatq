use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct Server(String);

#[derive(Debug, Clone, PartialEq)]
pub struct Plugin(String);

#[derive(Debug, Clone, PartialEq)]
pub enum MessageAudience {
    Players(Vec<Uuid>),
    Servers(Vec<Server>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageSource {
    Players(Vec<Uuid>),
    Plugins(Vec<Plugin>),
}
