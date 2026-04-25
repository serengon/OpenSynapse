pub mod adapter;
pub mod error;
pub mod id;
pub mod spec;

pub use adapter::{
    AudioAdapter, DeviceDiscovery, DpiAdapter, EventStream, ForegroundWatcher, LightingAdapter,
    MacroAdapter,
};
pub use error::{AdapterError, BoxError, Result};
pub use id::{DeviceId, DeviceKind, DiscoveredDevice};
pub use spec::{
    AudioSpec, Color, DpiSpec, ForegroundEvent, LightingMode, LightingSpec, MacroAction, MacroSpec,
};

pub const TRAIT_VERSION: u32 = 1;
