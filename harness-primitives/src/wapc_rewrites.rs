#![cfg(feature = "wasm-ext")]

//! This module contains the rewrites for the waPC module that is used in the harness system.
//! Going forward, we will retire this implementation and find out if it's possible to have this in the upstream waPC crate.

use std::{
    cell::RefCell,
    mem::transmute,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use tokio::sync::{Mutex, RwLock};
use wapc::{HostCallback, Invocation, ModuleState as WapcModuleState, WebAssemblyEngineProvider};

use crate::error::{Error, Result};

static GLOBAL_MODULE_COUNT: AtomicU64 = AtomicU64::new(1);

pub(crate) struct WapcHost<'a> {
    engine: Mutex<RefCell<Box<dyn WebAssemblyEngineProvider + 'a + Send>>>,
    state: Arc<ModuleState>,
}

impl<'a> WapcHost<'a> {
    pub async fn new<T: WebAssemblyEngineProvider + 'a + Send>(engine: T) -> Result<Self> {
        let id = GLOBAL_MODULE_COUNT.fetch_add(1, Ordering::SeqCst);

        let state = Arc::new(ModuleState {
            id,
            host_callback: Some(Box::new(move |_a, _b, _c, _d, _e| Ok(vec![]))), // todo?
            ..Default::default()
        });

        let mh = Self {
            engine: Mutex::new(RefCell::new(Box::new(engine))),
            state: state.clone(),
        };

        mh.initialize(state).await?;

        Ok(mh)
    }

    async fn initialize(&self, state: Arc<ModuleState>) -> Result<()> {
        // fixme: this is a bit of a hack, but it's the only way to get the types to line up!!
        let state = state.clone();
        let state = unsafe { transmute::<Arc<ModuleState>, Arc<WapcModuleState>>(state) };

        self.engine
            .lock()
            .await
            .borrow_mut()
            .init(state)
            .map_err(|e| Error::internal("error initializing wasm engine", e.into()))?;

        Ok(())
    }

    pub async fn call(&self, op: &str, payload: &[u8]) -> Result<Vec<u8>> {
        let inv = Invocation {
            operation: op.to_owned(),
            msg: payload.to_vec(),
        };

        let op_len = inv.operation.len();
        let msg_len = inv.msg.len();

        {
            *self.state.guest_response.write().await = None;
            *self.state.guest_request.write().await = Some(inv);
            *self.state.guest_error.write().await = None;
            *self.state.host_response.write().await = None;
            *self.state.host_error.write().await = None;
        }

        let callresult = match self
            .engine
            .lock()
            .await
            .borrow_mut()
            .call(op_len as i32, msg_len as i32)
        {
            Ok(c) => c,
            Err(e) => {
                return Err(Error::internal("wasm call failure", Some(e.to_string())));
            }
        };

        if callresult.eq(&0) {
            // invocation failed
            let lock = self.state.guest_error.read().await;
            return lock.as_ref().map_or_else(
                || {
                    Err(Error::internal(
                        "wasm call failure",
                        "No error message set for call failure".into(),
                    ))
                },
                |s| Err(Error::internal::<String>(s, None)),
            );
        }

        // invocation succeeded
        match self.state.guest_response.read().await.as_ref() {
            None => {
                let lock = self.state.guest_error.read().await;
                lock.as_ref().map_or_else(
                    || {
                        Err(Error::internal(
                            "wasm call failure",
                            "No error message OR response set for call success".into(),
                        ))
                    },
                    |s| Err(Error::internal::<String>(s, None)),
                )
            }
            Some(e) => Ok(e.clone()),
        }
    }
}

#[derive(Default)]
/// Module state is essentially a 'handle' that is passed to a runtime engine to allow it
/// to read and write relevant data as different low-level functions are executed during
/// a waPC conversation
pub struct ModuleState {
    pub(super) guest_request: RwLock<Option<Invocation>>,
    pub(super) guest_response: RwLock<Option<Vec<u8>>>,
    pub(super) host_response: RwLock<Option<Vec<u8>>>,
    pub(super) guest_error: RwLock<Option<String>>,
    pub(super) host_error: RwLock<Option<String>>,
    pub(super) host_callback: Option<Box<HostCallback>>,
    pub(super) id: u64,
}
