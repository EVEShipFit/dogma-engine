use std::io::Result;

#[cfg(feature = "rust")]
fn main() -> Result<()> {
    prost_build::compile_protos(&["esf.proto"], &["node_modules/@eveshipfit/data/dist/"])?;
    Ok(())
}

#[cfg(not(feature = "rust"))]
fn main() -> Result<()> {
    Ok(())
}
