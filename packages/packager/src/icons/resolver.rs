use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::cache::{CacheManifest, IconCache, IconData};
use super::sources::{self, IconOrigin, IconSourceDef};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct HikariIconsMetadata {
    #[serde(default)]
    pub sets: Vec<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub set: HashMap<String, SetConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SetConfig {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub formats: Vec<String>,
    #[serde(default)]
    pub subscripts: Vec<Subscript>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Subscript {
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(default)]
    pub globs: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct ResolvedSet {
    pub source: &'static IconSourceDef,
    pub version: String,
    pub formats: Vec<String>,
    pub subscripts: Vec<Subscript>,
    pub icons: HashMap<String, IconData>,
}

#[derive(Debug)]
pub struct ResolveResult {
    pub sets: Vec<ResolvedSet>,
    pub mode: String,
}

pub fn read_consumer_metadata(manifest_dir: &Path) -> crate::Result<HikariIconsMetadata> {
    let cargo_toml = manifest_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).map_err(|e| {
        crate::TairitsuPackagerError::InvalidConfig(format!(
            "Failed to read {}: {}",
            cargo_toml.display(),
            e
        ))
    })?;

    let value: toml::Value = toml::from_str(&content).map_err(|e| {
        crate::TairitsuPackagerError::InvalidConfig(format!("Failed to parse Cargo.toml: {}", e))
    })?;

    let icons_meta = value
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("hikari"))
        .and_then(|h| h.get("icons"));

    match icons_meta {
        Some(v) => {
            let meta: HikariIconsMetadata = v.clone().try_into().map_err(|e| {
                crate::TairitsuPackagerError::InvalidConfig(format!(
                    "Invalid [package.metadata.hikari.icons]: {}",
                    e
                ))
            })?;
            Ok(meta)
        }
        None => Ok(HikariIconsMetadata::default()),
    }
}

