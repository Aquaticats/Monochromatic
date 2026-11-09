//! What:
//!  The value of a Git alias,
//!  split the way Git splits it,
//!  and read from `git config --null` without losing a byte.
//! Why:
//!  An alias value is split by `split_cmdline`,
//!  which separates words on whitespace outside quotes,
//!  lets single and double quotes group,
//!  and lets a backslash escape the next byte except inside single quotes;
//!  Git decides what an alias is from its first byte and never trims the value,
//!  so a leading space is Git's own empty first word.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const words = splitAliasCommand(value); const value = aliasValueOf(gitConfigNullOutput);
//! ```

/// What:
///  Whether a byte is whitespace as C `isspace` sees it in the C locale.
/// Why:
///   `split_cmdline` separates words on exactly these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [' ', '\t', '\n', '\r', '\v', '\f'].includes(character)
/// ```
fn is_space(byte: u8) -> bool {
    return matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c);
}

/// What:
///  Split an alias value as Git's `split_cmdline` does:
///  whitespace separates words outside
///       quotes,
///  single and double quotes group,
///  and a backslash escapes the next byte except
///       inside single quotes.
///  `None` is an unclosed quote or a trailing backslash.
/// Why:
///   `splitAliasCommand`;
///  bytes are kept as they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitAliasCommand(value: string): string[] | typeof ALIAS_SPLIT_FAILED;
/// ```
pub fn split_alias_command(value: &[u8]) -> Option<Vec<Vec<u8>>> {
    let mut words: Vec<Vec<u8>> = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    // Starts true as the incumbent's `splitAliasCommand` does, so an empty value is one empty word.
    let mut started: bool = true;
    let mut quote: u8 = 0;
    let mut escaped: bool = false;
    for &byte in value {
        if escaped {
            current.push(byte);
            escaped = false;
        } else if quote == 0 && is_space(byte) {
            if started {
                words.push(std::mem::take(&mut current));
            }
            current.clear();
            started = false;
        } else if quote == 0 && (byte == b'\'' || byte == b'"') {
            quote = byte;
            started = true;
        } else if quote != 0 && byte == quote {
            quote = 0;
        } else if byte == b'\\' && quote != b'\'' {
            escaped = true;
            started = true;
        } else {
            current.push(byte);
            started = true;
        }
    }
    if quote != 0 || escaped {
        return None;
    }
    if started {
        words.push(current);
    }
    return Some(words);
}

/// What:
///  An alias value with exactly one trailing NUL removed,
///  the terminator `git config --null` adds to a value.
/// Why:
///   Git splits the value its configuration reader produced,
///  so no whitespace at either end may be
///       dropped here:
///  a leading space is Git's own empty first word (`git.c:414`),
///  the one that makes
///  `git -c 'alias.audit= reset --hard' audit` fail instead of resetting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value.endsWith('\0') ? value.slice(0, -1) : value
/// ```
pub fn alias_value_of(output: &[u8]) -> Vec<u8> {
    return match output.split_last() {
        Some((b'\0', rest)) => rest.to_vec(),
        _ => output.to_vec(),
    };
}
