//! The highlighting language loader that reads only the application's own runtime.
//!
//! What:
//!  [`OwnLoader`] answers the two questions the parser library (`tree-house`,
//!  Helix's syntax
//!       crate) asks while parsing and highlighting:
//!  which language an injection marker names,
//!  and
//!       the compiled grammar and queries of a language.
//!  Recognition comes from Helix's built-in
//!       language table;
//!  grammar libraries and query texts come from [`crate::runtime`].
//! Why:
//!  Helix's own loader (`helix_core::syntax::Loader`) compiles languages through
//!      `helix_loader::runtime_file`,
//!  which prefers `$XDG_CONFIG_HOME/helix/runtime` over any other
//!      directory.
//!  A stray empty `highlights.scm` there turned SQL into plain text (measured on
//!      2026-10-05:
//!  16 colored spans without it,
//!  0 with it).
//!  Implementing the loader here keeps every
//!      read inside the application's own runtime.

/// The only source of grammar libraries and query texts.
use crate::runtime;
/// Helix's built-in language table,
///  used for recognition only.
use helix_core::syntax::Loader;
/// One lazily filled cell per language,
///  and the set of bundled grammar names.
use std::{cell::OnceCell, collections::HashSet};
/// The parser library's loader interface,
///  compiled language type,
///  palette entry,
///  and grammar loader.
use tree_house::{
    InjectionLanguageMarker, Language, LanguageConfig, LanguageLoader, highlighter::Highlight,
    tree_sitter::Grammar,
};

/// What:
///  Helix's built-in language table plus one lazily compiled configuration per language.
///       `OnceCell` holds a value computed on first use and then only read,
///  on one thread (sibling:
///       `OnceLock`,
///  the same for several threads).
///  `Result<LanguageConfig, String>` keeps either the
///       compiled language or the message explaining why it could not be compiled.
/// Why:
///  Compiling every language at startup would load every parser library;
///  compiling on first use
///      loads only what the displayed files need,
///  and a failure is reported once,
///  not retried per file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnLoader = { helix: Loader; provisioned: Set<string>; scopes: string[];
///   configs: Array<{ value?: LanguageConfig | Error }> };
/// ```
pub(crate) struct OwnLoader {
    /// Filename,
    ///  shebang,
    ///  and injection-name rules;
    ///  never used to compile a language.
    pub(crate) helix: Loader,
    /// Grammar names the runtime manifest lists;
    ///  other grammars stay plain text.
    pub(crate) provisioned: HashSet<String>,
    /// Highlight names the engine paints,
    ///  in palette order.
    scopes: &'static [&'static str],
    /// One cell per language of `helix`,
    ///  in the same order as its language numbers.
    configs: Vec<OnceCell<Result<LanguageConfig, String>>>,
}

/// What:
///  The palette index for one capture name such as `keyword.control.import`:
///  the scope sharing
///       the most leading dot-separated parts wins.
///  `Option<Highlight>` is a palette entry or none.
/// Why:
///  The same rule as Helix's private `reconfigure_highlights`,
///  so colors match what the engine
///      painted before this loader existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bestScope(capture: string, scopes: string[]): number | undefined
/// ```
fn best_scope(capture: &str, scopes: &[&str]) -> Option<Highlight> {
    // What: `split('.').collect()` gathers the capture's parts into a list; `Vec<&str>` lends each part.
    // Why: Parts are compared position by position against each scope.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const captureParts = capture.split('.');
    // ```
    let capture_parts: Vec<&str> = capture.split('.').collect();
    let mut best: Option<u32> = None;
    let mut best_length = 0;
    // What: `zip(0u32..)` pairs each scope with its index counted as `u32` (sibling `usize`).
    // Why: Palette entries are numbered with `u32` in the parser library.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // scopes.forEach((scope, index) => { ... });
    // ```
    for (scope, index) in scopes.iter().zip(0u32..) {
        let mut length = 0;
        let mut matches = true;
        for (position, part) in scope.split('.').enumerate() {
            // What: `capture_parts.get(position) == Some(&part)` is true when the capture has the
            //       same part at that position; `Some(&part)` lends the part for the comparison.
            // Why: A scope matches only as a whole prefix of the capture.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (captureParts[position] === part) length++; else { matches = false; break; }
            // ```
            if capture_parts.get(position) == Some(&part) {
                length += 1;
            } else {
                matches = false;
                break;
            }
        }
        if matches && length > best_length {
            best = Some(index);
            best_length = length;
        }
    }
    // What: `.map(Highlight::new)` wraps the winning index in the palette-entry type.
    // Why: The parser library expects `Highlight` values from the configure callback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return best;
    // ```
    return best.map(Highlight::new);
}

