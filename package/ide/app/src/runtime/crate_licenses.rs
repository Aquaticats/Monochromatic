//! The license texts of the Rust crates the executable is built from,
//!  as cargo-about collected them.
//!
//! What:
//!  The `notices` task runs cargo-about with the package's `about.toml` and `about.json.hbs`,
//!       which writes `target/crate-licenses.json`;
//!  `build.rs` embeds that file as [`PATH`].
//!  [`parse`]
//!       reads it back,
//!  and [`CrateLicense::heading`] and [`CrateLicense::origin`] give each distinct
//!       license text the lines `--licenses` prints above it.
//! Why:
//!  The user chose on 2026-10-06 to carry every Rust crate's license text in the executable,
//!      collected by cargo-about.
//!  Its model is one entry per distinct license text with the crates
//!      that use it,
//!  so the listing prints one heading per text and names those crates under it.

/// A malformed list is an error naming the embedded file and the remedy.
use anyhow::Result;
/// The list is JSON that the template writes;
///  `serde` maps it onto these types.
use serde::Deserialize;

/// What:
///  Where the crate license list sits in the embedded table.
/// Why:
///  Beside the other license texts below `LICENSES/`;
///  `--licenses` prints it entry by entry
///      instead of as one raw file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PATH = 'LICENSES/crates.json';
/// ```
pub const PATH: &str = "LICENSES/crates.json";

/// What:
///  The width the crate names are wrapped to under a heading,
///  the same as the heading rules.
/// Why:
///  A license such as MIT covers hundreds of crates;
///  one line would not fit a terminal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WIDTH = 78;
/// ```
const WIDTH: usize = 78;

/// What:
///  The whole list.
///  `#[derive(Deserialize)]` lets `serde_json` build it from the JSON object.
/// Why:
///  The template writes one object with a `licenses` array.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CrateLicenses = { licenses: CrateLicense[] };
/// ```
#[derive(Debug, Deserialize)]
pub struct CrateLicenses {
    /// Every distinct license text,
    ///  in cargo-about's order.
    pub licenses: Vec<CrateLicense>,
}

/// What:
///  One distinct license text and the crates that use it.
///  `Option<String>` is a text or nothing.
/// Why:
///  Crates sharing an identical license file share one entry,
///  as cargo-about groups them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CrateLicense = { id: string; name: string; source: string | null; crates: CrateRef[]; text: string };
/// ```
#[derive(Debug, Deserialize)]
pub struct CrateLicense {
    /// The SPDX identifier,
    ///  for example `MIT`.
    pub id: String,
    /// The license's full name,
    ///  for example `MIT License`.
    pub name: String,
    /// The crate file the text was read from,
    ///  or nothing when it is the standard SPDX text.
    pub source: Option<String>,
    /// The crates whose license this text satisfies.
    pub crates: Vec<CrateRef>,
    /// The license text itself.
    pub text: String,
}

/// What:
///  One crate by name and version.
/// Why:
///  A dependency graph can hold two versions of one crate with different license files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CrateRef = { name: string; version: string };
/// ```
#[derive(Debug, Deserialize)]
pub struct CrateRef {
    /// The crate's name as published.
    pub name: String,
    /// The crate's exact version.
    pub version: String,
}

/// What:
///  Read the embedded list.
///  `serde_json::from_slice` decodes JSON bytes into the types here.
/// Why:
///  The bytes were digest-checked already,
///  so a decoding failure means the build wrote a list
///      this program does not understand,
///  which a fresh build fixes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse(bytes: Uint8Array): CrateLicenses // throws with the remedy
/// ```
pub fn parse(bytes: &[u8]) -> Result<CrateLicenses> {
    return match serde_json::from_slice(bytes) {
        Ok(parsed) => Ok(parsed),
        Err(error) => Err(anyhow::Error::new(error).context(format!(
            "The Rust crate license list {PATH} embedded in the executable cannot be read. Replace the executable with a fresh copy of the application, or build it again from source."
        ))),
    };
}

