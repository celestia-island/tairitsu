//! `tairitsu-browser-worlds`
//!
//! Provides embedded WIT world definitions for all supported browser/W3C API
//! surface packages. These are used as the offline fallback when the local
//! `target/tairitsu-wit` cache is empty and network access is unavailable.
//!
//! ## Package Layout
//!
//! - Phase 0 (hand-written, stable): `wit/*.wit`
//! - Phase A (generated, comprehensive): `wit/generated/*.wit`

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

// Helper macro to define an embedded package
macro_rules! wit_pkg {
    ($name:literal, $file:literal) => {
        EmbeddedPackage {
            id: concat!("tairitsu-browser:", $name, "@0.1.0"),
            namespace: "tairitsu-browser",
            name: $name,
            version: "0.1.0",
            files: &[($file, include_bytes!(concat!("../wit/generated/", $file)))],
        }
    };
}

/// All WIT packages embedded in this crate.
///
/// Includes both Phase 0 (hand-written) and Phase A (auto-generated) packages.
pub static EMBEDDED_PACKAGES: &[EmbeddedPackage] = &[
    // Phase 0 - Hand-written, stable interfaces
    EmbeddedPackage {
        id: "tairitsu-browser:full@0.1.0",
        namespace: "tairitsu-browser",
        name: "full",
        version: "0.1.0",
        files: &[("browser-full.wit", include_bytes!("../wit/browser-full.wit"))],
    },
    // Phase A - Auto-generated from W3C WebIDL (26 domains, 417 interfaces)
    wit_pkg!("canvas", "canvas.wit"),
    wit_pkg!("crypto", "crypto.wit"),
    wit_pkg!("css", "css.wit"),
    wit_pkg!("device", "device.wit"),
    wit_pkg!("dom", "dom.wit"),
    wit_pkg!("events", "events.wit"),
    wit_pkg!("fetch", "fetch.wit"),
    wit_pkg!("file-api", "file-api.wit"),
    wit_pkg!("geolocation", "geolocation.wit"),
    wit_pkg!("html", "html.wit"),
    wit_pkg!("indexed-db", "indexed-db.wit"),
    wit_pkg!("media", "media.wit"),
    wit_pkg!("notifications", "notifications.wit"),
    wit_pkg!("observers", "observers.wit"),
    wit_pkg!("performance", "performance.wit"),
    wit_pkg!("permissions", "permissions.wit"),
    wit_pkg!("resize-observer", "resize-observer.wit"),
    wit_pkg!("service-workers", "service-workers.wit"),
    wit_pkg!("storage", "storage.wit"),
    wit_pkg!("streams", "streams.wit"),
    wit_pkg!("url", "url.wit"),
    wit_pkg!("web-animations", "web-animations.wit"),
    wit_pkg!("webrtc", "webrtc.wit"),
    wit_pkg!("websocket", "websocket.wit"),
    wit_pkg!("websockets", "websockets.wit"),
    wit_pkg!("workers", "workers.wit"),
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
    EMBEDDED_PACKAGES
        .iter()
        .find(|p| p.namespace == namespace && p.name == name && p.version == version)
}

/// Get a list of all embedded package IDs.
pub fn list_embedded_packages() -> Vec<&'static str> {
    EMBEDDED_PACKAGES.iter().map(|p| p.id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_embedded_packages_present() {
        let expected = [
            "tairitsu-browser:full@0.1.0",
            "tairitsu-browser:canvas@0.1.0",
            "tairitsu-browser:crypto@0.1.0",
            "tairitsu-browser:css@0.1.0",
            "tairitsu-browser:device@0.1.0",
            "tairitsu-browser:dom@0.1.0",
            "tairitsu-browser:events@0.1.0",
            "tairitsu-browser:fetch@0.1.0",
            "tairitsu-browser:file-api@0.1.0",
            "tairitsu-browser:geolocation@0.1.0",
            "tairitsu-browser:html@0.1.0",
            "tairitsu-browser:indexed-db@0.1.0",
            "tairitsu-browser:media@0.1.0",
            "tairitsu-browser:notifications@0.1.0",
            "tairitsu-browser:observers@0.1.0",
            "tairitsu-browser:performance@0.1.0",
            "tairitsu-browser:permissions@0.1.0",
            "tairitsu-browser:resize-observer@0.1.0",
            "tairitsu-browser:service-workers@0.1.0",
            "tairitsu-browser:storage@0.1.0",
            "tairitsu-browser:streams@0.1.0",
            "tairitsu-browser:url@0.1.0",
            "tairitsu-browser:web-animations@0.1.0",
            "tairitsu-browser:webrtc@0.1.0",
            "tairitsu-browser:websocket@0.1.0",
            "tairitsu-browser:websockets@0.1.0",
            "tairitsu-browser:workers@0.1.0",
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
                let content = std::str::from_utf8(bytes).expect("WIT file should be valid UTF-8");
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

    #[test]
    fn count_embedded_packages() {
        // Should have 27 packages (1 hand-written + 26 generated)
        assert_eq!(EMBEDDED_PACKAGES.len(), 27);
    }
}
