use harness_node::{new_node_server, start_server, IcpAgentImpl};
use tokio::io::BufStream;

use harness_primitives::http::parse_request;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (port, listener) = start_server().await?;
    println!("connect on port '{port}'"); // todo: do telemetry properly

    let mut server = new_node_server(IcpAgentImpl);
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
