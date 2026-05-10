//! Build script: verify SCSS class extraction grammar exists.
//!
//! The actual parsing is done inside the proc macro itself using a
//! regex+state-machine extractor that mirrors the pest grammar in
//! `src/scss_classes.pest`. This avoids depending on `pest_generator`'s
//! unstable build-script API while keeping the grammar as the source
//! of truth for what constitutes a "class selector".

fn main() {
    println!("cargo:rerun-if-changed=src/scss_classes.pest");
    println!("cargo:rustc-cfg=has_scss_grammar");
}
