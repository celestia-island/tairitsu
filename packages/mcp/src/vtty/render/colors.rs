pub struct ColorScheme {
    #[allow(dead_code)]
    pub name: &'static str,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub cursor: [u8; 3],
    pub palette: [[u8; 3]; 16],
}

pub const SOLARIZED_DARK: ColorScheme = ColorScheme {
    name: "solarized-dark",
    fg: [147, 161, 161],
    bg: [0, 43, 54],
    cursor: [220, 50, 47],
    palette: [
        [7, 54, 66],
        [220, 50, 47],
        [133, 153, 0],
        [181, 137, 0],
        [38, 139, 210],
        [211, 54, 130],
        [42, 161, 152],
        [238, 232, 213],
        [0, 43, 54],
        [203, 75, 22],
        [88, 110, 117],
        [101, 123, 131],
        [131, 148, 150],
        [108, 113, 196],
        [147, 161, 161],
        [253, 246, 227],
    ],
};

pub const SOLARIZED_LIGHT: ColorScheme = ColorScheme {
    name: "solarized-light",
    fg: [101, 123, 131],
    bg: [253, 246, 227],
    cursor: [220, 50, 47],
    palette: [
        [7, 54, 66],
        [220, 50, 47],
        [133, 153, 0],
        [181, 137, 0],
        [38, 139, 210],
        [211, 54, 130],
        [42, 161, 152],
        [238, 232, 213],
        [0, 43, 54],
        [203, 75, 22],
        [88, 110, 117],
        [101, 123, 131],
        [131, 148, 150],
        [108, 113, 196],
        [147, 161, 161],
        [253, 246, 227],
    ],
};

pub const ONE_HALF_DARK: ColorScheme = ColorScheme {
    name: "one-half-dark",
    fg: [220, 223, 228],
    bg: [40, 44, 52],
    cursor: [198, 120, 221],
    palette: [
        [40, 44, 52],
        [224, 108, 117],
        [152, 195, 121],
        [229, 192, 123],
        [97, 175, 239],
        [198, 120, 221],
        [86, 182, 194],
        [220, 223, 228],
        [92, 99, 112],
        [224, 108, 117],
        [152, 195, 121],
        [229, 192, 123],
        [97, 175, 239],
        [198, 120, 221],
        [86, 182, 194],
        [220, 223, 228],
    ],
};

pub const ONE_HALF_LIGHT: ColorScheme = ColorScheme {
    name: "one-half-light",
    fg: [56, 58, 66],
    bg: [250, 250, 250],
    cursor: [152, 52, 115],
    palette: [
        [56, 58, 66],
        [228, 86, 73],
        [80, 161, 79],
        [204, 143, 44],
        [0, 82, 164],
        [152, 52, 115],
        [19, 135, 141],
        [250, 250, 250],
        [92, 99, 112],
        [228, 86, 73],
        [80, 161, 79],
        [204, 143, 44],
        [0, 82, 164],
        [152, 52, 115],
        [19, 135, 141],
        [220, 223, 228],
    ],
};

pub const IBM_5153: ColorScheme = ColorScheme {
    name: "ibm-5153",
    fg: [51, 255, 0],
    bg: [0, 0, 0],
    cursor: [51, 255, 0],
    palette: [
        [0, 0, 0],
        [170, 0, 0],
        [0, 170, 0],
        [170, 85, 0],
        [0, 0, 170],
        [170, 0, 170],
        [0, 170, 170],
        [170, 170, 170],
        [85, 85, 85],
        [255, 85, 85],
        [85, 255, 85],
        [255, 255, 85],
        [85, 85, 255],
        [255, 85, 255],
        [85, 255, 255],
        [255, 255, 255],
    ],
};

pub fn get_scheme(name: &str) -> &'static ColorScheme {
    match name {
        "solarized-dark" => &SOLARIZED_DARK,
        "solarized-light" => &SOLARIZED_LIGHT,
        "one-half-dark" => &ONE_HALF_DARK,
        "one-half-light" => &ONE_HALF_LIGHT,
        "ibm-5153" => &IBM_5153,
        _ => &SOLARIZED_DARK,
    }
}

#[allow(dead_code)]
pub fn scheme_names() -> &'static [&'static str] {
    &[
        "solarized-dark",
        "solarized-light",
        "one-half-dark",
        "one-half-light",
        "ibm-5153",
    ]
}

pub fn index_to_rgb(index: u8, scheme: &ColorScheme) -> [u8; 3] {
    match index {
        0..=15 => scheme.palette[index as usize],
        16..=231 => {
            let i = (index - 16) as usize;
            let cube = [0u8, 95, 135, 175, 215, 255];
            [cube[i / 36], cube[(i % 36) / 6], cube[i % 6]]
        }
        232..=255 => {
            let v = 8 + (index - 232) * 10;
            [v, v, v]
        }
    }
}

pub fn resolve_color(
    color: crate::vtty::screen::ColorKind,
    scheme: &ColorScheme,
    is_fg: bool,
) -> [u8; 3] {
    match color {
        crate::vtty::screen::ColorKind::Default => {
            if is_fg {
                scheme.fg
            } else {
                scheme.bg
            }
        }
        crate::vtty::screen::ColorKind::Index(i) => index_to_rgb(i, scheme),
        crate::vtty::screen::ColorKind::Rgb(r, g, b) => [r, g, b],
    }
}
