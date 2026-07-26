use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Creates a temporary directory available to the function as `tmp_path`, a `PathBuf`.
///
/// The directory is created inside the system temporary directory and is removed when the
/// thread that created it finishes, including when it panics. Since the test harness runs each
/// test on its own thread, the directory lives for the duration of the test, which allows an
/// annotated helper to return paths to the test that called it.
///
/// The directory that gets removed is the one originally created, so reassigning or mutating
/// `tmp_path` does not affect cleanup.
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
        struct TmpPathCleanup(Vec<std::path::PathBuf>);

        impl Drop for TmpPathCleanup {
            fn drop(&mut self) {
                for path in &self.0 {
                    let _ = std::fs::remove_dir_all(path);
                }
            }
        }

        std::thread_local! {
            static TMP_PATH_CLEANUP: std::cell::RefCell<TmpPathCleanup> =
                std::cell::RefCell::new(TmpPathCleanup(Vec::new()));
        }

        let mut #tmp_path_var = {
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
        };
        TMP_PATH_CLEANUP.with(|cleanup| cleanup.borrow_mut().0.push(#tmp_path_var.clone()));

        #block
    }};
    let output = quote! {
        #(#attrs)*
        #vis #sig
        #new_block
    };

    TokenStream::from(output)
}
