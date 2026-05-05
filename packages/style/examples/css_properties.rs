// Example demonstrating the new CSS property system
use tairitsu_style::{CssProperty, Property};

fn main() {
    // Test basic property access
    let prop = CssProperty::FlexDirection;
    println!("Property: {}", prop);
    println!("CSS name: {}", prop.as_str());
    println!("Category: {}", prop.category());
    println!("Is shorthand: {}", prop.is_shorthand());
    println!("MDN URL: {}", prop.mdn_url());

    println!();

    // Test categorization
    let props = vec![
        CssProperty::Display,
        CssProperty::Width,
        CssProperty::FlexDirection,
        CssProperty::GridTemplateColumns,
        CssProperty::FontSize,
        CssProperty::Animation,
    ];

    for prop in props {
        println!("{} is in category: {}", prop, prop.category());
    }

    println!();

    // Test Property enum (for custom properties)
    let custom: Property = "--my-var".into();
    let known: Property = CssProperty::Color.into();

    println!(
        "Custom property: {} (category: {:?})",
        custom.as_str(),
        custom.category()
    );
    println!(
        "Known property: {} (category: {:?})",
        known.as_str(),
        known.category()
    );
}
