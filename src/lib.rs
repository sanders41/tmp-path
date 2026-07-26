use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Creates a temporary directory available to the function as `tmp_path`, a `&Path`.
///
/// The directory is created inside the system temporary directory and is removed when the
/// function returns, including when it panics.
#[proc_macro_attribute]
pub fn tmp_path(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
        ..
    } = input;
    let tmp_path_var = quote! { tmp_path };
    let new_block = quote! {{
        struct TmpPathGuard(std::path::PathBuf);

        impl Drop for TmpPathGuard {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        let _tmp_path_guard = TmpPathGuard({
            let base = std::env::temp_dir();
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0u128, |elapsed| elapsed.as_nanos());
            let mut created = None;

            for attempt in 0..32u32 {
                let candidate =
                    base.join(format!("tmp-path-{}-{}-{}", std::process::id(), nanos, attempt));

                match std::fs::create_dir(&candidate) {
                    Ok(()) => {
                        created = Some(candidate);
                        break;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(err) => panic!(
                        "tmp_path: failed to create temporary directory {}: {err}",
                        candidate.display()
                    ),
                }
            }

            match created {
                Some(path) => path,
                None => panic!(
                    "tmp_path: no unused temporary directory name found in {}",
                    base.display()
                ),
            }
        });
        let #tmp_path_var = _tmp_path_guard.0.as_path();

        #block
    }};
    let output = quote! {
        #(#attrs)*
        #vis #sig
        #new_block
    };

    TokenStream::from(output)
}
