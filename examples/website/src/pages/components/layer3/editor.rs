use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-editor", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 \u{2014} Complex", "/components/layer3/editor"), ("Editor", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Editor" }
                p { class: "card__body",
                    "Rich text editor with markdown support, toolbar formatting, code highlighting, and image embedding."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Markdown Editor" }
                    div { class: "demo-block__body",
                        div { class: "editor-toolbar",
                            button { class: "hi-button hi-button-borderless", style: "font-weight:bold;", "B" }
                            button { class: "hi-button hi-button-borderless", style: "font-style:italic;", "I" }
                            button { class: "hi-button hi-button-borderless", style: "text-decoration:underline;", "U" }
                            button { class: "hi-button hi-button-borderless", style: "text-decoration:line-through;", "S" }
                            button { class: "hi-button hi-button-borderless", "|" }
                            button { class: "hi-button hi-button-borderless", style: "font-family:var(--ts-font-mono);font-size:0.8125rem;", "Code" }
                            button { class: "hi-button hi-button-borderless", "Link" }
                            button { class: "hi-button hi-button-borderless", "Image" }
                            button { class: "hi-button hi-button-borderless", "|" }
                            button { class: "hi-button hi-button-borderless", "H1" }
                            button { class: "hi-button hi-button-borderless", "H2" }
                            button { class: "hi-button hi-button-borderless", "H3" }
                            button { class: "hi-button hi-button-borderless", "|" }
                            button { class: "hi-button hi-button-borderless", "\u{1F4DD}" }
                        }
                        textarea { class: "editor-textarea",
                            placeholder: "Write your content here...",
                            rows: "8",
"# Tairitsu Editor

This is a **rich text editor** component with
markdown support.

- List item 1
- List item 2
- List item 3

```rust
pub fn render() -> VNode {
    rsx! { div { \"Hello\" } }
}
```"
                        }
                        div { class: "editor-preview",
                            div { style: "margin-bottom:8px;",
                                h3 { style: "font-size:1.25rem;font-weight:700;color:var(--hi-color-text-primary);margin-bottom:8px;", "Tairitsu Editor" }
                                p { "This is a rich text editor component with markdown support." }
                                ul { style: "padding-left:1.5rem;margin:8px 0;",
                                    li { "List item 1" }
                                    li { "List item 2" }
                                    li { "List item 3" }
                                }
                                div { class: "demo-code", "pub fn render() -> VNode {\n    rsx! { div { \"Hello\" } }\n}" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Editor Modes" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;gap:8px;",
                            a { href: "#", class: "hi-button hi-button-primary", style: "padding:4px 12px;font-size:0.8125rem;", "Edit" }
                            a { href: "#", class: "hi-button hi-button-secondary", style: "padding:4px 12px;font-size:0.8125rem;", "Preview" }
                            a { href: "#", class: "hi-button hi-button-tertiary", style: "padding:4px 12px;font-size:0.8125rem;", "Split" }
                        }
                        p { style: "font-size:0.8125rem;color:var(--hi-color-text-disabled);margin-top:8px;",
                            "Switch between edit, preview, and split view modes."
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "API" }
                    div { class: "demo-block__body",
                        table { class: "api-table",
                            thead {
                                tr { th { "Property" } th { "Type" } th { "Description" } }
                            }
                            tbody {
                                tr { td { code { "value" } } td { code { "string" } } td { "Editor content" } }
                                tr { td { code { "mode" } } td { code { "edit | preview | split" } } td { "Editor view mode" } }
                                tr { td { code { "language" } } td { code { "markdown | rich-text" } } td { "Editor language mode" } }
                                tr { td { code { "readonly" } } td { code { "bool" } } td { "Read-only mode" } }
                                tr { td { code { "toolbar" } } td { code { "ToolbarItem[]" } } td { "Toolbar configuration" } }
                                tr { td { code { "onChange" } } td { code { "(value: string) => void" } } td { "Content change callback" } }
                                tr { td { code { "placeholder" } } td { code { "string" } } td { "Placeholder text" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
