use std::{
    io::Cursor,
    net::{Ipv4Addr, SocketAddrV4},
};

use ic_agent::{export::Principal, Agent};
use tokio::{net::TcpListener, sync::Mutex};

use harness_primitives::{
    error::{Error, Result as HarnessResult},
    http::{get_header, Header, Method, PullProgram, Request, Response},
    HarnessOs,
};

#[derive(Default)]
pub struct NodeServer {
    harness_os: HarnessOs,
    lock_: Mutex<()>,
}

impl NodeServer {
    pub async fn handler(&mut self, req: Request) -> HarnessResult<Response<Cursor<Vec<u8>>>> {
        match (Method::try_from(req.method.as_str())?, req.path.as_str()) {
            (Method::GET, "/hello") => Ok(Response::hello()),

            // TODO: rework with tests and proper implementation
            (Method::POST, "/program") => {
                let program = match serde_json::from_slice::<PullProgram>(&req.data) {
                    Ok(programs) => programs,
                    Err(err) => {
                        return Ok(Response {
                            status_code: 400,
                            data: Cursor::new(err.to_string().into_bytes()),
                            headers: vec![],
                        })
                    }
                };

                let agent = Agent::builder().with_url(program.url).build().unwrap();
                agent.fetch_root_key().await.unwrap();

                // get the program code, calling the IC canister
                let response = agent
                    .query(
                        &Principal::from_text(program.canister_id).unwrap(),
                        "get_program_code",
                    )
                    .await
                    .unwrap();

                let _unused = self.lock_.lock().await;
                self.harness_os
                    .add_program(program.program_id.parse()?, &response)?;

                Ok(Response {
                    status_code: 202,
                    data: Cursor::new(vec![]),
                    headers: vec![],
                })
            }

            (Method::POST, "/procedure") => {
                let program_id = match get_header(&Header::ProgramId.to_string(), &req.headers) {
                    Some(program_id) => program_id,
                    None => {
                        return Ok(Response {
                            status_code: 400,
                            data: Cursor::new(
                                "Program-Identifier header could not be retrieved".into(),
                            ),
                            headers: vec![],
                        })
                    }
                };

                let procedure = match get_header(&Header::ProgramProc.to_string(), &req.headers) {
                    Some(procedure) => procedure,
                    None => {
                        return Ok(Response {
                            status_code: 400,
                            data: Cursor::new(
                                "Program-Procedure header could not be retrieved".into(),
                            ),
                            headers: vec![],
                        })
                    }
                };

                let _unused = self.lock_.lock().await;
                match self
                    .harness_os
                    .call_operation(&program_id.parse()?, &procedure, &req.data)
                {
                    Ok(res) => Ok(Response {
                        status_code: 200,
                        data: Cursor::new(res),
                        headers: vec![],
                    }),
                    Err(err) => {
                        eprintln!("{err}");
                        Ok(Response {
                            status_code: 400,
                            data: Cursor::new(err.to_string().into_bytes()),
                            headers: vec![],
                        })
                    }
                }
            }

            (Method::DELETE, "/program") => {
                let program_id =
                    get_header(&Header::ProgramId.to_string(), &req.headers).ok_or(Error::IO {
                        message: "Program-Identifier header could not be retrieved".to_string(),
                        inner: None,
                    })?;

                let _unused = self.lock_.lock().await;
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
pub async fn start_server() -> HarnessResult<(u16, TcpListener)> {
    let port_ = std::env::var("HARNESS_PORT").unwrap_or_else(|_| String::from("0"));
    let listener = TcpListener::bind(SocketAddrV4::new(
        Ipv4Addr::LOCALHOST,
        port_.parse::<u16>().unwrap(),
    ))
    .await
    .map_err(|err| Error::io("failed to bind to a port", err.into()))?;

    let port = listener
        .local_addr()
        .map_err(|err| Error::io("failed to get local address for port", err.into()))?
        .port();

    Ok((port, listener))
}
