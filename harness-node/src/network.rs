//! This holds the networking code for a node. TODO/FIXME!?: if target is wasm use fetch api, else continue in tokio

use std::{
    io::Cursor,
    net::{Ipv4Addr, SocketAddrV4},
};

use tokio::net::TcpListener;

use harness_primitives::{
    error::{Error, Result},
    http::{get_header, Header, Method, Request, Response},
    HarnessOs,
};

#[derive(Default)]
pub(crate) struct NodeServer {
    pub harness_os: HarnessOs,
}

impl NodeServer {
    pub fn handler(&mut self, req: Request) -> Result<Response<Cursor<Vec<u8>>>> {
        match (Method::try_from(req.method.as_str())?, req.path.as_str()) {
            (Method::GET, "/hello") => Ok(Response::hello()),

            (Method::POST, "/program") => {
                let program_id =
                    get_header(&Header::ProgramId.to_string(), &req.headers).ok_or(Error::IO {
                        message: "Program-Identifier header could not be retrieved".to_string(),
                        inner: None,
                    })?;

                self.harness_os
                    .add_program(program_id.parse()?, &req.data)?;

                Ok(Response {
                    status_code: 202,
                    data: Cursor::new(vec![]),
                    headers: vec![],
                })
            }

            (Method::POST, "/procedure") => {
                let program_id =
                    get_header(&Header::ProgramId.to_string(), &req.headers).ok_or(Error::IO {
                        message: "Program-Identifier header could not be retrieved".to_string(),
                        inner: None,
                    })?;

                let procedure = get_header(&Header::ProgramProc.to_string(), &req.headers).ok_or(
                    Error::IO {
                        message: "Program-Procedure header could not be retrieved".to_string(),
                        inner: None,
                    },
                )?;

                let res =
                    self.harness_os
                        .call_operation(&program_id.parse()?, &procedure, &req.data)?;

                Ok(Response {
                    status_code: 200,
                    data: Cursor::new(res),
                    headers: vec![],
                })
            }

            (Method::DELETE, "/program") => {
                let program_id =
                    get_header(&Header::ProgramId.to_string(), &req.headers).ok_or(Error::IO {
                        message: "Program-Identifier header could not be retrieved".to_string(),
                        inner: None,
                    })?;

                self.harness_os.remove_program(&program_id.parse()?);

                Ok(Response {
                    status_code: 204,
                    data: Cursor::new(vec![]),
                    headers: vec![],
                })
            }

            (_, _) => Ok(Response {
                status_code: 404,
                data: Cursor::new(vec![]),
                headers: vec![],
            }),
        }
    }
}

/// Starts a server on a random port and returns the port and the listener.
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
