use proc_macro2::TokenStream;
use quote::quote;

// This struct is a wrapper around the method to be used in the schema. Allowing for type
// information to be processed at compile time.
pub(crate) struct SchemaMethodWrapper {
    pub name: String,
    pub args: Vec<TokenStream>,
    pub rets: Vec<TokenStream>,
}

impl SchemaMethodWrapper {
    pub fn create_method(self) -> TokenStream {
        let name = self.name;
        let args = self.args;
        let rets = self.rets;

        quote! {
            ::harness_cdk::Method {
                name: #name,
                args: vec![#(#args),*],
                rets: vec![#(#rets),*],
            }
        }
    }
}
