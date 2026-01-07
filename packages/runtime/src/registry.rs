//! Registry - Manages Images and Containers (similar to Docker registry/daemon)

use anyhow::{Context, Result};
use bytes::Bytes;
use std::{collections::HashMap, sync::{Arc, Mutex}};

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

        let mut images = self.images.lock().unwrap();
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

        let mut images = self.images.lock().unwrap();
        images.insert(name, image);

        Ok(())
    }

    /// Get image by name
    pub fn get_image(&self, name: &str) -> Option<Image> {
        let images = self.images.lock().unwrap();
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
        let mut containers = self.containers.lock().unwrap();
        containers.get_mut(name).map(f)
    }

    /// Stop and remove container (similar to docker stop/rm)
    pub fn stop_container(&self, name: &str) -> Option<Container> {
        let mut containers = self.containers.lock().unwrap();
        containers.remove(name)
    }

    /// List all registered image names
    pub fn list_images(&self) -> Vec<String> {
        let images = self.images.lock().unwrap();
        images.keys().cloned().collect()
    }

    /// List all running container names
    pub fn list_containers(&self) -> Vec<String> {
        let containers = self.containers.lock().unwrap();
        containers.keys().cloned().collect()
    }

    /// Remove image by name
    pub fn remove_image(&self, name: &str) -> Option<Image> {
        let mut images = self.images.lock().unwrap();
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
