fn main() {
    tonic_build::configure()
        .build_server(true)
        .compile_protos(
            &["../../integration/protos/counter.proto"], // Path to your .proto file
            &["../../integration/protos"],               // Include path for .proto files
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
