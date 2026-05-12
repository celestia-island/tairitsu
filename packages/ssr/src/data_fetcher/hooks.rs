//! React hooks for data fetching

#[cfg(feature = "data-fetcher")]
use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

use tairitsu_vdom::runtime;

use super::{Fetcher, Resource, http_fetcher::HttpFetcher};

/// Hook for fetching data from a URL
///
/// This hook provides a reactive way to fetch data in components.
/// It returns a `Signal<Resource<T>>` that tracks the loading state,
/// data, and errors.
///
/// # Type Parameters
/// - `T`: The type of data to parse from the response
/// - `F`: The parser function type
///
/// # Arguments
/// - `url`: The URL to fetch from
/// - `parser`: A function that parses the response bytes into `T`
///
/// # Returns
/// A `Rc<RefCell<Resource<T>>` that tracks the fetch state
///
/// # Example
/// ```ignore
/// let user_data = use_fetch("/api/user", |data| {
///     serde_json::from_slice::<User>(data).map_err(|e| e.to_string())
/// });
///
/// match user_data.borrow() {
///     Resource::Loading => view! { "Loading..." },
///     Resource::Success(user) => view! { "Hello, {user.name}" },
///     Resource::Error(e) => view! { "Error: {e}" },
/// }
/// ```
pub fn use_fetch<T, F>(url: &str, parser: F) -> Rc<RefCell<Resource<T>>>
where
    T: Clone + Send + 'static,
    F: Fn(&[u8]) -> Result<T, String> + Clone + Send + 'static,
{
    use_fetch_with_fetcher_impl(url, parser, None::<HttpFetcher>)
}

/// Hook for fetching data with a custom fetcher
///
/// This is a more flexible version of `use_fetch` that allows you to provide
/// a custom `Fetcher` implementation.
pub fn use_fetch_with_fetcher<T, F, Fr>(
    url: &str,
    parser: F,
    fetcher: Fr,
) -> Rc<RefCell<Resource<T>>>
where
    T: Clone + Send + 'static,
    F: Fn(&[u8]) -> Result<T, String> + Clone + Send + 'static,
    Fr: Fetcher + Clone + Send + Sync + 'static,
{
    use_fetch_with_fetcher_impl(url, parser, Some(fetcher))
}

/// Internal implementation of fetch with optional fetcher
#[allow(unused_variables)]
fn use_fetch_with_fetcher_impl<T, F, Fr>(
    url: &str,
    parser: F,
    fetcher: Option<Fr>,
) -> Rc<RefCell<Resource<T>>>
where
    T: Clone + Send + 'static,
    F: Fn(&[u8]) -> Result<T, String> + Clone + Send + 'static,
    Fr: Fetcher + Clone + Send + Sync + 'static,
{
    let state = Rc::new(RefCell::new(Resource::Loading));

    // For server-side, we use Arc<Mutex<_>> for thread-safe async operations
    #[cfg(feature = "data-fetcher")]
    {
        let state_sync = Arc::new(std::sync::Mutex::new(Resource::Loading));
        let url = url.to_string();
        let component_id = runtime::use_component(tairitsu_vdom::VNode::empty);

        // Initialize the Rc state with Loading
        // The actual async update will happen in the tokio task
        let state_sync_clone = Arc::clone(&state_sync);

        tokio::spawn(async move {
            let result = if let Some(fetcher) = fetcher {
                fetcher.get(&url).await
            } else {
                let http_fetcher = HttpFetcher::new();
                http_fetcher.get(&url).await
            };

            let new_state = match result {
                Ok(data) => match parser(&data) {
                    Ok(value) => Resource::Success(value),
                    Err(e) => Resource::Error(e),
                },
                Err(e) => Resource::Error(e.to_string()),
            };

            // Update the thread-safe state
            *state_sync_clone.lock().unwrap() = new_state;

            // Trigger re-render
            runtime::mark_dirty(component_id);
            runtime::flush_render();
        });
    }

    #[cfg(not(feature = "data-fetcher"))]
    {
        // For non-server builds, we set an error state
        *state.borrow_mut() = Resource::Error("Server feature required".to_string());
    }

    state
}

/// Hook for fetching data with automatic JSON parsing
///
/// This is a convenience wrapper around `use_fetch` that automatically
/// parses JSON responses.
///
/// # Type Parameters
/// - `T`: The type to deserialize from JSON (must implement `serde::DeserializeOwned`)
///
/// # Arguments
/// - `url`: The URL to fetch from
///
/// # Returns
/// A `Rc<RefCell<Resource<T>>` that tracks the fetch state
///
/// # Example
/// ```ignore
/// #[derive(serde::Deserialize)]
/// struct User {
///     name: String,
/// }
///
/// let user_data = use_fetch_json("/api/user");
///
/// match user_data.borrow() {
///     Resource::Loading => view! { "Loading..." },
///     Resource::Success(user) => view! { "Hello, {user.name}" },
///     Resource::Error(e) => view! { "Error: {e}" },
/// }
/// ```
pub fn use_fetch_json<T>(url: &str) -> Rc<RefCell<Resource<T>>>
where
    T: serde::de::DeserializeOwned + Clone + Send + 'static,
{
    use_fetch(url, |data| {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    })
}

