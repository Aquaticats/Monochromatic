//! What:
//!  Renders a resolved target list as `JSON.stringify(targets, null, 2)` followed by a newline.
//! Why:
//!  `.browserslistrc.resolved.local.json` is written by file-enforcer in exactly that form,
//!  so the Rust engine must reproduce it byte for byte.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = `${JSON.stringify(targets, null, 2)}\n`;
//! ```

/// What:
///  Format the list:
/// `[]` for an empty list,
/// else one two-space-indented JSON string per line.
/// Why:
///  `serde_json::to_string` on a single string escapes exactly the characters `JSON.stringify` escapes
///  for the ASCII names and versions browserslist produces,
/// and the layout around it is fixed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderResolvedJson(targets: string[]): string { return `${JSON.stringify(targets, null, 2)}\n`; }
/// ```
pub fn render_resolved_json(targets: &[String]) -> String {
    if targets.is_empty() {
        return "[]\n".to_string();
    }
    let mut text = String::from("[\n");
    for (index, target) in targets.iter().enumerate() {
        text.push_str("  ");
        text.push_str(&serde_json::to_string(target).expect("serializing a string cannot fail"));
        if index + 1 < targets.len() {
            text.push(',');
        }
        text.push('\n');
    }
    text.push_str("]\n");
    return text;
}

/// What:
///  The unit tests of `render.rs`,
/// kept in the sidecar file `render_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './render.test';
/// ```
#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
