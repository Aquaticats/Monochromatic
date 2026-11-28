//! What:
//!  The value of a Git alias:
//!  the value Git keeps,
//!  read from one `git config --null --list`;
//!  that value split the way Git splits it;
//!  and the words of a `!` shell alias's value.
//! Why:
//!  An alias value is split by `split_cmdline`,
//!  which separates words on `GIT_SPACE` outside quotes,
//!  lets single and double quotes group,
//!  and lets a backslash escape the next byte except inside single quotes;
//!  Git decides what an alias is from its first byte and never trims the value,
//!  so a leading space is Git's own empty first word and a trailing one its own empty last word.
//!  A `!` alias's value is read by the shell instead,
//!  so it is only read here when the shell passes every byte through unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const value = await lookupAlias(name); const words = splitAliasCommand(value);
//! // const shellWords = shellGitWords(value); const selected = aliasSelectionOf(gitConfigNullList);
//! ```

/// Debug diagnostics for a read that answered nothing.
use crate::diagnostic_log::debug;
/// Running real Git,
///  for the one read of the value Git keeps.
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git,
};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path;
///  the read names the directory it runs in.
use std::path::Path;

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
    /// The variable name as Git wrote it,
    /// `section.subsection.key` or `section.key`.
    pub name: &'a [u8],
    /// The value,
    /// or nothing for a variable that stands alone (`alias.x` with no `=`).
    pub value: Option<&'a [u8]>,
}

/// What:
///  Whether a byte is the terminator `git config --null` writes after a variable.
/// Why:
///  Splitting the listing on it is what turns one buffer into variables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte) => byte === 0
/// ```
fn is_variable_end(byte: &u8) -> bool {
    return *byte == b'\0';
}

