//! Error handling primitives for the harness project.
use thiserror::Error;

#[cfg(feature = "wasm-ext")]
use wapc::errors::Error as WapcError;
#[cfg(feature = "wasm-ext")]
use wasmtime_provider::errors::Error as WasmtimeError;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Custom(#[from] anyhow::Error),

    #[error("IO error: {message}")]
    IO {
        message: String,
        #[source]
        inner: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        inner: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Error {
    pub fn io<T>(message: &str, err: Option<T>) -> Self
    where
        T: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::IO {
            message: message.into(),
            inner: err.map(|val| val.into()),
        }
    }

    pub fn internal<T>(message: &str, err: Option<T>) -> Self
    where
        T: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::IO {
            message: message.into(),
            inner: err.map(|val| val.into()),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::IO {
            message: e.to_string(),
            inner: None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO {
            message: e.to_string(),
            inner: None,
        }
    }
}

#[cfg(feature = "wasm-ext")]
impl From<WapcError> for Error {
    fn from(err: WapcError) -> Self {
        match err {
            WapcError::IO(e) => Error::io("wapc protocol io error", Some(e)),
            WapcError::NoSuchFunction(e) => Error::io("wapc protocol did not find method", Some(e)),
            WapcError::General(e) => Error::internal("wapc protocol internal error", Some(e)),
            WapcError::GuestCallFailure(e) => {
                Error::io("wapc protocol trouble communicating with host", Some(e))
            }
            _ => Error::Custom(anyhow::anyhow!(err)), // todo
        }
    }
}

#[cfg(feature = "wasm-ext")]
impl From<WasmtimeError> for Error {
    fn from(err: WasmtimeError) -> Self {
        match err {
            WasmtimeError::InitializationFailed(e) => {
                Error::internal("wastime initialization failed", Some(e))
            }
            _ => Error::Custom(anyhow::anyhow!(err)), // todo
        }
    }
}
