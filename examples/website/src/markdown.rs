use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use tairitsu_vdom::{el, txt, VElement, VNode};

pub fn render_markdown(md: &str) -> VNode {
    let parser = Parser::new(md);
    let mut stack: Vec<VElement> = Vec::new();
    let mut root: Vec<VNode> = Vec::new();
    let mut code_lang: Option<String> = None;

    for event in parser {
        match event {
            Event::Start(tag) => match &tag {
                Tag::Paragraph => stack.push(el("p")),
                Tag::Heading { level, .. } => stack.push(el(heading_tag(*level))),
                Tag::BlockQuote(_) => stack.push(el("blockquote")),
                Tag::CodeBlock(kind) => {
                    code_lang = match kind {
                        CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                        CodeBlockKind::Indented => None,
                    };
                    stack.push(el("code"));
                }
                Tag::List(_) => {
                    stack.push(el("ul"));
                }
                Tag::Item => stack.push(el("li")),
                Tag::Emphasis => stack.push(el("em")),
                Tag::Strong => stack.push(el("strong")),
                Tag::Strikethrough => stack.push(el("del")),
                Tag::Link { dest_url, .. } => stack.push(el("a").attr("href", dest_url.as_ref())),
                Tag::Image {
                    dest_url, title, ..
                } => {
                    let img = el("img")
                        .attr("src", dest_url.as_ref())
                        .attr("alt", title.as_ref());
                    push_child(&mut stack, &mut root, VNode::Element(img));
                    stack.push(el("span"));
                }
                Tag::Table(_) => stack.push(el("table")),
                Tag::TableHead => {
                    stack.push(el("thead-tr"));
                }
                Tag::TableRow => stack.push(el("tr")),
                Tag::TableCell => {
                    let in_head = stack.last().map_or(false, |p| p.tag == "thead-tr");
                    if in_head {
                        stack.push(el("th"))
                    } else {
                        stack.push(el("td"))
                    }
                }
                _ => stack.push(el("div")),
            },
            Event::End(tag_end) => {
                let elem = stack.pop().unwrap_or_else(|| el("div"));
                match tag_end {
                    TagEnd::CodeBlock => {
                        let class_str = code_lang
                            .take()
                            .map(|l| format!("hi-code-block language-{}", l))
                            .unwrap_or_else(|| "hi-code-block".to_string());
                        let wrapped = el("div")
                            .class(class_str.as_str())
                            .child(VNode::Element(el("pre").child(VNode::Element(elem))));
                        push_child(&mut stack, &mut root, VNode::Element(wrapped));
                    }
                    TagEnd::List(is_ordered) => {
                        let mut list = elem;
                        if is_ordered {
                            list.tag = "ol".to_string();
                        }
                        push_child(&mut stack, &mut root, VNode::Element(list));
                    }
                    TagEnd::TableHead => {
                        let thead = el("thead").child(VNode::Element(elem));
                        push_child(&mut stack, &mut root, VNode::Element(thead));
                    }
                    TagEnd::Image => {
                        stack.pop();
                    }
                    _ => {
                        push_child(&mut stack, &mut root, VNode::Element(elem));
                    }
                }
            }
            Event::Text(text) => {
                push_child(&mut stack, &mut root, txt(&text));
            }
            Event::Code(code) => {
                push_child(
                    &mut stack,
                    &mut root,
                    VNode::Element(el("code").class("hi-inline-code").child(txt(&code))),
                );
            }
            Event::SoftBreak => {
                push_child(&mut stack, &mut root, txt(" "));
            }
            Event::HardBreak => {
                push_child(&mut stack, &mut root, VNode::Element(el("br")));
            }
            Event::Rule => {
                push_child(&mut stack, &mut root, VNode::Element(el("hr")));
            }
            Event::TaskListMarker(checked) => {
                let cb = el("input")
                    .attr("type", "checkbox")
                    .attr("checked", checked)
                    .attr("disabled", true);
                push_child(&mut stack, &mut root, VNode::Element(cb));
            }
            _ => {}
        }
    }

    if root.len() == 1 {
        root.into_iter().next().unwrap()
    } else {
        VNode::Fragment(root)
    }
}

pub fn markdown_content(md: &str) -> VNode {
    VNode::Element(
        el("div")
            .class("hi-markdown-content")
            .child(render_markdown(md)),
    )
}

fn heading_tag(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "h1",
        HeadingLevel::H2 => "h2",
        HeadingLevel::H3 => "h3",
        HeadingLevel::H4 => "h4",
        HeadingLevel::H5 => "h5",
        HeadingLevel::H6 => "h6",
    }
}

fn push_child(stack: &mut Vec<VElement>, root: &mut Vec<VNode>, child: VNode) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(child);
    } else {
        root.push(child);
    }
}
