use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = find_workspace_root(&manifest_dir);

    // Watch for template changes
    let template_js = manifest_dir.join("src/wasm/component-wrapper-loader.template.js");
    println!("cargo:rerun-if-changed={}", template_js.display());

    // Watch for runtime changes (all files in the runtime/ folder)
    let runtime_dir = workspace_root.join("packages/browser-glue/src/runtime");
    if runtime_dir.is_dir() {
        for entry in std::fs::read_dir(&runtime_dir)
            .into_iter()
            .flatten()
            .flatten()
        {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    } else {
        // Fallback: watch legacy monolithic file if runtime/ dir doesn't exist
        println!(
            "cargo:rerun-if-changed={}",
            runtime_dir.with_extension("ts").display()
        );
    }

    // Watch for glue files in src/glue/
    let glue_files = [
        "console.ts",
        "style.ts",
        "event-target.ts",
        "css.ts",
        "dom.ts",
    ];
    for file in &glue_files {
        let path = workspace_root
            .join("packages/browser-glue/src/glue")
            .join(file);
        println!("cargo:rerun-if-changed={}", path.display());
    }
    // Also watch the shared helpers and runtime modules
    for file in &["handles.ts", "async.ts"] {
        let path = workspace_root.join("packages/browser-glue/src").join(file);
        println!("cargo:rerun-if-changed={}", path.display());
    }
    // Watch runtime modules (these are bundled into the glue)
    if runtime_dir.is_dir() {
        for entry in std::fs::read_dir(&runtime_dir)
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let generated_rs = out_dir.join("browser_glue_bundle.rs");

    // Try to compile runtime.ts with SWC
    let bundle_content = if let Some(compiled) = compile_with_swc(&workspace_root) {
        compiled
    } else if let Ok(src) = std::fs::read_to_string(runtime_dir.join("index.ts")) {
        // Fallback: use TypeScript source (browser will handle it)
        src
    } else {
        // Ultimate fallback: minimal inline implementation
        generate_minimal_fallback()
    };

    let escaped = escape_rust_string(&bundle_content);
    let rs_content = format!(
        r#"/// Auto-generated browser glue bundle - DO NOT EDIT
pub const BROWSER_GLUE_BUNDLE: &str = "{}";
#[allow(dead_code)]
pub const BROWSER_GLUE_BUNDLE_SIZE: usize = {};"#,
        escaped,
        bundle_content.len()
    );

    if let Err(e) = std::fs::write(&generated_rs, rs_content) {
        println!("cargo:warning=Failed to write browser-glue bundle: {}", e);
    }
}

fn compile_with_swc(workspace_root: &Path) -> Option<String> {
    let src_file = workspace_root.join("packages/browser-glue/src/runtime/index.ts");
    let out_file = workspace_root.join("packages/browser-glue/dist/runtime.js");

    // Ensure the dist directory exists
    if let Some(parent) = out_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let output = Command::new("npx")
        .args([
            "esbuild",
            &src_file.to_string_lossy(),
            "--bundle",
            &format!("--outfile={}", out_file.to_string_lossy()),
            "--format=esm",
            "--platform=browser",
        ])
        .current_dir(workspace_root.join("packages/browser-glue"))
        .output();

    if let Ok(output) = output
        && output.status.success()
        && let Ok(content) = std::fs::read_to_string(&out_file)
    {
        return Some(content);
    }

    None
}

fn escape_rust_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            c if c.is_ascii_control() => format!("\\u{:04x}", c as u32),
            c => c.to_string(),
        })
        .collect()
}

fn find_workspace_root(manifest_dir: &Path) -> PathBuf {
    let mut current = manifest_dir.parent();
    while let Some(dir) = current {
        if dir.join("Cargo.toml").exists()
            && let Ok(cargo_toml) = std::fs::read_to_string(dir.join("Cargo.toml"))
            && cargo_toml.contains("[workspace]")
        {
            return dir.to_path_buf();
        }
        current = dir.parent();
    }
    manifest_dir.parent().unwrap().to_path_buf()
}

