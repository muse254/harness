use core::panic;
use std::str::FromStr as _;

use tokio::io::BufStream;
use url::Url;

use harness_primitives::{http::parse_request, program::ProgramId, HarnessOs};

mod network;
use network::start_server;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // https://CANISTER_ID.ic0.app/sample-asset.txt
    let canister_id = std::env::var("CANISTER_ID")?;
    let canister_network = std::env::var("CANISTER_NETWORK")?; // Sample: ic0.app
    let canister_url = Url::parse(&format!(
        "https://{}.{}/harness_code.wasm",
        canister_id, canister_network
    ))?;

    let res = reqwest::get(canister_url).await?;
    if !res.status().is_success() {
        panic!("failed to fetch wasm: {res:?}");
    }

    let wasm = res.bytes().await?;

    let (port, listener) = start_server().await?;
    println!("connect on port '{port}'"); // todo: do telemetry properly

    let mut server = network::NodeServer {
        harness_os: HarnessOs::new(ProgramId::from_str("res_from_query").expect(""), &wasm)?, // todo
    };

    loop {
        let (stream, _) = listener.accept().await?;
        let mut stream = BufStream::new(stream);

        // the wasm engine is single-threaded, todo: allow async ops by replication
        // also multiple programs so really should not be sequential here
        match parse_request(&mut stream).await {
            Ok(req) => {
                let resp = server.handler(req).unwrap_or_else(|e| e.into());
                if let Err(err) = resp.write(&mut stream).await {
                    println!("{err}")
                }
            }
            Err(err) => {
                eprintln!("{err}")
            }
        }
    }
}
