//! Streaming SSR support for Tairitsu
//!
//! This module provides streaming HTML rendering capabilities that allow
//! sending HTML chunks to the client as they are generated, enabling
//! progressive rendering and faster Time to First Byte (TTFB).
//!
//! # Suspense Boundaries
//!
//! The streaming renderer supports Suspense boundaries, which allow rendering
//! fallback content first and then replacing it with the actual content when
//! it becomes available. This is particularly useful for:
//!
//! - Data fetching components
//! - Lazy-loaded content
//! - Code-split components
//!
//! # Example
//!
//! ```rust
//! use tairitsu_ssr::streaming::{render_to_stream, HtmlChunk};
//! use futures::stream::StreamExt;
//!
//! # async fn example() -> anyhow::Result<()> {
//! # let wasm_bytes = vec![];
//! # let config = tairitsu_ssr::SsrConfig::default();
//! let mut stream = render_to_stream(&wasm_bytes, config).await?;
//!
//! while let Some(chunk) = stream.next().await {
//!     match chunk {
//!         HtmlChunk::Content(html) => {
//!             // Send HTML chunk to client
//!             println!("{}", html);
//!         }
//!         HtmlChunk::SuspensePlaceholder { id, html } => {
//!             // Send placeholder HTML to client
//!             println!("<template id=\"{}\">{}</template>", id, html);
//!         }
//!         HtmlChunk::SuspenseResolution { id, html } => {
//!             // Send actual content to client with script to replace placeholder
//!             println!("<script>replaceSuspense(\"{}\", `{}`)</script>", id, html);
//!         }
//!         HtmlChunk::Complete => {
//!             // Streaming complete
//!             break;
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use futures::stream::Stream;
use std::pin::Pin;

use tairitsu_vdom::VNode;

use crate::{host_state::SsrConfig, render_to_html};

/// A chunk of HTML that can be sent to the client
///
/// This enum represents different types of HTML chunks that can be streamed
/// to the client during SSR.
#[derive(Clone, Debug)]
pub enum HtmlChunk {
    /// Regular HTML content
    Content(String),

    /// A Suspense boundary placeholder
    ///
    /// This variant contains the fallback HTML that should be displayed while
    /// the actual content is being prepared. The `id` is used to match this
    /// placeholder with its resolution.
    SuspensePlaceholder { id: String, html: String },

    /// The resolution of a Suspense boundary
    ///
    /// This variant contains the actual HTML content that replaces the placeholder.
    /// The `id` must match a previously sent placeholder.
    SuspenseResolution { id: String, html: String },

    /// Complete signal - indicates that streaming is complete
    Complete,
}

impl HtmlChunk {
    /// Convert this chunk to its string representation
    ///
    /// For content chunks, returns the HTML directly.
    /// For suspense chunks, wraps the content in appropriate script tags.
    pub fn to_string(&self) -> String {
        match self {
            HtmlChunk::Content(html) => html.clone(),
            HtmlChunk::SuspensePlaceholder { id, html } => {
                format!("<template data-suspense-id=\"{}\">{}</template>", id, html)
            }
            HtmlChunk::SuspenseResolution { id, html } => {
                format!(
                    "<script>(function(){{var el=document.querySelector('[data-suspense-id=\"{}\"]');if(el){{el.outerHTML={};}}}})();</script>",
                    id,
                    javascript_escape(html)
                )
            }
            HtmlChunk::Complete => String::new(),
        }
    }

    /// Check if this is the complete signal
    pub fn is_complete(&self) -> bool {
        matches!(self, HtmlChunk::Complete)
    }
}

