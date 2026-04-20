fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("failed to find protoc");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["proto/event_bridge.proto"], &["proto"])
        .expect("failed to compile protos");
}
