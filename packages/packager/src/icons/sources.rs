
#[derive(Debug, Clone)]
pub struct IconSourceDef {
    pub name: &'static str,
    pub display: &'static str,
    pub origins: &'static [IconOrigin],
    pub svg_glob: &'static str,
    pub font_file: Option<&'static str>,
    pub font_family: Option<&'static str>,
    pub meta_file: Option<&'static str>,
    pub view_box: &'static str,
}

#[derive(Debug, Clone)]
pub enum IconOrigin {
    Npm(&'static str, &'static str),
    Github(&'static str, &'static str, &'static str),
    GithubMirror(&'static str, &'static str, &'static str, &'static str),
}

pub const ICON_SOURCES: &[IconSourceDef] = &[
    IconSourceDef {
        name: "mdi",
        display: "Material Design Icons",
        origins: &[
            IconOrigin::Npm("@mdi/svg", "svg"),
            IconOrigin::Github("Templarian", "MaterialDesign", "master"),
        ],
        svg_glob: "svg/*.svg",
        font_file: Some("fonts/MaterialDesignIconsDesktop.ttf"),
        font_family: Some("Material Design Icons"),
        meta_file: Some("meta.json"),
        view_box: "0 0 24 24",
    },
    IconSourceDef {
        name: "lucide",
        display: "Lucide",
        origins: &[
            IconOrigin::Npm("lucide", "icons"),
            IconOrigin::Github("lucide-icons", "lucide", "main"),
        ],
        svg_glob: "icons/*.svg",
        font_file: None,
        font_family: None,
        meta_file: Some("packages.json"),
        view_box: "0 0 24 24",
    },
    IconSourceDef {
        name: "fa",
        display: "Font Awesome",
        origins: &[
            IconOrigin::Npm("@fortawesome/fontawesome-free", "svgs"),
            IconOrigin::Github("FortAwesome", "Font-Awesome", "6.x"),
        ],
        svg_glob: "svgs/**/*.svg",
        font_file: Some("webfonts/fa-solid-900.woff2"),
        font_family: Some("Font Awesome 6 Free"),
        meta_file: Some("metadata/icons.json"),
        view_box: "0 0 512 512",
    },
    IconSourceDef {
        name: "hero",
        display: "Heroicons",
        origins: &[
            IconOrigin::Npm("heroicons", "24/outline"),
            IconOrigin::Github("tailwindlabs", "heroicons", "master"),
        ],
        svg_glob: "24/outline/*.svg",
        font_file: None,
        font_family: None,
        meta_file: None,
        view_box: "0 0 24 24",
    },
    IconSourceDef {
        name: "phosphor",
        display: "Phosphor Icons",
        origins: &[
            IconOrigin::Npm("@phosphor-icons/react", "src/icons"),
            IconOrigin::Github("phosphor-icons", "phosphor-icons", "main"),
        ],
        svg_glob: "src/icons/**/*.svg",
        font_file: None,
        font_family: None,
        meta_file: None,
        view_box: "0 0 256 256",
    },
    IconSourceDef {
        name: "remix",
        display: "Remix Icon",
        origins: &[
            IconOrigin::Npm("remixicon", "icons"),
            IconOrigin::Github("Remix-Design", "RemixIcon", "master"),
        ],
        svg_glob: "icons/**/*.svg",
        font_file: Some("fonts/remixicon.woff2"),
        font_family: Some("remixicon"),
        meta_file: None,
        view_box: "0 0 24 24",
    },
    IconSourceDef {
        name: "simple",
        display: "Simple Icons",
        origins: &[
            IconOrigin::Npm("simple-icons", "icons"),
            IconOrigin::Github("simple-icons", "simple-icons", "develop"),
        ],
        svg_glob: "icons/*.svg",
        font_file: None,
        font_family: None,
        meta_file: Some("_data/simple-icons.json"),
        view_box: "0 0 24 24",
    },
    IconSourceDef {
        name: "tabler",
        display: "Tabler Icons",
        origins: &[
            IconOrigin::Npm("@tabler/icons", "icons"),
            IconOrigin::Github("tabler", "tabler-icons", "main"),
        ],
        svg_glob: "icons/*.svg",
        font_file: None,
        font_family: None,
        meta_file: None,
        view_box: "0 0 24 24",
    },
    IconSourceDef {
        name: "feather",
        display: "Feather Icons",
        origins: &[
            IconOrigin::Npm("feather-icons", "icons"),
            IconOrigin::Github("feathericons", "feather", "master"),
        ],
        svg_glob: "src/*.svg",
        font_file: None,
        font_family: None,
        meta_file: None,
        view_box: "0 0 24 24",
    },
    IconSourceDef {
        name: "bootstrap",
        display: "Bootstrap Icons",
        origins: &[
            IconOrigin::Npm("bootstrap-icons", "icons"),
            IconOrigin::Github("twbs", "icons", "main"),
        ],
        svg_glob: "icons/*.svg",
        font_file: Some("font/fonts/bootstrap-icons.woff2"),
        font_family: Some("bootstrap-icons"),
        meta_file: None,
        view_box: "0 0 16 16",
    },
    IconSourceDef {
        name: "iconoir",
        display: "Iconoir",
        origins: &[
            IconOrigin::Npm("iconoir", "icons"),
            IconOrigin::Github("iconoir-icons", "iconoir", "main"),
        ],
        svg_glob: "icons/*.svg",
        font_file: None,
        font_family: None,
        meta_file: None,
        view_box: "0 0 24 24",
    },
    IconSourceDef {
        name: "radix",
        display: "Radix Icons",
        origins: &[
            IconOrigin::Npm("@radix-ui/react-icons", "icons"),
            IconOrigin::Github("radix-ui", "icons", "main"),
        ],
        svg_glob: "packages/radix-ui-icons/icons/*.svg",
        font_file: None,
        font_family: None,
        meta_file: None,
        view_box: "0 0 15 15",
    },
    IconSourceDef {
        name: "fluent",
        display: "Fluent UI System Icons",
        origins: &[
            IconOrigin::Npm("@fluentui/svg-icons", "icons"),
            IconOrigin::Github("microsoft", "fluentui-system-icons", "main"),
        ],
        svg_glob: "icons/**/*.svg",
        font_file: None,
        font_family: None,
        meta_file: None,
        view_box: "0 0 24 24",
    },
    IconSourceDef {
        name: "ionicons",
        display: "Ionicons",
        origins: &[
            IconOrigin::Npm("ionicons", "src/svg"),
            IconOrigin::Github("ionic-team", "ionicons", "main"),
        ],
        svg_glob: "src/svg/*.svg",
        font_file: None,
        font_family: None,
        meta_file: None,
        view_box: "0 0 512 512",
    },
    IconSourceDef {
        name: "boxicons",
        display: "Boxicons",
        origins: &[
            IconOrigin::Npm("boxicons", "svg"),
            IconOrigin::Github("atisawd", "boxicons", "master"),
        ],
        svg_glob: "svg/**/*.svg",
        font_file: None,
        font_family: None,
        meta_file: None,
        view_box: "0 0 24 24",
    },
];

pub fn find_source(name: &str) -> Option<&'static IconSourceDef> {
    ICON_SOURCES.iter().find(|s| s.name == name)
}

impl IconOrigin {
    pub fn npm_package(&self) -> Option<&'static str> {
        match self {
            IconOrigin::Npm(pkg, _) => Some(pkg),
            _ => None,
        }
    }

    pub fn github_repo(&self) -> Option<(&str, &str)> {
        match self {
            IconOrigin::Github(owner, repo, _) => Some((owner, repo)),
            IconOrigin::GithubMirror(_, owner, repo, _) => Some((owner, repo)),
            _ => None,
        }
    }
}
