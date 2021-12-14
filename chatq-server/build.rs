fn main() {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");

    tonic_build::compile_protos("proto/chatq.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
