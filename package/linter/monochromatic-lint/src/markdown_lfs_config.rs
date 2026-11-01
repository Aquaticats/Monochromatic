//! What:
//!  Read the Git LFS endpoint declarations of a repository's committed `.lfsconfig`.
//! Why:
//!  `markdown/lfs-image-url` builds object URLs on the server that file declares;
//! the scan keeps the incumbent's line grammar so the same declarations are found in the same order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseLfsConfig(text) -> normalized bases in file order; readLfsObjectBase(root) -> first or none.
//! ```

/// Import the single endpoint normalizer;
///  no other code turns an endpoint into an object base.
/// Import ECMAScript's trim set,
///  which differs from Rust's `str::trim` (it includes U+FEFF,
///  excludes U+0085).
use crate::{
    markdown_lfs_endpoint::{LfsUrlRejection, lfs_object_base},
    markdown_table_text::trim_space,
};
/// Import native paths and I/O error classification.
use std::{io::ErrorKind, path::Path};

/// What:
///  The file name git-lfs reads for repository-level configuration.
/// Why:
///  Its directory is also the repository root every object path is relative to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LFS_CONFIG_FILENAME = '.lfsconfig';
/// ```
pub(crate) const LFS_CONFIG_FILENAME: &str = ".lfsconfig";

/// What:
///  Remove ECMAScript whitespace and line terminators from both ends of a borrowed string.
/// Why:
///  The incumbent calls `String.prototype.trim`,
///  so a trailing carriage return or BOM is not part of a value.
/// The result is a `&str` view into the input,
///  not a new `String`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsTrim(text: string): string { return text.trim(); }
/// ```
pub(crate) fn js_trim(text: &str) -> &str {
    return text.trim_matches(trim_space);
}

/// What:
///  Collect endpoint values exactly as written,
///  in file order.
/// Why:
///  `url` under `[lfs]` and `lfsurl` under any `[remote "..."]` section name the LFS server;
/// comments,
///  blank lines and unrelated keys pass through.
///  Lines split on U+000A only.
/// Section and key names fold ASCII case:
///  the evaluation measured over every code point that no
/// non-ASCII character lowercases to text containing a letter of the compared names.
/// `Vec<String>` owns each value because the caller outlives the borrowed file text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lfsEndpoints(text: string): string[];
/// ```
pub(crate) fn lfs_endpoints(text: &str) -> Vec<String> {
    let mut section: String = String::new();
    let mut endpoints: Vec<String> = Vec::<String>::new();
    for raw_line in text.split('\n') {
        let line: &str = js_trim(raw_line);
        // Blank lines and `#` or `;` comments need no branch of their own. A blank line has no `=`.
        // A comment is not a section header, because it does not start with `[`, and its key
        // starts with the comment character, so it can never equal `url` or `lfsurl`.
        if line.starts_with('[') && line.ends_with(']') {
            // Both brackets are single ASCII bytes and distinct, so the line holds at least two bytes.
            let inner: &str = &line[1..line.len() - 1];
            section = js_trim(inner).to_ascii_lowercase();
            continue;
        }
        let Some(equals): Option<usize> = line.find('=') else {
            continue;
        };
        let key: String = js_trim(&line[..equals]).to_ascii_lowercase();
        let value: &str = js_trim(&line[equals + 1..]);
        let lfs_url: bool = (section == "lfs" && key == "url")
            || (section.starts_with("remote ") && key == "lfsurl");
        if lfs_url && !value.is_empty() {
            endpoints.push(String::from(value));
        }
    }
    return endpoints;
}

/// What:
///  Normalize every declared endpoint,
///  in file order.
/// Why:
///  The incumbent normalizes all declarations before selecting one,
/// so a malformed later declaration fails the read instead of being skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLfsConfig(text: string): string[]; // throws on a malformed endpoint
/// ```
pub(crate) fn parse_lfs_config(text: &str) -> Result<Vec<String>, LfsUrlRejection> {
    let mut bases: Vec<String> = Vec::<String>::new();
    for endpoint in lfs_endpoints(text) {
        // `?` returns the normalizer's failure to the caller instead of continuing with fewer bases.
        bases.push(lfs_object_base(endpoint.as_str())?);
    }
    return Ok(bases);
}

/// What:
///  A failed read of repository LFS configuration.
/// Why:
///  An unreadable or malformed `.lfsconfig` must surface as a processing failure,
///  never as "no LFS here".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LfsConfigError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LfsConfigError {
    /// Affected file and failed operation.
    pub message: String,
}

/// Render the failure through the ordinary error interface.
impl std::fmt::Display for LfsConfigError {
    /// Borrow the formatter only while writing the stored explanation.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.message.as_str());
    }
}

/// Mark the failure as a standard error for application error handling.
impl std::error::Error for LfsConfigError {}

/// What:
///  Read one text file,
///  mapping absence to `None` and every other failure to a typed error.
/// Why:
///  A missing `.lfsconfig` or `.gitattributes` is ordinary;
///  a permission or encoding failure is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readOptionalText(path: Path): string | undefined; // rethrows non-ENOENT errors
/// ```
pub(crate) fn read_optional_text(path: &Path) -> Result<Option<String>, LfsConfigError> {
    match std::fs::read_to_string(path) {
        Ok(text) => return Ok(Some(text)),
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(LfsConfigError {
                message: format!("Cannot read {}: {error}.", path.display()),
            });
        }
    }
}

/// What:
///  The first object base a repository root declares,
///  or `None` without a file or declaration.
/// Why:
///  The rule is inert for a repository that names no LFS server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLfsObjectBase(repoRoot: Path): string | undefined;
/// ```
pub(crate) fn read_lfs_object_base(repo_root: &Path) -> Result<Option<String>, LfsConfigError> {
    let path: std::path::PathBuf = repo_root.join(LFS_CONFIG_FILENAME);
    let Some(text): Option<String> = read_optional_text(&path)? else {
        return Ok(None);
    };
    match parse_lfs_config(text.as_str()) {
        Ok(bases) => return Ok(bases.into_iter().next()),
        Err(error) => {
            // The endpoint itself is not echoed: it may hold credentials.
            return Err(LfsConfigError {
                message: format!(
                    "{} declares an LFS endpoint this linter cannot use: {error}",
                    path.display()
                ),
            });
        }
    }
}

/// Line-grammar and file-read controls stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_lfs_config_tests.rs"]
mod tests;
