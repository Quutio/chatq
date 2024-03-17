pub mod data;

#[cfg(feature = "proto")]
pub mod chatq {
    tonic::include_proto!("chatq");
}
