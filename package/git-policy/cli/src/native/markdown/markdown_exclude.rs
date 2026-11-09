//! What:
//!  The `exclude` option of `markdown/autofix`:
//!  gitignore-syntax patterns,
//!  relative to
//!       the repository's top level,
//!  naming candidates the policy leaves alone.
//! Why:
//!  An excluded candidate never starts a linter and never fails for one:
//!      `policy_markdown` asks this matcher about a candidate and skips it when the patterns
//!      name it,
//!  before any linter reads it.
//!  The filtering sits in the wrapper because the wrapper starts the linter.
//!  The TypeScript wrapper this replaced filtered with the same `ignore` package in
//!      `package/git-policy/cli/src/optional/markdown-lint/rewrite-candidates.ts`
//!      (`rewriteCandidates`),
//!  deleted on 2026-10-13 in `642b6d72cb`,
//!  and this module keeps that contract.
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
use crate::git_metadata::path_from_git_bytes;
/// The line a pattern is compiled as,
///  so a literal brace stays literal (`monochromatic_lint::gitignore_pattern`).
use monochromatic_lint::gitignore_pattern::for_matcher;
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
        // What: The pattern reaches the compiler in gitignore syntax, with its literal braces escaped for the glob
        //       grammar beneath it.
        // Why:  Git reads a brace as ordinary text, and the compiler would read `{a,b}` as an alternate group and
        //       refuse an unclosed one.
        //
        // In TS you'd write (pseudocode):
        // builder.addLine(forMatcher(pattern));
        if let Err(error) = builder.add_line(None, for_matcher(pattern.as_str()).as_str()) {
            return Err(format!(
                "monochromatic-git-policy-cli could not compile the markdown/autofix exclude pattern {pattern:?}: {error}"
            ));
        }
    }
    match builder.build() {
        Ok(matcher) => return Ok(ExcludeMatcher { matcher }),
        Err(error) => {
            return Err(format!(
                "monochromatic-git-policy-cli could not compile the markdown/autofix exclude patterns: {error}"
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
///  because `rust/avoidable-closure` reports a closure argument that a named function could replace;
///  the syntax-only anonymous-function ban once cited here was removed on 2026-10-18
///  (`doc/decision/rust-closure-lint-dylint.md`).
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
