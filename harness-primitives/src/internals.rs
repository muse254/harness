//! This module contains the internal data structures used by the harness sdk.
use proc_macro2::TokenStream;

/// This is the schema for a harness program, it is not intended to be used directly by the crate
/// consumer, but rather by the `harness` macro to generate the necessary code.
///
/// Ok to access once it's present in the [`Program`](crate::Program) struct.
#[derive(serde::Deserialize, Clone)]
pub struct Schema {
    pub version: String,
    pub program: String,
    pub services: Vec<Service>,
}

/// This is the intermediate schema that is used to generate the code for the harness program. It
/// contains the same information as the `Schema` struct, but in a format that is easier to work with
/// when generating code.
#[derive(Clone)]
pub struct IntermediateSchema {
    pub version: String,
    pub program: String,
    pub services: Vec<IntermediateService>,
}

impl From<Schema> for IntermediateSchema {
    fn from(schema: Schema) -> Self {
        IntermediateSchema {
            version: schema.version,
            program: schema.program,
            services: schema
                .services
                .iter()
                .map(|service| IntermediateService::from(service.clone()))
                .collect(),
        }
    }
}

/// This is the schema for a harness service, it is not intended to be used directly by the crate
/// consumer, but rather by the `harness` macro to generate the necessary code.
///
/// Ok to access once it's present in the [`Program`](crate::Program) struct.
#[derive(serde::Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub args: Vec<String>,
    pub rets: String,
}

/// This is the intermediate schema that is used to generate the code for the harness service. It
/// contains the same information as the `Service` struct, but in a format that is easier to work with
/// when generating code.
#[derive(Clone)]
pub struct IntermediateService {
    pub name: String,
    pub args: Vec<TokenStream>,
    pub rets: TokenStream,
}

impl From<Service> for IntermediateService {
    fn from(service: Service) -> Self {
        IntermediateService {
            name: service.name,
            args: service
                .args
                .iter()
                .map(|arg| {
                    arg.parse()
                        .expect("failed to parse argument as token stream")
                })
                .collect(),
            rets: service
                .rets
                .parse()
                .expect("Failed to parse return type as token stream"),
        }
    }
}
