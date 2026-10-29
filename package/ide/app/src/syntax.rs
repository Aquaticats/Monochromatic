//! Helix language recognition and tree-house highlighting,
//!  independent of native widgets.
//!
//! Language files come only from the application's own runtime ([`crate::runtime`]) through
//! [`crate::syntax_loader::OwnLoader`];
//!  Helix's runtime directory search is never consulted.

/// Source paint spans remain unrelated to native pixel coordinates.
use crate::source_style::{SourceStyles, StyleSpan};
/// The upstream error enum needs explicit conversion and operation-specific remedies.
use crate::syntax_error::parser_failure;
/// Reuse Helix's built-in language table for recognition;
///  grammars and queries come from the own loader.
use crate::{runtime, syntax_loader::OwnLoader};
/// Errors distinguish unavailable parsers/rules from a successful plain-text language.
use anyhow::{Context, Result, anyhow, bail};
/// Helix's built-in language table and rope text type.
use helix_core::{
    Language, Rope,
    syntax::{Loader, config::Configuration},
};
/// Decode the bundled manifest into a typed record instead of probing untyped JSON.
use serde::Deserialize;
/// What:
///  Paths identify the source language;
///  Rope retains scalar/byte conversion boundaries.
///       `HashSet` is a set of unique values;
///  `fs` reads the bundled manifest file.
/// Why:
///  Syntax positions must map to canonical source characters,
///  not rendered columns,
///      and provisioned-grammar lookup needs membership tests rather than ordered scans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync } from 'node:fs';
/// import { type Path, Rope, LanguageLoader, Syntax } from './helix';
/// ```
use std::{collections::HashSet, fs, path::Path, time::Duration};
/// The parser library Helix builds on,
///  driven here with the application's own loader.
use tree_house::{Syntax, highlighter::Highlighter};

/// What:
///  How long one parse may take;
///  `Duration` is a span of time (sibling:
///  `Instant`,
///  a moment).
/// Why:
///  The same half-second limit as Helix's `PARSE_TIMEOUT` in `helix-core/src/syntax.rs`,
///  which
///      the engine used before it drove the parser library directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARSE_TIMEOUT_MS = 500;
/// ```
const PARSE_TIMEOUT: Duration = Duration::from_millis(500);

/// What:
///  Ordered palette roles;
///  role zero stays ordinary source and 64 stays selection ink.
///  A capture takes the entry sharing the most leading dot-separated parts,
///  so `markup.heading.marker` takes `markup.heading` and `markup.link.text` takes `markup.link`.
/// Why:
///  The Markdown entries follow the original thirteen so that those keep their numbers.
///  `markup.raw.block` has no entry:
///  a fenced block's own language paints its content,
///  and a block color would show through every token that language leaves unpainted.
///  `label` has no entry because Rust and JavaScript use it for loop labels too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SCOPES = ['keyword', 'string', /* ... */ 'markup.quote'] as const;
/// ```
pub const SCOPES: &[&str] = &[
    "keyword",
    "string",
    "comment",
    "constant",
    "type",
    "function",
    "variable",
    "operator",
    "punctuation",
    "tag",
    "attribute",
    "namespace",
    "special",
    "markup.heading",
    "markup.bold",
    "markup.italic",
    "markup.strikethrough",
    "markup.raw.inline",
    "markup.link",
    "markup.link.url",
    "markup.list",
    "markup.quote",
];

/// What:
///  `#[derive(Deserialize)]` asks the serde library to generate this record's JSON decoder
///       at compile time;
///  `struct Manifest` is the record the package runtime task writes as
///       `manifest.json` beside the grammars.
/// Why:
///   A typed record rejects a malformed manifest instead of guessing which grammars exist.
///       Keys this reader does not name,
///  such as `helixRevision`,
///  are ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Manifest = { grammars: string[] };
/// ```
#[derive(Deserialize)]
struct Manifest {
    /// What:
    ///  `Vec<String>` is a growable list of owned text values,
    ///  one shared-object file name
    ///       per grammar.
    ///  Siblings:
    ///  `&[&str]` (borrowed,
    ///  fixed) and `[String; N]` (fixed length).
    /// Why:
    ///   The count comes from the file,
    ///  and the decoded names must outlive the file buffer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// grammars: string[];
    /// ```
    grammars: Vec<String>,
}

