use crate::id::DeviceId;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("device not found: {0:?}")]
    DeviceNotFound(DeviceId),

    #[error("capability not supported: {detail}")]
    UnsupportedCapability { detail: String },

    #[error("backend transient failure: {source}")]
    Transient {
        #[source]
        source: BoxError,
    },

    #[error("backend fatal failure: {source}")]
    Fatal {
        #[source]
        source: BoxError,
    },
}

impl AdapterError {
    pub fn unsupported(detail: impl Into<String>) -> Self {
        Self::UnsupportedCapability {
            detail: detail.into(),
        }
    }

    pub fn transient(e: impl Into<BoxError>) -> Self {
        Self::Transient { source: e.into() }
    }

    pub fn fatal(e: impl Into<BoxError>) -> Self {
        Self::Fatal { source: e.into() }
    }
}

pub type Result<T> = std::result::Result<T, AdapterError>;
