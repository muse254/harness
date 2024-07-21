//! This is is where the harness program is loaded at compile time, we create the arbiter to arbiter operations of the harness program.

use candid::Nat;
use ic_cdk::api::management_canister::http_request::{HttpHeader, HttpResponse, TransformArgs};

use harness_macros::get_program;
use harness_primitives::program::{Program, ProgramId};

pub struct Arbiter {
    // The collection of device urls that have been registered with the arbiter.
    devices: Vec<String>,
    // The harness program that is loaded into the arbiter at compile time.
    program: Program,
}

impl Arbiter {
    pub fn new(code: &'static [u8]) -> Self {
        let mut program = get_program!();
        if !code.is_empty() {
            program.wasm = Some(code);
        }

        Self {
            devices: Vec::new(),
            program,
        }
    }

    pub fn add_device(&mut self, device: String) {
        self.devices.push(device);
    }

    pub const fn get_program_code(&self) -> Option<&'static [u8]> {
        self.program.wasm
    }

    pub fn get_program_id(&self) -> ProgramId {
        self.program.id.clone()
    }
}

pub fn get_next_device(
    counter: &std::cell::Cell<usize>,
    arbiter: &std::cell::RefCell<Arbiter>,
) -> String {
    let n_device_val = counter.get();
    if n_device_val < arbiter.borrow().devices.len() {
        counter.set(n_device_val + 1);
    } else {
        counter.set(0);
    }

    arbiter.borrow().devices[n_device_val].clone()
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

// #[ic_cdk::query]
// fn get_program_code() -> Vec<u8> {
//     Arbiter::new(&[]).get_program_code().unwrap().to_vec()
// }
