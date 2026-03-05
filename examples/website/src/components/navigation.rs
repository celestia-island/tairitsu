use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn navigation() -> VNode {
    rsx! {
        nav {
            class: "navigation",
            ul {
                li {
                    a {
                        href: "#",
                        "Home"
                    }
                }
                li {
                    a {
                        href: "#rsx",
                        "rsx! Macro"
                    }
                }
                li {
                    a {
                        href: "#builders",
                        "Builders"
                    }
                }
                li {
                    a {
                        href: "#reactive",
                        "Reactive System"
                    }
                }
            }
        }
    }
}
