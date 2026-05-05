use tairitsu_style::*;

fn main() {
    let registry = create_default_registry();

    let classes = ["text-white", "bg-black"];

    for class in classes {
        if let Some(util) = registry.find(class) {
            println!("Found utility for: {}", class);
            println!("  Pattern: {}", util.pattern());

            let parsed = ParsedUtility::parse(class);
            if let Some(css) = util.generate_css(class, &parsed) {
                println!("  CSS: {}", css);
                println!(
                    "  Contains 'color:#ffffff': {}",
                    css.contains("color:#ffffff")
                );
            }
        } else {
            println!("No utility found for: {}", class);
        }
        println!();
    }
}
