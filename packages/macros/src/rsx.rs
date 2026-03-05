use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, Expr, Ident, LitStr, Token};

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
    Other { name: Ident, value: Expr },
}

pub enum RsxChild {
    Text(LitStr),
    Element(RsxElement),
    Dynamic(Expr),
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
                if content.peek(LitStr) {
                    let text: LitStr = content.parse()?;
                    children.push(RsxChild::Text(text));
                } else if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                } else if content.peek(Token![..]) {
                    content.parse::<Token![..]>()?;
                    let expr: Expr = content.parse()?;
                    children.push(RsxChild::Dynamic(expr));
                } else if content.peek2(Token![:]) {
                    let name: Ident = content.parse()?;
                    content.parse::<Token![:]>()?;

                    let attr = match name.to_string().as_str() {
                        "class" => RsxAttr::Class(content.parse()?),
                        "style" => RsxAttr::Style(content.parse()?),
                        "id" => RsxAttr::Id(content.parse()?),
                        "onclick" => RsxAttr::Onclick(content.parse()?),
                        _ => RsxAttr::Other {
                            name,
                            value: content.parse()?,
                        },
                    };
                    attrs.push(attr);
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

pub fn expand_rsx(element: RsxElement) -> TokenStream2 {
    let tag = element.tag.to_string();

    let mut class_code = quote! { tairitsu_vdom::Classes::new() };
    let mut style_code = quote! { tairitsu_vdom::Style::new() };
    let mut event_handlers = Vec::new();
    let mut other_attrs = Vec::new();

    for attr in element.attrs {
        match attr {
            RsxAttr::Class(expr) => {
                class_code = quote! { #expr };
            }
            RsxAttr::Style(expr) => {
                style_code = quote! { #expr };
            }
            RsxAttr::Id(expr) => {
                let value = expr;
                other_attrs.push(quote! { .attr("id", #value) });
            }
            RsxAttr::Onclick(expr) => {
                let handler = expr;
                event_handlers.push(quote! { .on_event("click", move |e| { (#handler)(e); }) });
            }
            RsxAttr::Other { name, value } => {
                let name_str = name.to_string();
                if let Some(event_name) = name_str.strip_prefix("on") {
                    event_handlers
                        .push(quote! { .on_event(#event_name, move |e| { (#value)(e); }) });
                } else {
                    other_attrs.push(quote! { .attr(#name_str, #value) });
                }
            }
        }
    }

    let mut children_code = Vec::new();
    for child in element.children {
        match child {
            RsxChild::Text(text) => {
                let text_value = text.value();
                children_code.push(quote! {
                    .child(tairitsu_vdom::VNode::Text(tairitsu_vdom::VText::new(#text_value)))
                });
            }
            RsxChild::Element(elem) => {
                let child_code = expand_rsx(elem);
                children_code.push(quote! { .child(#child_code) });
            }
            RsxChild::Dynamic(expr) => {
                children_code.push(quote! { .children(#expr) });
            }
        }
    }

    quote! {
        tairitsu_vdom::VNode::Element(
            tairitsu_vdom::VElement::new(#tag)
                .class(#class_code)
                .style(#style_code)
                #(#other_attrs)*
                #(#event_handlers)*
                #(#children_code)*
        )
    }
}
