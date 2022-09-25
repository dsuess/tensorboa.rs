use std::io::Result;

extern crate prost_build;
extern crate protobuf_src;

const PROTOS_TO_INCLUDE: [&str; 6] = [
    "src/proto/event.proto",
    "src/proto/summary.proto",
    "src/proto/tensor_shape.proto",
    "src/proto/resource_handle.proto",
    "src/proto/tensor.proto",
    "src/proto/types.proto",
];

fn main() -> Result<()> {
    std::env::set_var("PROTOC", protobuf_src::protoc());
    prost_build::compile_protos(&PROTOS_TO_INCLUDE, &["src/"]).unwrap();

    Ok(())
}
