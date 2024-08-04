use std::{
    future::Future,
    io::Cursor,
    net::{Ipv4Addr, SocketAddrV4},
};

use candid::Decode;
use ic_agent::{export::Principal, Agent, AgentError};
use tokio::{net::TcpListener, sync::Mutex};

use harness_primitives::{
    error::{Error, Result as HarnessResult},
    http::{get_header, Header, Method, PullProgram, Request, Response},
    HarnessOs,
};

pub struct NodeServer<T: IcpAgent> {
    harness_os: HarnessOs,
    lock_: Mutex<()>,
    icp_agent: T,
}

pub fn new_node_server<T>(agent: T) -> NodeServer<T>
where
    T: IcpAgent + Send,
{
    NodeServer {
        harness_os: HarnessOs::default(),
        lock_: Mutex::new(()),
        icp_agent: agent,
    }
}

/// This is the interface for the ICP agent that is used to poll the IC canister.
pub trait IcpAgent {
    fn get_program_code(
        &self,
        canister_id: &str,
        icp_url: &str,
    ) -> impl Future<Output = core::result::Result<Vec<u8>, AgentError>>;
}

/// This is the implementation of the ICP agent.
pub struct IcpAgentImpl;

impl IcpAgent for IcpAgentImpl {
    async fn get_program_code(
        &self,
        canister_id: &str,
        icp_url: &str,
    ) -> core::result::Result<Vec<u8>, AgentError> {
        let agent = Agent::builder().with_url(icp_url).build().unwrap();
        agent.fetch_root_key().await?;

        let response = agent
            .query(
                &Principal::from_text(canister_id).unwrap(),
                "get_program_code",
            )
            .with_arg(candid::encode_one(()).unwrap())
            .call()
            .await?;

        Ok(Decode!(&response, Vec<u8>).unwrap())
    }
}

impl<T: IcpAgent> NodeServer<T> {
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

                let response = self
                    .icp_agent
                    .get_program_code(&program.canister_id, &program.url)
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
                println!("Headers: {:?}", req.headers);

                let program_id = match get_header(&Header::ProgramId.to_string(), &req.headers) {
                    Some(program_id) => program_id,
                    None => {
                        return Ok(Response {
                            status_code: 400,
                            data: Cursor::new(
                                "Program-Identifier header could not be retrieved".into(),
                            ),
                            headers: vec![],
                        });
                    }
                };

                let program_id = program_id.trim().to_string();

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

                println!("Program-ids: {:?}", self.harness_os.program_ids());

                match self.harness_os.call_operation(
                    &program_id.parse()?,
                    procedure.trim(),
                    &req.data,
                ) {
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

// curl --header "Content-Type: application/json" \
//  --request POST \
//  --data '{"canister_id":"bkyz2-fmaaa-aaaaa-qaaaq-cai","program_id":"hello","url":"http://localhost:59258"}' \
//   http://localhost:8080/program
