use std::sync::{Arc, RwLock};

use anyhow::Result;

pub trait Component {
    fn step(&mut self) -> Result<u64>;
}

pub type ComponentRef = Arc<RwLock<dyn Component + Send + Sync>>;