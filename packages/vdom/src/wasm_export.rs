wit_bindgen::generate!({
    path: "wit",
    world: "vdom",
});

pub struct VdomExports;

impl exports::tairitsu::vdom::version::Guest for VdomExports {
    fn get_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

impl exports::tairitsu::vdom::svg::Guest for VdomExports {
    fn sanitize_svg(content: String) -> String {
        crate::svg::SafeSvg::new(&content).into_content()
    }
}

export!(VdomExports);
