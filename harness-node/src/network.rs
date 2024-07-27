//! This holds the networking code for a node. TODO/FIXME!?: if target is wasm use fetch api, else continue in tokio
use std::io::Cursor;

use ic_agent::{export::Principal, Agent};

use harness_primitives::{
    error::{Error, Result},
    http::{get_header, Header, Method, PoolPrograms, Request, Response},
    HarnessOs,
};

#[derive(Default)]
pub struct NodeServer<'a> {
    pub harness_os: HarnessOs<'a>,
}

impl<'a> NodeServer<'a> {
    pub async fn handler(&mut self, req: Request) -> Result<Response<Cursor<Vec<u8>>>> {
        match (Method::try_from(req.method.as_str())?, req.path.as_str()) {
            (Method::GET, "/hello") => Ok(Response::hello()),

            // TODO: rework with tests and proper implementation
            (Method::POST, "/program") => {
                let programs = match serde_json::from_slice::<PoolPrograms>(&req.data) {
                    Ok(programs) => programs,
                    Err(err) => {
                        return Ok(Response {
                            status_code: 400,
                            data: Cursor::new(err.to_string().into_bytes()),
                            headers: vec![],
                        })
                    }
                };

                let agent = Agent::builder().build().unwrap();
                // Only do the following call when not contacting the IC main net (e.g. a local emulator).
                // This is important as the main net public key is static and a rogue network could return
                // a different key.
                // If you know the root key ahead of time, you can use `agent.set_root_key(root_key);`.
                agent.fetch_root_key().await.unwrap();

                // get the program code, calling the IC canister
                for harness_canister in programs.ic_canisters {
                    let effective_canister_id =
                        Principal::from_text(harness_canister.canister_id).unwrap();
                    let response = agent
                        .query(&effective_canister_id, "account_balance")
                        .await
                        .unwrap();

                    self.harness_os
                        .add_program(harness_canister.program_id.parse()?, &response)
                        .await?;
                }

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

                match self
                    .harness_os
                    .call_operation(&program_id.parse()?, &procedure, &req.data)
                    .await
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