/// What:
///  `fn provisioned_grammars(manifest: &Path) -> Result<HashSet<String>>` reads a manifest file
///       from a prepared runtime directory and returns the grammar ids it lists,
///  or an error.
///       `&Path` borrows the path (the caller keeps ownership).
///  Siblings of `HashSet<String>`:
///       `Vec<String>` (ordered,
///  linear lookup) and `HashSet<&str>` (borrowed names that could not
///       outlive the decoded file).
/// Why:
///   Tests and development tools check a runtime directory directly;
///  the application reads its
///       embedded manifest through [`provisioned_grammars_from`] instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provisionedGrammars(manifest: string): Set<string> {
///   return provisionedGrammarsFrom(readFileSync(manifest, 'utf8'), manifest, DIRECTORY_REMEDY);
/// }
/// ```
pub fn provisioned_grammars(manifest: &Path) -> Result<HashSet<String>> {
    let remedy = runtime::DIRECTORY_REMEDY;
    // What: `fs::read_to_string` returns `Result<String, io::Error>`; `.context(...)`
    //       attaches a message only when reading failed, and the error returns to the caller early.
    // Why:  A missing manifest means language assets were never prepared, and the diagnostic
    //       must name the file and the remedy rather than a bare operating-system error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let encoded: string;
    // try { encoded = readFileSync(manifest, 'utf8'); }
    // catch (error) { throw new Error(`Cannot read ${manifest} ...`, { cause: error }); }
    // ```
    let encoded = match fs::read_to_string(manifest) {
        Ok(text) => text,
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!(
                "Cannot read the language manifest {}. {remedy}",
                manifest.display()
            )));
        }
    };
    // `&encoded` lends the text; the location names the file in later messages.
    return provisioned_grammars_from(&encoded, &manifest.display().to_string(), remedy);
}

/// What:
///  Decode manifest text into the set of grammar ids it lists.
///  `location` names where the text
///       came from and `remedy` what the reader should do,
///  both only for error messages.
/// Why:
///   The manifest names exactly the grammars this build ships.
///  A language outside it is
///       unsupported and stays plain text;
///  a listed grammar that then fails to load is a
///       broken installation and stays a visible failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provisionedGrammarsFrom(encoded: string, location: string, remedy: string): Set<string> {
///   const { grammars } = JSON.parse(encoded) as Manifest;
///   return new Set(grammars.map(file => file.slice(0, -'.so'.length)));
/// }
/// ```
pub fn provisioned_grammars_from(
    encoded: &str,
    location: &str,
    remedy: &str,
) -> Result<HashSet<String>> {
    // A wrong shape returns early with the location and the remedy.
    let decoded: Manifest = match serde_json::from_str(encoded) {
        Ok(manifest) => manifest,
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!(
                "Cannot decode the language manifest {location}. {remedy}"
            )));
        }
    };
    // What: `HashSet::new()` creates an empty set; `mut` allows the loop to insert into it.
    // Why:  Later lookups ask only "is this grammar bundled?".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const names = new Set<string>();
    // ```
    let mut names = HashSet::new();
    for file in decoded.grammars {
        // What: `strip_suffix(".so")` returns `Some(rest)` when the name ends in `.so` and
        //       `None` otherwise; `let Some(name) = ... else { ... }` extracts `rest` or runs
        //       the `else` block, which must leave the function.
        // Why:  The manifest lists shared-object file names, while Helix addresses grammars by
        //       bare id; any other entry is an unexpected manifest and must not be skipped.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!file.endsWith('.so')) throw new Error(`unexpected entry ${file}`);
        // const name = file.slice(0, -'.so'.length);
        // ```
        let Some(name) = file.strip_suffix(".so") else {
            bail!(
                "Cannot use the language manifest {location}: entry {file} is not a grammar shared object. {remedy}"
            );
        };
        // `to_string` copies the borrowed slice into an owned value the set can keep.
        names.insert(name.to_string());
    }
    // `Ok(...)` is the success variant of `Result`: hand the completed set to the caller.
    return Ok(names);
}

/// Merge touching intervals with identical paint roles,
///  not unrelated syntax nodes.
fn append_span(spans: &mut Vec<StyleSpan>, incoming: StyleSpan) {
    // last_mut lends only the final owned interval; source text remains immutable.
    // Extract a present interval only when its paint role and boundary both match.
    if let Some(previous) = spans.last_mut()
        && previous.end == incoming.start
        && previous.style == incoming.style
    {
        previous.end = incoming.end;
        return;
    }
    spans.push(incoming);
}

/// Own the language registry and its lazy grammar/query caches on the source worker.
pub struct SyntaxEngine {
    /// Built-in recognition rules plus languages compiled from the application's own runtime only;
    /// project-local and user Helix configuration is never loaded.
    loader: OwnLoader,
}

