extern crate proc_macro;

use anyhow::Context;
use proc_macro::TokenStream;
use quote::quote;

mod kiwi_project;

/// Makes your `main()` function accessible to the WASM host, and generates a `components` module with your project's components.
///
/// If you do not add this attribute to your `main()` function, your module will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let fn_name = item.sig.ident.clone();
    if item.sig.asyncness.is_none() {
        panic!("the `{fn_name}` function must be async");
    }

    let project_boilerplate = kiwi_project_pm2(false).unwrap();

    quote! {
        #project_boilerplate

        #item

        #[no_mangle]
        pub extern "C" fn call_main(runtime_interface_version: u32) {
            if INTERFACE_VERSION != runtime_interface_version {
                panic!("This module was compiled with interface version {{INTERFACE_VERSION}}, but the host is running with version {{runtime_interface_version}}.");
            }
            run_async(#fn_name());
        }
    }.into()
}

/// Generates global components and other boilerplate for the API crate.
#[proc_macro]
pub fn api_project(_input: TokenStream) -> TokenStream {
    TokenStream::from(kiwi_project_pm2(true).unwrap())
}

fn kiwi_project_pm2(global: bool) -> anyhow::Result<proc_macro2::TokenStream> {
    kiwi_project::implementation(
        kiwi_project::read_file("kiwi.toml".to_string())
            .context("Failed to load kiwi.toml")
            .unwrap(),
        global,
    )
}