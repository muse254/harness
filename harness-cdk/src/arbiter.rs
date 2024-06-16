use std::cell::RefCell;

use candid::Nat;
use ic_cdk::{api::management_canister::http_request::HttpResponse, query};

use crate::device::Device;

use harness_macros::get_harness_schema;
use harness_primitives::{
    http::{get_header, Header, HeaderField, Method, Request},
    internals::Schema,
    program::Program,
    ProgramId,
};

// FIXME: placeholder
const HARNESS_WASM: &[u8] =
    include_bytes!("../../examples/hello/target/wasm32-unknown-unknown/release/hello_backend.wasm");

pub(crate) struct Arbiter {
    devices: Vec<Device>,
    programs: Vec<Program>,
}

impl Arbiter {
    fn new() -> Self {
        // let schema = get_harness_schema!();
        todo!()
    }
}

thread_local! {
    static Devices: RefCell<Vec<Device>> = RefCell::new(Vec::new());
}

#[query]
fn http_request(req: Request) -> HttpResponse {
    let method = Method::try_from(req.method.as_str()).unwrap(); //todo

    match (method, req.path.as_str()) {
        (Method::GET, "/program") => {
            // register the device with the arbiter
            match get_header(&Header::HarnessNodeUrl.to_string(), &req.headers) {
                Some(url) => {
                    Devices.with(|devices| {
                        devices.borrow_mut().push(Device {
                            id: ic_cdk::api::caller(),
                            url,
                            programs: vec!["todo!"
                                .parse()
                                .expect("todo: should be in the format of a program id")],
                        })
                    });

                    return HttpResponse {
                        status: Nat::from(200u16),
                        headers: vec![],
                        body: HARNESS_WASM.to_vec(),
                    };
                }
                None => {
                    todo!()
                }
            }
        }

        (_, path) => {
            return HttpResponse {
                status: Nat::from(404u16),
                headers: Vec::new(),
                body: path.as_bytes().to_vec(),
            };
        }
    }
}
