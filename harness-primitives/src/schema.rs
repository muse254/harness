//! This module defines the schema of a harness program.

use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// This represents the schema of a harness program.
#[derive(Clone)]
pub struct Schema {
    pub version: Option<String>,
    pub program: Option<String>,
    pub services: Vec<Method>,
}

impl quote::ToTokens for Schema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let version = &self.version;
        let program = &self.program;
        let services = &self.services;

        tokens.extend(quote! {
            Schema {
                version: #version,
                program: #program,
                services: vec![#(#services),*],
            }
        });
    }
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

    pub fn add_service(&mut self, method: Method) {
        self.services.push(method);
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

/// This defines a harness service
///
/// We employ a hack on the args and ret fields to process type information. We store the `CandidType` as a list of
/// token streams to be evaluated later.
#[derive(Clone)]
pub struct Method {
    pub name: String,
    pub args: Vec<String>,
    pub rets: Vec<String>,
}

impl quote::ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(&self.name, proc_macro2::Span::call_site());
        let args = &self.args;
        let rets = &self.rets;

        tokens.extend(quote! {
            Method {
                name: String::from(#name),
                args: vec![#(#args),*],
                rets: vec![#(#rets),*],
            }
        });
    }
}
