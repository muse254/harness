use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::{io::BufStream, net::TcpListener};

use harness_primitives::{error::Error, http::parse_request, HarnessOs};

mod network;

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

/// Starts a server on a random port and returns the port and the listener.
async fn start_server() -> harness_primitives::error::Result<(u16, TcpListener)> {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|err| Error::io("failed to bind to a port", err.into()))?;

    let port = match std::env::var("HARNESS_PORT") {
        Ok(port) => port.parse::<u16>().unwrap(),
        Err(_) => listener
            .local_addr()
            .map_err(|err| Error::io("failed to get local address for port", err.into()))?
            .port(),
    };

    Ok((port, listener))
}
