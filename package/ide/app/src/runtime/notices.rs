//! The license and notice texts embedded in the executable,
//!  as `monochromatic-ide --licenses`
//! prints them.
//!
//! What:
//!  [`is_notice`] decides which embedded files are license or notice texts,
//!  [`collect`] checks
//!       their digests and adds one entry per Rust crate license text,
//!  and [`write`] prints each
//!       one in full under a heading.
//! Why:
//!  The user chose on 2026-10-06 that the executable shows the texts it carries through a
//!      `--licenses` flag;
//!  the files sit inside the executable,
//!  so nothing else can show them.

/// The Rust crate license list and the lines printed above each of its texts.
use super::crate_licenses;
/// Each text is digest-checked before anything is printed.
use super::embedded::EmbeddedRuntime;
/// Damage and a missing crate list are errors with the file and the remedy.
use anyhow::{Result, bail};
/// Texts are borrowed from the executable or owned after decoding;
///  printing goes to any writer.
use std::{borrow::Cow, io::Write};

/// What:
///  One license or notice text and the lines printed above it.
///  `Cow<'static, [u8]>` is either
///       bytes borrowed from the executable or bytes owned by this value (a crate text decoded from
///       the embedded list).
/// Why:
///  Files print as they are;
///  crate texts come out of one embedded list,
///  so they are owned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Notice = { heading: string; origin: string[]; text: Uint8Array };
/// ```
#[derive(Debug)]
pub struct Notice {
    /// The heading line,
    ///  for example `Language grammar rust: LICENSE`.
    pub heading: String,
    /// The lines between the heading and the text:
    ///  the embedded path,
    ///  or the crates and source.
    pub origin: Vec<String>,
    /// The text exactly as it was when the executable was built.
    pub text: Cow<'static, [u8]>,
}

/// What:
///  Whether an embedded path is a license or notice text printed as one file:
///  everything below
///       `LICENSES/` and `runtime/licenses/` except the Rust crate list,
///  and any other file whose
///       name (ignoring case) contains `LICENSE` or `LICENCE` or starts with `COPYING` or `NOTICE`.
///       `rsplit('/')` walks the path's parts from the end,
///  so its first item is the file name.
/// Why:
///  Most texts sit in those two folders,
///  but Helix also ships a license beside some query files
///      (`runtime/queries/snakemake/LICENSE`).
///  Query read-me files are documentation or a source
///      link,
///  not license terms,
///  so they are left out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isNotice(path: string): boolean
/// ```
pub fn is_notice(path: &str) -> bool {
    // The crate list is printed entry by entry by `collect`, never as one raw file.
    if path == crate_licenses::PATH {
        return false;
    }
    if path.starts_with("LICENSES/") || path.starts_with("runtime/licenses/") {
        return true;
    }
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_uppercase();
    return name.contains("LICENSE")
        || name.contains("LICENCE")
        || name.starts_with("COPYING")
        || name.starts_with("NOTICE");
}

/// What:
///  Every license and notice text of `runtime`:
///  the files in path order,
///  then one entry per
///       Rust crate license text in the list's order,
///  each digest-checked.
///  `Result<Vec<Notice>>` is
///       the list or the first problem found.
/// Why:
///  All texts are checked before any is printed,
///  so a damaged executable prints no partial list.
///      The application's build always embeds the crate list,
///  so its absence is an error too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collect(runtime: EmbeddedRuntime): Notice[] // throws on damage
/// ```
pub fn collect(runtime: &EmbeddedRuntime) -> Result<Vec<Notice>> {
    let mut notices = Vec::new();
    // What: the loop skips every file that is not a notice.
    // Why: Only license and notice texts are printed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const file of runtime.files.filter(file => isNotice(file.path))) { ... }
    // ```
    for file in runtime.files.iter() {
        // Only notice files are listed.
        if !is_notice(file.path) {
            continue;
        }
        // `required` checks the digest; the file is in the table, so only damage can fail here.
        let text = runtime.required(file.path)?;
        notices.push(Notice {
            heading: heading(file.path),
            origin: vec![format!("Embedded as {}", file.path)],
            text: Cow::Borrowed(text),
        });
    }
    // What: `let Some(list) = ... else { bail!(...) }` unpacks the checked bytes or fails.
    // Why: An executable without the crate list would silently omit every crate's terms.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list = runtime.verified(PATH); if (!list) throw new Error('...');
    // ```
    let Some(list) = runtime.verified(crate_licenses::PATH)? else {
        bail!(
            "The executable carries no Rust crate license list ({}), so it was not built by this package's build tasks. Replace it with a fresh copy of the application, or build it again from source.",
            crate_licenses::PATH
        );
    };
    for license in crate_licenses::parse(list)?.licenses {
        notices.push(Notice {
            heading: license.heading(),
            origin: license.origin(),
            text: Cow::Owned(license.text.into_bytes()),
        });
    }
    return Ok(notices);
}

/// What:
///  The heading line for one text,
///  saying whose text it is.
///       `strip_prefix` returns the rest of the text after a prefix,
///  or `None` without it.
/// Why:
///  A reader scanning the output must see which component each text belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function heading(path: string): string
/// ```
fn heading(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("runtime/licenses/") {
        // `split_once('/')` divides `<grammar>/<file>` at its first slash.
        let (grammar, file) = rest.split_once('/').unwrap_or((rest, ""));
        return format!("Language grammar {grammar}: {file}");
    }
    if let Some(rest) = path.strip_prefix("runtime/queries/") {
        let (language, file) = rest.split_once('/').unwrap_or((rest, ""));
        return format!("Helix highlighting queries for {language}: {file}");
    }
    if let Some(file) = path.strip_prefix("LICENSES/font/") {
        return format!("Font compiled into the application: {file}");
    }
    if let Some(file) = path.strip_prefix("LICENSES/") {
        return format!("Monochromatic IDE: {file}");
    }
    if path == "runtime/Helix-LICENSE" {
        return "Helix (highlighting queries and the Helix crates compiled in): Helix-LICENSE"
            .to_string();
    }
    return path.to_string();
}

/// What:
///  Print an introduction,
///  then every text in full under a framed heading that names its
///       component and where the text comes from.
///  `&mut impl Write` lends any writer for writing.
/// Why:
///  One plain-text stream that a pager,
///  a file,
///  or a terminal can hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function write(notices: Notice[], version: string, out: Writable): void
/// ```
pub fn write(notices: &[Notice], version: &str, out: &mut impl Write) -> std::io::Result<()> {
    let rule = "=".repeat(78);
    writeln!(
        out,
        "Monochromatic IDE {version} carries these {} license and notice texts, each in full below.",
        notices.len()
    )?;
    writeln!(
        out,
        "The Rust crate texts were collected by cargo-about from every crate the executable is built from,"
    )?;
    writeln!(
        out,
        "procedural-macro crates that run only while compiling included; each names the crates it covers."
    )?;
    for notice in notices {
        writeln!(out)?;
        writeln!(out, "{rule}")?;
        writeln!(out, "{}", notice.heading)?;
        for line in &notice.origin {
            writeln!(out, "{line}")?;
        }
        writeln!(out, "{rule}")?;
        writeln!(out)?;
        out.write_all(&notice.text)?;
        // A text without a final line break still ends its line before the next heading.
        if !notice.text.ends_with(b"\n") {
            writeln!(out)?;
        }
    }
    return Ok(());
}

/// Selection,
///  headings,
///  and damage,
///  on a small table.
#[cfg(test)]
#[path = "notices_tests.rs"]
mod tests;
