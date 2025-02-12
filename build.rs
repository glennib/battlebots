fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    tonic_build::configure()
        .build_server(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["proto/battlebots.proto"], &["proto"])
        .unwrap();
}
