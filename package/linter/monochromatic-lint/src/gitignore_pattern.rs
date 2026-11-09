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
//!  A character range that runs backwards keeps only the character that opens it,
//!  which is what Git matches
//!       there;
//!  the glob compiler refuses such a range,
//!  and a pattern Git accepts would otherwise fail the
//!       policy that read it.
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
                    line.push_str(class_as_git_reads_it(&characters[index..=close]).as_str());
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
///  One character class,
/// from its `[` to its `]`,
/// as the glob compiler must read it for Git's meaning.
/// Why:
///  Git tests every member character as itself before it can open a range,
///  so a range that runs backwards
///       names its first character and nothing else;
///  the glob compiler refuses such a range outright,
///  and a
///       pattern Git accepts would fail the policy that reads it.
///  Every other member is copied exactly as it
///       was written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const classAsGitReadsIt = (members) => `[${members.slice(1, -1).map((member, at, body) => backwardsRangeReduced(member, at, body)).join('')}]`;
/// ```
fn class_as_git_reads_it(members: &[char]) -> String {
    // What: The class as it is built, with its `[` first and its `]` last.
    // Why:  Only the members between them are rewritten.
    let mut built: String = String::with_capacity(members.len());
    built.push('[');
    let close: usize = members.len() - 1;
    let mut index: usize = 1;
    // A class that negates itself names no member where the `!` or `^` stands.
    if matches!(members.get(index), Some('!' | '^')) {
        built.push(members[index]);
        index += 1;
    }
    // What: The last member, which a `-` needs to open a range.
    // Why:  Git clears it after a range, so a `-` that follows one is a member of its own.
    let mut previous: Option<usize> = None;
    while index < close {
        let character: char = members[index];
        if character == '\\' && index + 1 < close {
            // An escape and what it escapes are members as they were written.
            built.push(character);
            built.push(members[index + 1]);
            previous = Some(index + 1);
            index += 2;
            continue;
        }
        if character == '-'
            && index + 1 < close
            && let Some(start) = previous
        {
            // What: The character the range ends at, and the escape before it when it has one.
            // Why:  Git reads an escaped character as that character, and a range's end may be escaped.
            let mut end: usize = index + 1;
            let escaped: bool = members[end] == '\\' && end + 1 < close;
            if escaped {
                end += 1;
            }
            let last: char = members[end];
            // What: A range that runs backwards keeps only the character that opened it, which is already built.
            // Why:  Git matches no input with such a range, and the glob compiler refuses it.
            if members[start] <= last {
                built.push('-');
                if escaped {
                    built.push('\\');
                }
                built.push(last);
            }
            previous = None;
            index = end + 1;
            continue;
        }
        built.push(character);
        previous = Some(index);
        index += 1;
    }
    built.push(']');
    return built;
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
