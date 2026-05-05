//! Theme integration — uses hikari's palette system to inject CSS variables.

use hikari_components::{ComponentPalette, Style, ThemePalette};
use hikari_palette::{get_palette, Tairitsu};

pub fn get_theme_style(theme_name: &str) -> Style {
    let palette = get_palette(theme_name).unwrap_or_else(Tairitsu::palette);

    let theme_palette = ThemePalette::from_palette(&palette);
    let component_palette = ComponentPalette::from_palette(&palette);

    let css_vars = format!(
        "{} {}",
        theme_palette.css_variables(),
        component_palette.css_variables()
    );

    Style::from(css_vars)
}

pub fn tairitsu_style() -> Style {
    get_theme_style("tairitsu")
}
