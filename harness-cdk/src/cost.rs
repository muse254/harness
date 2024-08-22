use ic_cdk::api::management_canister::http_request::{http_request, CanisterHttpRequestArgument};

pub fn calculate_cycles(request: &CanisterHttpRequestArgument) -> u64 {
    //  request.max_response_bytes
    todo!()
}

#[test]
fn test_calculate_cycles() {
    let request = CanisterHttpRequestArgument::default();
    let cycles = calculate_cycles(&request);

    // http_request(arg, cycles)

    assert_eq!(cycles, 0);
}