/// What:
///  The part of a crate file path from the crate's own folder on,
///  for example
///       `anyhow-1.0.104/LICENSE-MIT`.
///  `split_once` divides a text at the first occurrence.
/// Why:
///  cargo-about records where the build container keeps crate sources (`/cargo/registry/src/
///      <index>/...` or `/cargo/git/checkouts/<repository>/<revision>/...`);
///  only the crate part
///      means anything to a reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function crateRelative(source: string): string
/// ```
fn crate_relative(source: &str) -> &str {
    // What: `if let Some((_, rest)) = ...` keeps the text after the marker when it occurs.
    // Why: Registry sources sit one folder (the index) below the marker, git sources two (the
    //      repository and the revision).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rest = source.split('/registry/src/')[1]; if (rest) return rest.split('/').slice(1).join('/');
    // ```
    if let Some((_, rest)) = source.split_once("/registry/src/") {
        return match rest.split_once('/') {
            Some((_, crate_part)) => crate_part,
            None => rest,
        };
    }
    if let Some((_, rest)) = source.split_once("/git/checkouts/") {
        let after_repository = match rest.split_once('/') {
            Some((_, part)) => part,
            None => rest,
        };
        return match after_repository.split_once('/') {
            Some((_, part)) => part,
            None => after_repository,
        };
    }
    return source;
}

/// What:
///  Lay `items` out after `prefix` in lines of at most [`WIDTH`] characters,
///  continuing lines
///       indented by the prefix's width.
///  An item longer than a line stays whole on its own line.
/// Why:
///  Keeps a long crate list readable in a terminal and in a pager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wrap(prefix: string, items: string[]): string[]
/// ```
fn wrap(prefix: &str, items: &[String]) -> Vec<String> {
    let indent = " ".repeat(prefix.len());
    let mut lines = Vec::new();
    let mut line = prefix.to_string();
    let mut empty = true;
    for item in items {
        // What: `line.len() + 1 + item.len()` is the length after adding a space and the item.
        // Why: A full line is closed before the item that would overflow it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!empty && line.length + 1 + item.length > WIDTH) { lines.push(line); line = indent; empty = true; }
        // ```
        if !empty && line.len() + 1 + item.len() > WIDTH {
            lines.push(line);
            line = indent.clone();
            empty = true;
        }
        if !empty {
            line.push(' ');
        }
        line.push_str(item);
        empty = false;
    }
    lines.push(line);
    return lines;
}

/// Heading and origin lines of one license text.
impl CrateLicense {
    /// What:
    ///  The heading line:
    ///  whose terms these are and how many crates use them.
    /// Why:
    ///  A reader scanning the listing sees the license and its reach first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// heading(): string // 'Rust crates under MIT License (MIT): 12 crates'
    /// ```
    pub fn heading(&self) -> String {
        let count = self.crates.len();
        let noun = if count == 1 { "crate" } else { "crates" };
        return format!(
            "Rust crates under {} ({}): {count} {noun}",
            self.name, self.id
        );
    }

    /// What:
    ///  The lines between the heading and the text:
    ///  the crates,
    ///  comma-separated and wrapped,
    ///       then where the text comes from.
    /// Why:
    ///  Each crate's terms must be findable by its name and version;
    ///  the source tells whether
    ///      the text is the crate's own file or the standard text of the license.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// origin(): string[]
    /// ```
    pub fn origin(&self) -> Vec<String> {
        let last = self.crates.len().saturating_sub(1);
        // What: `enumerate()` pairs each crate with its position; all but the last get a comma.
        // Why: The commas keep the list readable where the wrapping breaks it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const items = crates.map((c, index) => `${c.name} ${c.version}${index < last ? ',' : ''}`);
        // ```
        let mut items: Vec<String> = Vec::new();
        for (index, used) in self.crates.iter().enumerate() {
            let comma = if index < last { "," } else { "" };
            items.push(format!("{} {}{comma}", used.name, used.version));
        }
        let mut lines = wrap("Used by: ", &items);
        lines.push(match &self.source {
            Some(source) => format!("Text from the crate file {}", crate_relative(source)),
            None => {
                "Text: the standard text of this license (no crate file was recognized)".to_string()
            }
        });
        return lines;
    }
}

/// Parsing,
///  headings,
///  wrapping,
///  and source paths,
///  on small lists.
#[cfg(test)]
#[path = "crate_licenses_tests.rs"]
mod tests;
