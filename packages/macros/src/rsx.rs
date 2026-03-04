use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, Ident, LitStr, Token};

pub struct RsxElement {
    pub tag: Ident,
    pub attrs: Vec<RsxAttr>,
    pub children: Vec<RsxChild>,
}

pub enum RsxAttr {
    Class(LitStr),
    Style(LitStr),
    Id(LitStr),
    Onclick(LitStr),
    Other { name: Ident, value: LitStr },
}

pub enum RsxChild {
    Text(LitStr),
    Element(RsxElement),
}

impl Parse for RsxElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tag: Ident = input.parse()?;
        let mut attrs = Vec::new();
        let mut children = Vec::new();

        // Parse attributes { ... }
        if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);

            while !content.is_empty() {
                if content.peek(LitStr) {
                    // Text child
                    let text: LitStr = content.parse()?;
                    children.push(RsxChild::Text(text));
                } else if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                } else if content.peek2(Token![:]) {
                    // Attribute
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
                    // Nested element
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

    let mut class_value = String::new();
    let mut style_value = String::new();
    let mut id_value = String::new();
    let mut onclick_value = String::new();
    let mut other_attrs = Vec::new();

    for attr in element.attrs {
        match attr {
            RsxAttr::Class(v) => class_value = v.value(),
            RsxAttr::Style(v) => style_value = v.value(),
            RsxAttr::Id(v) => id_value = v.value(),
            RsxAttr::Onclick(v) => onclick_value = v.value(),
            RsxAttr::Other { name, value } => {
                other_attrs.push((name.to_string(), value.value()));
            }
        }
    }

    let mut children_code = Vec::new();
    for child in element.children {
        match child {
            RsxChild::Text(text) => {
                let text_value = text.value();
                children_code.push(quote! {
                    tairitsu_vdom::VNode::Text(tairitsu_vdom::VText::new(#text_value))
                });
            }
            RsxChild::Element(elem) => {
                let child_code = expand_rsx(elem);
                children_code.push(child_code);
            }
        }
    }

    let class_code = if class_value.is_empty() {
        quote! { tairitsu_vdom::Classes::new() }
    } else {
        quote! { tairitsu_vdom::Classes::new().add(#class_value) }
    };

    let style_code = if style_value.is_empty() {
        quote! { tairitsu_vdom::Style::new() }
    } else {
        quote! { tairitsu_vdom::Style::new().add("", #style_value) }
    };

    let attr_code: Vec<_> = other_attrs
        .iter()
        .map(|(name, value)| {
            let name = name.as_str();
            let value = value.as_str();
            quote! { .attr(#name, #value) }
        })
        .collect();

    quote! {
        tairitsu_vdom::VNode::Element(
            tairitsu_vdom::VElement::new(#tag)
                .class(#class_code)
                .style(#style_code)
                #(#attr_code)*
                #(
                    .child(#children_code)
                )*
        )
    }
}
