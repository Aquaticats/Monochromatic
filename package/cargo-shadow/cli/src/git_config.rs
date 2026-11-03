//! What:
//! A reader of Git configuration files for the few keys the trust base needs
//! (`branch.<name>.remote`,
//! `branch.<name>.merge`,
//! `remote.<name>.fetch`,
//! `core.repositoryformatversion`,
//! `extensions.*`),
//! written without starting Git.
//! Why:
//! The owner's rules are "Invoke real git as rarely as possible."
//! and "embed instead of spawning"
//! (`package/git-policy/cli/SPEC.md`),
//! and the shadow runs before every cargo command.
//! The reader follows the lexer of Git's `config.c` character by character
//! (a comment never continues onto the next line,
//! only a value does;
//! a quoted subsection is read to its closing quote before the bracket;
//! a key without `=` has no value at all),
//! and answers only what it reads exactly:
//! text it cannot read exactly,
//! and every `include` or `includeIf` section (those pull in other files and conditions),
//! make it say "ask Git" instead of guessing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-config.ts
//! export type ConfigEntry = { name: string; value?: string };  // name: 'section.subsection.key', value undefined for a bare key
//! export type ParsedConfig = { entries: ConfigEntry[]; hasInclude: boolean };
//! export function parseConfig(text: string): ParsedConfig  // throws on text it cannot read exactly
//! export function parseBool(value: string | undefined): boolean  // throws on a spelling it does not know
//! ```

/// One variable of a configuration file,
/// named the way Git names it:
/// the section lowercased,
/// the subsection exactly as written (lowercased only in the old dotted header form),
/// the key lowercased,
/// joined by dots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigEntry {
    /// The flat variable name,
    /// for example `branch.topic.merge`.
    pub name: String,
    /// The value,
    /// or `None` for a key written without `=`
    /// (Git calls that a missing value;
    /// it reads as true only where a boolean is asked for).
    pub value: Option<String>,
}

/// The variables of one or more configuration files,
/// in the order Git reads them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParsedConfig {
    /// Every variable,
    /// in file order.
    pub entries: Vec<ConfigEntry>,
    /// Whether an `include` or `includeIf` section was seen.
    pub has_include: bool,
}

/// What:
/// `pub fn variable_name(section: &str, subsection: Option<&str>, key: &str) -> String` builds the flat name Git uses for a variable.
/// Why:
/// Git compares section and key without case and the subsection with case,
/// and a dotted section with a quoted subsection is one flat name,
/// so lookups build the same flat name the lexer builds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const variableName = (section: string, subsection: string | undefined, key: string): string =>
///   [section.toLowerCase(), ...(subsection === undefined ? [] : [subsection]), key.toLowerCase()].join('.');
/// ```
pub fn variable_name(section: &str, subsection: Option<&str>, key: &str) -> String {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    return match subsection {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        Some(found) => format!("{}.{found}.{}", section.to_lowercase(), key.to_lowercase()),
        // `None`: the present or the absent value of an `Option`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        None => format!("{}.{}", section.to_lowercase(), key.to_lowercase()),
    };
}

/// The lookups of a parsed configuration.
impl ParsedConfig {
    /// What:
    /// `pub fn values(&self, section, subsection, key) -> Vec<Option<&str>>` lists every value of a variable in file order,
    /// with `None` for an occurrence written without a value.
    /// Why:
    /// Callers that need a value must see a bare occurrence as the error it is in Git,
    /// not as a string.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// values(section: string, subsection: string | undefined, key: string): (string | undefined)[]
    /// ```
    pub fn values(&self, section: &str, subsection: Option<&str>, key: &str) -> Vec<Option<&str>> {
        // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
        // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name: Type = value;
        // ```
        // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
        // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let wanted: String = variable_name(section, subsection, key);
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
        // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Array.from(iterable)  // or [...iterable]
        // ```
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      They express one step of a pipeline without an index loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        let mut found: Vec<Option<&str>> = Vec::new();
        for entry in &self.entries {
            if entry.name == wanted {
                found.push(entry.value.as_deref());
            }
        }
        return found;
    }

    /// What:
    /// `pub fn last(&self, section, subsection, key) -> Option<&str>` is the value of the last occurrence of a variable,
    /// or `None` when there is none or the last one has no value.
    /// Why:
    /// Single-valued variables take the last occurrence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// last(section: string, subsection: string | undefined, key: string): string | undefined
    /// ```
    pub fn last(&self, section: &str, subsection: Option<&str>, key: &str) -> Option<&str> {
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        return self.values(section, subsection, key).last().copied().flatten();
    }

