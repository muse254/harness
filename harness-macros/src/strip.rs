use proc_macro::TokenStream;
use syn::ItemFn;

struct Strip {
    _values: Vec<String>,
}

impl syn::parse::Parse for Strip {
    fn parse(_input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Strip { _values: vec![] }) // todo
    }
}

pub(crate) fn stripper(_attr: TokenStream, func: ItemFn) -> ItemFn {
    // match syn::parse::<Strip>(attr) {
    //     Ok(strip) => {
    //         let mut preserve = Vec::new();
    //         func.attrs.iter().for_each(|attr| {
    //             if strip.values.iter().any(|v| attr.path().eq(v)) {
    //                 // return;
    //             }
    //             //  strip.values.contains(attr.path());

    //             if let Some(ident) = attr.path.get_ident() {
    //                 if strip.values.contains(&ident.to_string()) {
    //                     *attr = Default::default();
    //                 }
    //             }
    //         });

    //         let mut func = func;

    //         func.attrs = preserve;
    //         func
    //     }
    //     Err(_) => {
    //         return func;
    //     }
    // }
    let mut func = func;
    func.attrs = Vec::new();
    func
}
