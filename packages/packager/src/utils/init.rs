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
tairitsu-web = {{ version = "0.4", features = ["wit-bindings"] }}
tairitsu-macros = "0.4"

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
    let lib_rs = r#"use tairitsu_macros::{component, rsx};
use tairitsu_vdom::{VNode, Signal};
use tairitsu_web::WitPlatform;

#[component]
fn App() -> VNode {
    let count = Signal::new(0);

    let count_display = count.clone();
    let increment = count.clone();
    let decrement = count.clone();

    rsx! {
        div {
            style: "font-family: sans-serif; max-width: 600px; margin: 4rem auto; text-align: center",
            h1 { "Hello, Tairitsu!" }
            p { "A full-stack framework powered by the WASM Component Model." }
            div {
                style: "margin-top: 2rem",
                button {
                    style: "font-size: 1.2rem; padding: 0.5rem 1rem; margin: 0 0.5rem; cursor: pointer",
                    onclick: move |_| {
                        let c = decrement.get();
                        decrement.set(c - 1);
                    },
                    "-"
                }
                span {
                    style: "font-size: 1.5rem; margin: 0 1rem",
                    ..txt(&count_display.get().to_string())
                }
                button {
                    style: "font-size: 1.2rem; padding: 0.5rem 1rem; margin: 0 0.5rem; cursor: pointer",
                    onclick: move |_| {
                        let c = increment.get();
                        increment.set(c + 1);
                    },
                    "+"
                }
            }
        }
    }
}

fn txt(text: &str) -> Vec<VNode> {
    vec![VNode::text(text)]
}

#[export_name = "tairitsu_component_bootstrap"]
pub extern "C" fn bootstrap() {
    let platform = WitPlatform::new().expect("Failed to create WitPlatform");
    let app = App::builder().build();
    platform.mount_vnode_to_app(app).expect("Failed to mount");
}
"#;
    std::fs::write(format!("{}/src/lib.rs", name), lib_rs)?;

    let gitignore = "/target
/dist
*.wasm
node_modules/
";
    std::fs::write(format!("{}/.gitignore", name), gitignore)?;

    let msg = t.cli.init_project_created.replace("{name}", &name);
    pb.finish_with_message(msg);

    crate::log_ok!("{}:", t.cli.init_next_steps);
    crate::log_info!("  cd {}", name);
    crate::log_info!("  tairitsu dev");

    Ok(())
}
