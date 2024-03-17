fn main() {
    #[cfg(feature = "proto")]
    tonic()
}

#[cfg(feature = "proto")]
fn tonic() {
    tonic_build::compile_protos("../proto/chatq.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
