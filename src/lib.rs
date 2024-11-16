//! A library for generating Open Graph images for blog posts.
//!
//! This is intended to be compiled to WASM and called from a Cloudflare Worker.  However the
//! library will also compile and run on Tier 1 Rust targets like Linux and Mac, for convenience of
//! testing and development.  In that case there are no exports from this library, the intended
//! interface is running of the tests to generate sample images.

// Make a macro to log strings to the console, using either the wasm console log function or just
// plain println for non-wasm targets
#[cfg(target_arch = "wasm32")]
macro_rules! info {
    ($($arg:tt)*) => (crate::wasm::console_log(&format!($($arg)*)));
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! info {
    ($($arg:tt)*) => (println!($($arg)*));
}

// Make another macro to log debug messages, which are only enabled in debug builds
#[cfg(debug_assertions)]
macro_rules! debug {
    ($($arg:tt)*) => (info!($($arg)*));
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($($arg:tt)*) => {
        if false {
            let _ = ($($arg)*);
        }
    };
}

mod png;
mod svg;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::generate_og_image;

/// When building for non-WASM targets, export this from the lib so the compiler doesn't think all
/// of this code is unused.  Just remember that the only reason we have this here is for
/// convenience of testing and development.
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_og_image(
    site_title: &str,
    post_title: &str,
    post_description: &str,
) -> anyhow::Result<Vec<u8>> {
    generate_og_image_internal(site_title, post_title, post_description)
}

/// Internal Rust impl that creates the OpenGraph image.
///
/// Called from the WASM wrapper in `wasm`, and also from tests.
fn generate_og_image_internal(
    site_title: &str,
    post_title: &str,
    post_description: &str,
) -> anyhow::Result<Vec<u8>> {
    info!("Generating OG image.  site_title='{site_title}', post_title='{post_title}', post_description='{post_description}'");

    let svg_image = svg::generate_svg(site_title, post_title, post_description)?;
    let png_bytes = png::render_svg(&svg_image)?;

    Ok(png_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    #[ignore = "This test should only be run explicitly to experiment with template modifications"]
    fn test_generate_og_image() {
        // Generate an image using a very long sequence of characters to test wrapping and
        // truncating
        let png_data = generate_og_image(
            "127.io | Creative Articulation",
            &"a B C !".repeat(1000),
            &"a B C !".repeat(1000),
        )
        .unwrap();
        std::fs::File::create("card.png")
            .unwrap()
            .write_all(&png_data)
            .unwrap();

        println!("Generated card.png");
    }
}
