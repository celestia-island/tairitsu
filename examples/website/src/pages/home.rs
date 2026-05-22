//! Home page — tairitsu framework documentation site.

use hikari_icons::MdiIcon;
use tairitsu_vdom::{el, txt, VElement, VNode};

use crate::components::{glow_wrapper, svg_icon};
use crate::i18n::{self, Language};

fn glow_btn(href: &str, class: &str, text: &str, arrow: Option<&str>) -> VNode {
    let mut btn = VElement::new("a")
        .attr("href", href)
        .class(class)
        .child(txt(text));
    if let Some(arrow_text) = arrow {
        btn = btn.child(VNode::Element(
            VElement::new("span")
                .class("btn-arrow")
                .child(txt(arrow_text)),
        ));
    }
    glow_wrapper(
        "medium",
        "soft",
        "rgba(20,110,116,0.5)",
        VNode::Element(btn),
    )
}

fn glow_card(title: &str, body: &str) -> VNode {
    let card = VNode::Element(
        el("div")
            .class("card")
            .child(VNode::Element(
                el("h3").class("card__title").child(txt(title)),
            ))
            .child(VNode::Element(el("p").class("card__body").child(txt(body)))),
    );
    VNode::Element(
        el("div")
            .class("hi-glow-wrapper-block hi-glow-blur-medium hi-glow-dim")
            .attr("style", "--glow-x:50%;--glow-y:50%;--glow-color:rgba(20,110,116,0.3);--glow-opacity:0;--glow-intensity-scale:0;--glow-spread:2.4;--glow-base-opacity:0.15;border-radius:var(--hi-card-radius,var(--hi-radius-lg,12px));")
            .child(card),
    )
}

pub fn render() -> VNode {
    let t = i18n::text(Language::default_lang());

    let logo = VNode::Element(el("div").class("page-hero__logo").child(svg_icon(
        MdiIcon::CubeOutline,
        64,
        "page-hero-logo-icon",
    )));

    let title = VNode::Element(el("h1").class("page-hero__title").child(txt(t.hero_title)));

    let subtitle = VNode::Element(el("p").class("page-hero__subtitle").child(txt(t.hero_copy)));

    let tagline = VNode::Element(
        el("p")
            .class("page-hero__tagline")
            .child(txt(t.section_arch_lead)),
    );

    let btn1 = glow_btn(
        "/system",
        "hi-button hi-button-primary hi-button-lg",
        t.action_primary,
        Some("\u{2192}"),
    );
    let btn2 = glow_btn(
        "/guides/quick-start",
        "hi-button hi-button-secondary hi-button-lg",
        t.action_secondary,
        None,
    );

    let actions = VNode::Element(
        el("div")
            .class("page-hero__actions")
            .children(vec![btn1, btn2]),
    );

    let hero_inner = VNode::Element(
        el("div")
            .class("page-hero__inner")
            .children(vec![logo, title, subtitle, tagline, actions]),
    );

    let hero_section = VNode::Element(el("section").class("page-hero").child(hero_inner));

    let section_title = VNode::Element(
        el("h2")
            .class("page-section__title")
            .child(txt(t.section_packages)),
    );

    let card1 = glow_card(t.section_arch, t.section_arch_lead);
    let card2 = glow_card(t.section_runtime, t.section_runtime_lead);
    let card3 = glow_card(t.section_workspace, t.section_workspace_lead);

    let card_grid = VNode::Element(
        el("div")
            .class("card-grid")
            .children(vec![card1, card2, card3]),
    );

    let section = VNode::Element(
        el("section")
            .class("page-section")
            .children(vec![section_title, card_grid]),
    );

    VNode::Element(
        el("div")
            .attr("id", "page-home")
            .class("hikari-page is-active")
            .children(vec![hero_section, section]),
    )
}
