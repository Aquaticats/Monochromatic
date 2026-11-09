//! What:
//!  The line the `ignore` crate's matcher is given for a pattern written in gitignore syntax,
//!  with the
//!       characters gitignore reads as ordinary text escaped for the richer glob grammar underneath.
//! Why:
//!  A gitignore pattern is documented as the language `.gitignore` holds,
//!  and Git reads `{` and `}` as
//!       ordinary characters there.
//!  The `ignore` crate compiles patterns with `globset`,
//!  which reads `{a,b}` as an
//!       alternate group,
//!  so a pattern naming a file like `notes{old,new}.md` would exclude the wrong
//!       file and let the named one through,
//!  and one like `a{b.md` would fail to compile at all.
//!  Both callers of the matcher,
//!  the wrapper's `exclude` option and the LFS rule's own reading of it,
//!  pass their
//!       patterns through here so they can never disagree.
//!  A brace inside a character class is left alone,
//!  because the class parser reads a backslash there as a
//!       literal backslash rather than as an escape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const line = forMatcher(pattern); // '\{a,b\}.md' for '{a,b}.md'
//! ```

/// What:
///  `pattern` as the matcher's glob compiler must read it.
/// Why:
///  Escaping is what makes a literal brace literal:
///  `\{` is the character `{` to the glob compiler,
///  so a pattern keeps the meaning gitignore gives it.
///  An escape the author already wrote is kept as one,
///  never doubled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forMatcher = (pattern) => pattern.replace(/(?<!\\)[{]/g, '\\{').replace(/(?<!\\)[}]/g, '\\}');
/// ```
pub fn for_matcher(pattern: &str) -> String {
    let characters: Vec<char> = pattern.chars().collect();
    let mut line: String = String::with_capacity(pattern.len());
    let mut index: usize = 0;
    while index < characters.len() {
        let character: char = characters[index];
        match character {
            // An escape and what it escapes are copied as they are: the glob compiler reads them the same way Git does.
            '\\' => {
                line.push(character);
                if let Some(escaped) = characters.get(index + 1) {
                    line.push(*escaped);
                    index += 2;
                } else {
                    index += 1;
                }
            }
            // A character class is copied whole, because its parser reads a backslash as itself.
            '[' => match closing_bracket(characters.as_slice(), index) {
                Some(close) => {
                    for member in &characters[index..=close] {
                        line.push(*member);
                    }
                    index = close + 1;
                }
                None => {
                    // An unclosed class is a literal `[` to the glob compiler, so what follows is ordinary pattern text.
                    line.push(character);
                    index += 1;
                }
            },
            '{' | '}' => {
                line.push('\\');
                line.push(character);
                index += 1;
            }
            other => {
                line.push(other);
                index += 1;
            }
        }
    }
    return line;
}

/// What:
///  Where the character class opened at `open` ends,
///  or nothing when it never does.
/// Why:
///  The class parser reads `]` as its first member when it comes first,
///  and after a `!` or `^` that negates the class,
///  so a naive search would end the class too early.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const closingBracket = (chars, open) => { let i = open + 1; if (chars[i] === '!' || chars[i] === '^') i += 1; if (chars[i] === ']') i += 1; while (i < chars.length) { if (chars[i] === ']') return i; i += 1; } return undefined; };
/// ```
fn closing_bracket(characters: &[char], open: usize) -> Option<usize> {
    let mut index: usize = open + 1;
    if matches!(characters.get(index), Some('!' | '^')) {
        index += 1;
    }
    if matches!(characters.get(index), Some(']')) {
        index += 1;
    }
    while let Some(character) = characters.get(index) {
        if *character == ']' {
            return Some(index);
        }
        index += 1;
    }
    return None;
}

/// Pattern controls stay outside release artifacts.
#[cfg(test)]
#[path = "gitignore_pattern_tests.rs"]
mod tests;
