//! Registry - Manages Images and Containers (like a Docker registry/daemon)

use anyhow::{Context, Result};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{Container, Image};

/// A Registry manages Images and running Containers
/// Similar to Docker's daemon, it keeps track of available images and running containers
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
    
    /// Register an Image with a name (like docker pull/build)
    /// 
    /// # Arguments
    /// * `name` - A unique name for the image (e.g., "my-app:v1.0")
    /// * `wasm_binary` - The WASM binary to create the image from
    pub fn register_image(&self, name: impl Into<String>, wasm_binary: Bytes) -> Result<()> {
        let name = name.into();
        let image = Image::new(wasm_binary)
            .context(format!("Failed to create image '{}'", name))?;
        
        let mut images = self.images.lock().unwrap();
        images.insert(name.clone(), image);
        
        Ok(())
    }
    
    /// Register a pre-compiled WIT component as an Image
    pub fn register_component(&self, name: impl Into<String>, component_binary: Bytes) -> Result<()> {
        let name = name.into();
        let image = Image::from_component(component_binary)
            .context(format!("Failed to create component image '{}'", name))?;
        
        let mut images = self.images.lock().unwrap();
        images.insert(name, image);
        
        Ok(())
    }
    
    /// Get an Image by name
    pub fn get_image(&self, name: &str) -> Option<Image> {
        let images = self.images.lock().unwrap();
        images.get(name).cloned()
    }
    
    /// Create and start a Container from an Image (like docker run)
    /// 
    /// # Arguments
    /// * `image_name` - The name of the image to instantiate
    /// * `container_name` - A unique name for the container
    pub fn run_container(&self, image_name: &str, container_name: impl Into<String>) -> Result<()> {
        let container_name = container_name.into();
        
        let image = self.get_image(image_name)
            .context(format!("Image '{}' not found", image_name))?;
        
        let container = Container::new(&image)
            .context(format!("Failed to create container '{}'", container_name))?;
        
        let mut containers = self.containers.lock().unwrap();
        containers.insert(container_name, container);
        
        Ok(())
    }
    
    /// Get a mutable reference to a Container
    pub fn get_container_mut<F, R>(&self, name: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut Container) -> R,
    {
        let mut containers = self.containers.lock().unwrap();
        containers.get_mut(name).map(f)
    }
    
    /// Stop and remove a Container (like docker stop/rm)
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
    
    /// Remove an Image by name
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
