use std::fmt::{Display, Formatter};
#[cfg(feature = "wasm-ext")]
use std::{collections::HashMap, io::Cursor};

use candid::{CandidType, Deserialize};

#[cfg(feature = "wasm-ext")]
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
};

use crate::error::Error;

// This struct is legacy code and is not really used in the code.
#[derive(serde::Serialize, serde:: Deserialize)]
pub struct Context {
    pub bucket_start_time_index: usize,
    pub closing_price_index: usize,
}
#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HeaderField(pub String, pub String);

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: Vec<HeaderField>,
    pub data: Vec<u8>,
}

#[cfg(feature = "wasm-ext")]
#[derive(CandidType)]
pub struct Response<T: AsyncRead + Unpin> {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub data: T,
}

#[cfg(feature = "wasm-ext")]
impl From<crate::error::Error> for Response<Cursor<Vec<u8>>> {
    fn from(value: crate::error::Error) -> Self {
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
            data: Cursor::new(val_str.as_bytes().to_vec()),
        }
    }
}

#[cfg(feature = "wasm-ext")]
impl Response<Cursor<Vec<u8>>> {
    pub fn hello() -> Self {
        let data = b"Hello, World!";

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

#[cfg(feature = "wasm-ext")]
impl<T: AsyncRead + Unpin + Send> Response<T> {
    pub fn status_and_headers(&self) -> String {
        let headers = self
            .headers
            .iter()
            .map(|header| format!("{}: {}", header.0, header.1))
            .collect::<Vec<_>>()
            .join("\r\n");

        format!("HTTP/1.1 {}\r\n{headers}\r\n\r\n", self.status_code)
    }

    pub async fn write<S: AsyncWrite + Unpin + Send>(
        mut self,
        stream: &mut S,
    ) -> anyhow::Result<()> {
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
    DeviceUrl,
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProgramId => write!(f, "Program-Identifier"),
            Self::ProgramProc => write!(f, "Program-Procedure"),
            Self::DeviceUrl => write!(f, "Device-Url"),
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
            Self::Ok => write!(f, "200 OK"),
            Self::Created => write!(f, "201 Created"),
            Self::BadRequest => write!(f, "400 Bad Request"),
            Self::NotFound => write!(f, "404 Not Found"),
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
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "DELETE" => Ok(Self::DELETE),
            "HEAD" => Ok(Self::HEAD),
            m => Err(Error::IO {
                message: format!("unsupported method: {m}"),
                inner: None,
            }),
        }
    }
}

#[cfg(feature = "wasm-ext")]
pub async fn parse_request<T: AsyncBufRead + Unpin + Send>(
    mut stream: T,
) -> crate::error::Result<Request> {
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
                let length = length.parse::<u32>().map_err(|e| Error::IO {
                    message: "Content-Length using unexpected format".to_string(),
                    inner: Some(e.into()),
                })? as usize;

                data = Vec::with_capacity(length);
                stream.read_buf(&mut data).await?;
            }

            break;
        }

        let mut comps = line_buffer.split(':');
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
