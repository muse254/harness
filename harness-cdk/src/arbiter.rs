use std::cell::RefCell;

use ic_cdk::api::management_canister::http_request::HttpResponse;

use crate::device::Device;

// FIXME: placeholder
const HARNESS_WASM: &[u8] =
    include_bytes!("../../examples/hello/target/wasm32-unknown-unknown/release/hello_backend.wasm");

pub struct Arbiter {
    devices: Vec<Device>,
}

thread_local! {
    static Devices: RefCell<Vec<Device>> = RefCell::new(Vec::new());
}

type HeaderField = (String, String);

struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<HeaderField>,
    body: Vec<u8>,
}

// struct HttpResponse {
//     status_code: u16,
//     headers: Vec<HeaderField>,
//     body: Vec<u8>,
// }

fn http_request(req: HttpRequest) -> HttpResponse {
    return todo!("http_request");
}

// #[update]
// pub fn register_device(url: String, headers: Vec<(String, String)>) {
//     Devices.with(|devices| {
//         devices.borrow_mut().push(Device {
//             id: ic_cdk::api::caller(),
//             url,
//             headers,
//         })
//     });
// }

// #[update]
// async fn register_device() -> [u8] {
//     return todo!();
// }

// enum Method {
//     CheckHealth,
//     SendCall { input: Vec<u8> },
// }
