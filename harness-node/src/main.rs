use core::panic;

use tokio::io::BufStream;
use url::Url;

use harness_primitives::{http::parse_request, HarnessOs};

mod network;
use network::start_server;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (port, listener) = start_server().await?;
    println!("connect on port '{port}'"); // todo: do telemetry properly

    let mut server = network::NodeServer {
        harness_os: HarnessOs::default(),
    };

    loop {
        let (stream, _) = listener.accept().await?;
        let mut stream = BufStream::new(stream);

        // the wasm engine is single-threaded, todo: allow async ops by replication
        // also multiple programs so really should not be sequential here
        match parse_request(&mut stream).await {
            Ok(req) => {
                let resp = server.handler(req).await.unwrap_or_else(|e| e.into());
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