/// What:
///  Whether a byte starts a variable's value in `git config --null --list`.
/// Why:
///  Git writes the name,
///  this byte,
///  then the value,
///  so the first one is where the name ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte) => byte === '\n'
/// ```
fn is_value_start(byte: &u8) -> bool {
    return *byte == b'\n';
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
    for record in output.split(is_variable_end) {
        // The terminator after the last variable leaves one empty record.
        if record.is_empty() {
            continue;
        }
        match record.iter().position(is_value_start) {
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
///  and nothing else:
///  a vertical tab and a form feed are `GIT_CNTRL` in the same table.
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
///  Whether a byte separates two words for the shell Git starts a `!` alias in.
///  The blanks are a space and a TAB;
///  a line feed ends the command and starts another,
///  and a carriage return is an ordinary byte of a word,
///  which is why neither is a blank here.
/// Why:
///   A separator read too widely joins two commands into one word,
///  and the reading then decides about a command nobody runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0x20 || byte === 0x09
/// ```
fn is_shell_blank(byte: u8) -> bool {
    return matches!(byte, b' ' | b'\t');
}

/// What:
///  The words after `!git` in an alias value,
///  when the value is exactly one plain `git` command;
///  no word at all when the value names `git` and nothing else,
///  whose command the caller's own words then make.
/// Why:
///   The shell splits and expands its command line,
///  so only a value whose bytes the shell passes through
///       unchanged reads here as the words the shell runs;
///  anything the shell would expand,
///  quote or separate
///  is left unread rather than guessed at.
///  Git hands the shell the value's bytes after the `!` (`git.c:401` to `git.c:402`),
///  and the shell reads a run of blanks there as one separator,
///  so `! git <words>` and a TAB after `!git` run the command `!git <words>` runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shellGitWords(value: string): string[] | undefined;
/// ```
pub(crate) fn shell_git_words(value: &[u8]) -> Option<Vec<Vec<u8>>> {
    let after_the_bang: &[u8] = value.strip_prefix(b"!")?;
    // `mut` allows the scan to pass the blanks between `!` and the command.
    let mut at: usize = 0;
    while at < after_the_bang.len() && is_shell_blank(after_the_bang[at]) {
        at += 1;
    }
    let remainder: &[u8] = after_the_bang[at..].strip_prefix(b"git")?;
    // `!gitk` names another program, so only a blank or the end of the value may follow.
    if !remainder.is_empty() && !is_shell_blank(remainder[0]) {
        return None;
    }
    let mut words: Vec<Vec<u8>> = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    for &byte in remainder {
        if is_shell_blank(byte) {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        if !is_plain_shell_byte(byte) {
            return None;
        }
        current.push(byte);
    }
    if !current.is_empty() {
        words.push(current);
    }
    return Some(words);
}

/// What:
///  Whether the shell passes a byte through in the same word,
///  without quoting it,
///  expanding it or separating
///       on it.
/// Why:
///   These are the bytes of ordinary arguments;
///  `$`,
///  a backtick,
///  a quote,
///  a backslash,
///  a glob,
///  a
///  redirection,
///  a separator,
///  a brace and a leading `~` all mean something to the shell,
/// and `#` can start a
///  comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => /[A-Za-z0-9_./:=@+,-]/.test(String.fromCharCode(byte))
/// ```
fn is_plain_shell_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'-' | b'_' | b'.' | b'/' | b':' | b'=' | b'@' | b'+' | b','
        );
}

/// What:
///  Whether a byte is the dot `parse_config_key` splits a variable name at.
/// Why:
///  Git finds the last one,
///  because a subsection may hold dots of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte) => byte === '.'
/// ```
fn is_subsection_dot(byte: &u8) -> bool {
    return *byte == b'.';
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
pub fn alias_variable_matches(
    name: &[u8],
    variable: &[u8],
) -> bool {
    // `let Some(x) = ... else { ... }` is "not an alias variable at all".
    let Some(rest) = variable.strip_prefix(b"alias.") else {
        return false;
    };
    // `rposition` is `strrchr`: the subsection may hold dots of its own.
    if let Some(last) = rest.iter().rposition(is_subsection_dot) {
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
pub fn alias_selection(
    variables: &[ConfigVariable<'_>],
    name: &[u8],
) -> Option<Vec<u8>> {
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

/// What:
///  The value of the alias Git keeps,
///  read from one `git config --null --list`,
///  or nothing when Git keeps none or dies on one.
///  `Option<&Path>` is the directory this command was started in,
///       which this read runs in,
///       and nothing when the caller named none.
/// Why:
///   `lookupAlias`:
///  `config_alias_cb` overwrites the value it kept on every matching variable,
///  so the last of `alias.<name>` and `[alias "<name>"] command` in config order wins,
///  and a matching variable with no value is Git's own `missing value` refusal,
///  which leaving the alias to Git reproduces.
///  The read runs in the directory the command was started in,
///  because that is where Git reads the configuration this question is about:
///  a read that asked the process instead answered with whatever directory another
///  test had moved the process to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function lookupAlias({ gitPath, globalArgs, name, start }): Promise<string | typeof ALIAS_UNSET>;
/// ```
pub fn lookup_alias(
    context: &GitContext,
    global: &[OsString],
    name: &OsString,
    start: Option<&Path>,
) -> Option<Vec<u8>> {
    // A caller that named no directory reads no alias: Git cannot start somewhere this command was not started.
    let directory: &Path = start?;
    let mut arguments: Vec<OsString> = global.to_vec();
    arguments.push(OsString::from("config"));
    arguments.push(OsString::from("--null"));
    arguments.push(OsString::from("--list"));
    let mut request: GitRequest = GitRequest::new(directory, arguments.as_slice());
    request.without_prefix = true;
    let output = match run_git(context, &request) {
        Ok(output) if output.succeeded() => output,
        Ok(_) | Err(_) => {
            debug(
                "readConfigVariables",
                "git config --null --list answered nothing",
            );
            return None;
        }
    };
    let variables: Vec<ConfigVariable<'_>> = config_variables(output.stdout.as_slice());
    let selected: Option<Vec<u8>> = alias_selection(variables.as_slice(), name.as_encoded_bytes());
    if selected.is_none() {
        debug(
            "lookupAlias",
            format!("no {}", name.to_string_lossy()).as_str(),
        );
    }
    return selected;
}
