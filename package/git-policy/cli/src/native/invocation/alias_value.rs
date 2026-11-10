//! What:
//!  The value of a Git alias,
//!  split the way Git splits it,
//!  from the variables of `git config --null --list`.
//! Why:
//!  An alias value is split by `split_cmdline`,
//!  which separates words on `GIT_SPACE` outside quotes,
//!  lets single and double quotes group,
//!  and lets a backslash escape the next byte except inside single quotes;
//!  Git decides what an alias is from its first byte and never trims the value,
//!  so a leading space is Git's own empty first word and a trailing one its own empty last word.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const words = splitAliasCommand(value); const selected = aliasSelectionOf(gitConfigNullList);
//! ```

/// What:
///  One variable of `git config --null --list`,
///  with its value when it has one.
/// Why:
///  Git's alias reader keeps the value of the last matching variable and dies on a matching variable that holds none,
///  so both the value and its absence carry an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigVariable = { name: string; value?: string };
/// ```
pub struct ConfigVariable<'a> {
    /// The variable name as Git wrote it, `section.subsection.key` or `section.key`.
    pub name: &'a [u8],
    /// The value, or nothing for a variable that stands alone (`alias.x` with no `=`).
    pub value: Option<&'a [u8]>,
}

/// What:
///  The variables of `git config --null --list`,
///  in the order Git read them.
/// Why:
///  `--null --list` writes `name`,
///  a line feed,
///  the value,
///  then a NUL for each variable,
///  and a name alone for a variable with no value,
///  so the last matching variable is Git's own answer and a valueless one is its refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configVariables(output: Buffer): ConfigVariable[]
/// ```
pub fn config_variables(output: &[u8]) -> Vec<ConfigVariable<'_>> {
    let mut variables: Vec<ConfigVariable<'_>> = Vec::new();
    for record in output.split(|byte| return *byte == 0) {
        // The terminator after the last variable leaves one empty record.
        if record.is_empty() {
            continue;
        }
        match record.iter().position(|byte| return *byte == b'\n') {
            Some(separator) => variables.push(ConfigVariable {
                name: &record[..separator],
                value: Some(&record[separator + 1..]),
            }),
            None => variables.push(ConfigVariable {
                name: record,
                value: None,
            }),
        }
    }
    return variables;
}

/// What:
///  Whether a byte is whitespace to Git's `split_cmdline`.
///  `GIT_SPACE` is space,
///  TAB,
///  LF and CR,
///  and nothing else: a vertical tab and a form feed are `GIT_CNTRL` in the same table.
/// Why:
///  `split_cmdline` separates words on `isspace`,
///  which `git-compat-util.h` maps to `sane_isspace` over the `sane_ctype` table
///  (`ctype.c`),
///  so a wider class here splits a word Git keeps whole and runs a command Git never runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [' ', '\t', '\n', '\r'].includes(character)
/// ```
fn is_space(byte: u8) -> bool {
    return matches!(byte, b' ' | b'\t' | b'\n' | b'\r');
}

/// What:
///  Split an alias value as Git's `split_cmdline` does:
///  `GIT_SPACE` separates words outside quotes,
///  single and double quotes group,
///  and a backslash escapes the next byte except inside single quotes.
///  `None` is an unclosed quote or a trailing backslash.
///  There is always at least one word:
///  Git starts with one before it reads a byte,
///  and every whitespace run starts another,
///  so an empty value is one empty word and a trailing space adds a second.
/// Why:
///   `splitAliasCommand`;
///  bytes are kept as they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitAliasCommand(value: string): string[] | typeof ALIAS_SPLIT_FAILED;
/// ```
pub fn split_alias_command(value: &[u8]) -> Option<Vec<Vec<u8>>> {
    // Git starts with one word before it reads a byte, and every separator run closes the word it ends and starts
    //  another (`alias.c`, `split_cmdline`).
    let mut words: Vec<Vec<u8>> = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    let mut quote: u8 = 0;
    let mut at: usize = 0;
    while at < value.len() {
        let byte: u8 = value[at];
        if quote == 0 && is_space(byte) {
            words.push(std::mem::take(&mut current));
            at += 1;
            while at < value.len() && is_space(value[at]) {
                at += 1;
            }
        } else if quote == 0 && (byte == b'\'' || byte == b'"') {
            quote = byte;
            at += 1;
        } else if quote != 0 && byte == quote {
            quote = 0;
            at += 1;
        } else if byte == b'\\' && quote != b'\'' {
            at += 1;
            // `?` is Git's `SPLIT_CMDLINE_BAD_ENDING`: a value that ends in a backslash splits into nothing.
            let &escaped: &u8 = value.get(at)?;
            current.push(escaped);
            at += 1;
        } else {
            current.push(byte);
            at += 1;
        }
    }
    if quote != 0 {
        return None;
    }
    // The last word exists even when the value ended with a separator or held no byte at all.
    words.push(current);
    return Some(words);
}

/// What:
///  Whether an alias name is the alias a config variable defines,
///  by Git's own matching rules.
/// Why:
///  `config_alias_cb` splits the variable at the last dot into a subsection and a key,
///  reads a subsection with the key `command` as the `[alias "<name>"]` spelling and matches it byte for byte,
///  and reads any other variable as the plain `alias.<name>` spelling,
///  matched without regard to case,
///  where a variable with a dot in its name names an alias that holds that dot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aliasVariableMatches(name: string, variable: string): boolean
/// ```
pub fn alias_variable_matches(name: &[u8], variable: &[u8]) -> bool {
    // `let Some(x) = ... else { ... }` is "not an alias variable at all".
    let Some(rest) = variable.strip_prefix(b"alias.") else {
        return false;
    };
    // `rposition` is `strrchr`: the subsection may hold dots of its own.
    if let Some(last) = rest.iter().rposition(|byte| return *byte == b'.') {
        let subsection: &[u8] = &rest[..last];
        let key: &[u8] = &rest[last + 1..];
        // `[alias ""]` is the plain `[alias]` section (`config_alias_cb`).
        if !subsection.is_empty() {
            if key == b"command" {
                return subsection == name;
            }
            // A subsection whose key is not `command` is the plain spelling with a dot in the name.
            return rest.eq_ignore_ascii_case(name);
        }
    }
    return rest.eq_ignore_ascii_case(name);
}

/// What:
///  The value of the alias variable Git keeps,
///  or nothing when Git keeps none or dies on one.
/// Why:
///   `config_alias_cb` overwrites the value it kept on every matching variable,
///  so a repo holding both `alias.x` and `[alias "x"] command` runs the last one in config order,
///  and a matching variable with no value is `error: missing value for '<variable>'`,
///  which the wrapper leaves to Git by answering no alias.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aliasSelection(variables: ConfigVariable[], name: string): string | undefined
/// ```
pub fn alias_selection(variables: &[ConfigVariable<'_>], name: &[u8]) -> Option<Vec<u8>> {
    let mut selected: Option<Vec<u8>> = None;
    for variable in variables {
        if !alias_variable_matches(name, variable.name) {
            continue;
        }
        // `?` is Git's own refusal: `config_alias_cb` dies on a matching variable that holds no value
        //  (`error: missing value for '<variable>'`), and leaving the alias to Git reproduces that message.
        let value: &[u8] = variable.value?;
        selected = Some(value.to_vec());
    }
    return selected;
}
