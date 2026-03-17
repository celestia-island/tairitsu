//! Shared layout components: top nav and sidebar.
//!
//! 照抄 Hikari 的设计模式

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

/// Sidebar navigation links grouped by section.
struct NavSection {
    label: &'static str,
    items: &'static [(&'static str, &'static str)], // (label, hash)
}

const NAV: &[NavSection] = &[
    NavSection {
        label: "Guides",
        items: &[
            ("快速开始", "#/guides/quick-start"),
            ("工作区地图", "#/guides/workspace-map"),
            ("构建/测试/发布", "#/guides/build-test-release"),
            ("迁移指南", "#/guides/migration"),
            ("术语对照表", "#/guides/glossary"),
        ],
    },
    NavSection {
        label: "System",
        items: &[
            ("系统总览", "#/system/overview"),
            ("运行时", "#/system/runtime"),
            ("WIT 流水线", "#/system/wit-pipeline"),
            ("Web 后端", "#/system/web-backends"),
            ("版本策略", "#/system/versioning"),
        ],
    },
    NavSection {
        label: "Packages",
        items: &[
            ("包总览", "#/packages/overview"),
            ("包清单", "#/packages/list"),
        ],
    },
];

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}

/// Render the top navigation bar.
pub fn top_nav() -> VNode {
    rsx! {
        header { class: "tairitsu-topnav",
            a { href: "#/", class: "tairitsu-topnav__brand",
                "Tairitsu"
            }
            nav { class: "tairitsu-topnav__links",
                a { href: "#/guides/quick-start", class: "tairitsu-topnav__link",
                    "Guides"
                }
                a { href: "#/system/overview", class: "tairitsu-topnav__link",
                    "System"
                }
                a { href: "#/packages/overview", class: "tairitsu-topnav__link",
                    "Packages"
                }
            }
        }
    }
}

/// Render the sidebar with hash-based navigation.
pub fn sidebar() -> VNode {
    let mut sections: Vec<VNode> = Vec::new();

    // Home link
    sections.push(VNode::Element(
        VElement::new("a")
            .attr("href", "#/")
            .class("sidebar-link sidebar-link--home")
            .child(txt("Home")),
    ));

    for section in NAV {
        let mut items: Vec<VNode> = Vec::new();
        for (label, href) in section.items {
            items.push(VNode::Element(
                VElement::new("a")
                    .attr("href", href)
                    .class("sidebar-link")
                    .child(txt(label)),
            ));
        }

        sections.push(VNode::Element(
            VElement::new("div")
                .class("sidebar-section")
                .child(VNode::Element(
                    VElement::new("span")
                        .class("sidebar-section__label")
                        .child(txt(section.label)),
                ))
                .children(items),
        ));
    }

    VNode::Element(
        VElement::new("aside")
            .class("tairitsu-sidebar")
            .children(sections),
    )
}
