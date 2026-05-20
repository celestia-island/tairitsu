use ab_glyph::FontVec;

pub struct Fonts {
    pub mono: FontVec,
    pub cjk: Option<FontVec>,
}

static MONO_CANDIDATES: &[&str] = &[
    "FiraCodeNerdFontMono-Regular.ttf",
    "FiraCodeNerdFont-Regular.ttf",
    "JetBrainsMonoNerdFontMono-Regular.ttf",
    "DejaVuSansMono.ttf",
    "NotoSansMono-Regular.ttf",
    "LiberationMono-Regular.ttf",
    "UbuntuMono-Regular.ttf",
    "Consolas.ttf",
];

static CJK_CANDIDATES: &[&str] = &[
    "NotoSansCJKsc-Regular.otf",
    "NotoSansCJK-Regular.ttc",
    "NotoSansSC-Regular.otf",
    "NotoSansSC-Regular.ttf",
    "WenQuanYiMicroHei.ttf",
    "wqy-microhei.ttc",
    "SimHei.ttf",
    "simhei.ttf",
    "SourceHanSansSC-Regular.otf",
    "PingFang.ttc",
    "HiraginoSansGB.ttc",
    "DroidSansFallbackFull.ttf",
];

#[cfg(unix)]
fn font_search_dirs() -> Vec<std::path::PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut dirs = vec![
        std::path::PathBuf::from("/usr/share/fonts"),
        std::path::PathBuf::from("/usr/local/share/fonts"),
    ];
    if !home.is_empty() {
        dirs.push(std::path::PathBuf::from(format!(
            "{}/.local/share/fonts",
            home
        )));
        dirs.push(std::path::PathBuf::from(format!("{}/.fonts", home)));
    }
    dirs.push(std::path::PathBuf::from("/System/Library/Fonts"));
    dirs.push(std::path::PathBuf::from("/Library/Fonts"));
    dirs
}

#[cfg(windows)]
fn font_search_dirs() -> Vec<std::path::PathBuf> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
    vec![std::path::PathBuf::from(windir).join("Fonts")]
}

fn find_font_file(candidates: &[&str]) -> Option<std::path::PathBuf> {
    let dirs = font_search_dirs();
    for dir in &dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if candidates.contains(&name) {
                            return Some(path);
                        }
                    }
                } else if path.is_dir() {
                    if let Ok(sub) = std::fs::read_dir(&path) {
                        for se in sub.flatten() {
                            let sp = se.path();
                            if sp.is_file() {
                                if let Some(name) = sp.file_name().and_then(|n| n.to_str()) {
                                    if candidates.contains(&name) {
                                        return Some(sp);
                                    }
                                }
                            }
                            if sp.is_dir() {
                                if let Ok(l3) = std::fs::read_dir(&sp) {
                                    for l3e in l3.flatten() {
                                        let l3p = l3e.path();
                                        if l3p.is_file() {
                                            if let Some(name) =
                                                l3p.file_name().and_then(|n| n.to_str())
                                            {
                                                if candidates.contains(&name) {
                                                    return Some(l3p);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn load_font_from(path: &std::path::Path) -> Result<FontVec, String> {
    let data = std::fs::read(path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    FontVec::try_from_vec(data).map_err(|e| format!("parse font {}: {}", path.display(), e))
}

impl Fonts {
    pub fn load() -> Result<Self, String> {
        let mono_path = find_font_file(MONO_CANDIDATES).ok_or_else(|| {
            format!(
                "No monospace font found. Searched for: {}",
                MONO_CANDIDATES.join(", ")
            )
        })?;
        let mono = load_font_from(&mono_path)?;
        let cjk = find_font_file(CJK_CANDIDATES).and_then(|p| load_font_from(&p).ok());
        Ok(Self { mono, cjk })
    }
}
