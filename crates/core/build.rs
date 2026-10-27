use std::io::Result;

fn main() -> Result<()> {
    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &["../../proto/omnim.proto", "../../proto/omcom.proto"],
            &["../../proto"],
        )
        .expect("Failed to compile protos");

    println!("cargo:rerun-if-changed=../../proto/omnim.proto");
    println!("cargo:rerun-if-changed=../../proto/omcom.proto");

    Ok(())
}
