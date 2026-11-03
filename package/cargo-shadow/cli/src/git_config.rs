//! What:
//! A reader of Git configuration files for the few keys the trust base needs
//! (`branch.<name>.remote`,
//! `branch.<name>.merge`,
//! `remote.<name>.fetch`,
//! `extensions.*`),
//! written without starting Git.
//! Why:
//! The owner's rules are "Invoke real git as rarely as possible."
//! and "embed instead of spawning"
//! (`package/git-policy/cli/SPEC.md`),
//! and the shadow runs before every cargo command.
//! The reader answers only what it reads exactly:
//! text it cannot read exactly,
//! and every `include` or `includeIf` section (those pull in other files and conditions),
//! make it say "ask Git" instead of guessing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-config.ts
//! export type ConfigEntry = { section: string; subsection?: string; key: string; value: string };
//! export type ParsedConfig = { entries: ConfigEntry[]; hasInclude: boolean };
//! export function parseConfig(text: string): ParsedConfig  // throws on text it cannot read exactly
//! ```

/// What:
/// `#[derive(Clone, Debug, Eq, PartialEq)]` asks the compiler to write copy,
/// debug-print and equality code for the type below.
/// Why:
/// Tests compare parsed entries with `assert_eq!`,
/// and callers keep copies of the values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects compare with a helper (`deepStrictEqual`) and copy with `structuredClone`; nothing is declared.
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigEntry {
    /// The section name,
    /// lowercased (Git compares section names without case).
    pub section: String,
    /// The subsection name when the header had one,
    /// exactly as written (Git compares subsections with case).
    pub subsection: Option<String>,
    /// The key,
    /// lowercased (Git compares key names without case).
    pub key: String,
    /// The value after quoting,
    /// escapes and comments were applied.
    pub value: String,
}

/// The entries of one file,
/// in file order,
/// and whether the file asks Git to read another file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParsedConfig {
    /// Every `key = value` line,
    /// in file order.
    pub entries: Vec<ConfigEntry>,
    /// A section named `include` or `includeIf` appeared,
    /// so this file's meaning depends on files not read here.
    pub has_include: bool,
}

/// The lookups on a parsed configuration file:
/// every value of a key,
/// the last one,
/// and the merge of a later file.
impl ParsedConfig {
    /// What:
    /// `pub fn values(&self, ...) -> Vec<&str>` lists every value of one key in file order.
    /// `&self` borrows the parsed file read-only;
    /// the returned `&str` values borrow from it (sibling:
    /// `String`,
    /// which would copy).
    /// Why:
    /// A key such as `remote.origin.fetch` may appear several times and Git keeps all of them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// values(section: string, subsection: string | undefined, key: string): string[]
    /// ```
    pub fn values(&self, section: &str, subsection: Option<&str>, key: &str) -> Vec<&str> {
        // What:     `let mut x: Vec<T> = Vec::new();` makes an empty growable list that can be changed (`mut`); `Vec<&str>` holds borrowed text (sibling: `Vec<String>`, which owns it).
        // Why:      The matching values are collected one by one below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const found: string[] = [];
        // ```
        let mut found: Vec<&str> = Vec::new();
        // What:     `for item in &list { ... }` visits each element by reference; `&list` lends the list read-only.
        // Why:      Every entry is tested; none is changed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const entry of this.entries) { ... }
        // ```
        for entry in &self.entries {
            // What:     `a == b` compares two values; `.as_deref()` turns `Option<String>` into `Option<&str>` so it can be compared with the borrowed `subsection`.
            // Why:      Section and key were lowercased when parsed, so the arguments are lowercased here too; the subsection is compared as written.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (entry.section === section.toLowerCase() && entry.subsection === subsection && entry.key === key.toLowerCase())
            // ```
            if entry.section == section.to_lowercase() && entry.subsection.as_deref() == subsection && entry.key == key.to_lowercase() {
                found.push(entry.value.as_str());
            }
        }
        // What:     A last line without `;` and without `return` is the function's tail expression: its value is what the function returns (this crate writes `return` explicitly, so it is spelled out).
        // Why:      Hand the collected values to the caller.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return found;
        // ```
        return found;
    }

    /// What:
    /// `pub fn last(&self, ...) -> Option<&str>` is the final value of one key,
    /// or `None` when the key never appears.
    /// `Option<&str>` is "a borrowed text or nothing" (a TS `string | undefined`).
    /// Why:
    /// For a single-valued key Git uses the last assignment,
    /// so a later file or line overrides an earlier one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// last(section: string, subsection: string | undefined, key: string): string | undefined
    /// ```
    pub fn last(&self, section: &str, subsection: Option<&str>, key: &str) -> Option<&str> {
        // `.last()` takes the final element of the list as an `Option` (`None` for an empty list).
        return self.values(section, subsection, key).last().copied();
    }

    /// Appends the entries of a later file,
    /// which override earlier single-valued keys,
    /// and keeps the include flag.
    pub fn extend_with(&mut self, later: ParsedConfig) {
        self.entries.extend(later.entries);
        self.has_include = self.has_include || later.has_include;
    }
}