    /// What:
    /// `pub fn extend_with(&mut self, later: ParsedConfig)` appends the variables of a file read after this one.
    /// Why:
    /// Later files override earlier ones for single-valued variables.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// extendWith(later: ParsedConfig): void
    /// ```
    pub fn extend_with(&mut self, later: ParsedConfig) {
        self.entries.extend(later.entries);
        self.has_include = self.has_include || later.has_include;
    }
}

/// What:
/// `pub fn parse_bool(value: Option<&str>) -> Result<bool, String>` reads a configuration value as Git's boolean:
/// no value or `true`,
/// `yes`,
/// `on` or a nonzero integer is true;
/// an empty value,
/// `false`,
/// `no`,
/// `off` or zero is false.
/// Why:
/// `extensions.worktreeConfig = yes` is as true as `true`;
/// a spelling the reader does not know is an error so the caller asks Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseBool(value: string | undefined): boolean  // throws on an unknown spelling
/// ```
pub fn parse_bool(value: Option<&str>) -> Result<bool, String> {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `text: &str`.
    let text: &str = match value {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // `None`: the present or the absent value of an `Option`.
        None => return Ok(true),
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
    };
    if text.is_empty() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(false);
    }
    // Typed binding: `lowered: String`.
    let lowered: String = text.to_ascii_lowercase();
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    if ["true", "yes", "on"].contains(&lowered.as_str()) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(true);
    }
    // Borrow `&lowered.as_str`: lends the value without copying or giving it away.
    if ["false", "no", "off"].contains(&lowered.as_str()) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(false);
    }
    // Borrow `&lowered`: lends the value without copying or giving it away.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // Typed binding: `digits: &str`.
    let digits: &str = lowered.strip_prefix(['-', '+']).unwrap_or(&lowered);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `number: &str`.
    let number: &str = digits.strip_suffix(['k', 'm', 'g']).unwrap_or(digits);
    // Closure `|character|`: an anonymous function, like a TS arrow function.
    if !number.is_empty() && number.as_bytes().iter().all(u8::is_ascii_digit) && !(number.len() > 1 && number.starts_with('0')) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Closure `|character|`: an anonymous function, like a TS arrow function.
        return Ok(!number.trim_start_matches('0').is_empty());
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Err(format!("{text:?} is not a boolean spelling this reader knows"));
}

/// How the lexer sees the text:
/// Git's `get_next_char`.
struct Lexer {
    /// The characters of the file,
    /// after the optional byte order mark.
    characters: Vec<char>,
    /// The index of the next character.
    position: usize,
    /// Whether the end of the text was reached;
    /// the end reads as one final newline.
    at_end: bool,
}

/// The character reader of the lexer.
impl Lexer {
    /// What:
    /// `fn next(&mut self) -> char` returns the next character,
    /// with `\r\n` as `\n` and the end of the text as a newline
    /// (and `at_end` set).
    /// Why:
    /// It is Git's `get_next_char`,
    /// so the rest of the lexer reads like Git's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// next(): string
    /// ```
    fn next(&mut self) -> char {
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Typed binding: `found: Option<char>`.
        let found: Option<char> = self.characters.get(self.position).copied();
        self.position += 1;
        // `match`: picks the arm whose pattern fits the value.
        return match found {
            // `None`: the present or the absent value of an `Option`.
            None => {
                self.at_end = true;
                '\n'
            }
            // `Some`: the present or the absent value of an `Option`.
            Some('\r') if self.characters.get(self.position) == Some(&'\n') => {
                self.position += 1;
                '\n'
            }
            // `Some`: the present or the absent value of an `Option`.
            Some(character) => character,
        };
    }
}

/// Git's `isspace` for configuration text:
/// space,
/// tab,
/// newline and carriage return only.
fn is_space(character: char) -> bool {
    // Macro `matches!`: builds or prints text (or a list) from a template.
    return matches!(character, ' ' | '\t' | '\n' | '\r');
}

/// Git's `iskeychar`:
/// ASCII letters,
/// digits and the dash.
fn is_key_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || character == '-';
}

