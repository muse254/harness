// //! This module is necessary to create a [`WapcHost`] that can be sent between threads

// use std::sync::{
//     atomic::{AtomicU64, Ordering},
//     Arc,
// };

// use wapc::ModuleState;

// use crate::error::Result;

// static GLOBAL_MODULE_COUNT: AtomicU64 = AtomicU64::new(1);

// pub trait WasmProvider: wapc::WebAssemblyEngineProvider + Send + Sync {}

// pub struct WapcHost <T>{
//     pub engine: Box<dyn WasmProvider>,
//     pub state: Arc<wapc::ModuleState>,
// }

// impl WapcHost {
//     pub fn new<T: WasmProvider>(engine: T) -> Result<Self> {
//         let id = GLOBAL_MODULE_COUNT.fetch_add(1, Ordering::SeqCst);

//         let state = Arc::new(ModuleState::new(None, id));

//         let mh = WapcHost {
//             engine: RefCell::new(engine),
//             state: state.clone(),
//         };

//         mh.initialize(state)?;

//         Ok(mh)
//     }

//     /// Invokes the `__guest_call` function within the guest module as per the waPC specification.
//     /// Provide an operation name and an opaque payload of bytes and the function returns a `Result`
//     /// containing either an error or an opaque reply of bytes.
//     ///
//     /// It is worth noting that the _first_ time `call` is invoked, the WebAssembly module
//     /// might incur a "cold start" penalty, depending on which underlying engine you're using. This
//     /// might be due to lazy initialization or JIT-compilation.
//     pub fn call(&self, op: &str, payload: &[u8]) -> Result<Vec<u8>> {
//         let inv = Invocation::new(op, payload.to_vec());
//         let op_len = inv.operation.len();
//         let msg_len = inv.msg.len();

//         {
//             *self.state.guest_response.write() = None;
//             *self.state.guest_request.write() = Some(inv);
//             *self.state.guest_error.write() = None;
//             *self.state.host_response.write() = None;
//             *self.state.host_error.write() = None;
//         }

//         let callresult = match self.engine.borrow_mut().call(op_len as i32, msg_len as i32) {
//             Ok(c) => c,
//             Err(e) => {
//                 return Err(errors::Error::GuestCallFailure(e.to_string()));
//             }
//         };

//         if callresult == 0 {
//             // invocation failed
//             let lock = self.state.guest_error.read();
//             lock.as_ref().map_or_else(
//                 || {
//                     Err(errors::Error::GuestCallFailure(
//                         "No error message set for call failure".to_owned(),
//                     ))
//                 },
//                 |s| Err(errors::Error::GuestCallFailure(s.clone())),
//             )
//         } else {
//             // invocation succeeded
//             self.state.guest_response.read().as_ref().map_or_else(
//                 || {
//                     let lock = self.state.guest_error.read();
//                     lock.as_ref().map_or_else(
//                         || {
//                             Err(errors::Error::GuestCallFailure(
//                                 "No error message OR response set for call success".to_owned(),
//                             ))
//                         },
//                         |s| Err(errors::Error::GuestCallFailure(s.clone())),
//                     )
//                 },
//                 |e| Ok(e.clone()),
//             )
//         }
//     }
// }

// impl From<wapc::WapcHost> for WapcHost {
//     fn from(value: wapc::WapcHost) -> Self {
//         todo!()
//     }
// }
