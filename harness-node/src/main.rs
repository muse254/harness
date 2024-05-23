use reqwest;
use tokio::io::BufStream;

use harness_primitives::http::parse_request;

mod network;
use network::start_server;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // let harness_network = std::env::var("HARNESS_NETWORK")?;

    let (port, listener) = start_server().await?;
    println!("connect on port '{port}'"); // todo: do telemetry properly

    let mut server = network::NodeServer::default();

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
