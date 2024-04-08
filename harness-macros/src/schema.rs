use candid::CandidType;
use syn::{self, Signature};

pub(crate) const HARNESS_SCHEMA: Option<&'static str> = option_env!("HARNESS_SCHEMA");

/// This represents the schema of a harness program.
pub(crate) struct Schema {
    version: Option<String>,
    program: Option<String>,
    services: Vec<Service>,
}

/// This defines a harness service
pub(crate) struct Service {
    name: String,
    //  sig: Signature,
}

impl Schema {
    pub fn new() -> Self {
        let version = std::env::var("CARGO_PKG_VERSION").ok();
        let program = std::env::var("CARGO_PKG_NAME").ok();

        Self {
            version,
            program,
            services: vec![],
        }
    }

    pub fn add_service(mut self, name: String, item: &syn::Signature) -> Self {
        let service = Service {
            name,
            //  sig: item.clone(),
        };
        self.services.push(service);
        self
    }
}