pub fn resolve(
    metadata: &HikariIconsMetadata,
    cache: &IconCache,
) -> crate::Result<ResolveResult> {
    let mode = metadata
        .mode
        .as_deref()
        .unwrap_or("embed-svg")
        .to_string();

    if metadata.sets.is_empty() {
        return Ok(ResolveResult {
            sets: Vec::new(),
            mode,
        });
    }

    let mut resolved = Vec::with_capacity(metadata.sets.len());

    for set_name in &metadata.sets {
        let source = sources::find_source(set_name).ok_or_else(|| {
            crate::TairitsuPackagerError::InvalidConfig(format!(
                "Unknown icon set: '{}'. Available: {}",
                set_name,
                sources::ICON_SOURCES
                    .iter()
                    .map(|s| s.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;

        let set_cfg = metadata.set.get(set_name);
        let version = set_cfg
            .and_then(|c| c.version.clone())
            .unwrap_or_else(|| "latest".to_string());
        let formats = set_cfg
            .map(|c| c.formats.clone())
            .unwrap_or_else(|| vec!["svg".to_string()]);
        let subscripts = set_cfg
            .map(|c| c.subscripts.clone())
            .unwrap_or_default();

        let manifest = if cache.has_cache(set_name, &version) {
            cache.load_manifest(set_name, &version)
        } else {
            None
        };

        let icons = match manifest {
            Some(m) => m.icons,
            None => {
                if cache.is_offline() {
                    crate::log_warn!(
                        "Icon set '{}' v{} not in cache and offline mode is active — skipping",
                        set_name,
                        version
                    );
                    continue;
                }

                match fetch_set(source, &version, cache) {
                    Ok(icons) => icons,
                    Err(e) => {
                        crate::log_warn!(
                            "Failed to fetch icon set '{}' v{}: {} — skipping",
                            set_name,
                            version,
                            e
                        );
                        continue;
                    }
                }
            }
        };

        let filtered = if subscripts.is_empty() {
            icons
        } else {
            apply_subscripts(&icons, &subscripts)
        };

        crate::log_info!(
            "Resolved {} icons for set '{}' ({} subscripts)",
            filtered.len(),
            set_name,
            subscripts.len()
        );

        resolved.push(ResolvedSet {
            source,
            version,
            formats,
            subscripts,
            icons: filtered,
        });
    }

    Ok(ResolveResult { sets: resolved, mode })
}

fn apply_subscripts(
    all: &HashMap<String, IconData>,
    subscripts: &[Subscript],
) -> HashMap<String, IconData> {
    let mut result = HashMap::new();

    for sub in subscripts {
        for name in &sub.names {
            if let Some(icon) = all.get(name) {
                result.insert(name.clone(), icon.clone());
            }
        }

        if !sub.globs.is_empty() {
            for (name, icon) in all {
                let mut matches = false;
                for glob in &sub.globs {
                    if glob_match(glob, name) {
                        matches = true;
                        break;
                    }
                }
                if matches {
                    result.insert(name.clone(), icon.clone());
                }
            }
        }

        if !sub.tags.is_empty() {
            for (name, icon) in all {
                for tag in &sub.tags {
                    if icon.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                        result.insert(name.clone(), icon.clone());
                        break;
                    }
                }
            }
        }
    }

    result
}

fn glob_match(pattern: &str, name: &str) -> bool {
    if !pattern.contains('*') && !pattern.contains('?') {
        return pattern == name;
    }

    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        if pattern.starts_with('*') {
            return name.ends_with(parts[0]);
        }
        if pattern.ends_with('*') {
            return name.starts_with(parts[0]);
        }
        return pattern == name;
    }

    if parts.len() == 2 {
        let prefix = parts[0];
        let suffix = parts[1];
        if !prefix.is_empty() && !name.starts_with(prefix) {
            return false;
        }
        if !suffix.is_empty() && !name.ends_with(suffix) {
            return false;
        }
        let min_len = prefix.len() + suffix.len();
        return name.len() >= min_len;
    }

    let mut rest = name;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        match rest.find(part) {
            Some(pos) => {
                if i == 0 && pos != 0 {
                    return false;
                }
                rest = &rest[pos + part.len()..];
            }
            None => return false,
        }
    }
    if let Some(last) = parts.last() {
        if !last.is_empty() && !name.ends_with(last) {
            return false;
        }
    }
    true
}

fn fetch_set(
    source: &IconSourceDef,
    version: &str,
    cache: &IconCache,
) -> crate::Result<HashMap<String, IconData>> {
    cache.ensure_dir(source.name, version)?;

    let ver = if version == "latest" {
        resolve_latest_version(source)?
    } else {
        version.to_string()
    };

    for origin in source.origins {
        match try_fetch_from_origin(source, origin, &ver, cache) {
            Ok(icons) => return Ok(icons),
            Err(e) => {
                crate::log_warn!(
                    "Failed to fetch '{}' from {:?}: {} — trying next origin",
                    source.name,
                    origin,
                    e
                );
            }
        }
    }

    Err(crate::TairitsuPackagerError::IconFetchError(format!(
        "All origins failed for icon set '{}'",
        source.name
    )))
}

fn resolve_latest_version(source: &IconSourceDef) -> crate::Result<String> {
    for origin in source.origins {
        if let Some(pkg) = origin.npm_package() {
            let url = format!("https://registry.npmjs.org/{}/latest", pkg);
            if let Ok(resp) = reqwest::blocking::get(&url) {
                if let Ok(json) = resp.json::<serde_json::Value>() {
                    if let Some(v) = json.get("version").and_then(|v| v.as_str()) {
                        return Ok(v.to_string());
                    }
                }
            }
        }
    }
    Ok("latest".to_string())
}

fn try_fetch_from_origin(
    source: &IconSourceDef,
    origin: &IconOrigin,
    version: &str,
    cache: &IconCache,
) -> crate::Result<HashMap<String, IconData>> {
    match origin {
        IconOrigin::Npm(pkg, subpath) => fetch_from_npm(source, pkg, subpath, version, cache),
        IconOrigin::Github(owner, repo, branch) => {
            fetch_from_github(source, owner, repo, branch, version, cache)
        }
        IconOrigin::GithubMirror(mirror, owner, repo, branch) => {
            fetch_from_github_mirror(source, mirror, owner, repo, branch, version, cache)
        }
    }
}

fn fetch_from_npm(
    source: &IconSourceDef,
    pkg: &str,
    _subpath: &str,
    version: &str,
    cache: &IconCache,
) -> crate::Result<HashMap<String, IconData>> {
    let tarball_url = format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", pkg, pkg.replace('/', "-"), version);
    let dir = cache.set_dir(source.name, version);
    let tgz_path = dir.join("package.tgz");

    crate::log_info!("Downloading {}@{} from npm...", pkg, version);

    download_file(&tarball_url, &tgz_path)?;

    let extract_dir = dir.join("extracted");
    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir)?;
    }
    std::fs::create_dir_all(&extract_dir)?;

    extract_tgz(&tgz_path, &extract_dir)?;

    let package_dir = extract_dir.join("package");
    scan_and_build_cache(source, &package_dir, version, cache)
}

