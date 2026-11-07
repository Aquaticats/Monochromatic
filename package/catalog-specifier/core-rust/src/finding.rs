//! What:
//! The result type of a check:
//! one `Finding` per broken rule,
//! tagged with a `FindingKind`.
//! Why:
//! Callers (a commit policy,
//! a warning wrapper,
//! a CI step) print or count findings
//! and must not parse prose,
//! so the kind is a closed list and the text is built in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // finding.ts
//! export type FindingKind = 'unsupported-manifest-format' | 'invalid-manifest' | /* ... */;
//! export type Finding = { kind: FindingKind; path: string; section?: string; name?: string; specifier?: string; detail?: string };
//! ```

/// What:
/// `use std::fmt;` imports the module that holds the text-formatting traits,
/// so `fmt::Display` can be named below.
/// Why:
/// `Display` is how a Rust type says "this is how I print as human text",
/// which the findings need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No import needed: every TS object already has a `toString()` hook.
/// ```
use std::fmt;

/// What:
/// `const REMEDY: &str = ...` is a constant text.
/// `&str` is a borrowed,
/// read-only view of text baked into the program
/// (the sibling is `String`,
/// a growable text the variable owns).
/// `'static` is not written because constants get it for free.
/// Why:
/// `&str` and not `String` because the text never changes and never needs to be owned or built at run time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REMEDY = '...';
/// ```
const REMEDY: &str = "move the version into the `catalog:` section of pnpm-workspace.yaml and write `catalog:`, \
                      or write `workspace:*` for a package of this workspace";

/// What:
/// `pub enum FindingKind { ... }` declares a type that is exactly one of six named cases.
/// `#[derive(...)]` asks the compiler to write the standard comparison and copy code for us:
/// `Clone` and `Copy` (duplicate it cheaply,
/// like a number),
/// `Debug` (printable for tests),
/// `Eq` and `PartialEq` (`==`).
/// Why:
/// A closed list of cases lets a consumer match on the kind instead of searching text for a phrase.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type FindingKind =
///   | 'unsupported-manifest-format' | 'invalid-manifest' | 'malformed-section'
///   | 'duplicate-key' | 'non-string-specifier' | 'disallowed-specifier';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FindingKind {
    /// The file name is `package.json5`,
    /// `package.yaml` or `package.yml`.
    UnsupportedManifestFormat,
    /// The bytes are not UTF-8,
    /// are empty,
    /// do not parse,
    /// or the root is not an object.
    InvalidManifest,
    /// A checked section holds something other than an object.
    MalformedSection,
    /// A checked section or a dependency name appears more than once.
    DuplicateKey,
    /// A dependency value is not a JSON string.
    NonStringSpecifier,
    /// A dependency string is not an accepted specifier for its section.
    DisallowedSpecifier,
}

/// What:
/// `impl FindingKind { ... }` attaches functions to the enum,
/// like methods on a TS class.
/// Why:
/// The vectors file and consumers name kinds by a stable kebab-case word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const kindName = (kind: FindingKind): string => kind;
/// ```
impl FindingKind {
    /// What:
    /// `pub fn as_str(self) -> &'static str` takes the kind by value (it is `Copy`,
    /// so that costs nothing) and returns a
    /// borrowed text that lives for the whole program (`'static`).
    /// Why:
    /// `&'static str` and not `String` because every answer is one of six literals,
    /// so nothing needs allocating.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// export function asStr(kind: FindingKind): string { return kind; }
    /// ```
    pub fn as_str(self) -> &'static str {
        // What: `match self { ... }` picks one branch per case; every case must be covered or it does not compile.
        // Why: Each branch yields the literal for its case, and the match is the function's tail expression, so it is the return.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case 'UnsupportedManifestFormat': return 'unsupported-manifest-format'; /* ... */ }
        // ```
        return match self {
            FindingKind::UnsupportedManifestFormat => "unsupported-manifest-format",
            FindingKind::InvalidManifest => "invalid-manifest",
            FindingKind::MalformedSection => "malformed-section",
            FindingKind::DuplicateKey => "duplicate-key",
            FindingKind::NonStringSpecifier => "non-string-specifier",
            FindingKind::DisallowedSpecifier => "disallowed-specifier",
        };
    }
}

