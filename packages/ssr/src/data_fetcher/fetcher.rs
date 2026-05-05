//! Trait definition for data fetchers

use async_trait::async_trait;

use super::error::FetchError;

/// Trait for async data fetching operations
///
/// Implementations of this trait can fetch data from various sources
/// (HTTP, filesystem, database, etc.)
#[async_trait]
pub trait Fetcher: Send + Sync {
    /// Perform a GET request to the given URL
    async fn get(&self, url: &str) -> Result<Vec<u8>, FetchError>;

    /// Perform a POST request to the given URL with the given body
    async fn post(&self, url: &str, body: Vec<u8>) -> Result<Vec<u8>, FetchError>;

    /// Perform a PUT request to the given URL with the given body
    async fn put(&self, url: &str, body: Vec<u8>) -> Result<Vec<u8>, FetchError> {
        // Default implementation using POST
        self.post(url, body)
            .await
            .map_err(|e| FetchError::Other(format!("PUT not supported: {}", e)))
    }

    /// Perform a DELETE request to the given URL
    async fn delete(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        // Default implementation using GET
        self.get(url)
            .await
            .map_err(|e| FetchError::Other(format!("DELETE not supported: {}", e)))
    }

    /// Fetch JSON and deserialize it
    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, FetchError> {
        let data = self.get(url).await?;
        serde_json::from_slice(&data).map_err(|e| FetchError::Serialization(e.to_string()))
    }

    /// Post JSON and deserialize the response
    async fn post_json<T, U>(&self, url: &str, body: &T) -> Result<U, FetchError>
    where
        T: serde::Serialize + Sync,
        U: serde::de::DeserializeOwned,
    {
        let body_bytes =
            serde_json::to_vec(body).map_err(|e| FetchError::Serialization(e.to_string()))?;
        let data = self.post(url, body_bytes).await?;
        serde_json::from_slice(&data).map_err(|e| FetchError::Serialization(e.to_string()))
    }
}

#[cfg(all(test, feature = "data-fetcher"))]
mod tests {
    use super::*;

    struct MockFetcher {
        response_data: Vec<u8>,
        should_fail: bool,
    }

    #[async_trait]
    impl Fetcher for MockFetcher {
        async fn get(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
            if self.should_fail {
                return Err(FetchError::network("mock error"));
            }
            Ok(self.response_data.clone())
        }

        async fn post(&self, _url: &str, _body: Vec<u8>) -> Result<Vec<u8>, FetchError> {
            if self.should_fail {
                return Err(FetchError::network("mock error"));
            }
            Ok(self.response_data.clone())
        }
    }

    #[tokio::test]
    async fn test_fetcher_get() {
        let fetcher = MockFetcher {
            response_data: b"hello".to_vec(),
            should_fail: false,
        };

        let result = fetcher.get("http://example.com").await;
        assert_eq!(result.unwrap(), b"hello");
    }

    #[tokio::test]
    async fn test_fetcher_post() {
        let fetcher = MockFetcher {
            response_data: b"posted".to_vec(),
            should_fail: false,
        };

        let result = fetcher.post("http://example.com", b"data".to_vec()).await;
        assert_eq!(result.unwrap(), b"posted");
    }

    #[tokio::test]
    async fn test_fetcher_error() {
        let fetcher = MockFetcher {
            response_data: vec![],
            should_fail: true,
        };

        let result = fetcher.get("http://example.com").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().is_network());
    }

    #[tokio::test]
    async fn test_get_json() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct TestData {
            message: String,
            count: i32,
        }

        let fetcher = MockFetcher {
            response_data: br#"{"message":"hello","count":42}"#.to_vec(),
            should_fail: false,
        };

        let result: TestData = fetcher.get_json("http://example.com").await.unwrap();
        assert_eq!(result.message, "hello");
        assert_eq!(result.count, 42);
    }

    #[tokio::test]
    async fn test_post_json() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct RequestData {
            value: i32,
        }

        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct ResponseData {
            result: String,
        }

        let fetcher = MockFetcher {
            response_data: br#"{"result":"ok"}"#.to_vec(),
            should_fail: false,
        };

        let result: ResponseData = fetcher
            .post_json("http://example.com", &RequestData { value: 123 })
            .await
            .unwrap();
        assert_eq!(result.result, "ok");
    }

    #[tokio::test]
    async fn test_get_json_error() {
        let fetcher = MockFetcher {
            response_data: b"invalid json".to_vec(),
            should_fail: false,
        };

        let result = fetcher
            .get_json::<serde_json::Value>("http://example.com")
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().is_serialization());
    }
}