fn fetch_from_github(
    source: &IconSourceDef,
    owner: &str,
    repo: &str,
    branch: &str,
    version: &str,
    cache: &IconCache,
) -> crate::Result<HashMap<String, IconData>> {
    let archive_url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        owner, repo, branch
    );
    let dir = cache.set_dir(source.name, version);
    let zip_path = dir.join("source.zip");

    crate::log_info!("Downloading {}/{} from GitHub...", owner, repo);

    download_file(&archive_url, &zip_path)?;

    let extract_dir = dir.join("extracted");
    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir)?;
    }
    std::fs::create_dir_all(&extract_dir)?;

    extract_zip(&zip_path, &extract_dir)?;

    let base_dir = find_extracted_dir(&extract_dir);
    scan_and_build_cache(source, &base_dir, version, cache)
}

fn fetch_from_github_mirror(
    source: &IconSourceDef,
    mirror: &str,
    owner: &str,
    repo: &str,
    branch: &str,
    version: &str,
    cache: &IconCache,
) -> crate::Result<HashMap<String, IconData>> {
    let archive_url = format!(
        "https://{}/{}/{}/archive/refs/heads/{}.zip",
        mirror, owner, repo, branch
    );
    let dir = cache.set_dir(source.name, version);
    let zip_path = dir.join("source.zip");

    crate::log_info!(
        "Downloading {}/{} from mirror {}...",
        owner,
        repo,
        mirror
    );

    download_file(&archive_url, &zip_path)?;

    let extract_dir = dir.join("extracted");
    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir)?;
    }
    std::fs::create_dir_all(&extract_dir)?;

    extract_zip(&zip_path, &extract_dir)?;

    let base_dir = find_extracted_dir(&extract_dir);
    scan_and_build_cache(source, &base_dir, version, cache)
}

fn scan_and_build_cache(
    source: &IconSourceDef,
    base_dir: &Path,
    version: &str,
    cache: &IconCache,
) -> crate::Result<HashMap<String, IconData>> {
    let mut icons = HashMap::new();
    let mut svg_entries = Vec::new();

    let svg_pattern = source.svg_glob;
    let svg_base = base_dir.join(svg_pattern.split_once('/').map(|(p, _)| p).unwrap_or("."));

    if svg_base.exists() {
        scan_svg_dir(&svg_base, source, &mut icons, &mut svg_entries);
    }

    scan_svg_dir(base_dir, source, &mut icons, &mut svg_entries);

    if let Some(meta_file) = &source.meta_file {
        let meta_path = base_dir.join(meta_file);
        if meta_path.exists() {
            load_meta_tags(&meta_path, &mut icons, source.name);
        }
    }

    let source_data = serde_json::to_vec(&icons).unwrap_or_default();
    let source_hash = CacheManifest::compute_hash(&source_data);

    let manifest = CacheManifest {
        set_name: source.name.to_string(),
        version: version.to_string(),
        source_hash,
        icon_count: icons.len(),
        icons: icons.clone(),
    };

    cache.save_manifest(&manifest)?;
    cache.save_svg_data(source.name, version, &svg_entries)?;

    crate::log_info!(
        "Cached {} icons for {} v{}",
        icons.len(),
        source.name,
        version
    );

    Ok(icons)
}