/// Translate pinned language recognition and highlighting into source-character paint ranges.
impl SyntaxEngine {
    /// Initialize language recognition from the manifest of the process's language runtime
    /// (embedded in the application binary;
    ///  a directory for tests and tools) without starting
    /// servers or downloading assets.
    pub fn new() -> Result<Self> {
        // The trailing `?` returns the "no runtime" error when this program has none.
        let source = runtime::current()?;
        let (encoded, location) = source.manifest()?;
        // Lend the text; a malformed manifest returns its diagnostic early.
        let provisioned = provisioned_grammars_from(&encoded, &location, source.remedy())?;
        tracing::debug!(source = %source.describe(), grammars = provisioned.len(), "language runtime manifest read");
        return Self::with_provisioned(provisioned);
    }

    /// Initialize language recognition for an explicit set of bundled grammar ids.
    /// `new` passes the manifest's set;
    ///  tests pass sets the manifest would not produce.
    pub fn with_provisioned(provisioned: HashSet<String>) -> Result<Self> {
        // What: try_into deserializes the pinned loader's TOML value into its own config type.
        // Why: Keep parser configuration aligned with the selected Helix revision.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const config = parseLanguageConfig(helix.defaultLanguageConfig());
        // ```
        let config: Configuration = helix_loader::config::default_lang_config()
            .try_into()
            .context("Cannot decode the bundled language configuration")?;
        // Helix numbers languages in table order; the own loader keeps one cell per number.
        let count = config.language.len();
        let helix =
            Loader::new(config).context("Cannot prepare bundled filename language rules")?;
        return Ok(Self {
            loader: OwnLoader::new(helix, count, provisioned, SCOPES),
        });
    }

    /// What:
    ///  Say whether the built-in language rules name a language for this file,
    ///  by filename or
    ///       shebang,
    ///  without reading the bundled manifest or any grammar.
    ///  `Result<bool>` is the answer
    ///       or the error of decoding the compiled-in rules.
    /// Why:
    ///  When highlighting cannot start,
    ///  a file no language applies to loses nothing,
    ///  so that is
    ///      not a failure worth a warning;
    ///  a file some language applies to loses its highlighting.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static namesALanguage(path: string, text: Rope): boolean
    /// ```
    pub fn names_a_language(path: &Path, text: &Rope) -> Result<bool> {
        // An empty grammar set: only the compiled-in filename and shebang rules are consulted.
        let rules = Self::with_provisioned(HashSet::new())?;
        // What: the shebang is tried only when the filename named nothing; `is_some` asks
        //       whether either found a language.
        // Why: The same order `recognize` uses, without its check for a bundled grammar.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return (byFilename(path) ?? byShebang(text)) !== undefined;
        // ```
        let mut found = rules.loader.helix.language_for_filename(path);
        if found.is_none() {
            found = rules.loader.helix.language_for_shebang(text.slice(..));
        }
        return Ok(found.is_some());
    }

    /// Recognize a bundled language by filename,
    ///  then by shebang.
    /// None means the registry does not know the file or this build does not ship its grammar.
    fn recognize(&self, path: &Path, text: &Rope) -> Option<Language> {
        // What: the trailing `?` on an `Option` returns `None` from this function when the
        //       registry recognized nothing, and otherwise yields the contained language handle.
        // Why:  An unknown filename and shebang is ordinary plain text, not a failure.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const language = byFilename(path) ?? byShebang(text);
        // if (language === undefined) return undefined;
        // ```
        let mut found = self.loader.helix.language_for_filename(path);
        if found.is_none() {
            found = self.loader.helix.language_for_shebang(text.slice(..));
        }
        let language = found?;
        let config = self.loader.helix.language(language).config();
        // What: `as_deref()` views `Option<String>` as `Option<&str>` without copying;
        //       `unwrap_or(&config.language_id)` falls back to the language's own id.
        // Why:  Helix loads the parser named by `grammar` when present (`qml` uses `qmljs`,
        //       `jsonc` uses `json`), so provisioning is decided per grammar, not per language.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const grammar = config.grammar ?? config.languageId;
        // ```
        let grammar = config.grammar.as_deref().unwrap_or(&config.language_id);
        if !self.loader.provisioned.contains(grammar) {
            tracing::debug!(
                path = %path.display(),
                language = %config.language_id,
                grammar,
                "recognized language has no bundled grammar; source stays plain text"
            );
            // `None` is the absent variant of `Option`: no bundled language applies.
            return None;
        }
        // `Some(...)` is the present variant: this handle addresses the bundled language.
        return Some(language);
    }

