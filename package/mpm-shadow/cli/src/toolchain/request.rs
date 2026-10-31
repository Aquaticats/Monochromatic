//! What:
//! A plan entry's request:
//! one text of the form `<origin>@<range>` that says where a tool's versions come from and which versions are wanted,
//! the way a `package.json` value (`"npm:real@^1"`) does.
//! Why:
//! The lock is derived from the plan alone,
//! so everything a resolver needs to find a tool (the repository,
//! the registry,
//! the channel) is in the request,
//! and no resolver reads a previous lock row to learn it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // request.ts
//! export type Request = { source: string; range: string };
//! export function parseRequest(text: string): Request   // "github:FiloSottile/age@latest"
//! export function requestText(request: Request): string
//! ```

/// Imports the shared named helpers.
use super::util::{all_ascii_digits, starts_with_ascii_digit};

/// The origins written with a prefix:
/// the text after the prefix names the thing in that origin.
const PREFIXED_ORIGINS: [&str; 7] = ["github:", "npm:", "cargo:", "pipx:", "conda:", "go:", "git:"];

/// The origins written as a bare name:
/// each has a resolver of its own.
pub const NAMED_ORIGINS: [&str; 8] = ["node", "go", "zig", "rustup", "aws", "rust", "android-sdk", "java:temurin"];

/// What:
/// Where a tool's versions come from.
/// Why:
/// The resolver switch matches on this,
/// so a new origin cannot be added without deciding how it resolves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Origin = { kind: 'github'; ownerRepo: string } | { kind: 'npm'; name: string } | ... | { kind: 'named'; name: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Origin {
    /// `github:<owner>/<repo>`:
    /// releases of a GitHub repository.
    Github(
        /// `owner/repo`.
        String,
    ),
    /// `npm:<name>`:
    /// a package of the npm registry.
    Npm(
        /// The package name (`@scope/name` for a scoped one).
        String,
    ),
    /// `cargo:<crate>`:
    /// a crate of crates.io.
    Cargo(
        /// The crate name.
        String,
    ),
    /// `pipx:<name>`:
    /// a project of PyPI.
    Pipx(
        /// The project name.
        String,
    ),
    /// `conda:<name>`:
    /// a package of the conda-forge channel.
    Conda(
        /// The package name.
        String,
    ),
    /// `go:<module>`:
    /// a Go module.
    GoModule(
        /// The module path.
        String,
    ),
    /// `git:<url>`:
    /// a git repository,
    /// with the revision as the range.
    Git(
        /// The clone URL.
        String,
    ),
    /// A bare name with a resolver of its own (`zig`,
    /// `node`,
    /// `java:temurin`).
    Named(
        /// The name.
        String,
    ),
}

/// What:
/// The parsed request:
/// the source text before the range separator and the range after it (`latest` when none is written).
/// Why:
/// The range is kept as text,
/// because each origin reads it its own way (an exact version,
/// `latest`,
/// a channel,
/// a Java major line).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Request = { source: string; range: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    /// The origin text (`github:FiloSottile/age`,
    /// `zig`).
    pub source: String,
    /// The wanted versions (`latest`,
    /// `0.15.2`,
    /// `nightly`,
    /// `21`).
    pub range: String,
}

/// The byte index of the range separator in `text`:
/// the last `@` that does not follow `:` or `/` (those belong to an npm scope or a git URL).
fn separator_index(text: &str) -> Option<usize> {
    let bytes: &[u8] = text.as_bytes();
    let mut index: usize = bytes.len();
    while index > 0 {
        index -= 1;
        if bytes[index] != b'@' {
            continue;
        }
        if index == 0 || bytes[index - 1] == b':' || bytes[index - 1] == b'/' {
            continue;
        }
        return Some(index);
    }
    return None;
}

/// The methods of `Request`.
impl Request {
    /// What:
    /// Parses `<origin>[@<range>]`.
    /// Why:
    /// An unknown origin is refused with the list of valid ones,
    /// so a typo cannot silently fall through to another resolver.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// export function parseRequest(text: string): Request  // throws
    /// ```
    pub fn parse(text: &str) -> Result<Request, String> {
        let (source, range): (&str, &str) = match separator_index(text) {
            Some(at) => (&text[..at], &text[at + 1..]),
            None => (text, "latest"),
        };
        if source.is_empty() || range.is_empty() || range.contains(char::is_whitespace) || source.contains(char::is_whitespace) {
            return Err(format!("{text:?} must be <origin> or <origin>@<range>, without spaces"));
        }
        let request: Request = Request { source: source.to_string(), range: range.to_string() };
        if let Origin::Named(name) = request.origin()
            && !NAMED_ORIGINS.contains(&name.as_str())
        {
            return Err(format!(
                "the origin {source:?} is not known; write one of {} or a name with one of the prefixes {}",
                NAMED_ORIGINS.join(", "),
                PREFIXED_ORIGINS.join(" ")
            ));
        }
        for prefix in PREFIXED_ORIGINS {
            if source == prefix {
                return Err(format!("the origin {source:?} names nothing after {prefix}"));
            }
        }
        return Ok(request);
    }

    /// The request as written:
    /// `<source>@<range>`.
    pub fn text(&self) -> String {
        return format!("{}@{}", self.source, self.range);
    }

    /// The origin the source text names.
    pub fn origin(&self) -> Origin {
        let source: &str = &self.source;
        if let Some(rest) = source.strip_prefix("github:") {
            return Origin::Github(rest.to_string());
        }
        if let Some(rest) = source.strip_prefix("npm:") {
            return Origin::Npm(rest.to_string());
        }
        if let Some(rest) = source.strip_prefix("cargo:") {
            return Origin::Cargo(rest.to_string());
        }
        if let Some(rest) = source.strip_prefix("pipx:") {
            return Origin::Pipx(rest.to_string());
        }
        if let Some(rest) = source.strip_prefix("conda:") {
            return Origin::Conda(rest.to_string());
        }
        if let Some(rest) = source.strip_prefix("go:") {
            return Origin::GoModule(rest.to_string());
        }
        if let Some(rest) = source.strip_prefix("git:") {
            return Origin::Git(rest.to_string());
        }
        return Origin::Named(source.to_string());
    }

    /// What:
    /// Whether the range names one version that never moves
    /// (so `update` keeps a lock entry for an unchanged request without a lookup).
    /// Why:
    /// `latest` moves;
    /// so do a Rust channel (`nightly`),
    /// a Java major line (`21`)
    /// and an npm dist-tag (`next`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isExact(): boolean
    /// ```
    pub fn is_exact(&self) -> bool {
        if self.range == "latest" {
            return false;
        }
        return match self.origin() {
            Origin::Named(name) if name == "rust" => !matches!(self.range.as_str(), "nightly" | "beta" | "stable"),
            Origin::Named(name) if name == "java:temurin" => !all_ascii_digits(&self.range),
            Origin::Npm(_) => starts_with_ascii_digit(&self.range),
            _ => true,
        };
    }
}
