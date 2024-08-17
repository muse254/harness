use std::io::prelude::*;

use candid::{Decode, Encode};
use ic_agent::AgentError;

use harness_node::{new_node_server, IcpAgent};
use harness_primitives::{
    http::{Header, HeaderField, PullProgram, Request},
    program::ProgramId,
    HarnessOs,
};

const HELLO_BIN: &[u8] = include_bytes!("../../assets/sample_harness_code.wasm");

pub struct IcpAgentMock;

impl IcpAgent for IcpAgentMock {
    async fn get_program_code(
        &self,
        _: &str,
        _: &str,
    ) -> core::result::Result<Vec<u8>, AgentError> {
        Ok(HELLO_BIN.to_vec())
    }
}

#[tokio::test]
async fn test_hello() {
    let program_id = "hello".parse::<ProgramId>().unwrap();
    let harness_os = HarnessOs::new("hello".parse().unwrap(), HELLO_BIN)
        .await
        .unwrap();
    let result = harness_os
        .call_operation(
            &program_id,
            "hello",
            &Encode!(&String::from("World")).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(Decode!(&result, String).unwrap(), "Hello, World!");
}

#[tokio::test]
async fn test_with_node_impl() {
    let mut node_server = new_node_server(IcpAgentMock);

    // program registration to the device
    {
        let payload = serde_json::to_string(&PullProgram {
            canister_id: "hello".to_string(),
            program_id: "hello".to_string(),
            url: "http://localhost:8000".to_string(),
        })
        .unwrap();

        let resp = node_server
            .handler(Request {
                method: "POST".to_string(),
                path: "/program".to_string(),
                headers: vec![],
                data: payload.as_bytes().to_vec(),
            })
            .await
            .unwrap();

        // response should be created status
        assert_eq!(resp.status_code, 202);
    }

    // procedure invocation for the loaded program
    {
        let resp = node_server
            .handler(Request {
                method: "POST".to_string(),
                path: "/procedure".to_string(),
                headers: vec![
                    HeaderField(Header::ProgramId.to_string(), "hello".to_string()),
                    HeaderField(Header::ProgramProc.to_string(), "hello".to_string()),
                ],
                data: Encode!(&String::from("World")).unwrap(),
            })
            .await
            .unwrap();

        // status ok
        assert_eq!(resp.status_code, 200);

        let mut data = resp.data;
        let mut buf = Vec::new();
        data.read_to_end(&mut buf).unwrap();

        println!("{:?}", buf);

        // response should be "Hello, World!"
        assert_eq!(Decode!(&buf, String).unwrap(), "Hello, World!");
    }
}
