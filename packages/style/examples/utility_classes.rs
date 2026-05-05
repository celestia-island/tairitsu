//! Example demonstrating the UtilityClass system and ClassesBuilder integration.
//!
//! This example shows how to:
//! - Use utility classes with ClassesBuilder
//! - Generate CSS from utility classes
//! - Work with variants (responsive, state)
//! - Use arbitrary values

use tairitsu_style::*;

fn main() {
    println!("=== Utility Class System Demo ===\n");

    // Basic utility class usage
    println!("1. Basic Utility Classes:");
    let classes = ClassesBuilder::new()
        .add_utility("flex")
        .add_utility("p-4")
        .add_utility("text-center")
        .build();
    println!("   Classes: {}", classes);

    // With CSS generation
    let css = ClassesBuilder::new()
        .add_utility("flex")
        .add_utility("p-4")
        .generate_stylesheet();
    println!("   CSS:\n{}\n", css);

    // Responsive variants
    println!("2. Responsive Variants:");
    let classes = ClassesBuilder::new()
        .add_utility("p-4")
        .add_utility("md:p-8")
        .add_utility("lg:p-12")
        .build();
    println!("   Classes: {}", classes);

    let css = ClassesBuilder::new()
        .add_utilities("p-4 md:p-8 lg:p-12")
        .generate_stylesheet();
    println!("   CSS:\n{}\n", css);

    // State variants
    println!("3. State Variants:");
    let classes = ClassesBuilder::new()
        .add_utility("text-center")
        .add_utility("hover:text-left")
        .add_utility("focus:text-right")
        .build();
    println!("   Classes: {}", classes);

    let css = ClassesBuilder::new()
        .add_utilities("text-center hover:text-left focus:text-right")
        .generate_stylesheet();
    println!("   CSS:\n{}\n", css);

    // Combined variants
    println!("4. Combined Responsive + State Variants:");
    let classes = ClassesBuilder::new().add_utility("md:hover:flex").build();
    println!("   Classes: {}", classes);

    let (_, css) = ClassesBuilder::new().add_utility_with_css("md:hover:flex");
    if let Some(css) = css {
        println!("   CSS: {}\n", css);
    }

    // Arbitrary values
    println!("5. Arbitrary Values:");
    let classes = ClassesBuilder::new()
        .add_utility("p-[10px]")
        .add_utility("m-[1.5rem]")
        .add_utility("text-[#ff0000]")
        .build();
    println!("   Classes: {}", classes);

    let css = ClassesBuilder::new()
        .add_utilities("p-[10px] m-[1.5rem]")
        .generate_stylesheet();
    println!("   CSS:\n{}\n", css);

    // Mixed regular and utility classes
    println!("6. Mixed Regular and Utility Classes:");
    let classes = ClassesBuilder::new()
        .add("container")
        .add_utility("flex")
        .add_if("active", true)
        .add_utility_if("p-4", true)
        .add_utility_if("m-4", false)
        .build();
    println!("   Classes: {}\n", classes);

    // Conditional utility classes
    println!("7. Conditional Utility Classes:");
    let is_large = true;
    let is_active = false;

    let classes = ClassesBuilder::new()
        .add_utility("flex")
        .add_utility_if("text-lg", is_large)
        .add_utility_if("text-sm", !is_large)
        .add_utility_if("bg-blue-500", is_active)
        .build();
    println!("   Classes: {}\n", classes);

    // Typography utilities
    println!("8. Typography Utilities:");
    let classes = ClassesBuilder::new()
        .add_utilities("text-xl font-bold text-center")
        .build();
    println!("   Classes: {}", classes);

    let css = ClassesBuilder::new()
        .add_utilities("text-xl font-bold text-center")
        .generate_stylesheet();
    println!("   CSS:\n{}\n", css);

    // Layout utilities
    println!("9. Layout Utilities:");
    let classes = ClassesBuilder::new()
        .add_utilities("flex flex-col justify-center items-center")
        .build();
    println!("   Classes: {}", classes);

    let css = ClassesBuilder::new()
        .add_utilities("flex flex-col justify-center items-center")
        .generate_stylesheet();
    println!("   CSS:\n{}\n", css);

    // Spacing utilities
    println!("10. Spacing Utilities:");
    let classes = ClassesBuilder::new()
        .add_utilities("p-4 m-2 px-6 py-8 mt-10 mb-20 ml-30 mr-40")
        .build();
    println!("   Classes: {}", classes);

    let css = ClassesBuilder::new()
        .add_utilities("p-4 m-2")
        .generate_stylesheet();
    println!("   CSS:\n{}\n", css);

    // Position utilities
    println!("11. Position Utilities:");
    let classes = ClassesBuilder::new()
        .add_utilities("relative absolute fixed")
        .build();
    println!("   Classes: {}", classes);

    let css = ClassesBuilder::new()
        .add_utilities("absolute")
        .generate_stylesheet();
    println!("   CSS:\n{}\n", css);

    // Complex real-world example
    println!("12. Real-world Card Component:");
    let card_classes = ClassesBuilder::new()
        .add("card")
        .add_utility("relative")
        .add_utility("bg-white")
        .add_utility("rounded-lg")
        .add_utility("shadow-lg")
        .add_utility("p-6")
        .add_utility("hover:shadow-xl")
        .add_utility("transition-shadow")
        .build();
    println!("   Classes: {}\n", card_classes);

    println!("=== Demo Complete ===");
}
