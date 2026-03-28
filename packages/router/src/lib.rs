//! File-system based routing for Tairitsu framework.
//!
//! This module provides a flexible routing system that supports:
//! - Static routes (e.g., `/about`, `/contact`)
//! - Dynamic routes (e.g., `/users/:id`, `/posts/:slug`)
//! - Route guards/middleware
//! - 404 fallback
//!
//! # Example
//!
//! ```ignore
//! use tairitsu_router::{Router, Route, RouteHandler};
//!
//! let router = Router::new()
//!     .route("/", home_handler)
//!     .route("/about", about_handler)
//!     .route("/users/:id", user_handler)
//!     .fallback(not_found_handler);
//!
//! let vnode = router.render("/users/123");
//! ```

use std::{
    collections::HashMap,
    fmt,
    sync::Arc,
};

use tairitsu_vdom::VNode;

pub mod segment;

pub use segment::{RouteSegment, SegmentType};

/// A route in the router
#[derive(Clone)]
pub struct Route {
    /// The path pattern (e.g., "/users/:id")
    pub path: String,
    /// The handler that renders the route
    pub handler: RouteHandler,
    /// Optional name for the route
    pub name: Option<String>,
    /// Middleware to apply before the route
    pub middleware: Vec<RouteMiddleware>,
    /// Whether this is an exact match or prefix match
    pub exact: bool,
}

impl Route {
    /// Create a new route
    pub fn new(path: impl Into<String>, handler: impl Into<RouteHandler>) -> Self {
        Self {
            path: path.into(),
            handler: handler.into(),
            name: None,
            middleware: Vec::new(),
            exact: true,
        }
    }

    /// Create a prefix route (matches path and all children)
    pub fn prefix(path: impl Into<String>, handler: impl Into<RouteHandler>) -> Self {
        Self {
            path: path.into(),
            handler: handler.into(),
            name: None,
            middleware: Vec::new(),
            exact: false,
        }
    }

    /// Set the route name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add middleware to this route
    pub fn middleware(mut self, middleware: RouteMiddleware) -> Self {
        self.middleware.push(middleware);
        self
    }
}

/// Handler function for a route
pub type RouteHandler = Arc<dyn Fn(Params) -> VNode + Send + Sync>;

/// Route parameters extracted from the URL
pub type Params = HashMap<String, String>;

/// Middleware function for routes
pub type RouteMiddleware = Arc<dyn Fn(&mut Params) -> Result<(), MiddlewareError> + Send + Sync>;

/// Error that can occur during middleware execution
#[derive(Clone, Debug, PartialEq)]
pub enum MiddlewareError {
    /// The route is not accessible
    Forbidden,
    /// The route requires authentication
    Unauthorized,
    /// A custom error message
    Custom(String),
}

impl fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MiddlewareError::Forbidden => write!(f, "Forbidden"),
            MiddlewareError::Unauthorized => write!(f, "Unauthorized"),
            MiddlewareError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

/// Result of matching a route
#[derive(Clone)]
pub struct RouteMatch {
    /// The matched route
    pub route: Route,
    /// The extracted parameters
    pub params: Params,
}

