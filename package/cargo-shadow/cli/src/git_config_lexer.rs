//! What:
//! The character lexer of Git configuration text:
//! the section header, the value and the key readers.
//! Why:
//! Split out of `git_config.rs`,
//! which reads the few keys the trust base needs,
//! to keep that file under the code-line limit.
//! The parent module reads through these items,
//! so the reader behaves as it did.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports the recorded variable that a key and its value become.
use crate::git_config::ConfigEntry;

/// How the lexer sees the text:
/// Git's `get_next_char`.
pub(crate) struct Lexer {
    /// The characters of the file,
    /// after the optional byte order mark.
    pub(crate) characters: Vec<char>,
    /// The index of the next character.
    pub(crate) position: usize,
    /// Whether the end of the text was reached;
    /// the end reads as one final newline.
    pub(crate) at_end: bool,
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
    pub(crate) fn next(&mut self) -> char {
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
pub(crate) fn is_space(character: char) -> bool {
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
fn read_extended_base(
    lexer: &mut Lexer,
    name: &mut String,
    first: char,
) -> Result<(), String> {
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
        return Err(
            "a section header has something between the quoted subsection and its closing bracket"
                .to_string(),
        );
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
pub(crate) fn read_base(lexer: &mut Lexer) -> Result<String, String> {
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
            return Err(format!(
                "a section header holds {character:?}, which a section name cannot"
            ));
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
                other => {
                    return Err(format!(
                        "an unknown escape \\{other} in a configuration value"
                    ));
                }
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
pub(crate) fn read_variable(
    lexer: &mut Lexer,
    stem: &str,
    first: char,
) -> Result<ConfigEntry, String> {
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
        return Ok(ConfigEntry {
            name,
            value: None,
        });
    }
    if character != '=' {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!(
            "{name} is followed by {character:?} where a value or a newline is expected"
        ));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    return Ok(ConfigEntry {
        name,
        value: Some(read_value(lexer)?),
    });
}
