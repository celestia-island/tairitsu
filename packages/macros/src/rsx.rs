use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Expr, Ident, LitStr, Pat, Token,
};

pub struct RsxElement {
    pub tag: Ident,
    pub attrs: Vec<RsxAttr>,
    pub children: Vec<RsxChild>,
}

pub enum RsxAttr {
    Class(Expr),
    Style(Expr),
    Id(Expr),
    Onclick(Expr),
    InnerHtml(Expr), // dangerous_inner_html attribute
    Other { name: String, value: Expr },
}

pub enum RsxChild {
    Text(LitStr),
    Element(RsxElement),
    Dynamic(Expr),
    Spread(Expr), // ..expr syntax for spreading Vec<VNode>
    If(RsxIf),
    Match(RsxMatch),
    For(Box<RsxFor>),
}

/// Root content of an rsx! macro
pub enum RsxRoot {
    Element(RsxElement),
    Fragment(Vec<RsxChild>), // Multiple root elements
    If(RsxIf),
    Match(RsxMatch),
    For(Box<RsxFor>),
}

/// If expression with rsx body
pub struct RsxIf {
    pub condition: Expr,
    pub then_branch: Vec<RsxChild>,
    pub else_branch: Option<RsxElse>,
}

pub enum RsxElse {
    Block(Vec<RsxChild>),
    If(Box<RsxIf>),
}

/// Match expression with rsx arms
pub struct RsxMatch {
    pub scrutinee: Expr,
    pub arms: Vec<RsxMatchArm>,
}

pub struct RsxMatchArm {
    pub pattern: Pat,
    pub guard: Option<Expr>,
    pub body: Vec<RsxChild>,
}

/// For loop expression with rsx body
pub struct RsxFor {
    pub pattern: Pat,
    pub iterable: Expr,
    pub body: Vec<RsxChild>,
}

impl Parse for RsxElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tag: Ident = input.parse()?;
        let mut attrs = Vec::new();
        let mut children = Vec::new();

        if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);

            while !content.is_empty() {
                // Check for attribute pattern (name: value) or shorthand (name,)
                let fork = content.fork();
                let is_attr = if fork.peek(LitStr) {
                    fork.parse::<LitStr>().is_ok() && fork.peek(Token![:])
                } else if fork.peek(Ident) {
                    fork.parse::<Ident>().is_ok()
                        && (fork.peek(Token![:]) || fork.peek(Token![,]) || fork.is_empty())
                } else {
                    false
                };

                if is_attr {
                    // Check for shorthand syntax: identifier, or identifier }
                    let is_shorthand = content.peek(Ident) && {
                        let fork = content.fork();
                        fork.parse::<Ident>().ok();
                        !fork.peek(Token![:]) && (fork.peek(Token![,]) || fork.is_empty())
                    };

                    let name = if content.peek(LitStr) {
                        let lit: LitStr = content.parse()?;
                        lit.value()
                    } else {
                        let name: Ident = content.parse()?;
                        name.to_string()
                    };

                    // For non-shorthand, consume the colon
                    if !is_shorthand {
                        content.parse::<Token![:]>()?;
                    }

                    let attr = if is_shorthand {
                        // Shorthand: name, means name: name
                        let value: Expr = syn::parse_str(&name).unwrap();
                        RsxAttr::Other {
                            name: name.clone(),
                            value,
                        }
                    } else {
                        match name.as_str() {
                            "class" => RsxAttr::Class(content.parse()?),
                            "style" => RsxAttr::Style(content.parse()?),
                            "id" => RsxAttr::Id(content.parse()?),
                            "onclick" => RsxAttr::Onclick(content.parse()?),
                            "dangerous_inner_html" => RsxAttr::InnerHtml(content.parse()?),
                            _ => RsxAttr::Other {
                                name,
                                value: content.parse()?,
                            },
                        }
                    };
                    attrs.push(attr);
                } else if content.peek(LitStr) {
                    let text: LitStr = content.parse()?;
                    children.push(RsxChild::Text(text));
                } else if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                } else if content.peek(Token![..]) {
                    content.parse::<Token![..]>()?;
                    let expr: Expr = content.parse()?;
                    children.push(RsxChild::Spread(expr));
                } else if content.peek(syn::token::Brace) {
                    let expr: Expr = content.parse()?;
                    children.push(RsxChild::Dynamic(expr));
                } else if content.peek(Token![if]) {
                    let rsx_if: RsxIf = content.parse()?;
                    children.push(RsxChild::If(rsx_if));
                } else if content.peek(Token![match]) {
                    let rsx_match: RsxMatch = content.parse()?;
                    children.push(RsxChild::Match(rsx_match));
                } else if content.peek(Token![for]) {
                    let rsx_for: RsxFor = content.parse()?;
                    children.push(RsxChild::For(Box::new(rsx_for)));
                } else if content.peek(Ident) {
                    let elem: RsxElement = content.parse()?;
                    children.push(RsxChild::Element(elem));
                } else {
                    break;
                }
            }
        }

        Ok(RsxElement {
            tag,
            attrs,
            children,
        })
    }
}

