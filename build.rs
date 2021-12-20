use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/fsevent_pb.proto"], &["src/"])?;
    Ok(())
}
