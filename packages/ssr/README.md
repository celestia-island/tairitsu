# tairitsu-ssr

Server-side rendering and static site generation for Tairitsu.

## Overview

`tairitsu-ssr` enables server-side rendering of Tairitsu applications for improved SEO, faster initial page loads, and better user experience.

## Features

- **SSR**: Render Tairitsu apps on the server
- **SSG**: Generate static HTML at build time
- **Hydration**: Attach event listeners after client-side load
- **Streaming**: Progressive HTML rendering
- **Island Architecture**: Partial hydration for interactivity

## Usage

### Server-Side Rendering

```rust
use tairitsu_ssr::{SsrPlatform, render_to_string};

let platform = SsrPlatform::new();
let html = render_to_string(&platform, &app_vnode)?;
```

### Static Site Generation

```bash
tairitsu build --ssg
```

Generates static HTML for all routes.

### Hydration

Attach event listeners to server-rendered HTML:

```rust
use tairitsu_web::WitPlatform;

let platform = WitPlatform::new()?;
platform.hydrate(&app_vnode)?;
```

## Configuration

SSR options in `tairitsu.toml`:

```toml
[ssr]
enabled = true
inline_scripts = false
minify_html = true

[ssr.routes]
# Routes to pre-render
"/about" = "pages/about.rs"
"/blog/**" = "pages/blog/**/*.rs"
```

## Platform

### `SsrPlatform`

Server-side rendering platform that:

- Generates HTML string output
- Handles virtual DOM rendering
- Collects styles for injection
- Generates hydration metadata

### Client Hydration

Hydration attaches event listeners to server-rendered HTML:

```rust
#[no_mangle]
pub extern "C" fn run() {
    let platform = WitPlatform::new().unwrap();

    // Hydrate the app
    platform.hydrate_from_selector("#app").unwrap();
}
```

## Streaming

Stream HTML for faster Time to First Byte (TTFB):

```rust
use tairitsu_ssr::render_to_stream;

let mut writer = std::io::stdout();
render_to_stream(&platform, &vnode, &mut writer)?;
```

## Islands

Islands allow partial hydration - only hydrate interactive components:

```rust
use tairitsu_ssr::island;

div {
    island("interactive-counter", rsx! {
        Counter {}
    })
}
```

## Best Practices

1. **Pre-render critical pages**: Home, about, blog posts
2. **Use islands for interactivity**: Only hydrate what's needed
3. **Stream when possible**: Faster perceived performance
4. **Cache rendered HTML**: Reduce server load

## See Also

- [SSR Guide](../../docs/en-US/guides/ssr.md): Detailed SSR documentation
- [Hydration](../../docs/en-US/guides/hydration.md): Hydration patterns
- [tairitsu-web](../web): Client-side platforms
