use tokio::io::BufStream;

mod network;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let harness_network = std::env::var("HARNESS_NETWORK")?;

    // before starting the node's server, we ping the harness network for loadable programs

    let (port, listener) = network::start_server().await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        let mut stream = BufStream::new(stream);

        tokio::spawn(async move {
            match network::parse_request(&mut stream).await {
                Ok(req) => {
                    // todo
                    println!("incoming request")
                }
                Err(err) => {
                    eprintln!("{err}")
                }
            }
        });
    }
}