impl Parse for RsxRoot {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![if]) {
            Ok(RsxRoot::If(input.parse()?))
        } else if input.peek(Token![match]) {
            Ok(RsxRoot::Match(input.parse()?))
        } else if input.peek(Token![for]) {
            Ok(RsxRoot::For(Box::new(input.parse()?)))
        } else if input.peek(Ident) {
            // Parse first element
            let first: RsxElement = input.parse()?;

            // Check if there are more elements after this one
            if !input.is_empty()
                && (input.peek(Ident)
                    || input.peek(Token![if])
                    || input.peek(Token![match])
                    || input.peek(Token![for]))
            {
                // Multiple root elements - parse as fragment
                let mut children = vec![RsxChild::Element(first)];
                while !input.is_empty() {
                    if input.peek(Ident) {
                        children.push(RsxChild::Element(input.parse()?));
                    } else if input.peek(Token![if]) {
                        children.push(RsxChild::If(input.parse()?));
                    } else if input.peek(Token![match]) {
                        children.push(RsxChild::Match(input.parse()?));
                    } else if input.peek(Token![for]) {
                        children.push(RsxChild::For(Box::new(input.parse()?)));
                    } else {
                        break;
                    }
                }
                Ok(RsxRoot::Fragment(children))
            } else {
                Ok(RsxRoot::Element(first))
            }
        } else {
            Err(syn::Error::new(
                input.span(),
                "Expected element or control flow",
            ))
        }
    }
}

impl Parse for RsxIf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![if]>()?;
        // Use parse_without_eager_brace to prevent parsing { as struct literal
        let condition: Expr = Expr::parse_without_eager_brace(input)?;

        let content;
        syn::braced!(content in input);
        let then_branch = parse_rsx_children(&content)?;

        let else_branch = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            if input.peek(Token![if]) {
                Some(RsxElse::If(Box::new(input.parse()?)))
            } else {
                let else_content;
                syn::braced!(else_content in input);
                Some(RsxElse::Block(parse_rsx_children(&else_content)?))
            }
        } else {
            None
        };

        Ok(RsxIf {
            condition,
            then_branch,
            else_branch,
        })
    }
}

impl Parse for RsxMatch {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![match]>()?;
        // Use parse_without_eager_brace to prevent parsing { as struct literal
        let scrutinee: Expr = Expr::parse_without_eager_brace(input)?;

        let content;
        syn::braced!(content in input);
        let mut arms = Vec::new();
        while !content.is_empty() {
            let arm = parse_rsx_match_arm(&content)?;
            arms.push(arm);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        Ok(RsxMatch { scrutinee, arms })
    }
}

impl Parse for RsxFor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![for]>()?;
        let pattern: Pat = syn::Pat::parse_single(input)?;
        input.parse::<Token![in]>()?;
        let iterable: Expr = input.parse()?;

        let content;
        syn::braced!(content in input);
        let body = parse_rsx_children(&content)?;

        Ok(RsxFor {
            pattern,
            iterable,
            body,
        })
    }
}

fn parse_rsx_match_arm(input: ParseStream) -> syn::Result<RsxMatchArm> {
    // Parse pattern - use PatType which handles various pattern forms
    let pattern: Pat = syn::Pat::parse_single(input)?;

    let guard = if input.peek(Token![if]) {
        input.parse::<Token![if]>()?;
        Some(input.parse()?)
    } else {
        None
    };

    input.parse::<Token![=>]>()?;

    let body = if input.peek(syn::token::Brace) {
        let content;
        syn::braced!(content in input);
        parse_rsx_children(&content)?
    } else {
        let expr: Expr = input.parse()?;
        vec![RsxChild::Dynamic(expr)]
    };

    Ok(RsxMatchArm {
        pattern,
        guard,
        body,
    })
}

