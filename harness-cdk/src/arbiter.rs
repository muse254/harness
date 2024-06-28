#!(cfg(not(feature = "__harness-build")))

//! This is is where the harness program is loaded at compile time, we create the arbiter to arbiter operations of the harness program.

use std::cell::RefCell;
use std::io::prelude::*;

use candid::Nat;
use ic_cdk::{api::management_canister::http_request::HttpResponse, query};

use crate::device::Device;

use harness_primitives::{
    http::{get_header, Header, Method, Request},
    program::Program,
};

pub(crate) struct Arbiter {
    devices: Vec<Device>,
    program: Program,
}

impl Arbiter {
    pub(crate) fn new() -> Result<Self, String> {
        // read the harness code bytes to memory at compile time
        let program = harness_macros::get_program!();

        Ok(Self {
            devices: Vec::new(),
            program,
        })
    }

    pub(crate) fn add_device(&mut self, device: Device) {
        self.devices.push(device);
    }
}

thread_local! {
    static DEVICES: RefCell<Vec<Device>> = RefCell::new(Vec::new());
}

#[query]
fn http_request(req: Request) -> HttpResponse {
    let method = Method::try_from(req.method.as_str()).unwrap(); //todo

    match (method, req.path.as_str()) {
        (Method::GET, "/program") => {
            // register the device with the arbiter
            match get_header(&Header::HarnessNodeUrl.to_string(), &req.headers) {
                Some(url) => {
                    DEVICES.with(|devices| {
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
                        body: vec![], //HARNESS_WASM.to_vec(),
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
