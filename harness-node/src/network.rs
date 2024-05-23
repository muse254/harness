//! This holds the networking code for a node. TODO/FIXME!?: if target is wasm use fetch api, else continue in tokio

use std::{
    collections::HashMap,
    io::Cursor,
    net::{Ipv4Addr, SocketAddrV4},
};

use tokio::net::TcpListener;

use harness_primitives::{
    error::{Error, Result},
    http::{Header, Method, Request, Response, Status},
    HarnessOs,
};

#[derive(Default)]
pub struct NodeServer {
    harness_os: HarnessOs,
}

impl NodeServer {
    pub fn handler(&mut self, req: Request) -> Result<Response<Cursor<Vec<u8>>>> {
        let pattern = (req.method, req.path.as_str());

        match pattern {
            (Method::GET, "/hello") => Ok(Response::hello()),

            (Method::POST, "/program") => {
                let program_id =
                    req.headers
                        .get(&Header::ProgramId.to_string())
                        .ok_or(Error::IO {
                            message: "Program-Identifier header could not be retrieved".to_string(),
                            inner: None,
                        })?;

                self.harness_os
                    .add_program(program_id.parse()?, &req.data)?;

                Ok(Response {
                    status: Status::Created,
                    headers: HashMap::new(),
                    data: Cursor::new(vec![]),
                })
            }

            (Method::POST, "/procedure") => {
                let program_id =
                    req.headers
                        .get(&Header::ProgramId.to_string())
                        .ok_or(Error::IO {
                            message: "Program-Identifier header could not be retrieved".to_string(),
                            inner: None,
                        })?;

                let procedure =
                    req.headers
                        .get(&Header::ProgramProc.to_string())
                        .ok_or(Error::IO {
                            message: "Program-Procedure header could not be retrieved".to_string(),
                            inner: None,
                        })?;

                let res =
                    self.harness_os
                        .call_operation(&program_id.parse()?, &procedure, &req.data)?;

                Ok(Response {
                    data: Cursor::new(res),
                    status: Status::Ok,
                    headers: HashMap::new(), // TODO
                })
            }

            (Method::DELETE, "/program") => {
                let program_id =
                    req.headers
                        .get(&Header::ProgramId.to_string())
                        .ok_or(Error::IO {
                            message: "Program-Identifier header could not be retrieved".to_string(),
                            inner: None,
                        })?;

                self.harness_os.remove_program(&program_id.parse()?);

                Ok(Response {
                    status: Status::Ok,
                    data: Cursor::new(vec![]),
                    headers: HashMap::new(),
                })
            }

            (_, _) => Ok(Response {
                status: Status::NotFound,
                data: Cursor::new(vec![]),
                headers: HashMap::new(),
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
