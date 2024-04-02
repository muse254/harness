pub(crate) const HARNESS_SCHEMA: Option<&'static str> = option_env!("HARNESS_SCHEMA");

/// This struct represents the schema of a harness program.
pub(crate) struct Schema {
    version: Option<String>,
    program: Option<String>,
    services: Vec<Services>,
}

/// This struct identifies a service of the harness program.
struct Service {
    name: String,
}

impl Schema {
    fn new(functions: &[(String, String)]) -> Self {
        let version = std::env::var("CARGO_PKG_VERSION");
        let program = std::env::var("CARGO_PKG_NAME");

        Self {
            version,
            program,
            services: functions
                .iter()
                .map(|(name, doc)| Service {
                    name: name.clone(),
                    doc: doc.clone(),
                })
                .collect(),
        }
    }
}

pub(crate) fn generate_schema() -> Schema {}
