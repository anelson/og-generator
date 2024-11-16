//! Almost all of the WASM-specific plumbing
use js_sys::JsString;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Used by the log macro in the root module to log messages to the Javascript console
pub(super) fn console_log(s: &str) {
    log(s);
}

/// WASM-specific interface for generating an OpenGraph image.
/// Translates types into regular Rust types and invokes the real implementation.
#[wasm_bindgen]
pub fn generate_og_image(
    site_title: JsString,
    post_title: JsString,
    post_description: JsString,
) -> Result<Vec<u8>, JsString> {
    set_panic_hook();

    info!("Inside WASM entry point");
    info!("site_title: {:?}", site_title);
    info!("post_title: {:?}", post_title);
    info!("post_description: {:?}", post_description);

    let site_title: String = site_title.into();
    let post_title: String = post_title.into();
    let post_description: String = post_description.into();
    crate::generate_og_image_internal(&site_title, &post_title, &post_description).map_err(|e| {
        info!("Error generating OG image: {:?}", e);

        e.to_string().into()
    })
}

/// Install a panic hook to help debug panics in WASM code.
///
/// NOTE: `#[wasm_bindgen(start)]` is not available to us, because the Cloudflare Pages (and
/// presumably Cloudflare Workers in general) runtime doesn't work right w/ the
/// wasm-bindgen-generated wrappers.  We need to explicitly install the panic hook.
#[cfg(target_arch = "wasm32")]
fn set_panic_hook() {
    use std::{backtrace, panic};
    panic::set_hook(Box::new(|info| {
        // Extract file and line info
        let location = info.location().unwrap();
        let file = location.file();
        let line = location.line();

        // Get the panic message
        let msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        // Log the panic information
        info!("Panic occurred in file '{}' at line {}", file, line);
        info!("Panic message: {}", msg);

        // Print the backtrace
        let backtrace = backtrace::Backtrace::force_capture();
        info!("Backtrace:\n{:?}", backtrace);
    }));
}
