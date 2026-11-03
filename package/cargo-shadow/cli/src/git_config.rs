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
    /// Every variable, in file order.
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
    return match subsection {
        Some(found) => format!("{}.{found}.{}", section.to_lowercase(), key.to_lowercase()),
        None => format!("{}.{}", section.to_lowercase(), key.to_lowercase()),
    };
}

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
        let wanted: String = variable_name(section, subsection, key);
        return self.entries.iter().filter(|entry| { return entry.name == wanted; }).map(|entry| { return entry.value.as_deref(); }).collect();
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
/// no value or `true`, `yes`, `on` or a nonzero integer is true;
/// an empty value, `false`, `no`, `off` or zero is false.
/// Why:
/// `extensions.worktreeConfig = yes` is as true as `true`;
/// a spelling the reader does not know is an error so the caller asks Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseBool(value: string | undefined): boolean  // throws on an unknown spelling
/// ```
pub fn parse_bool(value: Option<&str>) -> Result<bool, String> {
    let text: &str = match value {
        None => return Ok(true),
        Some(found) => found,
    };
    if text.is_empty() {
        return Ok(false);
    }
    let lowered: String = text.to_ascii_lowercase();
    if ["true", "yes", "on"].contains(&lowered.as_str()) {
        return Ok(true);
    }
    if ["false", "no", "off"].contains(&lowered.as_str()) {
        return Ok(false);
    }
    let digits: &str = lowered.strip_prefix(['-', '+']).unwrap_or(&lowered);
    let number: &str = digits.strip_suffix(['k', 'm', 'g']).unwrap_or(digits);
    if !number.is_empty() && number.chars().all(|character| { return character.is_ascii_digit(); }) && !(number.len() > 1 && number.starts_with('0')) {
        return Ok(number.chars().any(|character| { return character != '0'; }));
    }
    return Err(format!("{text:?} is not a boolean spelling this reader knows"));
}

/// How the lexer sees the text: Git's `get_next_char`.
struct Lexer {
    /// The characters of the file, after the optional byte order mark.
    characters: Vec<char>,
    /// The index of the next character.
    position: usize,
    /// Whether the end of the text was reached; the end reads as one final newline.
    at_end: bool,
}

impl Lexer {
    /// What:
    /// `fn next(&mut self) -> char` returns the next character, with `\r\n` as `\n` and the end of the text as a newline
    /// (and `at_end` set).
    /// Why:
    /// It is Git's `get_next_char`, so the rest of the lexer reads like Git's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// next(): string
    /// ```
    fn next(&mut self) -> char {
        let found: Option<char> = self.characters.get(self.position).copied();
        self.position += 1;
        return match found {
            None => {
                self.at_end = true;
                '\n'
            }
            Some('\r') if self.characters.get(self.position) == Some(&'\n') => {
                self.position += 1;
                '\n'
            }
            Some(character) => character,
        };
    }
}

/// Git's `isspace` for configuration text: space, tab, newline and carriage return only.
fn is_space(character: char) -> bool {
    return matches!(character, ' ' | '\t' | '\n' | '\r');
}

/// Git's `iskeychar`: ASCII letters, digits and the dash.
fn is_key_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || character == '-';
}

/// What:
/// `fn read_extended_base(lexer, name, first) -> Result<(), String>` reads the quoted subsection of a header such as
/// `[branch "topic"]` after the section name,
/// appending a dot and the subsection exactly as written to `name`.
/// Why:
/// The subsection runs to the closing quote (a backslash makes the next character literal),
/// so a bracket, a hash or an escaped quote inside it belong to the name,
/// and nothing but the closing bracket may follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readExtendedBase(lexer: Lexer, name: string[], first: string): void  // throws on an incomplete header
/// ```
fn read_extended_base(lexer: &mut Lexer, name: &mut String, first: char) -> Result<(), String> {
    let mut character: char = first;
    loop {
        if character == '\n' {
            return Err("a section header ends before its closing bracket".to_string());
        }
        character = lexer.next();
        if !is_space(character) {
            break;
        }
    }
    if character != '"' {
        return Err("a subsection name is not in double quotes".to_string());
    }
    name.push('.');
    loop {
        let mut inner: char = lexer.next();
        if inner == '\n' {
            return Err("a subsection name ends before its closing quote".to_string());
        }
        if inner == '"' {
            break;
        }
        if inner == '\\' {
            inner = lexer.next();
            if inner == '\n' {
                return Err("a subsection name ends after a backslash".to_string());
            }
        }
        name.push(inner);
    }
    if lexer.next() != ']' {
        return Err("a section header has something between the quoted subsection and its closing bracket".to_string());
    }
    return Ok(());
}

/// What:
/// `fn read_base(lexer) -> Result<String, String>` reads a section header after its opening bracket and returns the flat stem
/// (`branch.topic` for `[branch "topic"]`, lowercased only before a quoted subsection).
/// Why:
/// Every key that follows is named by this stem and a dot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readBase(lexer: Lexer): string  // throws on a header Git rejects
/// ```
fn read_base(lexer: &mut Lexer) -> Result<String, String> {
    let mut name: String = String::new();
    loop {
        let character: char = lexer.next();
        if lexer.at_end {
            return Err("a section header has no closing bracket".to_string());
        }
        if character == ']' {
            return Ok(name);
        }
        if is_space(character) {
            read_extended_base(lexer, &mut name, character)?;
            return Ok(name);
        }
        if !is_key_character(character) && character != '.' {
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
    let mut quoted: bool = false;
    let mut comment: bool = false;
    let mut pending_spaces: usize = 0;
    loop {
        let mut character: char = lexer.next();
        if character == '\n' {
            if quoted {
                return Err("a configuration value has an unclosed double quote".to_string());
            }
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
            let escaped: char = match character {
                '\n' => continue,
                't' => '\t',
                'b' => '\u{8}',
                'n' => '\n',
                '\\' | '"' => character,
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
/// A key ends at the first character that is not a letter, digit or dash;
/// after optional blanks a newline means no value,
/// an `=` starts the value,
/// and anything else is a syntax error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readVariable(lexer: Lexer, stem: string, first: string): ConfigEntry  // throws on a syntax error
/// ```
fn read_variable(lexer: &mut Lexer, stem: &str, first: char) -> Result<ConfigEntry, String> {
    let mut name: String = format!("{stem}{}", first.to_ascii_lowercase());
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
        return Ok(ConfigEntry { name, value: None });
    }
    if character != '=' {
        return Err(format!("{name} is followed by {character:?} where a value or a newline is expected"));
    }
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
    let without_mark: &str = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lexer: Lexer = Lexer { characters: without_mark.chars().collect(), position: 0, at_end: false };
    let mut parsed: ParsedConfig = ParsedConfig::default();
    let mut stem: Option<String> = None;
    let mut comment: bool = false;
    loop {
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
            let base: String = read_base(&mut lexer)?;
            let section: &str = base.split('.').next().unwrap_or("");
            if section == "include" || section == "includeif" {
                parsed.has_include = true;
            }
            stem = Some(format!("{base}."));
            continue;
        }
        if !character.is_ascii_alphabetic() {
            return Err(format!("a configuration line starts with {character:?}, which starts neither a section nor a key"));
        }
        let Some(current) = &stem else {
            return Err("a configuration line appears before any section header".to_string());
        };
        parsed.entries.push(read_variable(&mut lexer, current, character)?);
    }
    for entry in &parsed.entries {
        if entry.name.starts_with("include.") || entry.name.starts_with("includeif.") {
            parsed.has_include = true;
        }
    }
    return Ok(parsed);
}