    /// Name the bundled Helix language a file is read as,
    ///  such as `jsonc` for `tsconfig.json`.
    /// None means plain text:
    ///  an unknown file,
    ///  or a language this build does not ship.
    pub fn language_id(&self, path: &Path, text: &Rope) -> Option<String> {
        let language = self.recognize(path, text)?;
        // What: `.clone()` duplicates the registry's owned id into a new `String`.
        //       Sibling: `&str`, a borrowed view tied to this engine's lifetime.
        // Why:  Callers on other threads may keep the id after the engine moves or drops.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return loader.language(language).config.languageId;
        // ```
        return Some(
            self.loader
                .helix
                .language(language)
                .config()
                .language_id
                .clone(),
        );
    }

    /// Load the grammar and compile the highlighting rules of one registry language by id,
    /// reporting why it fails.
    ///  Ok(false) means the id is unknown or its grammar is not bundled.
    /// Checks of a runtime use this to prove every bundled language compiles.
    pub fn compile_language(&self, language_id: &str) -> Result<bool> {
        // What: `language_for_name(language_id.to_string())` looks the id up in Helix's table.
        // Why: The table compares owned names.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const language = helix.languageForName(languageId); if (!language) return false;
        // ```
        let Some(language) = self.loader.helix.language_for_name(language_id.to_string()) else {
            return Ok(false);
        };
        let config = self.loader.helix.language(language).config();
        let grammar = config.grammar.as_deref().unwrap_or(&config.language_id);
        if !self.loader.provisioned.contains(grammar) {
            return Ok(false);
        }
        // What: `anyhow!(...)` turns the stored message into an error value.
        // Why: The caller collects every failing language with its reason.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // loader.prepared(language); // throws `${languageId}: ${reason}`
        // ```
        if let Err(reason) = self.loader.prepared(language) {
            return Err(anyhow!("{language_id}: {reason}"));
        }
        return Ok(true);
    }

    /// Return None when the registry does not recognize the filename or shebang,
    /// or when this build does not ship the recognized language's grammar.
    /// Bundled languages with missing,
    ///  damaged,
    ///  or incompatible assets return a visible failure.
    /// Returned intervals are sorted,
    ///  non-overlapping,
    ///  and merge adjacent identical paint roles.
    pub fn highlight(&self, path: &Path, text: &Rope) -> Result<Option<SourceStyles>> {
        let Some(language) = self.recognize(path, text) else {
            return Ok(None);
        };
        let name = &self.loader.helix.language(language).config().language_id;
        tracing::debug!(path = %path.display(), language = %name, "preparing source syntax");
        // Compile first so a missing or damaged runtime file is reported with its own reason.
        if let Err(reason) = self.loader.prepared(language) {
            tracing::warn!(path = %path.display(), language = %name, %reason, "bundled language could not be prepared");
            bail!(
                "Cannot highlight {} as {name}. The bundled parser or highlighting rules could not be loaded: {reason}",
                path.display()
            );
        }
        let syntax = match Syntax::new(text.slice(..), language, PARSE_TIMEOUT, &self.loader) {
            Ok(parsed) => parsed,
            Err(error) => return Err(parser_failure(path, name, error)),
        };
        let length = match u32::try_from(text.len_bytes()) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Err(anyhow::Error::new(error).context(format!(
                    "Cannot highlight {}: source exceeds the parser byte-offset limit",
                    path.display()
                )));
            }
        };
        let mut highlighter = Highlighter::new(&syntax, text.slice(..), &self.loader, ..);
        let mut spans = Vec::new();
        let mut start = 0;
        while start < length {
            let end = highlighter.next_event_offset().min(length);
            if start < end {
                // The last active capture is the most specific current source classification.
                if let Some(highlight) = highlighter.active_highlights().next_back() {
                    append_span(
                        &mut spans,
                        StyleSpan {
                            start: text.byte_to_char(start as usize),
                            end: text.byte_to_char(end as usize),
                            style: highlight.idx() + 1,
                        },
                    );
                }
            }
            if end == length {
                break;
            }
            // advance updates the engine's active set; no duplicate external highlight stack is needed.
            highlighter.advance();
            start = end;
        }
        tracing::debug!(path = %path.display(), spans = spans.len(), "source syntax prepared");
        return Ok(Some(SourceStyles::from(spans)));
    }
}