fn parse_rsx_children(content: &syn::parse::ParseBuffer) -> syn::Result<Vec<RsxChild>> {
    let mut children = Vec::new();
    while !content.is_empty() {
        let fork = content.fork();
        let is_attr = if fork.peek(LitStr) {
            fork.parse::<LitStr>().is_ok() && fork.peek(Token![:])
        } else if fork.peek(Ident) {
            fork.parse::<Ident>().is_ok() && fork.peek(Token![:])
        } else {
            false
        };

        if is_attr {
            if content.peek(LitStr) {
                content.parse::<LitStr>()?;
            } else {
                content.parse::<Ident>()?;
            }
            content.parse::<Token![:]>()?;
            content.parse::<Expr>()?;
        } else if content.peek(LitStr) {
            children.push(RsxChild::Text(content.parse()?));
        } else if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        } else if content.peek(Token![..]) {
            content.parse::<Token![..]>()?;
            children.push(RsxChild::Spread(content.parse()?));
        } else if content.peek(syn::token::Brace) {
            children.push(RsxChild::Dynamic(content.parse()?));
        } else if content.peek(Token![if]) {
            children.push(RsxChild::If(content.parse()?));
        } else if content.peek(Token![match]) {
            children.push(RsxChild::Match(content.parse()?));
        } else if content.peek(Token![for]) {
            children.push(RsxChild::For(Box::new(content.parse()?)));
        } else if content.peek(Ident) {
            children.push(RsxChild::Element(content.parse()?));
        } else {
            break;
        }
    }
    Ok(children)
}

