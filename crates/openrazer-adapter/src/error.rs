use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("dbus error: {0}")]
    Dbus(#[from] zbus::Error),

    #[error("dbus fdo error: {0}")]
    Fdo(#[from] zbus::fdo::Error),
}

pub type Result<T> = std::result::Result<T, AdapterError>;