/// What:
/// `pub struct Finding { ... }` declares a record of named fields.
/// `Option<String>` means "either `Some(text)` or `None`",
/// the Rust stand-in for a TS field that may be `undefined`.
/// `String` is an owned,
/// growable text (the sibling is `&str`,
/// a borrowed view).
/// Why:
/// `String` and not `&str` because a finding outlives the manifest bytes it was read from,
/// so it must own its text;
/// `Option` because only some kinds have a section,
/// a name or a specifier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type Finding = { kind: FindingKind; path: string; section?: string; name?: string; specifier?: string; detail?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    /// Which rule was broken.
    pub kind: FindingKind,
    /// The manifest path the caller passed in,
    /// copied unchanged.
    pub path: String,
    /// The checked section (`dependencies`,
    /// ...),
    /// when the finding is about one.
    pub section: Option<String>,
    /// The dependency name,
    /// when the finding is about one;
    /// shown with the replacement character if it held a lone surrogate.
    pub name: Option<String>,
    /// The offending specifier text,
    /// when the finding is about one.
    pub specifier: Option<String>,
    /// Extra text for `InvalidManifest`:
    /// why the bytes could not be read.
    pub detail: Option<String>,
}

/// What:
/// `impl fmt::Display for Finding` teaches Rust how to print a `Finding` as human text,
/// so `format!("{finding}")` works.
/// Why:
/// The text names the file,
/// the section and the dependency and states the remedy,
/// in one place for every consumer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function describe(finding: Finding): string { /* ... */ }
/// ```
impl fmt::Display for Finding {
    /// What:
    /// `fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result` is the one function `Display` requires.
    /// `&self` lends the finding read-only;
    /// `&mut` lends the formatter so we can write into it;
    /// `'_` is an anonymous
    /// lifetime (the compiler infers how long the loan lasts);
    /// `fmt::Result` is `Ok(())` on success or an error.
    /// Why:
    /// The shape is fixed by the trait,
    /// so this is the only legal signature.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { /* build and return the text */ }
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What: `.as_deref()` turns `&Option<String>` into `Option<&str>`; `.unwrap_or("")` takes the text or an empty text.
        // Why: Most branches print an optional field, and an absent one should print as nothing rather than fail.
        // Gotcha: `.unwrap_or(...)` silently replaces `None`; that is intended here because absence is not an error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const section = this.section ?? '';
        // ```
        let section: &str = self.section.as_deref().unwrap_or("");
        let name: &str = self.name.as_deref().unwrap_or("");
        let specifier: &str = self.specifier.as_deref().unwrap_or("");
        let detail: &str = self.detail.as_deref().unwrap_or("");
        let path: &str = self.path.as_str();

        // What: `write!(formatter, "...", args)` writes formatted text into the formatter and returns a `fmt::Result`;
        //       `{path}` inserts the variable named `path`. As the last expression of each branch it is the function's return.
        // Why: Each kind has its own sentence naming the file, section and dependency.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return `${path}: ...`;
        // ```
        return match self.kind {
            FindingKind::UnsupportedManifestFormat => write!(
                formatter,
                "{path}: this manifest format is not supported by the catalog specifier rule; use package.json"
            ),
            FindingKind::InvalidManifest => {
                write!(formatter, "{path}: not a readable package.json object ({detail})")
            }
            FindingKind::MalformedSection => {
                write!(formatter, "{path}: `{section}` must be an object of dependency names to specifiers")
            }
            FindingKind::DuplicateKey => {
                // What: `self.name.is_none()` is true when no dependency name was recorded, so the section itself repeats.
                // Why: A repeated section and a repeated dependency need different sentences.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (this.name === undefined) return `${path}: \`${section}\` appears more than once`;
                // ```
                if self.name.is_none() {
                    write!(formatter, "{path}: `{section}` appears more than once")
                } else {
                    write!(formatter, "{path}: {section} lists `{name}` more than once")
                }
            }
            FindingKind::NonStringSpecifier => write!(
                formatter,
                "{path}: {section} `{name}` must be a string specifier; {REMEDY}"
            ),
            FindingKind::DisallowedSpecifier => write!(
                formatter,
                "{path}: {section} `{name}` is \"{specifier}\"; {REMEDY}"
            ),
        };
    }
}