/// What:
/// `fn read_extended_base(lexer, name, first) -> Result<(), String>` reads the quoted subsection of a header such as
/// `[branch "topic"]` after the section name,
/// appending a dot and the subsection exactly as written to `name`.
/// Why:
/// The subsection runs to the closing quote (a backslash makes the next character literal),
/// so a bracket,
/// a hash or an escaped quote inside it belong to the name,
/// and nothing but the closing bracket may follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readExtendedBase(lexer: Lexer, name: string[], first: string): void  // throws on an incomplete header
/// ```
fn read_extended_base(lexer: &mut Lexer, name: &mut String, first: char) -> Result<(), String> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `character: char`.
    let mut character: char = first;
    loop {
        if character == '\n' {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            return Err("a section header ends before its closing bracket".to_string());
        }
        character = lexer.next();
        if !is_space(character) {
            break;
        }
    }
    if character != '"' {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a subsection name is not in double quotes".to_string());
    }
    name.push('.');
    loop {
        // `let mut`: a binding that may change.
        // Typed binding: `inner: char`.
        let mut inner: char = lexer.next();
        if inner == '\n' {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Err("a subsection name ends before its closing quote".to_string());
        }
        if inner == '"' {
            break;
        }
        if inner == '\\' {
            inner = lexer.next();
            if inner == '\n' {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                return Err("a subsection name ends after a backslash".to_string());
            }
        }
        name.push(inner);
    }
    if lexer.next() != ']' {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a section header has something between the quoted subsection and its closing bracket".to_string());
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

/// What:
/// `fn read_base(lexer) -> Result<String, String>` reads a section header after its opening bracket and returns the flat stem
/// (`branch.topic` for `[branch "topic"]`,
/// lowercased only before a quoted subsection).
/// Why:
/// Every key that follows is named by this stem and a dot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readBase(lexer: Lexer): string  // throws on a header Git rejects
/// ```
fn read_base(lexer: &mut Lexer) -> Result<String, String> {
    // `let mut`: a binding that may change.
    // Typed binding: `name: String`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let mut name: String = String::new();
    loop {
        // Typed binding: `character: char`.
        let character: char = lexer.next();
        if lexer.at_end {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Err("a section header has no closing bracket".to_string());
        }
        if character == ']' {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(name);
        }
        if is_space(character) {
            // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
            // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
            // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const value = await expr;   // throws to the caller on failure
            // ```
            // Borrow `&mut name`: lends the value without copying or giving it away.
            read_extended_base(lexer, &mut name, character)?;
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(name);
        }
        if !is_key_character(character) && character != '.' {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("a section header holds {character:?}, which a section name cannot"));
        }
        name.push(character.to_ascii_lowercase());
    }
}

/// What:
/// `fn read_value(lexer) -> Result<String, String>` reads a value after the `=`,
/// the way Git's `parse_value` does.
/// Why:
/// Unquoted whitespace around the value is dropped and inside it kept character for character,
/// a quoted part keeps everything including trailing spaces,
/// a `#` or `;` outside quotes starts a comment that a backslash does not continue,
/// and a backslash at the end of a line continues the value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readValue(lexer: Lexer): string  // throws on an unknown escape or an unclosed quote
/// ```
fn read_value(lexer: &mut Lexer) -> Result<String, String> {
    let mut value: String = String::new();
    // `let mut`: a binding that may change.
    // Typed binding: `quoted: bool`.
    let mut quoted: bool = false;
    // `let mut`: a binding that may change.
    // Typed binding: `comment: bool`.
    let mut comment: bool = false;
    // `let mut`: a binding that may change.
    // Typed binding: `pending_spaces: usize`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let mut pending_spaces: usize = 0;
    loop {
        // `let mut`: a binding that may change.
        // Typed binding: `character: char`.
        let mut character: char = lexer.next();
        if character == '\n' {
            if quoted {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                return Err("a configuration value has an unclosed double quote".to_string());
            }
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(value);
        }
        if comment {
            continue;
        }
        if is_space(character) && !quoted {
            if !value.is_empty() {
                pending_spaces += 1;
            }
            continue;
        }
        if !quoted && (character == ';' || character == '#') {
            comment = true;
            continue;
        }
        value.push_str(&" ".repeat(pending_spaces));
        pending_spaces = 0;
        if character == '\\' {
            character = lexer.next();
            // `match`: picks the arm whose pattern fits the value.
            // Typed binding: `escaped: char`.
            let escaped: char = match character {
                '\n' => continue,
                't' => '\t',
                'b' => '\u{8}',
                'n' => '\n',
                '\\' | '"' => character,
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                other => return Err(format!("an unknown escape \\{other} in a configuration value")),
            };
            value.push(escaped);
            continue;
        }
        if character == '"' {
            quoted = !quoted;
            continue;
        }
        value.push(character);
    }
}