/// What:
/// `fn read_header(rest: &str) -> Result<(String, Option<String>, &str), String>` reads a section header after its `[`:
/// the section name,
/// the subsection if any,
/// and the text after the closing `]`.
/// `Result<T, String>` is "a value,
/// or an error message" (the Rust form of a function that throws).
/// Why:
/// Git writes `[branch "main"]`;
/// the older `[branch.main]` spelling means the same with a lowercased subsection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readHeader(rest: string): { section: string; subsection?: string; after: string }  // throws on a bad header
/// ```
fn read_header(rest: &str) -> Result<(String, Option<String>, &str), String> {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits; `Some(x)` / `None` are the "present" and "absent" cases of an `Option`; the compiler checks that every case is covered.
    // Why:      A header without a closing bracket is not a header, and that is reported instead of read past.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const close = rest.indexOf(']'); if (close === -1) throw new Error('...');
    // ```
    let close: usize = match rest.find(']') {
        // `Some`: the present or the absent value of an `Option`.
        Some(position) => position,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        None => return Err(format!("a section header has no closing bracket: [{rest}")),
    };
    // What:     `&rest[..close]` is a borrowed slice of the text up to (not including) byte `close`; `.trim()` drops surrounding whitespace.
    // Why:      The header's inner text holds the name and, in the quoted form, the subsection.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const inner = rest.slice(0, close).trim();
    // ```
    let inner: &str = rest[..close].trim();
    // Borrow `&rest`: lends the value without copying or giving it away.
    // Typed binding: `after: &str`.
    let after: &str = &rest[close + 1..];
    // The quoted form: `section "subsection"`.
    if let Some(space) = inner.find(char::is_whitespace) {
        // Typed binding: `section: String`.
        let section: String = inner[..space].to_lowercase();
        // Typed binding: `quoted: &str`.
        let quoted: &str = inner[space..].trim();
        // What:     `strip_prefix` / `strip_suffix` return the text without the given end, as an `Option`; `?` is not used here because the failure needs a message.
        // Why:      The subsection must be wrapped in double quotes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!quoted.startsWith('"') || !quoted.endsWith('"')) throw new Error('...');
        // ```
        let unquoted: Option<&str> = match quoted.strip_prefix('"') {
            // `Some`: the present or the absent value of an `Option`.
            Some(text) => text.strip_suffix('"'),
            // `None`: the present or the absent value of an `Option`.
            None => None,
        };
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `body: &str`.
        let body: &str = match unquoted {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            None => return Err(format!("a subsection name is not in double quotes: [{inner}]")),
        };
        // `let mut`: a binding that may change.
        // Typed binding: `subsection: String`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        let mut subsection: String = String::new();
        // `let mut`: a binding that may change.
        let mut characters = body.chars();
        // `while let Some(c) = iterator.next()` runs until the iterator is used up.
        while let Some(character) = characters.next() {
            if character == '\\' {
                // Inside a subsection name only `\"` and `\\` are escapes; a backslash before any other character is dropped.
                if let Some(escaped) = characters.next() {
                    subsection.push(escaped);
                }
            } else {
                subsection.push(character);
            }
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        return Ok((section, Some(subsection), after));
    }
    // The older form: `section.subsection`, whose subsection is lowercased.
    if let Some(dot) = inner.find('.') {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        return Ok((inner[..dot].to_lowercase(), Some(inner[dot + 1..].to_lowercase()), after));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    return Ok((inner.to_lowercase(), None, after));
}

/// What:
/// `fn read_value(raw: &str) -> Result<String, String>` turns the text after `=` into a value:
/// whitespace trimmed,
/// double quotes grouping,
/// backslash escapes applied,
/// and an unquoted `#` or `;` starting a comment.
/// Why:
/// A value such as `+refs/heads/*:refs/remotes/origin/*` must come out exactly as Git reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readValue(raw: string): string  // throws on an unknown escape or an unclosed quote
/// ```
fn read_value(raw: &str) -> Result<String, String> {
    // `let mut`: a binding that may change.
    // Typed binding: `value: String`.
    // Constructor `String::new()`: an empty owned collection or a boxed value.
    let mut value: String = String::new();
    // `let mut`: a binding that may change.
    // Typed binding: `in_quotes: bool`.
    let mut in_quotes: bool = false;
    // `let mut`: a binding that may change.
    let mut characters = raw.trim().chars();
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    while let Some(character) = characters.next() {
        if character == '\\' {
            // Git's escapes in a value: `\n`, `\t`, `\b`, `\\` and `\"`.
            let escaped: char = match characters.next() {
                // `Some`: the present or the absent value of an `Option`.
                Some('n') => '\n',
                // `Some`: the present or the absent value of an `Option`.
                Some('t') => '\t',
                // `Some`: the present or the absent value of an `Option`.
                Some('b') => '\u{8}',
                // `Some`: the present or the absent value of an `Option`.
                Some('\\') => '\\',
                // `Some`: the present or the absent value of an `Option`.
                Some('"') => '"',
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `Some`: the present or the absent value of an `Option`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                Some(other) => return Err(format!("an unknown escape \\{other} in a configuration value")),
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `None`: the present or the absent value of an `Option`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                None => return Err("a configuration value ends with a lone backslash".to_string()),
            };
            value.push(escaped);
        } else if character == '"' {
            in_quotes = !in_quotes;
        } else if !in_quotes && (character == '#' || character == ';') {
            break;
        } else {
            value.push(character);
        }
    }
    if in_quotes {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a configuration value has an unclosed double quote".to_string());
    }
    // Whitespace before an unquoted comment was collected into the value; Git drops trailing whitespace outside quotes.
    return Ok(value.trim_end().to_string());
}

/// What:
/// `fn is_valid_key(key: &str) -> bool` says whether a text is a configuration key:
/// a letter followed by letters,
/// digits and hyphens.
/// Why:
/// A key outside that grammar is text this reader does not read exactly,
/// so the parser reports it instead of storing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isValidKey = (key: string): boolean => /^[A-Za-z][A-Za-z0-9-]*$/.test(key);
/// ```
fn is_valid_key(key: &str) -> bool {
    // `let mut`: a binding that may change.
    let mut characters = key.chars();
    // `match`: picks the arm whose pattern fits the value.
    match characters.next() {
        // `Some`: the present or the absent value of an `Option`.
        Some(first) if first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    for character in characters {
        if !character.is_ascii_alphanumeric() && character != '-' {
            return false;
        }
    }
    return true;
}

/// What:
/// `pub fn parse_config(text: &str) -> Result<ParsedConfig, String>` parses one configuration file.
/// Why:
/// The caller needs the entries of `.git/config` and of the user's configuration files,
/// and needs to know whether any of them includes another file.
/// A line continued with a trailing backslash is joined to the next,
/// as Git does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseConfig(text: string): ParsedConfig  // throws on text it cannot read exactly
/// ```
pub fn parse_config(text: &str) -> Result<ParsedConfig, String> {
    // `let mut`: a binding that may change.
    // Typed binding: `parsed: ParsedConfig`.
    let mut parsed: ParsedConfig = ParsedConfig::default();
    // `None`: the present or the absent value of an `Option`.
    // `let mut`: a binding that may change.
    // Typed binding: `section: Option<(String, Option<String>)>`.
    let mut section: Option<(String, Option<String>)> = None;
    // Join continuation lines first, so every logical line is one string.
    let mut logical: Vec<String> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `pending: String`.
    // Constructor `String::new()`: an empty owned collection or a boxed value.
    let mut pending: String = String::new();
    for physical in text.lines() {
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(without) = physical.strip_suffix('\\') {
            pending.push_str(without);
            continue;
        }
        pending.push_str(physical);
        // Borrow `&mut pending`: lends the value without copying or giving it away.
        logical.push(std::mem::take(&mut pending));
    }
    if !pending.is_empty() {
        logical.push(pending);
    }
    // Borrow `&logical`: lends the value without copying or giving it away.
    for line in &logical {
        // `let mut`: a binding that may change.
        // Typed binding: `rest: &str`.
        let mut rest: &str = line.trim_start();
        if rest.is_empty() || rest.starts_with('#') || rest.starts_with(';') {
            continue;
        }
        // A header may be followed by a key on the same line.
        if let Some(after_bracket) = rest.strip_prefix('[') {
            // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
            // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
            // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const value = await expr;   // throws to the caller on failure
            // ```
            let (name, subsection, after) = read_header(after_bracket)?;
            if name == "include" || name == "includeif" {
                parsed.has_include = true;
            }
            // `Some`: the present or the absent value of an `Option`.
            section = Some((name, subsection));
            rest = after.trim_start();
            if rest.is_empty() || rest.starts_with('#') || rest.starts_with(';') {
                continue;
            }
        }
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&section`: lends the value without copying or giving it away.
        let (section_name, subsection_name) = match &section {
            // `Some`: the present or the absent value of an `Option`.
            // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
            // Why:      The copy can be stored or moved elsewhere while the original stays in place.
            // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // structuredClone(value)  // or [...list] / {...object}
            // ```
            Some(found) => found.clone(),
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            None => return Err("a configuration line appears before any section header".to_string()),
        };
        // `key = value`, or a bare `key` that means true.
        let (key, value) = match rest.split_once('=') {
            // `Some`: the present or the absent value of an `Option`.
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            Some((key_text, value_text)) => (key_text.trim(), read_value(value_text)?),
            // `None`: the present or the absent value of an `Option`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            None => (rest.trim(), "true".to_string()),
        };
        // A key is a letter followed by letters, digits and hyphens.
        if !is_valid_key(key) {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("a configuration key is not valid: {key:?}"));
        }
        parsed.entries.push(ConfigEntry { section: section_name, subsection: subsection_name, key: key.to_lowercase(), value });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(parsed);
}
