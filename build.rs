use std::error::Error;
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("paxos_descriptor.bin"))
        .compile_protos(&["./proto/paxos.proto"], &["proto"])?;

    //tonic_build::compile_protos("proto/paxos.proto")?;

    Ok(())
}
