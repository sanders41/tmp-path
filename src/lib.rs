use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Creates a temporary directory named `tmp_path` as a Pathbuff.
#[proc_macro_attribute]
pub fn tmp_path(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;
    let tmp_path_var = quote! { tmp_path };
    let new_block = quote! {{
        let mut #tmp_path_var = tempfile::tempdir().unwrap().into_path();
        std::fs::create_dir_all(&#tmp_path_var).unwrap();

        #block
    }};
    let output = quote! {
        #(#attrs)*
        #vis #sig
        #new_block
    };

    TokenStream::from(output)
}
