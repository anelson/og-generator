//! Uses the template to generate SVG for the OG title card.
//! This module doesn't actually know anything about SVG, it's just a template implementation
use askama::Template;

/// "external" resources referenced by the SVG template.  When we compile to WASM these can't be
/// loaded from a local filesystem, so instead they are embedded into the binary and resolved with
/// a custom resolver.
pub(crate) const EXTERNAL_RESOURCES: &[(&str, &[u8])] = &[(
    "devil.svg",
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/devil.svg")),
)];

/// fonts need to be embedded in the WASM binary as well, for the same reason.
pub(crate) const FONTS: &[&[u8]] = &[include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/noto-sans.ttf"
))];

#[derive(Debug, Template)]
#[template(path = "template.svg")]
struct SvgTemplate {
    site_title: String,
    post_title_lines: Vec<String>,
    post_description_lines: Vec<String>,
}

/// Generate an SVG OG title card from the given inputs.
///
/// Handles wrapping of text since SVG doesn't do that.
pub(super) fn generate_svg(
    site_title: &str,
    post_title: &str,
    post_description: &str,
) -> anyhow::Result<String> {
    // These constants are obtained experimentally by rendering the SVG in a browser and adjusting
    // If the dimensions of the text area, or the fonts, are ever changed, this will need to be
    // experimentally readjusted.
    const POST_TITLE_WIDTH: usize = 33;
    const POST_DESCRIPTION_WIDTH: usize = 34;
    const MAX_POST_TITLE_LINES: usize = 3;
    const MAX_TOTAL_LINES: usize = 13;

    let post_title_lines = wrap_text(post_title, POST_TITLE_WIDTH, MAX_POST_TITLE_LINES);
    let post_description_lines = wrap_text(
        post_description,
        POST_DESCRIPTION_WIDTH,
        MAX_TOTAL_LINES - post_title_lines.len(),
    );
    let template = SvgTemplate {
        site_title: site_title.to_string(),
        post_title_lines,
        post_description_lines,
    };

    let svg_data = template.render()?;

    debug!("Generated {} characters of SVG data", svg_data.len());

    Ok(svg_data)
}

/// Wrap text to a maximum width and truncate to a maximum number of lines.
///
/// This uses `textwrap`, which doesn't know anything about font rendering, but it does know about
/// the width of Unicode characters (ie, one character, two, etc), and it will try to wrap at word
/// boundaries to make the resulting text prettier.
fn wrap_text(text: &str, max_width: usize, max_lines: usize) -> Vec<String> {
    let lines = textwrap::wrap(text, max_width)
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    if lines.len() > max_lines {
        let mut new_lines = lines[..max_lines].to_vec();
        let last_line = new_lines[max_lines - 1].trim_end();

        new_lines[max_lines - 1] = if last_line.len() + 3 > max_width {
            format!("{}...", &last_line[..max_width - 3])
        } else {
            format!("{}...", last_line)
        };

        new_lines
    } else {
        lines
    }
}