fn scan_svg_dir(
    dir: &Path,
    source: &IconSourceDef,
    icons: &mut HashMap<String, IconData>,
    svg_entries: &mut Vec<(String, String)>,
) {
    if !dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let glob_pattern = source.svg_glob;
            if glob_pattern.contains("**") {
                scan_svg_dir(&path, source, icons, svg_entries);
            }
            continue;
        }

        if path.extension().map(|e| e == "svg").unwrap_or(false) {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                if icons.contains_key(name) {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Some(path_d) = extract_path_d(&content) {
                        let icon_data = IconData {
                            path_d: path_d.clone(),
                            tags: Vec::new(),
                            aliases: Vec::new(),
                        };
                        icons.insert(name.to_string(), icon_data);
                        svg_entries.push((name.to_string(), path_d));
                    }
                }
            }
        }
    }
}

fn extract_path_d(svg: &str) -> Option<String> {
    let start = svg.find("<path")?;
    let rest = &svg[start..];

    let d_attr = rest.find("d=\"")?;
    let d_start = d_attr + 3;
    let d_rest = &rest[d_start..];
    let d_end = d_rest.find('"')?;
    let path_d = &d_rest[..d_end];

    if path_d.is_empty() {
        return None;
    }

    Some(path_d.to_string())
}

