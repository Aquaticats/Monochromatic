//! What:
//!  Gitignore-syntax path matching for the LFS rule's two pattern lists.
//! Why:
//!  Root `.gitattributes` `filter=lfs` lines decide which files are LFS-tracked,
//!  and the rule's
//! `exclude` option names files the rule leaves alone.
//!  The incumbent evaluates both with one
//! gitignore matcher,
//!  so both keep last-match-wins and excluded-parent semantics here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const matcher = ignore().add(patterns); matcher.ignores(repoRelativePath)
//! ```

/// Import the shared ECMAScript trim so attribute lines split as the incumbent splits them.
use crate::markdown_lfs_config::js_trim;
/// Import the walker family's gitignore compiler and match outcome,
///  already a dependency of file discovery.
use ignore::{
    Match,
    gitignore::{Gitignore, GitignoreBuilder},
};

/// What:
///  A rejected pattern,
///  naming its text and the compiler's reason.
/// Why:
///  A pattern that cannot compile must not silently match nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PathPatternError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathPatternError {
    /// Affected pattern and compiler explanation.
    pub message: String,
}

/// Render the rejection through the ordinary error interface.
impl std::fmt::Display for PathPatternError {
    /// Borrow the formatter only while writing the stored explanation.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.message.as_str());
    }
}

/// Mark the rejection as a standard error for application error handling.
impl std::error::Error for PathPatternError {}

/// What:
///  A compiled ordered pattern list over forward-slash paths relative to the repository root.
/// Why:
///  Compiling once lets every image target and every linted file reuse the same matcher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PathPatterns { matches(repoRelativePath: string): boolean }
/// ```
#[derive(Clone, Debug)]
pub struct PathPatterns {
    /// Compiled globs in authored order;
    ///  a later match overrides an earlier one.
    matcher: Gitignore,
}

/// What:
///  Build and query one pattern list.
/// Why:
///  Callers never see glob compilation or parent-directory walking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new PathPatterns(patterns).matches(path)
/// ```
impl PathPatterns {
    /// What:
    ///  Compile patterns in order,
    ///  each in gitignore syntax (`!` negates,
    ///  a trailing `/` names a directory).
    /// Why:
    ///  The empty root keeps candidate paths exactly as given;
    ///  they are already repository-relative.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(patterns: readonly string[])
    /// ```
    pub fn new(patterns: &[String]) -> Result<PathPatterns, PathPatternError> {
        let mut builder: GitignoreBuilder = GitignoreBuilder::new("");
        for pattern in patterns {
            // The first argument records a source file for diagnostics; these lists have none.
            if let Err(error) = builder.add_line(None, pattern.as_str()) {
                return Err(PathPatternError {
                    message: format!("Cannot compile path pattern {pattern:?}: {error}."),
                });
            }
        }
        match builder.build() {
            Ok(matcher) => return Ok(PathPatterns { matcher }),
            Err(error) => {
                return Err(PathPatternError {
                    message: format!("Cannot compile path patterns: {error}."),
                });
            }
        }
    }

    /// What:
    ///  Decide whether a file path is selected,
    ///  checking each ancestor directory from the top first.
    /// Why:
    ///  In gitignore semantics a file under an excluded directory stays excluded even if a later
    /// pattern negates the file itself;
    ///  only when no ancestor is excluded does the file's own last match decide.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// matches(repoRelativePath: string): boolean
    /// ```
    pub fn matches(&self, repo_relative_path: &str) -> bool {
        let mut prefix: String = String::new();
        let mut components: std::iter::Peekable<std::str::Split<'_, char>> =
            repo_relative_path.split('/').peekable();
        while let Some(component) = components.next() {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(component);
            // The last component is the file itself; every earlier prefix is a directory.
            let is_file: bool = components.peek().is_none();
            let outcome: Match<&ignore::gitignore::Glob> =
                self.matcher.matched(prefix.as_str(), !is_file);
            if is_file {
                return outcome.is_ignore();
            }
            if outcome.is_ignore() {
                return true;
            }
        }
        return false;
    }
}

/// What:
///  Split one attribute line on runs of spaces and tabs.
/// Why:
///  The attribute grammar separates a pattern from its attributes by blanks only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitOnBlanks(line: string): string[];
/// ```
fn split_on_blanks(line: &str) -> Vec<&str> {
    let mut tokens: Vec<&str> = Vec::<&str>::new();
    for token in line.split([' ', '\t']) {
        if !token.is_empty() {
            tokens.push(token);
        }
    }
    return tokens;
}

/// What:
///  Translate `.gitattributes` text into the gitignore-syntax pattern list of LFS-tracked paths.
/// Why:
///  A line that sets `filter=lfs` selects its pattern;
///  a later line that unsets the filter
/// (`-filter` or `!filter`) becomes a negation,
///  matching git's last-match-wins attribute resolution.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lfsTrackedPatterns(text: string): string[];
/// ```
pub(crate) fn lfs_tracked_patterns(text: &str) -> Vec<String> {
    let mut patterns: Vec<String> = Vec::<String>::new();
    for raw_line in text.split('\n') {
        let line: &str = js_trim(raw_line);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let tokens: Vec<&str> = split_on_blanks(line);
        let Some((pattern, attributes)): Option<(&&str, &[&str])> = tokens.split_first() else {
            continue;
        };
        if attributes.contains(&"filter=lfs") {
            patterns.push(String::from(*pattern));
        } else if attributes.contains(&"-filter") || attributes.contains(&"!filter") {
            patterns.push(format!("!{pattern}"));
        }
    }
    return patterns;
}

/// Attribute-grammar and parent-directory controls stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_lfs_patterns_tests.rs"]
mod tests;
