//! Registry - Manages Images and Containers (similar to Docker registry/daemon)

use anyhow::{Context, Result};
use bytes::Bytes;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{Container, Image};

/// Registry manages images and containers
///
/// Similar to Docker's daemon, it tracks available images and running containers
pub struct Registry {
    images: Arc<Mutex<HashMap<String, Image>>>,
    containers: Arc<Mutex<HashMap<String, Container>>>,
}

impl Registry {
    /// Create a new empty Registry
    pub fn new() -> Self {
        Self {
            images: Arc::new(Mutex::new(HashMap::new())),
            containers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register an image with a name (similar to docker pull/build)
    ///
    /// # Arguments
    /// * `name` - Unique name for the image (e.g., "my-app:v1.0")
    /// * `wasm_binary` - WASM binary file to create the image from
    pub fn register_image(&self, name: impl Into<String>, wasm_binary: Bytes) -> Result<()> {
        let name = name.into();
        let image =
            Image::new(wasm_binary).context(format!("Failed to create image '{}'", name))?;

        let mut images = self.images.lock().expect("registry images lock poisoned");
        images.insert(name.clone(), image);

        Ok(())
    }

    /// Register a pre-compiled WIT component as an image
    pub fn register_component(
        &self,
        name: impl Into<String>,
        component_binary: Bytes,
    ) -> Result<()> {
        let name = name.into();
        let image = Image::from_component(component_binary)
            .context(format!("Failed to create component image '{}'", name))?;

        let mut images = self.images.lock().expect("registry images lock poisoned");
        images.insert(name, image);

        Ok(())
    }

    /// Get image by name
    pub fn get_image(&self, name: &str) -> Option<Image> {
        let images = self.images.lock().expect("registry images lock poisoned");
        images.get(name).cloned()
    }

    /// Get mutable reference to container by name
    ///
    /// # Arguments
    /// * `name` - Container name
    /// * `f` - Callback function to execute on the container
    pub fn get_container_mut<F, R>(&self, name: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut Container) -> R,
    {
        let mut containers = self
            .containers
            .lock()
            .expect("registry containers lock poisoned");
        containers.get_mut(name).map(f)
    }

    /// Stop and remove container (similar to docker stop/rm)
    pub fn stop_container(&self, name: &str) -> Option<Container> {
        let mut containers = self
            .containers
            .lock()
            .expect("registry containers lock poisoned");
        containers.remove(name)
    }

    /// List all registered image names
    pub fn list_images(&self) -> Vec<String> {
        let images = self.images.lock().expect("registry images lock poisoned");
        images.keys().cloned().collect()
    }

    /// List all running container names
    pub fn list_containers(&self) -> Vec<String> {
        let containers = self
            .containers
            .lock()
            .expect("registry containers lock poisoned");
        containers.keys().cloned().collect()
    }

    /// Remove image by name
    pub fn remove_image(&self, name: &str) -> Option<Image> {
        let mut images = self.images.lock().expect("registry images lock poisoned");
        images.remove(name)
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registry")
            .field("images", &self.list_images())
            .field("containers", &self.list_containers())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    const MINIMAL_WASM: &[u8] = b"\x00asm\x01\x00\x00\x00";

    #[test]
    fn test_new_creates_empty_registry() {
        let reg = Registry::new();
        assert!(reg.list_images().is_empty());
        assert!(reg.list_containers().is_empty());
    }

    #[test]
    fn test_default_creates_empty_registry() {
        let reg = Registry::default();
        assert!(reg.list_images().is_empty());
        assert!(reg.list_containers().is_empty());
    }

    #[test]
    fn test_register_image_and_get() {
        let reg = Registry::new();
        let wasm = Bytes::from_static(MINIMAL_WASM);
        let result = reg.register_image("test:v1", wasm);
        if result.is_ok() {
            let img = reg.get_image("test:v1");
            assert!(img.is_some(), "image should be retrievable after registration");

            let missing = reg.get_image("nonexistent");
            assert!(missing.is_none());
        }
    }

    #[test]
    fn test_register_image_invalid_bytes() {
        let reg = Registry::new();
        let bad = Bytes::from_static(b"not wasm");
        let result = reg.register_image("bad", bad);
        assert!(result.is_err());
        assert!(reg.get_image("bad").is_none());
    }

    #[test]
    fn test_register_image_overwrites() {
        let reg = Registry::new();
        let wasm = Bytes::from_static(MINIMAL_WASM);
        if reg.register_image("app", wasm.clone()).is_ok() {
            let _ = reg.register_image("app", wasm);
            let images = reg.list_images();
            assert_eq!(images.len(), 1);
        }
    }

    #[test]
    fn test_list_images() {
        let reg = Registry::new();
        let wasm = Bytes::from_static(MINIMAL_WASM);

        let mut registered = 0usize;
        if reg.register_image("alpha", wasm.clone()).is_ok() {
            registered += 1;
        }
        if reg.register_image("beta", wasm).is_ok() {
            registered += 1;
        }

        if registered == 2 {
            let images = reg.list_images();
            assert_eq!(images.len(), 2);
            assert!(images.contains(&"alpha".to_string()));
            assert!(images.contains(&"beta".to_string()));
        }
    }

    #[test]
    fn test_remove_image() {
        let reg = Registry::new();
        let wasm = Bytes::from_static(MINIMAL_WASM);

        if reg.register_image("removeme", wasm).is_ok() {
            let removed = reg.remove_image("removeme");
            assert!(removed.is_some(), "remove_image should return the removed image");

            let gone = reg.get_image("removeme");
            assert!(gone.is_none(), "image should be gone after removal");
        }

        let nope = reg.remove_image("never-existed");
        assert!(nope.is_none(), "removing nonexistent returns None");
    }

    #[test]
    fn test_list_containers_empty() {
        let reg = Registry::new();
        assert!(reg.list_containers().is_empty());
    }

    #[test]
    fn test_get_container_mut_not_found() {
        let reg = Registry::new();
        let result = reg.get_container_mut("nonexistent", |_c| 42);
        assert!(result.is_none());
    }

    #[test]
    fn test_stop_container_not_found() {
        let reg = Registry::new();
        let result = reg.stop_container("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_register_component_invalid_bytes() {
        let reg = Registry::new();
        let bad = Bytes::from_static(b"not a component");
        let result = reg.register_component("bad-comp", bad);
        assert!(result.is_err());
        assert!(reg.get_image("bad-comp").is_none());
    }
}