fn load_meta_tags(meta_path: &Path, icons: &mut HashMap<String, IconData>, _set_name: &str) {
    let content = match std::fs::read_to_string(meta_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    let entries = match parsed.as_array() {
        Some(arr) => arr,
        None => {
            if let Some(obj) = parsed.as_object() {
                for (name, val) in obj {
                    if let Some(icon) = icons.get_mut(name) {
                        if let Some(tags) = val.get("tags").and_then(|t| t.as_array()) {
                            icon.tags = tags
                                .iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect();
                        }
                        if let Some(aliases) = val.get("aliases").and_then(|a| a.as_array()) {
                            icon.aliases = aliases
                                .iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect();
                        }
                    }
                }
            }
            return;
        }
    };

    for entry in entries {
        let name = match entry.get("name").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => continue,
        };

        if let Some(icon) = icons.get_mut(name) {
            if let Some(tags) = entry.get("tags").and_then(|t| t.as_array()) {
                icon.tags = tags
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
            if let Some(aliases) = entry.get("aliases").and_then(|a| a.as_array()) {
                icon.aliases = aliases
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }
    }
}

fn download_file(url: &str, dest: &Path) -> crate::Result<()> {
    #[cfg(feature = "icon-fetch")]
    {
        let client = reqwest::blocking::Client::builder()
            .user_agent(format!("tairitsu-packager/{}", crate::VERSION))
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| crate::TairitsuPackagerError::HttpError(e.to_string()))?;

        let mut resp = client.get(url).send().map_err(|e| {
            crate::TairitsuPackagerError::HttpError(format!("Failed to download {}: {}", url, e))
        })?;

        if !resp.status().is_success() {
            return Err(crate::TairitsuPackagerError::HttpError(format!(
                "HTTP {} for {}",
                resp.status(),
                url
            )));
        }

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut f = std::fs::File::create(dest)?;
        resp.copy_to(&mut f)
            .map_err(|e| crate::TairitsuPackagerError::HttpError(e.to_string()))?;
        Ok(())
    }

    #[cfg(not(feature = "icon-fetch"))]
    {
        let _ = (url, dest);
        Err(crate::TairitsuPackagerError::IconFetchError(
            "icon-fetch feature not enabled".to_string(),
        ))
    }
}

fn extract_tgz(tgz_path: &Path, dest: &Path) -> crate::Result<()> {
    #[cfg(feature = "icon-fetch")]
    {
        use std::process::Command;

        let output = Command::new("tar")
            .args(["xzf", &tgz_path.to_string_lossy(), "-C", &dest.to_string_lossy()])
            .output()
            .map_err(|e| crate::TairitsuPackagerError::IconFetchError(e.to_string()))?;

        if !output.status.success() {
            return Err(crate::TairitsuPackagerError::IconFetchError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    #[cfg(not(feature = "icon-fetch"))]
    {
        let _ = (tgz_path, dest);
        Err(crate::TairitsuPackagerError::IconFetchError(
            "icon-fetch feature not enabled".to_string(),
        ))
    }
}

fn extract_zip(zip_path: &Path, dest: &Path) -> crate::Result<()> {
    #[cfg(feature = "icon-fetch")]
    {
        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            crate::TairitsuPackagerError::IconFetchError(format!("Failed to open zip: {}", e))
        })?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                crate::TairitsuPackagerError::IconFetchError(format!(
                    "Failed to read zip entry {}: {}",
                    i, e
                ))
            })?;

            let outpath = match entry.enclosed_name() {
                Some(path) => dest.join(path),
                None => continue,
            };

            if entry.is_dir() {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut entry, &mut outfile)?;
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "icon-fetch"))]
    {
        let _ = (zip_path, dest);
        Err(crate::TairitsuPackagerError::IconFetchError(
            "icon-fetch feature not enabled".to_string(),
        ))
    }
}

fn find_extracted_dir(base: &Path) -> PathBuf {
    if let Ok(entries) = std::fs::read_dir(base) {
        let mut dirs: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .collect();
        if dirs.len() == 1 {
            return dirs.remove(0).path();
        }
    }
    base.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_star_suffix() {
        assert!(glob_match("arrow-*", "arrow-left"));
        assert!(glob_match("arrow-*", "arrow-right"));
        assert!(!glob_match("arrow-*", "chevron-left"));
    }

    #[test]
    fn test_glob_match_star_prefix() {
        assert!(glob_match("*-outline", "home-outline"));
        assert!(!glob_match("*-outline", "home-filled"));
    }

    #[test]
    fn test_glob_match_star_both() {
        assert!(glob_match("*-*", "arrow-left"));
        assert!(glob_match("*", "anything"));
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("home", "home"));
        assert!(!glob_match("home", "home-outline"));
    }

    #[test]
    fn test_find_source() {
        assert!(sources::find_source("mdi").is_some());
        assert!(sources::find_source("lucide").is_some());
        assert!(sources::find_source("nonexistent").is_none());
    }

    #[test]
    fn test_all_15_sources() {
        assert_eq!(sources::ICON_SOURCES.len(), 15);
    }

    #[test]
    fn test_extract_path_d() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg>"#;
        assert_eq!(
            extract_path_d(svg),
            Some("M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z".to_string())
        );
    }

    #[test]
    fn test_extract_path_d_fill_first() {
        let svg = r#"<svg viewBox="0 0 24 24"><path fill="currentColor" d="M4 4h16v16H4V4z"/></svg>"#;
        assert_eq!(
            extract_path_d(svg),
            Some("M4 4h16v16H4V4z".to_string())
        );
    }

    #[test]
    fn test_read_consumer_metadata_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
        let meta = read_consumer_metadata(tmp.path()).unwrap();
        assert!(meta.sets.is_empty());
    }
}