/// What:
///  An error rendered with its whole chain of causes,
///  as the `{:#}` format shows it.
/// Why:
///  The loader reports failures as plain text,
///  so each fallible step converts its error with this one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alternateText = (error: Error): string => String(error);
/// ```
fn alternate_text<Failure: std::fmt::Display>(error: Failure) -> String {
    return format!("{error:#}");
}

/// What:
///  Read one query kind of a language,
///  following `; inherits:` lines into other languages'
///       files.
///  `tree_house::read_query` calls the arrow function once per language it needs.
/// Why:
///  That callback cannot return an error,
///  so the first failure is kept aside and returned after.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readRules(source: RuntimeSource, language: string, file: string): string // throws
/// ```
fn read_rules(
    source: &runtime::RuntimeSource,
    language: &str,
    file: &str,
) -> Result<String, String> {
    let mut failure: Option<String> = None;
    // What: `|name| match ... { ... }` is an arrow function returning query text or, after recording
    //       the failure, empty text. `String::new()` makes empty owned text.
    // Why: Inherited languages are read through the same source as the language itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = readQuery(language, name => { try { return source.query(name, file); }
    //   catch (error) { failure ??= String(error); return ''; } });
    // ```
    let text = tree_house::read_query(
        language,
        #[cfg_attr(
            dylint_lib = "monochromatic_lint_closure",
            expect(
                avoidable_closure,
                reason = "tree_house::read_query accepts only an FnMut(&str) -> String reader, and this one records the first failure in the caller's variable, which a named function cannot reach"
            )
        )]
        |name| {
            return match source.query(name, file) {
                Ok(text) => text,
                Err(error) => {
                    if failure.is_none() {
                        failure = Some(format!("{error:#}"));
                    }
                    String::new()
                }
            };
        },
    );
    if let Some(reason) = failure {
        return Err(reason);
    }
    return Ok(text);
}