/// What:
/// `fn read_variable(lexer, stem, first) -> Result<ConfigEntry, String>` reads a key and its optional value,
/// the way Git's `get_value` does.
/// Why:
/// A key ends at the first character that is not a letter,
/// digit or dash;
/// after optional blanks a newline means no value,
/// an `=` starts the value,
/// and anything else is a syntax error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readVariable(lexer: Lexer, stem: string, first: string): ConfigEntry  // throws on a syntax error
/// ```
fn read_variable(lexer: &mut Lexer, stem: &str, first: char) -> Result<ConfigEntry, String> {
    // `let mut`: a binding that may change.
    // Typed binding: `name: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let mut name: String = format!("{stem}{}", first.to_ascii_lowercase());
    // `let mut`: a binding that may change.
    let mut character: char;
    loop {
        character = lexer.next();
        if lexer.at_end || !is_key_character(character) {
            break;
        }
        name.push(character.to_ascii_lowercase());
    }
    while character == ' ' || character == '\t' {
        character = lexer.next();
    }
    if character == '\n' {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        return Ok(ConfigEntry { name, value: None });
    }
    if character != '=' {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{name} is followed by {character:?} where a value or a newline is expected"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    return Ok(ConfigEntry { name, value: Some(read_value(lexer)?) });
}

/// What:
/// `pub fn parse_config(text: &str) -> Result<ParsedConfig, String>` reads one configuration file the way Git's lexer does.
/// Why:
/// The error cases are the ones Git rejects or this reader cannot read exactly;
/// the caller then asks Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseConfig(text: string): ParsedConfig  // throws on text it cannot read exactly
/// ```
pub fn parse_config(text: &str) -> Result<ParsedConfig, String> {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `without_mark: &str`.
    let without_mark: &str = text.strip_prefix('\u{feff}').unwrap_or(text);
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // `let mut`: a binding that may change.
    // Typed binding: `lexer: Lexer`.
    let mut lexer: Lexer = Lexer { characters: without_mark.chars().collect(), position: 0, at_end: false };
    let mut parsed: ParsedConfig = ParsedConfig::default();
    // `None`: the present or the absent value of an `Option`.
    // `let mut`: a binding that may change.
    // Typed binding: `stem: Option<String>`.
    let mut stem: Option<String> = None;
    // `let mut`: a binding that may change.
    // Typed binding: `comment: bool`.
    let mut comment: bool = false;
    loop {
        // Typed binding: `character: char`.
        let character: char = lexer.next();
        if character == '\n' {
            if lexer.at_end {
                break;
            }
            comment = false;
            continue;
        }
        if comment || is_space(character) {
            continue;
        }
        if character == '#' || character == ';' {
            comment = true;
            continue;
        }
        if character == '[' {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Borrow `&mut lexer`: lends the value without copying or giving it away.
            // Typed binding: `base: String`.
            let base: String = read_base(&mut lexer)?;
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            // Typed binding: `section: &str`.
            let section: &str = base.split('.').next().unwrap_or("");
            if section == "include" || section == "includeif" {
                parsed.has_include = true;
            }
            // `Some`: the present or the absent value of an `Option`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            stem = Some(format!("{base}."));
            continue;
        }
        if !character.is_ascii_alphabetic() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("a configuration line starts with {character:?}, which starts neither a section nor a key"));
        }
        // `Some`: the present or the absent value of an `Option`.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        // Borrow `&stem`: lends the value without copying or giving it away.
        let Some(current) = &stem else {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Err("a configuration line appears before any section header".to_string());
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&mut lexer`: lends the value without copying or giving it away.
        parsed.entries.push(read_variable(&mut lexer, current, character)?);
    }
    // Borrow `&parsed.entries`: lends the value without copying or giving it away.
    for entry in &parsed.entries {
        if entry.name.starts_with("include.") || entry.name.starts_with("includeif.") {
            parsed.has_include = true;
        }
    }
    return Ok(parsed);
}
