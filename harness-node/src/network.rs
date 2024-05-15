use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::net::TcpListener;

use harness_primitives::error::{Error, Result};

pub async fn start_server() -> Result<(u16, TcpListener)> {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|err| Error::io("failed to bind to a port", err.into()))?;

    let port = listener
        .local_addr()
        .map_err(|err| Error::io("failed to get local address for port", err.into()))?
        .port();
    Ok((port, listener))
}

mod http_primitives {
    use std::{collections::HashMap, net::TcpStream};

    use tokio::io::{AsyncBufRead, AsyncBufReadExt};

    use harness_primitives::error::{Error, Result};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Request {
        pub method: Method,
        pub path: String,
        pub headers: HashMap<String, String>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Method {
        GET,
        POST,
        HEAD,
    }

    impl TryFrom<&str> for Method {
        type Error = Error;

        fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
            match value {
                "GET" => Ok(Method::GET),
                "POST" => Ok(Method::POST),
                "HEAD" => Ok(Method::HEAD),
                m => Err(Error::IO {
                    message: format!("unsupported method: {m}").into(),
                    nested: None,
                }),
            }
        }
    }

    pub async fn parse_request<T: AsyncBufRead + Unpin>(mut stream: T) -> Result<Request> {
        let mut line_buffer = String::new();
        stream.read_line(&mut line_buffer).await?;

        let mut parts = line_buffer.split_whitespace();

        let method = parts
            .next()
            .ok_or(Error::io::<anyhow::Error>("missing method", None))
            .and_then(TryFrom::try_from)?;

        let path = parts
            .next()
            .ok_or(Error::io::<anyhow::Error>("missing path", None))?
            .into();

        let mut headers = HashMap::new();
        loop {
            line_buffer.clear();
            stream.read_line(&mut line_buffer).await?;
            if line_buffer.is_empty() || line_buffer == "\n" || line_buffer == "\r\n" {
                break;
            }

            let mut comps = line_buffer.split(":");
            let key = comps
                .next()
                .ok_or(Error::io::<anyhow::Error>("missing header name", None))?
                .into();

            let value = comps
                .next()
                .ok_or(Error::io::<anyhow::Error>("missing header value", None))?
                .into();

            headers.insert(key, value);
        }

        Ok(Request {
            method,
            path,
            headers,
        })
    }
}

pub use http_primitives::{parse_request, Method, Request};
