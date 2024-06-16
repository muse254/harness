use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    io::Cursor,
};

use candid::{CandidType, Deserialize};
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
};

use crate::error::{self, Error, Result};

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HeaderField(pub String, pub String);

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: Vec<HeaderField>,
    pub data: Vec<u8>,
}

#[derive(CandidType)]
pub struct Response<T: AsyncRead + Unpin> {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub data: T,
}

impl From<error::Error> for Response<Cursor<Vec<u8>>> {
    fn from(value: error::Error) -> Self {
        let status_code = match &value {
            Error::IO { .. } => 400,
            Error::Internal { .. } | Error::Custom(_) => 500,
        };

        let val_str = value.to_string();
        Self {
            status_code: status_code as u16,
            headers: vec![HeaderField(
                "Content-Type".to_string(),
                "text/plain".to_string(),
            )],
            data: Cursor::new(val_str.as_bytes().to_vec().into()),
        }
    }
}

impl Response<Cursor<Vec<u8>>> {
    pub fn hello() -> Self {
        let data = "Hello, World!".as_bytes();

        Self {
            status_code: 202,
            headers: vec![HeaderField(
                "Content-Type".to_string(),
                "text/plain".to_string(),
            )],
            data: Cursor::new(data.to_vec()),
        }
    }

    pub fn from_bytes(status_code: u16, data: &[u8]) -> Self {
        let string = String::from;
        let headers = vec![
            HeaderField(string("Content-Type"), string("application/octet-stream")),
            HeaderField(string("Content-Length"), data.len().to_string()),
        ];

        Self {
            status_code,
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
            .map(|header| format!("{}: {}", header.0, header.1))
            .collect::<Vec<_>>()
            .join("\r\n");

        format!("HTTP/1.1 {}\r\n{headers}\r\n\r\n", self.status_code)
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
    /// The URL of the harness node
    HarnessNodeUrl,
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Header::ProgramId => write!(f, "Program-Identifier"),
            Header::ProgramProc => write!(f, "Program-Procedure"),
            Header::HarnessNodeUrl => write!(f, "Harness-Node-Url"),
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
        .ok_or(Error::io::<anyhow::Error>("missing method", None))?
        .to_string();

    let path = parts
        .next()
        .ok_or(Error::io::<anyhow::Error>("missing path", None))?
        .to_string();

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
        headers: headers
            .into_iter()
            .map(|(k, v)| HeaderField(k, v))
            .collect::<Vec<HeaderField>>(),
        data,
    };

    Ok(req)
}

pub fn get_header(header_key: &str, headers: &[HeaderField]) -> Option<String> {
    headers
        .iter()
        .find(|header| header.0.to_lowercase() == header_key.to_lowercase())
        .map(|v| v.1.clone())
}
