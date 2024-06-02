use std::cell::RefCell;

use candid::Nat;
use ic_cdk::{api::management_canister::http_request::HttpResponse, query};

use crate::device::Device;

use harness_primitives::http::{get_header, Header, HeaderField, Method, Request};

// FIXME: placeholder
const HARNESS_WASM: &[u8] =
    include_bytes!("../../examples/hello/target/wasm32-unknown-unknown/release/hello_backend.wasm");

pub struct Arbiter {
    devices: Vec<Device>,
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
                            programs: vec!["todo!".to_string()],
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

        (_, _) => {
            todo!()
        }
    }
}
