//! Decide which locations returned by a language server a read-only reader may open.

/// What:
///  `lsp::Url` is the parsed address type the protocol uses for documents.
/// Why:
///  Servers name definition and reference targets by address,
///  never by local path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { lsp } from 'helix-lsp';
/// ```
use helix_lsp::lsp;
/// What:
///  `Path` is a borrowed filesystem path and `PathBuf` its owned,
///  growable sibling,
///       like `&str` and `String`;
///  `fmt` holds the text-formatting traits.
/// Why:
///  Verdicts keep an owned path because they outlive the server reply they came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { realpathSync, statSync } from 'node:fs';
/// ```
use std::{
    fmt,
    path::{Path, PathBuf},
};

/// What:
///  An `enum` with data in a variant is a tagged union.
///  `#[derive(...)]` asks the compiler
///       to generate copying,
///  debug printing,
///  and equality for it.
/// Why:
///  A target is refused for exactly one of these reasons,
///  and the reader must be able to
///      say which instead of showing "no result".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TargetRefusal =
///   | { kind: 'unsupportedScheme'; scheme: string }
///   | { kind: 'notALocalPath' } | { kind: 'missing' } | { kind: 'notAFile' };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetRefusal {
    /// The address is not a `file` address,
    ///  for example `untitled:` or `jdt:`.
    UnsupportedScheme(
        /// The scheme the server used.
        String,
    ),
    /// A `file` address that cannot be turned into a local path.
    NotALocalPath,
    /// The path does not exist or cannot be resolved.
    Missing,
    /// The path resolves to a directory or a special file.
    NotAFile,
}

/// What:
///  `impl fmt::Display for X` defines how `X` is written by `{}` in format strings.
/// Why:
///  The native layer shows this sentence when a target cannot be opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describe(refusal: TargetRefusal): string { /* switch */ }
/// ```
impl fmt::Display for TargetRefusal {
    /// Write the user-facing reason;
    ///  `&mut fmt::Formatter` is the output sink being appended to.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Self::UnsupportedScheme(scheme) => write!(
                formatter,
                "the language server named a '{scheme}' address, and only local files can be opened"
            ),
            Self::NotALocalPath => {
                write!(formatter, "the address does not name a local file path")
            }
            Self::Missing => write!(formatter, "the file does not exist"),
            Self::NotAFile => write!(formatter, "the path is not a regular file"),
        };
    }
}

/// A target either resolves to a real file inside or outside the project,
///  or is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Classified =
///   | { kind: 'inside'; path: string } | { kind: 'outside'; path: string }
///   | { kind: 'refused'; refusal: TargetRefusal };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Classified {
    /// Canonical path below the project root.
    InsideProject(
        /// Resolved path of the file.
        PathBuf,
    ),
    /// Canonical path of an existing file elsewhere,
    ///  such as a toolchain's standard library.
    OutsideProject(
        /// Resolved path of the file.
        PathBuf,
    ),
    /// Never read and never opened.
    Refused(
        /// Why the address cannot be opened.
        TargetRefusal,
    ),
}

/// What:
///  Classify one address against the canonical project root.
///  `&lsp::Url` and `&Path` lend
///       the inputs read-only.
/// Why:
///  Containment is judged on the resolved location,
///  so a link inside the project that
///      leads elsewhere counts as outside,
///  and two spellings of one path compare equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classify(uri: URL, root: string): Classified {
///   if (uri.protocol !== 'file:') return refused('unsupportedScheme');
///   let real: string;
///   try { real = realpathSync(fileURLToPath(uri)); } catch { return refused('missing'); }
///   if (!statSync(real).isFile()) return refused('notAFile');
///   return real.startsWith(root + sep) ? inside(real) : outside(real);
/// }
/// ```
pub fn classify(uri: &lsp::Url, root: &Path) -> Classified {
    if uri.scheme() != "file" {
        // What: `to_string` copies the borrowed scheme text into an owned `String`.
        // Why: The verdict outlives the address it was read from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: 'refused', refusal: { kind: 'unsupportedScheme', scheme: uri.protocol } };
        // ```
        return Classified::Refused(TargetRefusal::UnsupportedScheme(uri.scheme().to_string()));
    }
    // What: `helix_core::Uri::try_from` percent-decodes the address and normalizes `.` and `..`;
    //       it returns `Result`, and `match` unpacks success (`Ok`) or failure (`Err`).
    // Why: Servers encode the same path differently (`%40` against `@`); only the decoded path
    //      may be compared or resolved.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let decoded: string;
    // try { decoded = fileURLToPath(uri); } catch { return refused('notALocalPath'); }
    // ```
    let decoded = match helix_core::Uri::try_from(uri) {
        Ok(decoded) => decoded,
        Err(error) => {
            tracing::debug!(%error, "language server target is not a local path");
            return Classified::Refused(TargetRefusal::NotALocalPath);
        }
    };
    // What: `let Some(x) = option else { ... }` binds the inner value or runs the else block,
    //       which must leave the function.
    // Why: Helix's address type is open to future non-path variants; they are refused, not guessed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (decoded.path === undefined) return refused('notALocalPath');
    // ```
    let Some(path) = decoded.as_path() else {
        return Classified::Refused(TargetRefusal::NotALocalPath);
    };
    // `canonicalize` resolves symbolic links and fails for a path that does not exist.
    let real = match std::fs::canonicalize(path) {
        Ok(real) => real,
        Err(error) => {
            tracing::debug!(path = %path.display(), %error, "language server target cannot be resolved");
            return Classified::Refused(TargetRefusal::Missing);
        }
    };
    if !real.is_file() {
        return Classified::Refused(TargetRefusal::NotAFile);
    }
    // `starts_with` compares whole path components, so `/project-other` is not inside `/project`.
    if real.starts_with(root) {
        return Classified::InsideProject(real);
    }
    return Classified::OutsideProject(real);
}

/// Real directories,
///  links,
///  and encoded addresses exercise every verdict.
#[cfg(test)]
#[path = "target_tests.rs"]
mod tests;
