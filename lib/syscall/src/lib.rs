extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::{format_ident, quote};
use syn::{FnArg, ReturnType, Pat};
use syn::Type;
use quote::ToTokens;

#[derive(Eq, PartialEq, Copy, Clone)]
enum InternalType {
    U16,
    Handle,
    U32,
    Pointer,
    HeapByteString,
    Ignored,
}

fn get_internal_type_size(ty: InternalType) -> u32 {
    match ty {
        InternalType::U16 | InternalType::Handle => 2,
        InternalType::U32 | InternalType::Pointer | InternalType::HeapByteString => 4,
        InternalType::Ignored => 0,
    }
}

fn get_internal_type_from_str(str: &str) -> InternalType {
    match str {
        "Handle" => InternalType::Handle,
        "u16" | "Result < u16, EmulatorError >" => InternalType::U16,
        "u32" | "Result < u32, EmulatorError >" => InternalType::U32,
        "Pointer" => InternalType::Pointer,
        "HeapByteString" => InternalType::HeapByteString,
        "EmulatorAccessor" => InternalType::Ignored,
        other => panic!("not supported: {}", other),
    }
}

fn get_internal_type_from_type(ty: &Type) -> InternalType {
    get_internal_type_from_str(ty.to_token_stream().to_string().as_str())
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn api_function(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("expected fn"),
    };

    let fn_self = fn_item.sig.inputs.iter().next().unwrap();
    let fn_name = &fn_item.sig.ident;
    let fn_return = &fn_item.sig.output;

    assert_eq!(fn_return.to_token_stream().to_string(), "-> Result < ReturnValue, EmulatorError >");

    let mut param_reading_code = Vec::new();
    let mut params = Vec::new();
    let mut argument_offset = 0;
    for input in fn_item.sig.inputs.iter().rev() {
        match input {
            FnArg::Typed(typed) => {
                let identifier = match &*typed.pat {
                    Pat::Ident(ident) => &ident.ident,
                    _ => panic!("unexpected case"),
                };

                let internal_type = get_internal_type_from_type(&typed.ty);
                let internal_type_size = get_internal_type_size(internal_type);

                if internal_type != InternalType::Ignored {
                    let code = match internal_type {
                        InternalType::Handle => quote! {
                            let #identifier = accessor.word_argument(#argument_offset)?.into();
                        },
                        InternalType::U16 => quote! {
                            let #identifier = accessor.word_argument(#argument_offset)?;
                        },
                        InternalType::U32 => quote! {
                            let #identifier = accessor.dword_argument(#argument_offset)?;
                        },
                        InternalType::Pointer => quote! {
                            let #identifier = accessor.pointer_argument(#argument_offset)?.into();
                        },
                        InternalType::HeapByteString => quote! {
                            let tmp_pointer = accessor.pointer_argument(#argument_offset)?;
                            let #identifier = accessor.clone_string(tmp_pointer)?;
                        },
                        _ => panic!("unexpected case"),
                    };

                    param_reading_code.push(code);

                    argument_offset += internal_type_size / 2;
                }

                params.insert(0, identifier);
            },
            _ => {},
        }
    }

    let mut streams: Vec<TokenStream> = Vec::new();

    let fn_name_str = fn_name.to_string().replace('_', " ").to_uppercase();
    let glue_name = format_ident!("__api_{}", fn_name);
    streams.push(quote! {
        fn #glue_name(#fn_self, mut accessor: EmulatorAccessor) -> Result<ReturnValue, EmulatorError> {
            debug!("ENTER {}", #fn_name_str);
            #(#param_reading_code)*
            self.#fn_name(#(#params),*)
        }
    }.into());

    streams.push(input_clone);
    TokenStream::from_iter(streams)
}
