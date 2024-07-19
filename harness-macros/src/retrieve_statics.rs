use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    type Buffer;
}

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync, catch)]
    fn read_file(path: &str) -> Result<Buffer, JsValue>;
}

fn write_to_schema(data: &[u8]) -> Result<(), JsValue> {
    todo!()
}

/// Allows us to retrieve the schema generated for the harness program.
fn get_schema() -> proc_macro2::TokenStream {
    let schema_file_path = match ensure_path_created() {
        Ok(path) => path.join("harness_schema.json"),
        Err(e) => {
            return syn::Error::new(Span::call_site(), e)
                .to_compile_error()
                .into();
        }
    };

    let schema_file = match std::fs::File::open(&schema_file_path) {
        Ok(val) => val,
        Err(err) => {
            if cfg!(feature = "__harness-build") {
                return syn::Error::new(Span::call_site(), "schema file not found, please call after the first build with `--features __harness-build`")
                    .to_compile_error()
                    .into();
            }

            return syn::Error::new(Span::call_site(), format!("schema file not found: {}", err))
                .to_compile_error()
                .into();
        }
    };

    let schema: Schema = serde_json::from_reader(BufReader::new(schema_file))
        .expect("schema file is not valid json");

    let inter_schema = IntermediateSchema::from(schema);

    let mut services = Vec::new();
    for mut service in inter_schema.services {
        service.args.iter_mut().for_each(|arg| {
            to_candid_type(arg);
        });
        to_candid_type(&mut service.rets);

        let name = service.name;
        let rets = service.rets;
        let args = service.args;
        services.push(quote! {
            ::harness_primitives::internals::Service {
                name: String::from(#name),
                args: vec![#(#args),*],
                rets: #rets,
            }
        });
    }

    let version = {
        if let Some(val) = inter_schema.version {
            quote!(Some(String::from(#val)))
        } else {
            quote!(None)
        }
    };

    let program = {
        if let Some(val) = inter_schema.program {
            quote!(Some(String::from(#val)))
        } else {
            quote!(None)
        }
    };

    quote! {
        ::harness_primitives::internals::Schema {
            program: #program,
            version: #version,
            services: vec![#(#services),*],
        }
    }
}

pub(crate) fn get_program_() -> TokenStream {
    // get harness program compiled code
    let wasm_file_path = match ensure_path_created() {
        Ok(path) => path.join("./harness_code.wasm"),
        Err(e) => {
            return syn::Error::new(Span::call_site(), e)
                .to_compile_error()
                .into();
        }
    };

    // only doing fs reads at compile time
    let mut f = match std::fs::File::open(&wasm_file_path) {
        Ok(val) => val,
        Err(err) => {
            if cfg!(feature = "__harness-build") {
                return syn::Error::new(Span::call_site(), "wasm file not found, please call after the first build with `--features __harness-build`")
                    .to_compile_error()
                    .into();
            }

            return syn::Error::new(Span::call_site(), format!("wasm file not found: {}", err))
                .to_compile_error()
                .into();
        }
    };

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .expect("file read to succeed; qed");

    // fixme: how reliable is metadata for exact file size?
    let _bytes = match std::fs::metadata(&wasm_file_path) {
        Ok(val) => val.len() as usize,
        Err(e) => {
            return syn::Error::new(Span::call_site(), e)
                .to_compile_error()
                .into();
        }
    };

    let schema = get_schema();
    TokenStream::from(quote! {
       {
        ::harness_primitives::program::Program {
            id: #schema.program.expect("program value expected").parse().unwrap(), // fixme: allow
            schema: #schema,
            wasm: &[#(#buffer),*],
        }
       }
    })
}
