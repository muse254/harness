use candid::{candid_method, Deserialize};
use ic_cdk::update;

#[derive(candid::CandidType, Deserialize)]
struct Request {}

#[derive(candid::CandidType)]
struct Response {}

#[update]
#[candid_method(update)]
async fn register_device(_req: Request) -> Response {
    todo!()
}
