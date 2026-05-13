use indicatif::{ProgressBar, ProgressStyle};

pub fn init_project(name: &str) -> crate::Result<()> {
    let t = crate::i18n::translations();
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    let name = name.to_string();

    pb.set_message(t.cli.init_creating_dir.as_str());
    std::fs::create_dir_all(&name)?;
    std::fs::create_dir_all(format!("{}/src", name))?;

    pb.set_message(t.cli.init_writing_cargo.as_str());
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
tairitsu-web = {{ version = "0.1", features = ["wit-bindings"] }}

[package.metadata.tairitsu]
app-name = "{name}"

[package.metadata.tairitsu.build]
target = "component"

[package.metadata.tairitsu.dev]
port = 3000

[profile.dev-wasm]
inherits = "release"
lto = true
opt-level = 'z'
codegen-units = 1
panic = "abort"
"#,
        name = name
    );
    std::fs::write(format!("{}/Cargo.toml", name), cargo_toml)?;

    pb.set_message(t.cli.init_writing_lib.as_str());
    let lib_rs = r#"use tairitsu_web::WitPlatform;

#[export_name = "tairitsu_component_bootstrap"]
pub extern "C" fn bootstrap() {
    let platform = WitPlatform::new().expect("Failed to create WitPlatform");
    let app = tairitsu_web::vdom::VNode::element("div", vec![], vec![
        tairitsu_web::vdom::VNode::text("Hello, Tairitsu!"),
    ]);
    platform.mount_vnode_to_app(app).expect("Failed to mount");
}
"#;
    std::fs::write(format!("{}/src/lib.rs", name), lib_rs)?;

    let msg = t.cli.init_project_created.replace("{name}", &name);
    pb.finish_with_message(msg);

    crate::log_ok!("{}:", t.cli.init_next_steps);
    crate::log_info!("  cd {}", name);
    crate::log_info!("  tairitsu dev");

    Ok(())
}
