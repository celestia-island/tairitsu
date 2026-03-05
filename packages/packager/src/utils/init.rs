use indicatif::{ProgressBar, ProgressStyle};

pub fn init_project(name: &str) -> crate::Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    let name = name.to_string();

    pb.set_message("Creating project directory...");
    std::fs::create_dir_all(&name)?;
    std::fs::create_dir_all(format!("{}/src", name))?;

    pb.set_message("Writing Cargo.toml...");
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
tairitsu-vdom = "0.1"
tairitsu-hooks = "0.1"
tairitsu-macros = "0.1"
wasm-bindgen = "0.2"

[package.metadata.tairitsu]
app-name = "{}"

[package.metadata.tairitsu.build]
target = "wasm"

[package.metadata.tairitsu.dev]
port = 3000
"#,
        name, name
    );
    std::fs::write(format!("{}/Cargo.toml", name), cargo_toml)?;

    pb.set_message("Writing src/lib.rs...");
    let lib_rs = r##"use wasm_bindgen::prelude::*;
use tairitsu_macros::rsx;

#[wasm_bindgen(start)]
pub fn main() {
    let _node = rsx! {
        div {
            "Hello, Tairitsu!"
        }
    };
}
"##;
    std::fs::write(format!("{}/src/lib.rs", name), lib_rs)?;

    let msg = format!("Project {} created! ✅", name);
    pb.finish_with_message(msg);

    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  tairitsu dev");

    Ok(())
}
