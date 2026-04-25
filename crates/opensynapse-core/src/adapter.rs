use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;

use crate::error::Result;
use crate::id::{DeviceId, DiscoveredDevice};
use crate::spec::{AudioSpec, DpiSpec, ForegroundEvent, LightingSpec, MacroSpec};

pub type EventStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;

#[async_trait]
pub trait DeviceDiscovery: Send + Sync {
    async fn discover(&self) -> Result<Vec<DiscoveredDevice>>;
}

#[async_trait]
pub trait LightingAdapter: Send + Sync {
    async fn apply_lighting(&self, device: &DeviceId, spec: &LightingSpec) -> Result<()>;
}

#[async_trait]
pub trait MacroAdapter: Send + Sync {
    async fn apply_macros(&self, device: &DeviceId, macros: &[MacroSpec]) -> Result<()>;
}

#[async_trait]
pub trait DpiAdapter: Send + Sync {
    async fn apply_dpi(&self, device: &DeviceId, spec: &DpiSpec) -> Result<()>;
}

#[async_trait]
pub trait AudioAdapter: Send + Sync {
    async fn apply_audio(&self, spec: &AudioSpec) -> Result<()>;
}

#[async_trait]
pub trait ForegroundWatcher: Send + Sync {
    async fn watch(&self) -> Result<EventStream<ForegroundEvent>>;
}
