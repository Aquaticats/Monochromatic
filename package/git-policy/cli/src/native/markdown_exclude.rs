//! What:
//!  The `exclude` option of `markdown/autofix`:
//!  gitignore-syntax patterns,
//!  relative to
//!       the repository's top level,
//!  naming candidates the policy leaves alone.
//! Why:
//!  The installed wrapper filtered candidates with the `ignore` package before starting
//!      the linter (`rewrite-candidates.ts`,
//!  `rewriteCandidates`),
//!  so an excluded file never
//!      starts a linter and never fails for one.
//!  The linter evaluates its own copy of the
//!      option with the `ignore` crate's gitignore matcher and a walk over ancestor
//!      directories (`package/linter/monochromatic-lint/src/markdown_lfs_patterns.rs`);
//!  this
//!      module uses the same matcher and the same walk,
//!  so both agree on every path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const excluded = ignore().add(exclude); if (excluded.ignores(candidate.path)) continue;
//! ```

/// Import the conversion of Git's pathname bytes to a path.
use super::git_metadata::path_from_git_bytes;
/// What:
///  `Gitignore` is a compiled pattern list;
///  `GitignoreBuilder` compiles one.
/// Why:
///   The same compiler the linter and the scanner use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import ignore from 'ignore';
/// ```
use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// What:
///  The compiled patterns.
///  The field is private:
///  build one with `compile_exclude`.
/// Why:
///   Compiled once per invocation,
///  then asked once per Markdown candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExcludeMatcher = { ignores(path: string): boolean };
/// ```
#[derive(Clone, Debug)]
pub struct ExcludeMatcher {
    /// The patterns in authored order;
    ///  a later match overrides an earlier one.
    matcher: Gitignore,
}

/// What:
///  Compile `patterns` in order.
///  `Result<ExcludeMatcher, String>` is the matcher or
///       the reason a pattern was refused,
///  naming it.
/// Why:
///   A pattern that cannot compile must not silently match nothing.
///  The empty root
///       keeps candidate paths as Git prints them:
///  relative to the top level.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compileExclude(patterns: string[]): ExcludeMatcher;
/// ```
pub fn compile_exclude(patterns: &[String]) -> Result<ExcludeMatcher, String> {
    let mut builder: GitignoreBuilder = GitignoreBuilder::new("");
    for pattern in patterns {
        // The first argument names a source file for diagnostics; these patterns have none.
        if let Err(error) = builder.add_line(None, pattern.as_str()) {
            return Err(format!(
                "cli-git could not compile the markdown/autofix exclude pattern {pattern:?}: {error}"
            ));
        }
    }
    match builder.build() {
        Ok(matcher) => return Ok(ExcludeMatcher { matcher }),
        Err(error) => {
            return Err(format!(
                "cli-git could not compile the markdown/autofix exclude patterns: {error}"
            ));
        }
    }
}

/// What:
///  `impl ExcludeMatcher { ... }` attaches the one question it answers.
/// Why:
///   Callers never see the walk over ancestor directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ExcludeMatcher { excludes(path: Uint8Array): boolean }
/// ```
impl ExcludeMatcher {
    /// What:
    ///  Whether the patterns name the candidate at `path`,
    ///  Git's pathname bytes.
    /// Why:
    ///   In gitignore semantics a file below an excluded directory stays excluded
    ///       even when a later pattern negates the file,
    ///  so each ancestor directory is
    ///       asked first,
    ///  from the top;
    ///  only when none is excluded does the file's own
    ///       last match decide.
    ///  A pathname this platform cannot spell is not excluded.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// excludes(path: Uint8Array): boolean
    /// ```
    pub fn excludes(&self, path: &[u8]) -> bool {
        let segments: Vec<&[u8]> = path.split(is_slash).collect();
        let mut prefix: Vec<u8> = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            prefix.extend_from_slice(segment);
            let Some(spelled) = path_from_git_bytes(prefix.as_slice()) else {
                return false;
            };
            // The last segment is the file itself; every earlier prefix is a directory.
            let is_file: bool = index + 1 == segments.len();
            let excluded: bool = self
                .matcher
                .matched(spelled.as_path(), !is_file)
                .is_ignore();
            if is_file || excluded {
                return excluded;
            }
            // The separator before the next segment.
            prefix.push(b'/');
        }
        return false;
    }
}

/// What:
///  Whether one byte is `/`,
///  the separator of Git pathnames.
/// Why:
///   A named predicate for `split`,
///  because the repository bans anonymous functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSlash = (byte: number) => byte === 0x2f;
/// ```
fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// Pattern and ancestor-walk controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_exclude_tests.rs"]
mod tests;