pub fn expand_rsx(element: RsxElement) -> TokenStream2 {
    let tag = element.tag.clone();
    let tag_str = tag.to_string();

    // Check if this is a custom component (starts with uppercase)
    let is_custom_component = tag_str.chars().next().is_some_and(|c| c.is_uppercase());

    if is_custom_component {
        return expand_custom_component(element);
    }

    // HTML element handling
    let mut class_code = quote! { tairitsu_vdom::Classes::new() };
    let mut style_code = quote! { tairitsu_vdom::Style::new() };
    let mut event_handlers = Vec::new();
    let mut other_attrs = Vec::new();
    let mut children_code = Vec::new();

    // Check if this is a known HTML element (lowercase tags)
    // Custom components start with uppercase
    let is_html_element = tag_str.chars().next().is_some_and(|c| c.is_lowercase());

    // List of known HTML elements that support event handlers
    let html_elements = [
        "div", "span", "p", "a", "button", "input", "textarea", "select", "form", "ul", "ol", "li",
        "table", "tr", "td", "th", "thead", "tbody", "tfoot", "h1", "h2", "h3", "h4", "h5", "h6",
        "img", "video", "audio", "canvas", "svg", "path", "rect", "circle", "line", "text",
        "header", "footer", "nav", "main", "section", "article", "aside", "label", "option",
        "optgroup", "fieldset", "legend", "progress", "meter", "details", "summary", "dialog",
        "iframe", "object", "embed", "source", "br", "hr", "wbr",
    ];
    let is_known_html =
        html_elements.contains(&tag_str.as_str()) || (is_html_element && !tag_str.contains('_'));

    for attr in element.attrs {
        match attr {
            RsxAttr::Class(expr) => {
                class_code = quote! { #expr };
            }
            RsxAttr::Style(expr) => {
                style_code = quote! { #expr };
            }
            RsxAttr::Id(expr) => {
                other_attrs.push(quote! { .attr("id", #expr) });
            }
            RsxAttr::Onclick(expr) => {
                // For onclick, we need to downcast Box<dyn EventData> to MouseEvent
                // The handler is expected to be a closure that takes MouseEvent
                event_handlers.push(quote! {
                    .on_event("click", {
                        let handler = #expr;
                        move |e: Box<dyn tairitsu_vdom::EventData>| {
                            if let Some(mouse_event) = e.as_any().downcast_ref::<tairitsu_vdom::MouseEvent>() {
                                handler(mouse_event.clone());
                            }
                        }
                    })
                });
            }
            RsxAttr::InnerHtml(expr) => {
                other_attrs.push(quote! { .inner_html(#expr) });
            }
            RsxAttr::Other { name, value } => {
                // Only treat on_* as event handlers for known HTML elements
                if is_known_html {
                    if let Some(event_name) = name.strip_prefix("on") {
                        // Map event names to their types
                        let event_type = match event_name {
                            "click" | "mousedown" | "mouseup" | "mousemove" | "mouseenter"
                            | "mouseleave" => {
                                quote! { tairitsu_vdom::MouseEvent }
                            }
                            "keydown" | "keyup" | "keypress" => {
                                quote! { tairitsu_vdom::KeyboardEvent }
                            }
                            "input" => {
                                quote! { tairitsu_vdom::InputEvent }
                            }
                            "change" => {
                                quote! { tairitsu_vdom::ChangeEvent }
                            }
                            "focus" | "blur" => {
                                quote! { tairitsu_vdom::FocusEvent }
                            }
                            "dragstart" | "dragend" | "dragover" | "dragleave" | "drop" => {
                                quote! { tairitsu_vdom::DragEvent }
                            }
                            _ => {
                                // For unknown events, just pass the boxed event
                                event_handlers.push(quote! {
                                    .on_event(#event_name, {
                                        let handler = #value;
                                        move |e| {
                                            handler(e);
                                        }
                                    })
                                });
                                continue;
                            }
                        };
                        // Capture handler in a binding
                        // The handler is expected to be a closure that takes the event type
                        event_handlers.push(quote! {
                            .on_event(#event_name, {
                                let handler = #value;
                                move |e: Box<dyn tairitsu_vdom::EventData>| {
                                    if let Some(typed_event) = e.as_any().downcast_ref::<#event_type>() {
                                        handler(typed_event.clone());
                                    }
                                }
                            })
                        });
                        continue;
                    }
                }

                // For custom components or non-event attributes, pass as regular attribute
                if name == "children" {
                    // Special handling for children - add as child, not attribute
                    children_code.push(quote! { .child(#value) });
                } else {
                    other_attrs.push(quote! { .attr(#name, #value) });
                }
            }
        }
    }

    // Add element children
    for child in element.children {
        children_code.push(expand_child_method(child));
    }

    quote! {
        tairitsu_vdom::VNode::Element(
            tairitsu_vdom::VElement::new(#tag_str)
                .class(#class_code)
                .style(#style_code)
                #(#other_attrs)*
                #(#event_handlers)*
                #(#children_code)*
        )
    }
}

pub fn expand_rsx_root(root: RsxRoot) -> TokenStream2 {
    match root {
        RsxRoot::Element(elem) => expand_rsx(elem),
        RsxRoot::Fragment(children) => {
            let children_code: Vec<_> = children.into_iter().map(expand_child).collect();
            quote! {
                tairitsu_vdom::VNode::Fragment(vec![#(#children_code),*])
            }
        }
        RsxRoot::If(rsx_if) => expand_rsx_if(rsx_if),
        RsxRoot::Match(rsx_match) => expand_rsx_match(rsx_match),
        RsxRoot::For(rsx_for) => expand_rsx_for(*rsx_for),
    }
}

fn expand_rsx_if(rsx_if: RsxIf) -> TokenStream2 {
    let condition = &rsx_if.condition;
    let then_code: Vec<_> = rsx_if.then_branch.into_iter().map(expand_child).collect();
    let else_code = match rsx_if.else_branch {
        Some(RsxElse::Block(children)) => {
            let children_code: Vec<_> = children.into_iter().map(expand_child).collect();
            quote! { else { tairitsu_vdom::VNode::Fragment(vec![#(#children_code),*]) } }
        }
        Some(RsxElse::If(inner_if)) => {
            let inner_code = expand_rsx_if(*inner_if);
            quote! { else { #inner_code } }
        }
        None => quote! { else { tairitsu_vdom::VNode::empty() } },
    };
    quote! {
        if #condition {
            tairitsu_vdom::VNode::Fragment(vec![#(#then_code),*])
        } #else_code
    }
}

fn expand_rsx_match(rsx_match: RsxMatch) -> TokenStream2 {
    let scrutinee = &rsx_match.scrutinee;
    let arms_code: Vec<_> = rsx_match.arms.into_iter().map(|arm| {
        let pattern = &arm.pattern;
        let guard_code = match &arm.guard {
            Some(guard) => quote! { if #guard },
            None => quote! {},
        };
        let body_code: Vec<_> = arm.body.into_iter().map(expand_child).collect();
        quote! { #pattern #guard_code => tairitsu_vdom::VNode::Fragment(vec![#(#body_code),*]), }
    }).collect();
    quote! {
        match #scrutinee {
            #(#arms_code)*
        }
    }
}

fn expand_rsx_for(rsx_for: RsxFor) -> TokenStream2 {
    let pattern = &rsx_for.pattern;
    let iterable = &rsx_for.iterable;
    let body_code: Vec<_> = rsx_for.body.into_iter().map(expand_child).collect();
    quote! {
        {
            let mut __children = Vec::new();
            for #pattern in #iterable {
                #(__children.push(#body_code);)*
            }
            tairitsu_vdom::VNode::Fragment(__children)
        }
    }
}

fn expand_child(child: RsxChild) -> TokenStream2 {
    match child {
        RsxChild::Text(text) => {
            let text_value = text.value();
            // Check if this is a format string like "{variable}"
            if text_value.starts_with('{')
                && text_value.ends_with('}')
                && text_value.matches('{').count() == 1
            {
                // This is a shorthand for displaying a variable: "{count}" -> count.to_string()
                let inner = &text_value[1..text_value.len() - 1];
                // Parse the inner as an expression
                if let Ok(expr) = syn::parse_str::<Expr>(inner) {
                    return quote! { tairitsu_vdom::VNode::Text(tairitsu_vdom::VText::new(&format!("{}", #expr))) };
                }
            }
            quote! { tairitsu_vdom::VNode::Text(tairitsu_vdom::VText::new(#text_value)) }
        }
        RsxChild::Element(elem) => expand_rsx(elem),
        RsxChild::Dynamic(expr) => quote! { #expr },
        RsxChild::Spread(expr) => quote! { tairitsu_vdom::VNode::Fragment(#expr) },
        RsxChild::If(rsx_if) => expand_rsx_if(rsx_if),
        RsxChild::Match(rsx_match) => expand_rsx_match(rsx_match),
        RsxChild::For(rsx_for) => expand_rsx_for(*rsx_for),
    }
}

/// Expands a custom component (PascalCase tag) into a component function call
fn expand_custom_component(element: RsxElement) -> TokenStream2 {
    let tag = &element.tag;
    let tag_str = tag.to_string();
    let props_name = syn::Ident::new(&format!("{}Props", tag_str), tag.span());

    // Collect all props from attributes
    let mut props_fields: Vec<TokenStream2> = Vec::new();

    for attr in element.attrs {
        match attr {
            RsxAttr::Class(expr) => {
                props_fields.push(quote! { class: #expr });
            }
            RsxAttr::Style(expr) => {
                props_fields.push(quote! { style: #expr });
            }
            RsxAttr::Id(expr) => {
                props_fields.push(quote! { id: #expr });
            }
            RsxAttr::Onclick(expr) => {
                // For custom components, pass onclick value directly without wrapping in EventHandler::new
                // The component's Props should define the type appropriately (Option<EventHandler<T>> or EventHandler<T>)
                props_fields.push(quote! { onclick: #expr });
            }
            RsxAttr::InnerHtml(expr) => {
                props_fields.push(quote! { dangerous_inner_html: #expr });
            }
            RsxAttr::Other { name, value } => {
                // Convert attribute name to Rust field name
                let field_name = name.as_str();
                let field_ident = syn::Ident::new(field_name, tag.span());
                props_fields.push(quote! { #field_ident: #value });
            }
        }
    }

    // Handle children
    if !element.children.is_empty() {
        // Check if there's already a children field
        let has_children_field = props_fields
            .iter()
            .any(|f| f.to_string().starts_with("children :"));

        if !has_children_field {
            let children_code: Vec<_> = element.children.into_iter().map(expand_child).collect();
            props_fields.push(
                quote! { children: tairitsu_vdom::VNode::Fragment(vec![#(#children_code),*]) },
            );
        }
    }

    quote! {
        #tag(#props_name {
            #(#props_fields,)*
            ..Default::default()
        })
    }
}

/// Expands a child into a method call for building VElement
fn expand_child_method(child: RsxChild) -> TokenStream2 {
    match child {
        RsxChild::Text(text) => {
            let text_value = text.value();
            // Check if this is a format string like "{variable}"
            if text_value.starts_with('{')
                && text_value.ends_with('}')
                && text_value.matches('{').count() == 1
            {
                // This is a shorthand for displaying a variable: "{count}" -> count.to_string()
                let inner = &text_value[1..text_value.len() - 1];
                // Parse the inner as an expression
                if let Ok(expr) = syn::parse_str::<Expr>(inner) {
                    return quote! { .child(tairitsu_vdom::VNode::Text(tairitsu_vdom::VText::new(&format!("{}", #expr)))) };
                }
            }
            quote! { .child(tairitsu_vdom::VNode::Text(tairitsu_vdom::VText::new(#text_value))) }
        }
        RsxChild::Element(elem) => {
            let expanded = expand_rsx(elem);
            quote! { .child(#expanded) }
        }
        RsxChild::Dynamic(expr) => quote! { .child(#expr) },
        RsxChild::Spread(expr) => quote! { .children(#expr) },
        RsxChild::If(rsx_if) => {
            let expanded = expand_rsx_if(rsx_if);
            quote! { .child(#expanded) }
        }
        RsxChild::Match(rsx_match) => {
            let expanded = expand_rsx_match(rsx_match);
            quote! { .child(#expanded) }
        }
        RsxChild::For(rsx_for) => {
            let expanded = expand_rsx_for(*rsx_for);
            quote! { .child(#expanded) }
        }
    }
}