fn generate_minimal_fallback() -> String {
    r#"const e=new Map,n=new Map,o=new Map;let t=1n;function i(e){const n=o.get(e)||o.get(e);if(!n)throw new Error("Node handle "+e+" not found");return n}globalThis.__elementHandles=o,globalThis.__nodeHandles=n;const l={log(e){console.log(e)},warn(e){console.warn(e)},error(e){console.error(e)}},r={setStyleProperty(e,n,t){try{(function(e){const n=o.get(e);if(!n)throw new Error("Element handle "+e+" not found");return n})(e).style.setProperty(n,t)}catch(e){return String(e)}},getStyleProperty(e,n){return(function(e){const n=o.get(e);if(!n)throw new Error("Element handle "+e+" not found");return n})(e).style.getPropertyValue(n)||void 0},removeStyleProperty(e,n){try{(function(e){const n=o.get(e);if(!n)throw new Error("Element handle "+e+" not found");return n})(e).style.removeProperty(n)}catch(e){return String(e)}}};let a=1n;const s=new Map,c=new Map;let d=1n;const u={addEventListener(t,i,l){try{const r=o.get(t);if(!r)return"Target handle "+t+" not found";const a=e=>{const n=d++;c.set(n,e)};return r.addEventListener(i,a,l),s.set(a++,{target:r,type:i,listener:a,useCapture:l}),a}catch(e){return String(e)}},removeEventListener(e,n){try{const o=s.get(n);if(!o)return"Listener ID "+n+" not found";o.target.removeEventListener(o.type,o.listener,o.useCapture),s.delete(n)}catch(e){return String(e)}},preventDefault(e){c.get(e)?.preventDefault()},stopPropagation(e){c.get(e)?.stopPropagation()}},g={createElement(e,n){const i=document.createElement(e);return function(e){if(!e)return;const n=t++;return o.set(n,e),n}(i)},createTextNode(e){const o=document.createTextNode(e);return function(e){if(!e)return;const o=t++;return n.set(o,e),o}(o)},getBody(){return function(e){if(!e)return;const n=t++;return o.set(n,e),n}(document.body)}},p={setAttribute(e,n,t){(function(e){const n=o.get(e);if(!n)throw new Error("Element handle "+e+" not found");return n})(e).setAttribute(n,t)},removeAttribute(e,n){(function(e){const n=o.get(e);if(!n)throw new Error("Element handle "+e+" not found");return n})(e).removeAttribute(n)}},f={appendChild(e,o){const t=i(e),l=i(o);return function(e){if(!e)return;const o=t++;return n.set(o,e),o}(t.appendChild(l))},removeChild(e,o){const t=i(e),l=i(o);return function(e){if(!e)return;const o=t++;return n.set(o,e),o}(t.removeChild(l))},setTextContent(e,n){i(e).textContent=n??null},getTextContent(e){return i(e).textContent??void 0}},m={getElementById(e,n){const t=document.getElementById(n);return function(e){if(!e)return;const n=t++;return o.set(n,e),n}(t)}},w={getInnerWidth:()=>window.innerWidth,getInnerHeight:()=>window.innerHeight},h={console:l,style:r,"event-target":u,document:g,element:p,node:f,"non-element-parent-node":m,window:w};function y(e){const n=[];for(const[o,t]of Object.entries(e))n.push("export const "+o+" = "+t.toString()+";");return n.join("\n")}!function(){const e={};for(const[n,o]of Object.entries(h)){const t=y(o),i=new Blob([t],{type:"application/javascript"}),l=URL.createObjectURL(i);e["@tairitsu-glue/"+n]=l}let n=document.querySelector('script[type="importmap"]');n?function(){try{const o=JSON.parse(n.textContent||"{}");o.imports={...o.imports,...e},n.textContent=JSON.stringify(o)}catch{n.textContent=JSON.stringify({imports:e})}}():((n=document.createElement("script")).type="importmap",n.textContent=JSON.stringify({imports:e}),document.head.prepend(n))}();globalThis.__TAIRITSU_GLUE__={INTERFACES:h,handles:{_elementHandles:o,_nodeHandles:n,_documentHandles:e}};"#.to_string()
}
