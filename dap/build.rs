use fuel_core::schema;
use schemafy_lib::{Expander, Schema};

use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../fuel_core/src/schema.rs");

    let path = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|p| {
            p.parent()
                .expect("Failed to uproot")
                .join("dap/assets/debugAdapterProtocol.json")
                .to_path_buf()
        })
        .expect("Failed to fetch JSON schema");

    let json = fs::read_to_string(&path).expect("Failed to parse JSON from schema");
    let schema: Schema = serde_json::from_str(&json).expect("Failed to parse Schema from JSON");
    let root_name = schema.title.clone().unwrap_or_else(|| "Root".to_owned());

    let path = path.into_os_string();
    let path = path.into_string().expect("Failed to convert path");
    let mut expander = Expander::new(Some(&root_name), path.as_str(), &schema);
    let contents = expander.expand(&schema).to_string();

    let schema = env::var("OUT_DIR")
        .map(PathBuf::from)
        .map(|mut f| {
            f.set_file_name("schema");
            f.set_extension("rs");
            f
        })
        .expect("Failed to fetch schema path");

    File::create(&schema)
        .and_then(|mut f| {
            f.write_all(contents.as_bytes())?;
            f.sync_all()
        })
        .expect("Failed to create schema.rs file");

    let sdl = env::var("OUT_DIR")
        .map(PathBuf::from)
        .map(|mut f| {
            f.set_file_name("debug");
            f.set_extension("sdl");
            f
        })
        .expect("Failed to fetch sdl path");

    let assets = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|f| f.as_path().join("assets/debug.sdl").to_path_buf())
        .expect("Failed to fetch assets path");

    File::create(&sdl)
        .and_then(|mut f| {
            f.write_all(schema::debug_schema().sdl().as_bytes())?;
            f.sync_all()
        })
        .and_then(|_| fs::copy(sdl, assets))
        .expect("Failed to create debug.sdl file");
}
