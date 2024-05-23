use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    io::Cursor,
};

use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
};

use crate::error::{self, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub data: Vec<u8>,
}

pub struct Response<T: AsyncRead + Unpin> {
    pub status: Status,
    pub headers: HashMap<String, String>,
    pub data: T,
}

impl<T: AsyncRead + Unpin> From<error::Error> for Response<T> {
    fn from(value: error::Error) -> Self {
        todo!()
    }
}

// impl From<serde_json::Error> for Error {
//     fn from(e: serde_json::Error) -> Self {
//         Error::IO {
//             message: e.to_string(),
//             inner: None,
//         }
//     }
// }

impl Response<Cursor<Vec<u8>>> {
    pub fn hello() -> Self {
        let data = "Hello, World!".as_bytes();
        Self {
            status: Status::Ok,
            headers: HashMap::from([("Content-Length".to_string(), data.len().to_string())]),
            data: Cursor::new(data.to_vec()),
        }
    }

    pub fn from_bytes(status: Status, data: &[u8]) -> Self {
        let string = String::from;
        let headers = HashMap::from([
            (string("Content-Type"), string("application/octet-stream")),
            (string("Content-Length"), data.len().to_string()),
        ]);

        Self {
            status,
            headers,
            data: Cursor::new(data.to_vec()),
        }
    }
}

impl<T: AsyncRead + Unpin> Response<T> {
    pub fn status_and_headers(&self) -> String {
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\r\n");

        format!("HTTP/1.1 {}\r\n{headers}\r\n\r\n", self.status)
    }

    pub async fn write<S: AsyncWrite + Unpin>(mut self, stream: &mut S) -> anyhow::Result<()> {
        stream
            .write_all(self.status_and_headers().as_bytes())
            .await?;

        tokio::io::copy(&mut self.data, stream).await?;

        Ok(())
    }
}

pub enum Header {
    /// The program identifier
    ProgramId,
    /// The procedure to call into
    ProgramProc,
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Header::ProgramId => write!(f, "Program-Identifier"),
            Header::ProgramProc => write!(f, "Program-Procedure"),
        }
    }
}

pub enum Status {
    Ok,
    Created,
    BadRequest,
    NotFound,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Ok => write!(f, "200 OK"),
            Status::Created => write!(f, "201 Created"),
            Status::BadRequest => write!(f, "400 Bad Request"),
            Status::NotFound => write!(f, "404 Not Found"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    DELETE,
    HEAD,
}

impl TryFrom<&str> for Method {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "DELETE" => Ok(Method::DELETE),
            "HEAD" => Ok(Method::HEAD),
            m => Err(Error::IO {
                message: format!("unsupported method: {m}").to_string(),
                inner: None,
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

    let mut headers: HashMap<String, String> = HashMap::new();
    let mut data = Vec::new();
    loop {
        line_buffer.clear();
        stream.read_line(&mut line_buffer).await?;
        if line_buffer.is_empty() || line_buffer == "\n" || line_buffer == "\r\n" {
            // fixme: rework this to not depend on content-length. Using `read_to_end` has blocking issues
            if let Some(mut length) = headers.get("Content-Length").cloned() {
                length.retain(|c| !c.is_whitespace() && !c.eq(&'\r') && !c.eq(&'\n'));
                let length = u32::from_str_radix(&length, 10).map_err(|e| Error::IO {
                    message: "Content-Length using unexpected format".to_string(),
                    inner: Some(e.into()),
                })? as usize;

                data = vec![0; length];
                stream.read_exact(&mut data).await?;
            }

            break;
        }

        let mut comps = line_buffer.split(":");
        let key = comps
            .next()
            .ok_or(Error::io::<anyhow::Error>("missing header name", None))?
            .to_string();

        let value = comps
            .next()
            .ok_or(Error::io::<anyhow::Error>("missing header value", None))?
            .to_string();

        headers.insert(key, value);
    }

    let req = Request {
        method,
        path,
        headers,
        data,
    };

    Ok(req)
}
