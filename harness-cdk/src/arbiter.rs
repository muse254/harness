//! This is is where the harness program is loaded at compile time, we create the arbiter to arbiter operations of the harness program.

use std::cell::{Cell, RefCell};

use candid::Nat;
use ic_cdk::api::management_canister::http_request::{HttpHeader, HttpResponse, TransformArgs};

use harness_macros::{get_binary__, get_schema};
use harness_primitives::program::{Program, ProgramId};

struct Arbiter {
    // The collection of device urls that have been registered with the arbiter.
    devices: Vec<String>,
    // The harness program that is loaded into the arbiter at compile time.
    program: Program,
}

#[cfg(not(feature = "__harness_build"))]
thread_local! {
    static NEXT_DEVICE_ID: Cell<usize> = Cell::new(0);// rudimentary round robin scheduling
    static ARBITER: RefCell<Arbiter> = RefCell::new( Arbiter {
        devices: Vec::new(),
        program: Program { schema: get_schema!(), wasm: get_binary__!() },
    });
}

#[cfg(not(feature = "__harness_build"))]
pub struct StateAccessor;

#[cfg(not(feature = "__harness_build"))]
impl StateAccessor {
    pub fn add_device(url: String) {
        ARBITER.with(|arbiter| arbiter.borrow_mut().devices.push(url));
    }

    pub fn get_program_code() -> Vec<u8> {
        ARBITER.with(|arbiter| arbiter.borrow().program.wasm.to_vec())
    }

    pub fn get_program_id() -> ProgramId {
        ARBITER.with(|arbiter| ProgramId::new(arbiter.borrow().program.schema.program.clone()))
    }

    pub fn get_next_device() -> String {
        ARBITER.with(|arbiter| {
            let devices = &arbiter.borrow().devices;
            let next_device_id = NEXT_DEVICE_ID.with(|next_device_id| {
                let id = next_device_id.get();
                next_device_id.set((id + 1) % devices.len());
                id
            });
            devices[next_device_id].clone()
        })
    }
}

// Copied over from example `send_http_post_rust`
// Strips all data that is not needed from the original response.
#[ic_cdk::query]
fn harness_transform(raw: TransformArgs) -> HttpResponse {
    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
        ..Default::default()
    };

    if res.status == Nat::from(200u8) {
        res.body = raw.response.body;
    } else {
        ic_cdk::api::print(format!(
            "Received an error from harness node: err = {:?}",
            raw
        ));
    }
    res
}