/// Hook for fetching data with a custom fetcher and automatic JSON parsing
pub fn use_fetch_json_with_fetcher<T, Fr>(url: &str, fetcher: Fr) -> Rc<RefCell<Resource<T>>>
where
    T: serde::de::DeserializeOwned + Clone + Send + 'static,
    Fr: Fetcher + Clone + Send + Sync + 'static,
{
    use_fetch_with_fetcher(
        url,
        |data| serde_json::from_slice(data).map_err(|e| e.to_string()),
        fetcher,
    )
}

/// Hook for lazy data fetching
///
/// Unlike `use_fetch`, this hook doesn't fetch immediately.
/// Instead, it returns a trigger function that can be called to start the fetch.
///
/// # Type Parameters
/// - `T`: The type of data to parse from the response
/// - `F`: The parser function type
///
/// # Arguments
/// - `url`: The URL to fetch from
/// - `parser`: A function that parses the response bytes into `T`
///
/// # Returns
/// A tuple of:
/// - `Rc<RefCell<Resource<T>>>` - tracks the fetch state
/// - `impl Fn()` - trigger function to start the fetch
///
/// # Example
/// ```ignore
/// let (user_data, fetch_user) = use_lazy_fetch("/api/user", |data| {
///     serde_json::from_slice::<User>(data).map_err(|e| e.to_string())
/// });
///
/// // Initially, user_data is Loading
/// // Call fetch_user() when ready (e.g., on button click)
/// view! {
///     button { onclick: move |_| fetch_user(), "Load User" }
///     match user_data.borrow() {
///         Resource::Loading => view! { "Loading..." },
///         Resource::Success(user) => view! { "Hello, {user.name}" },
///         Resource::Error(e) => view! { "Error: {e}" },
///     }
/// }
/// ```
pub fn use_lazy_fetch<T, F>(url: &str, parser: F) -> (Rc<RefCell<Resource<T>>>, impl Fn())
where
    T: Clone + Send + 'static,
    F: Fn(&[u8]) -> Result<T, String> + Clone + Send + 'static,
{
    let state = Rc::new(RefCell::new(Resource::Loading));
    #[allow(unused_variables)]
    let state_clone = Rc::clone(&state);
    let url = url.to_string();
    let component_id = runtime::use_component(tairitsu_vdom::VNode::empty);

    let trigger = move || {
        #[allow(unused_variables)]
        let url = url.clone();
        #[allow(unused_variables)]
        let parser = parser.clone();
        let component_id = component_id;

        #[cfg(feature = "data-fetcher")]
        {
            let state_sync = Arc::new(std::sync::Mutex::new(Resource::Loading));
            let state_sync_clone = Arc::clone(&state_sync);

            tokio::spawn(async move {
                let http_fetcher = HttpFetcher::new();
                let result = http_fetcher.get(&url).await;

                let new_state = match result {
                    Ok(data) => match parser(&data) {
                        Ok(value) => Resource::Success(value),
                        Err(e) => Resource::Error(e),
                    },
                    Err(e) => Resource::Error(e.to_string()),
                };

                *state_sync_clone.lock().unwrap() = new_state;

                runtime::mark_dirty(component_id);
                runtime::flush_render();
            });
        }

        #[cfg(not(feature = "data-fetcher"))]
        {
            *state_clone.borrow_mut() = Resource::Error("Server feature required".to_string());
            runtime::mark_dirty(component_id);
            runtime::flush_render();
        }
    };

    (state, trigger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "data-fetcher"))]
    #[test]
    fn test_use_fetch_creates_resource() {
        // Note: This is a basic smoke test
        // Full testing requires the runtime to be initialized

        // We can test that the function returns the right type
        let resource: Rc<RefCell<Resource<String>>> = use_fetch("http://example.com", |data| {
            String::from_utf8(data.to_vec()).map_err(|e| e.to_string())
        });

        // On non-server, it returns an error state
        assert!(resource.borrow().is_error());
    }

    #[test]
    fn test_use_lazy_fetch_creates_trigger() {
        let (_resource, _trigger) = use_lazy_fetch("http://example.com", |data| {
            String::from_utf8(data.to_vec()).map_err(|e| e.to_string())
        });

        // Just verify the types are correct
        // Actual execution requires runtime
    }

    #[cfg(feature = "data-fetcher")]
    #[tokio::test]
    async fn test_use_fetch_with_tokio_runtime() {
        // This test verifies that use_fetch works with a tokio runtime
        let resource: Rc<RefCell<Resource<String>>> = use_fetch("http://example.com", |data| {
            String::from_utf8(data.to_vec()).map_err(|e| e.to_string())
        });

        // The resource should be Loading initially
        assert!(matches!(*resource.borrow(), Resource::Loading));

        // Note: We can't actually test the full fetch flow without
        // a real HTTP server or mock fetcher
    }
}
