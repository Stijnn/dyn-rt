use dyn_rt_utils::PLUGIN_DECL_APPENDIX;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat, parse_macro_input};

#[proc_macro_attribute]
pub fn plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| fn_name.to_string());
    let description = "No description provided".to_string();

    let out = quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn _impl_attach_dyn_plugin() -> dyn_rt::utils::Plugin {
            #fn_name();

            dyn_rt::utils::Plugin::new(
                concat!(#name, "\0"),
                concat!(#description, "\0"),
                concat!(env!("CARGO_PKG_VERSION"), "\0")
            )
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn _dyn_rt_free_string(ptr: *mut std::os::raw::c_char) {
            if ptr.is_null() { return; }
            unsafe {
                let _ = std::ffi::CString::from_raw(ptr);
            }
        }
    };

    TokenStream::from(out)
}

///
/// TODO: Implement Asyncness through `sig.asyncness`.
#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let (return_type, return_type_str) = match &input_fn.sig.output {
        syn::ReturnType::Type(_, ty) => {
            use quote::ToTokens;
            (quote! { #ty }, ty.to_token_stream().to_string().replace(" ", ""))
        }
        syn::ReturnType::Default => (quote! { () }, "()".to_string()),
    };

    let wrapper_name_str = format!("{}{}", PLUGIN_DECL_APPENDIX, fn_name);
    let wrapper_ident = format_ident!("{}", wrapper_name_str);
    let args_struct_ident = format_ident!("_Args_{}", fn_name);
    let original_function_descriptor_fn_ident = format_ident!("__impl_fd_schematic_{}", wrapper_name_str);

    let mut struct_fields = Vec::new();
    let mut call_args = Vec::new();
    let mut field_names = Vec::new(); // Collect strings here
    let mut field_types = Vec::new(); // Collect strings here
    
    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            let name = &pat_type.pat;
            let ty = &pat_type.ty;
            
            struct_fields.push(quote! { pub #name: #ty });

            use quote::ToTokens;
            field_names.push(name.to_token_stream().to_string());
            
            field_types.push(ty.to_token_stream().to_string().replace(" ", ""));
    
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let id = &pat_ident.ident;
                call_args.push(quote! { args.#id });
            }
        }
    }

    let out = quote! {
        #input_fn

        #[derive(::dyn_rt::serde::Deserialize)]
        #[serde(crate = "::dyn_rt::serde")]
        #[allow(non_camel_case_types)]
        struct #args_struct_ident {
            #(#struct_fields),*
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn #original_function_descriptor_fn_ident() -> *const ::std::os::raw::c_char {
            let details = ::dyn_rt::serde_json::json!({
                "function_name": stringify!(#fn_name),
                "parameters": [
                    #(
                        {
                            "name": #field_names,
                            "type": #field_types
                        }
                    ),*
                ],
                "return_type": #return_type_str
            });

            let wrapped = ::dyn_rt::WrappedResult {
                data: Some(details),
                error: None
            };

            let json_string = ::dyn_rt::serde_json::to_string(&wrapped).unwrap_or_else(|_| "{\"error\": \"Serialization failed\"}".into());
            let c_result = ::std::ffi::CString::new(json_string).unwrap();
            
            c_result.into_raw()
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn #wrapper_ident(json_ptr: *const ::std::os::raw::c_char) -> *const ::std::os::raw::c_char {
            let c_str = unsafe { ::std::ffi::CStr::from_ptr(json_ptr) };
            let json_str = c_str.to_string_lossy();

            let result_string = match ::dyn_rt::serde_json::from_str::<#args_struct_ident>(&json_str) {
                Ok(args) => {
                    let call_result = #fn_name(#(#call_args),*);
                    let wrapped = ::dyn_rt::WrappedResult {
                        data: Some(call_result),
                        error: None
                    };
                    ::dyn_rt::serde_json::to_string(&wrapped).unwrap_or_else(|_| "{\"error\": \"Serialization failed\"}".into())
                },
                Err(e) => {
                    let wrapped: ::dyn_rt::WrappedResult<#return_type> = ::dyn_rt::WrappedResult {
                        data: None,
                        error: Some(format!("Invalid arguments: {}", e))
                    };
                    ::dyn_rt::serde_json::to_string(&wrapped).unwrap()
                }
            };

            let c_result = ::std::ffi::CString::new(result_string).unwrap();
            c_result.into_raw()
        }
    };

    TokenStream::from(out)
}
