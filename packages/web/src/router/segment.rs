//! Route segment parsing and matching.

use std::fmt;

/// Type of route segment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentType {
    /// Static segment (e.g., "users", "posts")
    Static,
    /// Dynamic segment (e.g., ":id", ":slug")
    Dynamic,
    /// Wildcard segment (matches anything)
    Wildcard,
}

/// A segment of a route path
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteSegment {
    /// The raw segment string
    raw: String,
    /// The type of segment
    segment_type: SegmentType,
}

impl RouteSegment {
    /// Parse a path string into segments
    ///
    /// # Example
    ///
    /// ```
    /// use tairitsu_web::router::RouteSegment;
    ///
    /// let segments = RouteSegment::parse_path("/users/:id/posts");
    /// assert_eq!(segments.len(), 3);
    /// assert_eq!(segments[1].is_dynamic(), true);
    /// ```
    pub fn parse_path(path: &str) -> Vec<Self> {
        let mut segments = Vec::new();

        // Skip leading slash and split
        let path = path.trim_start_matches('/');
        let path = path.trim_end_matches('/');

        if path.is_empty() {
            return segments;
        }

        for part in path.split('/') {
            let segment_type = if part == "*" {
                SegmentType::Wildcard
            } else if part.starts_with(':') {
                SegmentType::Dynamic
            } else {
                SegmentType::Static
            };

            segments.push(RouteSegment {
                raw: part.to_string(),
                segment_type,
            });
        }

        segments
    }

    /// Get the segment type
    pub fn segment_type(&self) -> SegmentType {
        self.segment_type
    }

    /// Check if this is a dynamic segment
    pub fn is_dynamic(&self) -> bool {
        self.segment_type == SegmentType::Dynamic
    }

    /// Check if this is a wildcard segment
    pub fn is_wildcard(&self) -> bool {
        self.segment_type == SegmentType::Wildcard
    }

    /// Get the parameter name for dynamic segments
    ///
    /// Returns None for static segments
    pub fn param_name(&self) -> Option<&str> {
        if self.segment_type == SegmentType::Dynamic {
            Some(&self.raw[1..]) // Skip the ':' prefix
        } else {
            None
        }
    }

    /// Check if this segment matches the given path segment
    pub fn matches(&self, segment: &str) -> bool {
        match self.segment_type {
            SegmentType::Static => self.raw == segment,
            SegmentType::Dynamic => true, // Dynamic segments match anything
            SegmentType::Wildcard => true, // Wildcards match anything
        }
    }
}

impl fmt::Display for RouteSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_path() {
        let segments = RouteSegment::parse_path("/");
        assert!(segments.is_empty());
    }

    #[test]
    fn test_parse_simple_path() {
        let segments = RouteSegment::parse_path("/users");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].raw, "users");
        assert_eq!(segments[0].segment_type, SegmentType::Static);
    }

    #[test]
    fn test_parse_nested_path() {
        let segments = RouteSegment::parse_path("/users/posts");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].raw, "users");
        assert_eq!(segments[1].raw, "posts");
    }

    #[test]
    fn test_parse_dynamic_path() {
        let segments = RouteSegment::parse_path("/users/:id");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].raw, "users");
        assert_eq!(segments[1].raw, ":id");
        assert_eq!(segments[1].segment_type, SegmentType::Dynamic);
        assert_eq!(segments[1].param_name(), Some("id"));
    }

    #[test]
    fn test_parse_wildcard_path() {
        let segments = RouteSegment::parse_path("/files/*");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].raw, "files");
        assert_eq!(segments[1].raw, "*");
        assert_eq!(segments[1].segment_type, SegmentType::Wildcard);
    }

    #[test]
    fn test_parse_mixed_path() {
        let segments = RouteSegment::parse_path("/users/:id/posts/:post_id");
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].segment_type, SegmentType::Static);
        assert_eq!(segments[1].segment_type, SegmentType::Dynamic);
        assert_eq!(segments[2].segment_type, SegmentType::Static);
        assert_eq!(segments[3].segment_type, SegmentType::Dynamic);
    }

    #[test]
    fn test_static_segment_matches() {
        let segment = RouteSegment {
            raw: "users".to_string(),
            segment_type: SegmentType::Static,
        };

        assert!(segment.matches("users"));
        assert!(!segment.matches("posts"));
    }

    #[test]
    fn test_dynamic_segment_matches() {
        let segment = RouteSegment {
            raw: ":id".to_string(),
            segment_type: SegmentType::Dynamic,
        };

        assert!(segment.matches("123"));
        assert!(segment.matches("abc"));
        assert!(segment.matches(""));
    }

    #[test]
    fn test_wildcard_segment_matches() {
        let segment = RouteSegment {
            raw: "*".to_string(),
            segment_type: SegmentType::Wildcard,
        };

        assert!(segment.matches("anything"));
    }
}