/// Construction and per-language compilation.
impl OwnLoader {
    /// What:
    ///  Wrap Helix's language table;
    ///  `count` is its number of languages.
    /// Why:
    ///  The cells must line up with Helix's language numbers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(helix: Loader, count: number, provisioned: Set<string>, scopes: string[])
    /// ```
    pub(crate) fn new(
        helix: Loader,
        count: usize,
        provisioned: HashSet<String>,
        scopes: &'static [&'static str],
    ) -> Self {
        // What: `(0..count).map(|_| OnceCell::new()).collect()` builds a list of `count` empty cells.
        // Why: `vec![cell; count]` would need cells to be copyable, which an unfilled cell is not.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const configs = Array.from({ length: count }, () => ({}));
        // ```
        let mut configs: Vec<OnceCell<Result<LanguageConfig, String>>> = Vec::new();
        for _ in 0..count {
            configs.push(OnceCell::new());
        }
        return Self {
            helix,
            provisioned,
            scopes,
            configs,
        };
    }

    /// What:
    ///  Load the grammar library and compile the three query kinds of one language.
    /// Why:
    ///  Called once per language through its cell.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// compile(language: Language): LanguageConfig // throws a message
    /// ```
    fn compile(&self, language: Language) -> Result<LanguageConfig, String> {
        let config = self.helix.language(language).config();
        let name = &config.language_id;
        // What: `as_deref()` views `Option<String>` as `Option<&str>`; `unwrap_or(name)` uses the
        //       language's own name when no separate grammar is named.
        // Why: Several languages share one grammar, for example TSX's queries over the TSX grammar.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const grammarName = config.grammar ?? name;
        // ```
        let grammar_name = config.grammar.as_deref().unwrap_or(name);
        if !self.provisioned.contains(grammar_name) {
            tracing::debug!(language = %name, grammar = grammar_name, "language has no bundled grammar; it stays plain text");
            return Err(format!("the grammar {grammar_name} is not bundled"));
        }
        // What: `map_err(alternate_text)` turns any error into its message text.
        // Why: Cells keep plain text, which can be shown again for every file of the language.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const source = runtime.current(); // message on failure
        // ```
        let source = runtime::current().map_err(alternate_text)?;
        let library = source
            .grammar_library(grammar_name)
            .map_err(alternate_text)?;
        // SAFETY: `Grammar::new` opens a shared library and runs its initialization code. The library
        // is either the file in the prepared runtime directory of a test or tool program, or a cached
        // copy just compared byte for byte with the bytes the build embedded and digested.
        //
        // What: `unsafe { ... }` marks code whose correctness the compiler cannot check.
        // Why: Loading native parser code is the only way to use a compiled tree-sitter grammar.
        // Gotcha: TS has no equivalent; a damaged library here could crash the whole process, which
        //         is why the bytes are checked first.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const grammar = loadNativeLibrary(library).symbol(`tree_sitter_${grammarName}`);
        // ```
        let grammar = match unsafe { Grammar::new(grammar_name, &library) } {
            Ok(loaded) => loaded,
            Err(error) => {
                return Err(format!(
                    "cannot load the parser library {}: {error}. If its directory is on a file system that forbids running code (mounted noexec), point XDG_CACHE_HOME at another directory and restart the application",
                    library.display()
                ));
            }
        };
        let highlights = read_rules(source, name, "highlights.scm")?;
        let injections = read_rules(source, name, "injections.scm")?;
        let locals = read_rules(source, name, "locals.scm")?;
        let compiled = match LanguageConfig::new(grammar, &highlights, &injections, &locals) {
            Ok(configured) => configured,
            Err(error) => {
                return Err(format!(
                    "cannot compile the highlighting rules of {name}: {error}"
                ));
            }
        };
        compiled.configure(
            #[cfg_attr(
                dylint_lib = "monochromatic_lint_closure",
                expect(
                    avoidable_closure,
                    reason = "LanguageConfig::configure accepts only an FnMut(&str) -> Option<Highlight> callback, and this one reads the scope table of self, which a named function cannot reach"
                )
            )]
            |capture| return best_scope(capture, self.scopes),
        );
        tracing::debug!(language = %name, grammar = grammar_name, library = %library.display(), "language highlighting prepared from the application's own runtime");
        return Ok(compiled);
    }

    /// What:
    ///  The compiled language,
    ///  compiling it on first use.
    ///  `Result<&LanguageConfig, &String>`
    ///       lends the compiled value or the stored message.
    /// Why:
    ///  The engine reports the reason a displayed file's language failed;
    ///  injections only skip it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prepared(language: Language): LanguageConfig // throws the stored message
    /// ```
    pub(crate) fn prepared(&self, language: Language) -> Result<&LanguageConfig, String> {
        let Some(cell) = self.configs.get(language.idx()) else {
            return Err(format!("language number {} is unknown", language.idx()));
        };
        // What: `get_or_init(|| self.compile(language))` compiles once and keeps the result;
        //       `.as_ref()` lends either side; `.map_err(Clone::clone)` copies the message.
        // Why: Later files of the same language reuse the first outcome.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // cell.value ??= tryCompile(language); if (cell.value instanceof Error) throw cell.value; return cell.value;
        // ```
        // The cell is filled on first use; compiling never touches this cell, so the set cannot fail.
        if cell.get().is_none() {
            let compiled = self.compile(language);
            let _stored = cell.set(compiled);
        }
        let Some(outcome) = cell.get() else {
            return Err(format!("language number {} was not stored", language.idx()));
        };
        return outcome.as_ref().map_err(Clone::clone);
    }
}

/// What:
///  `impl LanguageLoader for OwnLoader` makes this type usable wherever the parser library
///       wants a loader,
///  like implementing a TS interface.
/// Why:
///  Parsing,
///  injections,
///  and highlighting all ask the loader,
///  so this is the single gate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class OwnLoader implements LanguageLoader { languageForMarker(...) {...} getConfig(...) {...} }
/// ```
impl LanguageLoader for OwnLoader {
    /// What:
    ///  Which language an injection marker (a name,
    ///  a filename,
    ///  a shebang) refers to.
    /// Why:
    ///  Recognition is Helix's built-in table;
    ///  it reads no runtime files.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// languageForMarker(marker: InjectionLanguageMarker): Language | undefined
    /// ```
    fn language_for_marker(&self, marker: InjectionLanguageMarker) -> Option<Language> {
        return self.helix.language_for_marker(marker);
    }

    /// What:
    ///  The compiled language,
    ///  or `None` when it cannot be compiled.
    /// Why:
    ///  An injected language that fails is left uncolored instead of failing the whole file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// getConfig(language: Language): LanguageConfig | undefined
    /// ```
    fn get_config(&self, lang: Language) -> Option<&LanguageConfig> {
        return self.prepared(lang).ok();
    }
}
