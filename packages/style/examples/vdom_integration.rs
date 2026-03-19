//! Example demonstrating tairitsu-style integration with VDOM

use tairitsu_style::{ClassesBuilder, CssProperty, Property};

fn main() {
    println!("Tairitsu Style + VDOM Integration Example\n");
    println!("==========================================\n");

    // Example 1: Using ClassesBuilder to create utility classes
    println!("1. Utility Classes with ClassesBuilder:");
    let utility_classes = ClassesBuilder::new()
        .add("flex")
        .add("flex-col")
        .add("gap-4")
        .add("p-4")
        .add_utility("text-center")
        .build();

    println!("   Generated classes: {}", utility_classes);

    // Example 2: Converting ClassesBuilder to VDOM classes
    println!("\n2. Converting to VDOM Classes:");
    let _vdom_classes = ClassesBuilder::new()
        .add("flex")
        .add("flex-col")
        .add("gap-4")
        .to_vdom_classes();

    println!("   VDOM classes created successfully");

    // Example 3: Generating CSS from utility classes
    println!("\n3. Generating CSS from Utility Classes:");
    let css_stylesheet = ClassesBuilder::new()
        .add_utility("flex")
        .add_utility("text-center")
        .add_utility("p-4")
        .generate_stylesheet();

    println!("   Generated CSS:\n{}", css_stylesheet);

    // Example 4: Using CSS property enums
    println!("\n4. CSS Property Enums:");
    let property = CssProperty::Display;
    println!("   Property: {}", property.as_str());
    println!("   Category: {}", property.category().name());
    println!("   Is Shorthand: {}", property.is_shorthand());
    println!("   MDN URL: {}", property.mdn_url());

    // Example 5: Custom property
    println!("\n5. Custom Properties:");
    let custom_property = Property::Custom("my-custom-property".to_string());
    println!("   Custom property: {}", custom_property.as_str());

    // Example 6: Full CSS generation
    println!("\n6. Full CSS Generation Example:");
    let full_css = ClassesBuilder::new()
        .add_utility("flex")
        .add_utility("flex-col")
        .add_utility("gap-4")
        .add_utility("p-4")
        .add_utility("text-center")
        .add_utility("text-lg")
        .add_utility("font-bold")
        .generate_stylesheet();

    println!("   Full CSS:\n{}", full_css);

    println!("\n✅ All examples completed successfully!");
}
