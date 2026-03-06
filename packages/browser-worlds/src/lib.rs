//! `tairitsu-browser-worlds`
//!
//! Provides embedded WIT world definitions for all supported browser/W3C API
//! surface packages. These are used as the offline fallback when the local
//! `target/tairitsu-wit` cache is empty and network access is unavailable.

/// Version metadata for each embedded WIT package.
#[derive(Debug, Clone)]
pub struct EmbeddedPackage {
    /// Package identifier (e.g. `tairitsu-browser:dom@0.1.0`).
    pub id: &'static str,
    /// Namespace component.
    pub namespace: &'static str,
    /// Name component.
    pub name: &'static str,
    /// Semver version string.
    pub version: &'static str,
    /// Map of filename → WIT source bytes (embedded at compile time).
    pub files: &'static [(&'static str, &'static [u8])],
}

/// All WIT packages embedded in this crate.
pub static EMBEDDED_PACKAGES: &[EmbeddedPackage] = &[
    EmbeddedPackage {
        id: "tairitsu-browser:dom@0.1.0",
        namespace: "tairitsu-browser",
        name: "dom",
        version: "0.1.0",
        files: &[("dom.wit", include_bytes!("../wit/dom.wit"))],
    },
    EmbeddedPackage {
        id: "tairitsu-browser:events@0.1.0",
        namespace: "tairitsu-browser",
        name: "events",
        version: "0.1.0",
        files: &[("events.wit", include_bytes!("../wit/events.wit"))],
    },
    EmbeddedPackage {
        id: "tairitsu-browser:fetch@0.1.0",
        namespace: "tairitsu-browser",
        name: "fetch",
        version: "0.1.0",
        files: &[("fetch.wit", include_bytes!("../wit/fetch.wit"))],
    },
    EmbeddedPackage {
        id: "tairitsu-browser:canvas@0.1.0",
        namespace: "tairitsu-browser",
        name: "canvas",
        version: "0.1.0",
        files: &[("canvas.wit", include_bytes!("../wit/canvas.wit"))],
    },
    EmbeddedPackage {
        id: "tairitsu-browser:full@0.1.0",
        namespace: "tairitsu-browser",
        name: "full",
        version: "0.1.0",
        files: &[("browser-full.wit", include_bytes!("../wit/browser-full.wit"))],
    },
];

/// Look up an embedded package by its identifier string.
pub fn find_embedded(id: &str) -> Option<&'static EmbeddedPackage> {
    EMBEDDED_PACKAGES.iter().find(|p| p.id == id)
}

/// Look up an embedded package by namespace, name, and version.
pub fn find_embedded_by_parts(
    namespace: &str,
    name: &str,
    version: &str,
) -> Option<&'static EmbeddedPackage> {
    EMBEDDED_PACKAGES.iter().find(|p| {
        p.namespace == namespace && p.name == name && p.version == version
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_embedded_packages_present() {
        let expected = [
            "tairitsu-browser:dom@0.1.0",
            "tairitsu-browser:events@0.1.0",
            "tairitsu-browser:fetch@0.1.0",
            "tairitsu-browser:canvas@0.1.0",
            "tairitsu-browser:full@0.1.0",
        ];
        for id in &expected {
            assert!(
                find_embedded(id).is_some(),
                "Missing embedded package: {id}"
            );
        }
    }

    #[test]
    fn embedded_wit_files_are_non_empty() {
        for pkg in EMBEDDED_PACKAGES {
            for (filename, bytes) in pkg.files {
                assert!(
                    !bytes.is_empty(),
                    "WIT file {filename} in package {} is empty",
                    pkg.id
                );
            }
        }
    }

    #[test]
    fn find_by_parts() {
        let pkg = find_embedded_by_parts("tairitsu-browser", "dom", "0.1.0");
        assert!(pkg.is_some());
        assert_eq!(pkg.unwrap().id, "tairitsu-browser:dom@0.1.0");
    }

    #[test]
    fn wit_files_parse_with_wit_parser() {
        use wit_parser::Resolve;

        for pkg in EMBEDDED_PACKAGES {
            for (filename, bytes) in pkg.files {
                let content = std::str::from_utf8(bytes)
                    .expect("WIT file should be valid UTF-8");
                let mut resolve = Resolve::default();
                let result = resolve.push_str(filename, content);
                assert!(
                    result.is_ok(),
                    "WIT file {} in package {} failed to parse: {:?}",
                    filename,
                    pkg.id,
                    result.err()
                );
            }
        }
    }
}
