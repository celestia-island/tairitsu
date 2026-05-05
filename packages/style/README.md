# tairitsu-style

Styling utilities and CSS-in-JS implementation for Tairitsu.

## Overview

`tairitsu-style` provides a type-safe, ergonomic way to style components in Tairitsu applications.

## Core Types

### `Style`

Builder pattern for inline styles:

```rust
use tairitsu_style::Style;

let style = Style::new()
    .background("blue")
    .color("white")
    .padding("10px")
    .border_radius("4px");
```

### `Classes`

Builder for CSS classes:

```rust
use tairitsu_style::Classes;

let classes = Classes::from(["container", "active"])
    .add("hover")
    .remove("disabled");
```

## Usage

### Inline Styles

```rust
use tairitsu_macros::rsx;
use tairitsu_style::Style;

div {
    style: Style::new()
        .background("linear-gradient(135deg, #667eea 0%, #764ba2 100%)")
        .color("white")
        .padding("12px 24px")
        .border_radius("6px"),
    "Styled content"
}
```

### CSS Variables

```rust
div {
    style: Style::new()
        .css_var("--primary-color", "blue")
        .css_var("--secondary-color", "red"),
    "Content with CSS variables"
}
```

### Dynamic Styles

```rust
let (is_hovered, set_hovered) = use_state(false);

div {
    style: Style::new()
        .background(if *is_hovered.borrow() { "blue" } else { "gray" })
        .cursor("pointer")
        .onmouseenter(move |_| set_hovered(true))
        .onmouseleave(move |_| set_hovered(false)),
    "Hover me"
}
```

## CSS-in-JS

Define component-scoped styles:

```rust
use tairitsu_style::css;

let button_style = css!(
    ".my-button {
        background: blue;
        color: white;
        padding: 10px 20px;
        border-radius: 4px;
        &:hover {
            background: darkblue;
        }
    }
);
```

## Style Injection

The packager automatically injects styles into the document head.

## Features

- **Type-safe**: No string concatenation for styles
- **Builder pattern**: Fluent API for constructing styles
- **CSS Variables**: Full support for CSS custom properties
- **Responsive**: Media query helpers
- **Pseudo-classes**: :hover, :active, etc.
- **Auto-prefixing**: Vendor prefixes handled automatically

## Best Practices

1. **Use Styles for dynamic values**: When styles depend on state
2. **Use CSS classes for static styles**: For consistent styling
3. **Combine both**: Classes for base styles, Style for overrides
4. **Avoid inline styles for layout**: Prefer CSS classes

## See Also

- [tairitsu-vdom](../vdom/): Virtual DOM integration
- [tairitsu-web](../web/): Platform styling support
- [CSS Guide](../../docs/en-US/guides/styling.md): Styling patterns