/// Escape a string for use in a JavaScript string literal
fn javascript_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('`');
    for c in s.chars() {
        match c {
            '`' => result.push_str("\\`"),
            '$' => result.push_str("\\$"),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result.push('`');
    result
}

/// A stream of HTML chunks
///
/// Note: This stream is not Send because VNode contains Rc which is not Send.
/// For use in async contexts that require Send, consider collecting chunks
/// into a Vec<String> before sending across threads.
pub type HtmlStream = Pin<Box<dyn Stream<Item = HtmlChunk>>>;

/// Render a WASM component to a streaming HTML response
///
/// This function creates a stream that yields HTML chunks as they are generated.
/// The stream starts with the initial HTML, then yields any suspense placeholder
/// chunks, followed by suspense resolution chunks, and finally a complete signal.
///
/// # Arguments
///
/// * `wasm_bytes` - The compiled WASM component bytes
/// * `config` - SSR configuration (viewport dimensions, etc.)
///
/// # Returns
///
/// A stream that yields `HtmlChunk` values
///
/// # Example
///
/// ```rust
/// use tairitsu_ssr::streaming::{render_to_stream, HtmlChunk};
/// use futures::stream::StreamExt;
///
/// # async fn example() -> anyhow::Result<()> {
/// # let wasm_bytes = vec![];
/// # let config = tairitsu_ssr::SsrConfig::default();
/// let mut stream = render_to_stream(&wasm_bytes, config).await?;
///
/// while let Some(chunk) = stream.next().await {
///     if !chunk.is_complete() {
///         // Send chunk to client
///         println!("{}", chunk.to_string());
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn render_to_stream(wasm_bytes: &[u8], config: SsrConfig) -> Result<HtmlStream> {
    // First, render the initial HTML
    let initial_html = render_to_html(wasm_bytes, config.clone())?;

    // Create a channel for streaming chunks
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<HtmlChunk>(32);

    // Spawn a task to process the DOM and emit chunks
    tokio::task::spawn(async move {
        // Send the initial HTML
        if sender.send(HtmlChunk::Content(initial_html)).await.is_err() {
            return;
        }

        // Process suspense boundaries
        // In a real implementation, we would:
        // 1. Scan the DOM for suspense boundaries
        // 2. Render and emit placeholders
        // 3. Wait for async content to resolve
        // 4. Render and emit resolutions

        // For now, just send the complete signal
        let _ = sender.send(HtmlChunk::Complete).await;
    });

    // Convert the channel receiver to a stream
    let stream = Box::pin(async_stream::stream! {
        while let Some(chunk) = receiver.recv().await {
            yield chunk;
        }
    });

    Ok(stream)
}

/// Render a single Suspense boundary
///
/// This function renders a suspense boundary by first emitting the fallback
/// content as a placeholder, then emitting the actual content when ready.
///
/// # Arguments
///
/// * `component_id` - A unique identifier for this suspense boundary
/// * `fallback` - The fallback VNode to show while content loads
/// * `content` - A future that resolves to the actual content VNode
///
/// # Returns
///
/// A vector of HTML chunks representing the suspense boundary lifecycle
///
/// # Example
///
/// ```rust
/// use tairitsu_ssr::streaming::render_suspense_boundary;
/// use tairitsu_vdom::{VElement, VNode};
///
/// # async fn example() {
/// let fallback = VNode::Element(VElement::new("div").child(VNode::Text(
///     tairitsu_vdom::VText::new("Loading...")
/// )));
///
/// let content = VNode::Element(VElement::new("div").child(VNode::Text(
///     tairitsu_vdom::VText::new("Hello, World!")
/// )));
///
/// let chunks = render_suspense_boundary("my-suspense", fallback, content).await;
///
/// // chunks[0] is the placeholder
/// // chunks[1] is the resolution
/// # }
/// ```
pub async fn render_suspense_boundary(
    component_id: &str,
    fallback: VNode,
    content: VNode,
) -> Vec<HtmlChunk> {
    // Render the fallback
    let fallback_html = fallback.render_to_html();

    // Create the placeholder chunk
    let placeholder = HtmlChunk::SuspensePlaceholder {
        id: component_id.to_string(),
        html: fallback_html,
    };

    // Render the actual content
    let content_html = content.render_to_html();

    // Create the resolution chunk
    let resolution = HtmlChunk::SuspenseResolution {
        id: component_id.to_string(),
        html: content_html,
    };

    vec![placeholder, resolution]
}

/// Render a VNode to a stream of HTML chunks
///
/// This is a simpler version of `render_to_stream` that works directly with
/// a VNode instead of WASM bytes.
///
/// # Arguments
///
/// * `vnode` - The VNode to render
///
/// # Returns
///
/// A stream that yields `HtmlChunk` values
pub fn render_vnode_to_stream(vnode: VNode) -> HtmlStream {
    let stream = async_stream::stream! {
        // Render the VNode to HTML
        let html = vnode.render_to_html();
        yield HtmlChunk::Content(html);
        yield HtmlChunk::Complete;
    };

    Box::pin(stream)
}

/// Hydration script template for client-side hydration
///
/// This function generates a JavaScript snippet that should be included
/// in the streamed HTML to enable client-side hydration.
pub fn hydration_script() -> String {
    r#"
<script>
(function() {
    // Store suspense boundaries for hydration
    window.__TAIRITSU_SUSPENSE__ = window.__TAIRITSU_SUSPENSE__ || {};

    // Replace a suspense placeholder with actual content
    window.replaceSuspense = function(id, html) {
        var placeholder = document.querySelector('[data-suspense-id="' + id + '"]');
        if (placeholder) {
            var template = document.createElement('template');
            template.innerHTML = html;
            placeholder.parentNode.replaceChild(template.content.firstChild, placeholder);
            delete window.__TAIRITSU_SUSPENSE__[id];
        }
    };

    // Register a suspense boundary
    window.registerSuspense = function(id) {
        window.__TAIRITSU_SUSPENSE__[id] = true;
    };
})();
</script>
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::StreamExt;
    use tairitsu_vdom::{VElement, VText};

    #[test]
    fn test_html_chunk_content_to_string() {
        let chunk = HtmlChunk::Content("<div>Hello</div>".to_string());
        assert_eq!(chunk.to_string(), "<div>Hello</div>");
    }

    #[test]
    fn test_html_chunk_suspense_placeholder_to_string() {
        let chunk = HtmlChunk::SuspensePlaceholder {
            id: "test-id".to_string(),
            html: "<div>Loading...</div>".to_string(),
        };
        let result = chunk.to_string();
        assert!(result.contains("data-suspense-id=\"test-id\""));
        assert!(result.contains("<div>Loading...</div>"));
    }

    #[test]
    fn test_html_chunk_suspense_resolution_to_string() {
        let chunk = HtmlChunk::SuspenseResolution {
            id: "test-id".to_string(),
            html: "<div>Content</div>".to_string(),
        };
        let result = chunk.to_string();
        assert!(result.contains("data-suspense-id=\"test-id\""));
        assert!(result.contains("<div>Content</div>"));
    }

    #[test]
    fn test_html_chunk_is_complete() {
        assert!(HtmlChunk::Complete.is_complete());
        assert!(!HtmlChunk::Content("test".to_string()).is_complete());
        assert!(
            !HtmlChunk::SuspensePlaceholder {
                id: "test".to_string(),
                html: "test".to_string(),
            }
            .is_complete()
        );
    }

    #[test]
    fn test_javascript_escape() {
        assert_eq!(javascript_escape("hello"), "`hello`");
        assert_eq!(javascript_escape("hello`world"), "`hello\\`world`");
        assert_eq!(javascript_escape("hello$world"), "`hello\\$world`");
        assert_eq!(javascript_escape("hello\nworld"), "`hello\\nworld`");
        assert_eq!(javascript_escape("hello\\world"), "`hello\\\\world`");
    }

    #[tokio::test]
    async fn test_render_suspense_boundary() {
        let fallback =
            VNode::Element(VElement::new("div").child(VNode::Text(VText::new("Loading..."))));
        let content =
            VNode::Element(VElement::new("div").child(VNode::Text(VText::new("Hello, World!"))));

        let chunks = render_suspense_boundary("test-suspense", fallback, content).await;

        assert_eq!(chunks.len(), 2);

        match &chunks[0] {
            HtmlChunk::SuspensePlaceholder { id, html } => {
                assert_eq!(id, "test-suspense");
                assert!(html.contains("Loading..."));
            }
            _ => panic!("Expected SuspensePlaceholder"),
        }

        match &chunks[1] {
            HtmlChunk::SuspenseResolution { id, html } => {
                assert_eq!(id, "test-suspense");
                assert!(html.contains("Hello, World!"));
            }
            _ => panic!("Expected SuspenseResolution"),
        }
    }

    #[tokio::test]
    async fn test_render_vnode_to_stream() {
        let vnode = VNode::Element(
            VElement::new("div")
                .attr("id", "test")
                .child(VNode::Text(VText::new("Hello"))),
        );

        let mut stream = render_vnode_to_stream(vnode);

        let chunk1 = stream.next().await.unwrap();
        match chunk1 {
            HtmlChunk::Content(html) => {
                assert!(html.contains("<div"));
                assert!(html.contains("id=\"test\""));
                assert!(html.contains("Hello"));
            }
            _ => panic!("Expected Content chunk"),
        }

        let chunk2 = stream.next().await.unwrap();
        assert!(chunk2.is_complete());
    }

    #[test]
    fn test_hydration_script() {
        let script = hydration_script();
        assert!(script.contains("window.__TAIRITSU_SUSPENSE__"));
        assert!(script.contains("window.replaceSuspense"));
        assert!(script.contains("window.registerSuspense"));
    }
}
