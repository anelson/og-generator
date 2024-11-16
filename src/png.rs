//! The heavy-lifting (and the cause of most of the code bloat) that actually renders SVG to PNG
use crate::svg;
use resvg::usvg;
use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn render_svg(svg: &str) -> anyhow::Result<Vec<u8>> {
    let tree = {
        debug!("Instantiating usvg::Tree");

        let mut opt = usvg::Options {
            // When running in WASM, there's no filesystem to load external resources from, so we
            // have to intercept these requests and resolve them with bundled resources
            image_href_resolver: usvg::ImageHrefResolver {
                resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
                resolve_string: Box::new(|href, _options| {
                    debug!("Resolving image href: {}", href);
                    let contents = svg::EXTERNAL_RESOURCES
                        .iter()
                        .find(|(name, _)| *name == href);

                    if let Some((_, data)) = contents {
                        let href = PathBuf::from(href);
                        match href.extension().and_then(|ext| ext.to_str()) {
                            Some("png") => Some(usvg::ImageKind::PNG(Arc::new(data.to_vec()))),
                            Some("svg") => usvg::Tree::from_data(data, &usvg::Options::default())
                                .ok()
                                .map(usvg::ImageKind::SVG),
                            Some(other) => {
                                info!("WARNING: Unrecognized image extension: {:?}", other);
                                None
                            }
                            None => {
                                info!("WARNING: Image href has no extension: {}", href.display());
                                None
                            }
                        }
                    } else {
                        info!("WARNING: Could not resolve image href: {}", href);
                        None
                    }
                }),
            },
            ..Default::default()
        };

        // Load the font data compiled into the binary
        for font_data in svg::FONTS {
            opt.fontdb_mut().load_font_data(font_data.to_vec());
        }

        debug!("Loading SVG");
        usvg::Tree::from_data(svg.as_bytes(), &opt)?
    };

    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    debug!("Rendering SVG to pixmap");
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap_to_optimized_png(pixmap)
}

/// tiny-skia's Pixmap has an `encode_png` method, but we disable the `png` feature because that
/// method doesn't provide enough control over the PNG encoding.  Instead we'll use oxipng to
/// create the PNG and optimize it at the same time.
///
/// Without this, the generated PNGs are around 600KB, which is absurdly large for such a simple
/// image.  With the optimization they're still heavy, 400KB or so, and this takes two seconds when
/// running in WASM.  I hope it's worth the savings in file size.  If I decide it isn't, I can
/// re-enable the `png` feature on `tiny-skia` and use its built-in PNG encoding, which runs in
/// around one second
fn pixmap_to_optimized_png(pixmap: tiny_skia::Pixmap) -> anyhow::Result<Vec<u8>> {
    debug!("Encoding pixmap as optimized PNG");

    // This code is copied from the `Pixmap::encode_png` source code, but modified to use oxipng
    // for the actual PNG encoding

    // Demultiply alpha.
    //
    // RasterPipeline is 15% faster here, but produces slightly different results
    // due to rounding. So we stick with this method for now.

    // Unlike the code in `tink-skia`, we can't use `from_rgba_unchecked` since it's private
    // But the "checked" version is making sure the pixel value is valid, which it won't be
    // because we demultplied it and `from_rgba` expects premultiplied values.
    //
    // So instead, we'll just create a new Vec<u8> and fill it directly with the byte
    // representation of the demultiplied pixels.  This probably even works out to be a bit more
    // efficient, since `RawImage::new` requires taking ownership of the data in a Vec anyway,
    // so `pixmap.data().to_vec()` would have incurred a memcopy as well.
    debug!("Demultiplying pixels");
    let mut data = Vec::<u8>::with_capacity(pixmap.data().len());
    for pixel in pixmap.pixels() {
        let pixel = pixel.demultiply();

        data.extend(&[pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]);
    }
    debug!("Finished demultiplying pixels");

    let raw_image = oxipng::RawImage::new(
        pixmap.width(),
        pixmap.height(),
        oxipng::ColorType::RGBA,
        oxipng::BitDepth::Eight,
        data,
    )?;

    // TODO: experiment with different optimization levels.  2 is the default used by the opipng
    // CLI.  0 is the fastest.  In my testing, not using opipng results in a file over 600KiB.
    // Optimization preset 0 makes 393KiB, optimization level 2 is a bit smaller at 334KiB.
    debug!("Creating optimized PNG (this takes a while)");
    let data = raw_image.create_optimized_png(&oxipng::Options::from_preset(0))?;
    debug!("Created optimized PNG");

    debug!("Generated {} bytes of optimized PNG data", data.len());

    Ok(data)
}