/// A router that matches paths to routes
pub struct Router {
    /// Registered routes
    routes: Vec<Route>,
    /// Fallback handler for 404
    fallback: Option<RouteHandler>,
    /// Global middleware applied to all routes
    global_middleware: Vec<RouteMiddleware>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            fallback: None,
            global_middleware: Vec::new(),
        }
    }

    /// Add a route to the router
    ///
    /// # Example
    ///
    /// ```ignore
    /// router.route("/users/:id", |params| {
    ///     let id = params.get("id").unwrap();
    ///     rsx! { h1 { "User {id}" } }
    /// });
    /// ```
    pub fn route(mut self, path: impl Into<String>, handler: impl Into<RouteHandler>) -> Self {
        self.routes.push(Route::new(path, handler));
        self
    }

    /// Add a prefix route
    ///
    /// # Example
    ///
    /// ```ignore
    /// router.prefix("/app", app_layout_handler);
    /// ```
    pub fn prefix(mut self, path: impl Into<String>, handler: impl Into<RouteHandler>) -> Self {
        self.routes.push(Route::prefix(path, handler));
        self
    }

    /// Add a named route
    ///
    /// # Example
    ///
    /// ```ignore
    /// router.named_route("user_profile", "/users/:id", handler);
    /// ```
    pub fn named_route(
        mut self,
        name: impl Into<String>,
        path: impl Into<String>,
        handler: impl Into<RouteHandler>,
    ) -> Self {
        let mut route = Route::new(path, handler);
        route.name = Some(name.into());
        self.routes.push(route);
        self
    }

    /// Set the fallback handler for unmatched routes
    ///
    /// # Example
    ///
    /// ```ignore
    /// router.fallback(|_params| {
    ///     rsx! {
    ///         h1 { "404 - Not Found" }
    ///         p { "The page you requested could not be found." }
    ///     }
    /// });
    /// ```
    pub fn fallback(mut self, handler: impl Into<RouteHandler>) -> Self {
        self.fallback = Some(handler.into());
        self
    }

    /// Add global middleware applied to all routes
    ///
    /// # Example
    ///
    /// ```ignore
    /// router.middleware(|params| {
    ///     if params.get("auth").is_none() {
    ///         return Err(MiddlewareError::Unauthorized);
    ///     }
    ///     Ok(())
    /// });
    /// ```
    pub fn middleware(mut self, middleware: RouteMiddleware) -> Self {
        self.global_middleware.push(middleware);
        self
    }

    /// Match a path against the registered routes
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(matched) = router.match_route("/users/123") {
    ///     let vnode = (matched.route.handler)(matched.params);
    /// }
    /// ```
    pub fn match_route(&self, path: impl AsRef<str>) -> Option<RouteMatch> {
        let path = path.as_ref().trim_start_matches('/');
        let path = path.trim_end_matches('/');

        for route in &self.routes {
            let pattern = route.path.trim_start_matches('/');
            let pattern = pattern.trim_end_matches('/');

            if let Some(params) = self.match_pattern(pattern, path) {
                return Some(RouteMatch {
                    route: route.clone(),
                    params,
                });
            }
        }

        None
    }

    /// Match a pattern against a path
    fn match_pattern(&self, pattern: &str, path: &str) -> Option<Params> {
        let pattern_segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        let mut params = Params::new();

        // For exact match, segment counts must match
        if pattern_segments.len() != path_segments.len() {
            return None;
        }

        for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
            if let Some(param_name) = pattern_seg.strip_prefix(':') {
                // Dynamic segment - extract parameter
                params.insert(param_name.to_string(), path_seg.to_string());
            } else if **pattern_seg == *"*" {
                // Wildcard - match anything
            } else if pattern_seg != path_seg {
                // Static segment - must match exactly
                return None;
            }
        }

        Some(params)
    }

    /// Render a route by matching the path
    ///
    /// Returns the VNode for the matched route, or the fallback if no match.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let vnode = router.render("/users/123");
    /// ```
    pub fn render(&self, path: impl AsRef<str>) -> VNode {
        if let Some(matched) = self.match_route(path) {
            // Apply global middleware first
            let mut params = matched.params;

            for middleware in &self.global_middleware {
                if let Err(e) = middleware(&mut params) {
                    return self.render_error(e);
                }
            }

            // Apply route-specific middleware
            for middleware in &matched.route.middleware {
                if let Err(e) = middleware(&mut params) {
                    return self.render_error(e);
                }
            }

            // Render the route
            (matched.route.handler)(params)
        } else if let Some(fallback) = &self.fallback {
            (fallback)(Params::new())
        } else {
            self.default_404()
        }
    }

    /// Render an error page
    fn render_error(&self, error: MiddlewareError) -> VNode {
        let message = error.to_string();
        VNode::Element(
            tairitsu_vdom::VElement::new("div")
                .attr("class", "error-page")
                .child(VNode::Element(
                    tairitsu_vdom::VElement::new("h1").child(VNode::Text(
                        tairitsu_vdom::VText::new(&format!("Error: {}", message)),
                    )),
                ))
                .child(VNode::Element(
                    tairitsu_vdom::VElement::new("p").child(VNode::Text(
                        tairitsu_vdom::VText::new("An error occurred while processing your request."),
                    )),
                )),
        )
    }

    /// Default 404 page
    fn default_404(&self) -> VNode {
        VNode::Element(
            tairitsu_vdom::VElement::new("div")
                .attr("class", "error-404")
                .child(VNode::Element(
                    tairitsu_vdom::VElement::new("h1").child(VNode::Text(tairitsu_vdom::VText::new("404"))),
                ))
                .child(VNode::Element(
                    tairitsu_vdom::VElement::new("p").child(VNode::Text(tairitsu_vdom::VText::new("Page not found"))),
                )),
        )
    }

    /// Generate a URL for a named route with parameters
    ///
    /// # Example
    ///
    /// ```ignore
    /// let url = router.url_for("user_profile", &[("id", "123")]);
    /// // Returns: Some("/users/123".to_string())
    /// ```
    pub fn url_for(&self, name: &str, params: &[(&str, &str)]) -> Option<String> {
        for route in &self.routes {
            if route.name.as_deref() == Some(name) {
                let mut url = route.path.clone();
                for (key, value) in params {
                    url = url.replace(&format!(":{}", key), value);
                }
                return Some(url);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn mock_handler(_params: Params) -> VNode {
        VNode::Text(tairitsu_vdom::VText::new("mock"))
    }

    fn wrap_handler(f: fn(Params) -> VNode) -> RouteHandler {
        Arc::new(f)
    }

    #[test]
    fn test_route_creation() {
        let route = Route::new("/test", wrap_handler(mock_handler));
        assert_eq!(route.path, "/test");
        assert!(route.exact);
    }

    #[test]
    fn test_router_static_route() {
        let router = Router::new().route("/test", wrap_handler(mock_handler));

        let matched = router.match_route("/test");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().params.len(), 0);
    }

    #[test]
    fn test_router_dynamic_route() {
        let router = Router::new().route("/users/:id", wrap_handler(mock_handler));

        let matched = router.match_route("/users/123");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_router_no_match() {
        let router = Router::new().route("/test", wrap_handler(mock_handler));

        let matched = router.match_route("/other");
        assert!(matched.is_none());
    }

    #[test]
    fn test_router_multiple_params() {
        let router = Router::new().route("/posts/:post_id/comments/:comment_id", wrap_handler(mock_handler));

        let matched = router.match_route("/posts/abc/comments/xyz");
        assert!(matched.is_some());
        let params = matched.unwrap().params;
        assert_eq!(params.get("post_id"), Some(&"abc".to_string()));
        assert_eq!(params.get("comment_id"), Some(&"xyz".to_string()));
    }

    #[test]
    fn test_router_fallback() {
        let router = Router::new()
            .route("/test", wrap_handler(mock_handler))
            .fallback(wrap_handler(mock_handler));

        let vnode = router.render("/other");
        // Should return the fallback handler's result
        assert!(matches!(vnode, VNode::Text(_)));
    }

    #[test]
    fn test_named_route() {
        let router = Router::new().named_route("user", "/users/:id", wrap_handler(mock_handler));

        let url = router.url_for("user", &[("id", "123")]);
        assert_eq!(url, Some("/users/123".to_string()));
    }

    #[test]
    fn test_segment_parsing() {
        let segments = RouteSegment::parse_path("/users/:id/posts/:post_id");
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].to_string(), "users");
        assert!(segments[1].is_dynamic());
        assert_eq!(segments[2].to_string(), "posts");
        assert!(segments[3].is_dynamic());
    }
}
