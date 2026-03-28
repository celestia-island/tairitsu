//! Build script for web-next package
//!
//! This build script generates WIT bindings from the browser-worlds package.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Add the browser-worlds directory to OUT_DIR
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let browser_worlds_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../browser-worlds");

    // Copy WIT files to OUT_DIR
    let wit_full_dir = out_dir.join("browser");
    fs::create_dir_all(&wit_full_dir).unwrap();
    fs::copy(
        browser_worlds_dir.join("wit/browser-full.wit"),
        wit_full_dir.join("browser-full.wit"),
    ).unwrap();

    let wit_geometry_dir = out_dir.join("geometry");
    fs::create_dir_all(&wit_geometry_dir).unwrap();
    fs::copy(
        browser_worlds_dir.join("wit/dom-geometry.wit"),
        wit_geometry_dir.join("dom-geometry.wit"),
    ).unwrap();

    let wit_animation_dir = out_dir.join("animation");
    fs::create_dir_all(&wit_animation_dir).unwrap();
    fs::copy(
        browser_worlds_dir.join("wit/animation.wit"),
        wit_animation_dir.join("animation.wit"),
    ).unwrap();

    let wit_media_dir = out_dir.join("media");
    fs::create_dir_all(&wit_media_dir).unwrap();
    fs::copy(
        browser_worlds_dir.join("wit/media-query.wit"),
        wit_media_dir.join("media-query.wit"),
    ).unwrap();

    // Configure cargo to regenerate when WIT files change
    println!("cargo:rerun-if-changed=../browser-worlds/wit/*.wit");
    println!("cargo:rerun-if-changed=build.rs");
}