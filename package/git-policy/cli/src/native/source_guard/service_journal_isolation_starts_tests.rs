//! What:
//!  The reader behind the journal-isolation guard:
//! for every source the build compiles for tests only it says where a child is started,
//! which program that start names,
//! and whether the start gives the child a standard error of its own.
//! Why:
//!  A start can hand a child the streams the run was given without ever writing `Stdio::inherit`:
//! `std::process::Command`'s default is to inherit all three,
//! so a `spawn()` or a `status()` with no setter is the same hazard spelled differently,
//! and a guard that looked only for the token passed such a start
//! (the review of 2026-11-14, finding 2).
//!  The reading is textual and says where it is coarse:
//! it masks strings,
//! characters and comments to spaces so a token inside prose or a format text is no start;
//! it splits a source at every `;` outside parentheses and brackets;
//! it reads a start as the clause that ends it,
//! plus the clauses of the binding that clause calls the terminal on;
//! and it follows that binding back only to its `let` or its last assignment,
//! so a command built by a helper function of another file is a start whose program this reader cannot name.
//!  A definition (`fn spawn_owned(`) is no start,
//! and `.spawn(` with an argument is a thread, not a child.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const statement of statements(masked(source))) for (const terminal of terminals(statement)) starts.push(read(statement, terminal));
//! ```

use std::collections::{BTreeMap, BTreeSet};

/// Where a start's standard error goes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardError {
    /// The start names a standard error of its own:
    /// a pipe,
    /// a file,
    /// a descriptor the fixture holds,
    /// or nothing at all.
    Given,
    /// The start names none,
    /// so the child inherits the stream the process running the tests was given.
    Inherited,
}

/// What ends a start.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Terminal {
    /// `output()`, which captures standard output and standard error unless the clause names them.
    Output,
    /// `status()`, which waits for the child.
    Status,
    /// `spawn()` with no argument, which starts the child and returns.
    Spawn,
    /// `spawn_owned(...)`, the support that starts a child in a group of its own.
    SpawnOwned,
    /// A call to a function of this crate that ends in starting a child,
    /// which this reader follows rather than names
    /// (`status_retrying`, `git`, `run`, `observe`, …).
    Followed,
}

/// What a function of this crate that starts a child gives that child.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Shape {
    /// Whether the function captures standard output and standard error itself,
    /// as `output()` does.
    pub captures: bool,
    /// Whether the function gives the child a standard error of its own.
    pub sets_error: bool,
    /// Whether the function gives the child all three streams of its own.
    pub sets_all: bool,
}

/// One start of a child as this reader sees it.
#[derive(Clone, Debug)]
pub struct Start {
    /// The line the terminal is on,
    /// counting from one.
    pub line: usize,
    /// What ends the start.
    pub terminal: Terminal,
    /// The program the start names,
    /// as the source spells it,
    /// or nothing when the command comes from somewhere this reader does not follow.
    pub program: Option<String>,
    /// The raw text of the terminal line,
    /// trimmed,
    /// which is what an inventory lists a start by.
    pub anchor: String,
    /// Where the child's standard error goes.
    pub standard_error: StandardError,
    /// The raw text this start was read from:
    /// its own clause and the clauses of the binding it is called on,
    /// which is what a finding quotes.
    pub chain: String,
    /// Whether the start's clause names the wrapper's service mode,
    /// which makes it a start of the service itself.
    pub runs_the_service: bool,
    /// The helper this start is a call to, when the reader followed one,
    /// which a finding names beside the program the reader could not read.
    pub followed: Option<String>,
}

/// What:
///  The source with every string,
/// character and comment masked to spaces,
/// keeping every line's length.
/// Why:
///   The reader must not find a start in a format text or a comment
/// (`source_guard/blocking_code_off_the_runtime_fixture_tests.rs` holds `.spawn()` in a constant on purpose),
/// and keeping the lengths lets a finding name the line and column it came from.
///
/// What:
///  Reads one Rust character or byte-character literal whole, escapes included,
///  and answers where it ends;
///  a lifetime's quote opens no literal, so there is no answer.
/// Why:
///   A quote that opens a literal blinds the mask to every quote after it:
///  the escaped quote of a character literal taken for three bytes leaves it to open a string,
///  the code that follows is blanked as that string's contents,
///  and a service start with `Stdio::inherit()` is then invisible to both scans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const end = charLiteralEnd(bytes, at) // `'x'`, `'\\''`, `'\\x41'`, `'\\u{1F600}'`, `b'\\''` -> after it; `'a` -> undefined
/// ```
fn char_literal_end(bytes: &[u8], at: usize) -> Option<usize> {
    // `at` is the literal's opening quote; a byte literal's `b` stands before it and is no part of it.
    let mut cursor: usize = at + 1;
    match *bytes.get(cursor)? {
        // A lifetime holds no closing quote, so the quote that closes nothing opens no literal.
        b'\'' => return None,
        b'\\' => {
            cursor += 1;
            match *bytes.get(cursor)? {
                b'n' | b'r' | b't' | b'\\' | b'\'' | b'"' | b'0' => cursor += 1,
                b'x' => {
                    cursor += 1;
                    for _ in 0..2 {
                        if !hex_digit(*bytes.get(cursor)?) {
                            return None;
                        }
                        cursor += 1;
                    }
                }
                b'u' => {
                    cursor += 1;
                    if *bytes.get(cursor)? != b'{' {
                        return None;
                    }
                    cursor += 1;
                    let mut digits: usize = 0;
                    while *bytes.get(cursor)? != b'}' {
                        if !hex_digit(*bytes.get(cursor)?) || digits == 6 {
                            return None;
                        }
                        digits += 1;
                        cursor += 1;
                    }
                    cursor += 1;
                }
                _ => return None,
            }
        }
        _ => {
            // One character, which may take more than one byte.
            cursor += character_width(bytes, cursor)?;
        }
    }
    if *bytes.get(cursor)? != b'\'' {
        return None;
    }
    return Some(cursor + 1);
}

/// What:
///  How many bytes the character at `at` takes.
/// Why:
///   A character literal may hold any character, and a mask that steps one byte at a time would miss its closing quote.
fn character_width(bytes: &[u8], at: usize) -> Option<usize> {
    let rest: &str = std::str::from_utf8(bytes.get(at..)?).ok()?;
    return rest.chars().next().map(char::len_utf8);
}

/// What:
///  Whether `byte` is a hexadecimal digit.
/// Why:
///  An escape's hex digits are what says the escape is whole.
fn hex_digit(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// In TS you'd write (pseudocode):
/// ```ts
/// const masked = source.replace(/\/\/[^\n]*|\/\*[\s\S]*?\*\/|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])'/g, (m) => m.replace(/[^\n]/g, ' '));
/// ```
pub fn mask(text: &str) -> String {
    let bytes: &[u8] = text.as_bytes();
    let mut out: Vec<u8> = bytes.to_vec();
    let mut at: usize = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'/' if bytes.get(at + 1) == Some(&b'/') => {
                while at < bytes.len() && bytes[at] != b'\n' {
                    out[at] = b' ';
                    at += 1;
                }
            }
            b'/' if bytes.get(at + 1) == Some(&b'*') => {
                // A block comment ends at the `*/` that matches its opening, and one inside it opens another:
                //  the depth is kept, so the end of the inner comment is no end of the outer one.
                out[at] = b' ';
                out[at + 1] = b' ';
                at += 2;
                let mut depth: usize = 1;
                while at < bytes.len() {
                    if bytes[at] == b'/' && bytes.get(at + 1) == Some(&b'*') {
                        depth += 1;
                        out[at] = b' ';
                        if at + 1 < bytes.len() {
                            out[at + 1] = b' ';
                        }
                        at += 2;
                        continue;
                    }
                    if bytes[at] == b'*' && bytes.get(at + 1) == Some(&b'/') {
                        depth -= 1;
                        out[at] = b' ';
                        if at + 1 < bytes.len() {
                            out[at + 1] = b' ';
                        }
                        at += 2;
                        if depth == 0 {
                            break;
                        }
                        continue;
                    }
                    if bytes[at] != b'\n' {
                        out[at] = b' ';
                    }
                    at += 1;
                }
            }
            b'r' if matches!(bytes.get(at + 1), Some(b'#' | b'"')) => {
                let mut cursor: usize = at + 1;
                let mut hashes: usize = 0;
                while bytes.get(cursor) == Some(&b'#') {
                    hashes += 1;
                    cursor += 1;
                }
                if bytes.get(cursor) != Some(&b'"') {
                    at += 1;
                    continue;
                }
                cursor += 1;
                let mut closing: Vec<u8> = vec![b'"'];
                closing.extend(std::iter::repeat_n(b'#', hashes));
                let end: usize = find(bytes, cursor, closing.as_slice()).unwrap_or(bytes.len());
                let stop: usize = (end + closing.len()).min(bytes.len());
                for slot in at..stop {
                    if bytes[slot] != b'\n' {
                        out[slot] = b' ';
                    }
                }
                at = stop;
            }
            b'"' => {
                out[at] = b' ';
                at += 1;
                while at < bytes.len() {
                    match bytes[at] {
                        b'\\' => {
                            out[at] = b' ';
                            if at + 1 < bytes.len() {
                                if bytes[at + 1] != b'\n' {
                                    out[at + 1] = b' ';
                                }
                                at += 2;
                            } else {
                                at += 1;
                            }
                        }
                        b'"' => {
                            out[at] = b' ';
                            at += 1;
                            break;
                        }
                        _ => {
                            if bytes[at] != b'\n' {
                                out[at] = b' ';
                            }
                            at += 1;
                        }
                    }
                }
            }
            b'\'' => match char_literal_end(bytes, at) {
                Some(end) => {
                    for slot in at..end {
                        if bytes[slot] != b'\n' {
                            out[slot] = b' ';
                        }
                    }
                    at = end;
                }
                // A lifetime (`'a`, `'static`) is no literal, and its quote opens nothing to mask.
                None => at += 1,
            },
            _ => at += 1,
        }
    }
    return String::from_utf8(out).expect("masking keeps the text's bytes");
}

/// What:
///  The source with every comment masked to spaces and every string kept as written,
///  keeping every line's length.
/// Why:
///   A guard that looks for what a string *says* (a path,
/// a quoted variable name)
///  must read the string,
///  and must not read a `//` inside one as the start of a comment:
/// the tokens of Rust are the grammar both scans need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const masked = source.replace(/\/\*[\s\S]*?\*\/|\/\/[^\n]*/g, (m) => m.replace(/[^\n]/g, ' '));
/// ```
pub fn mask_comments(text: &str) -> String {
    let bytes: &[u8] = text.as_bytes();
    let mut out: Vec<u8> = bytes.to_vec();
    let mut at: usize = 0;
    let mut in_string: bool = false;
    while at < bytes.len() {
        if in_string {
            match bytes[at] {
                b'\\' => at += 2,
                b'"' => {
                    in_string = false;
                    at += 1;
                }
                _ => at += 1,
            }
            continue;
        }
        match bytes[at] {
            b'"' => {
                in_string = true;
                at += 1;
            }
            // A raw string's `//` is no comment's start either: what it holds is a value, not source.
            b'r' if matches!(bytes.get(at + 1), Some(b'#' | b'"')) => {
                let mut cursor: usize = at + 1;
                let mut hashes: usize = 0;
                while bytes.get(cursor) == Some(&b'#') {
                    hashes += 1;
                    cursor += 1;
                }
                if bytes.get(cursor) != Some(&b'"') {
                    at += 1;
                    continue;
                }
                let mut closing: Vec<u8> = vec![b'"'];
                closing.extend(std::iter::repeat_n(b'#', hashes));
                let end: usize = find(bytes, cursor + 1, closing.as_slice()).unwrap_or(bytes.len());
                at = (end + closing.len()).min(bytes.len());
            }
            b'/' if bytes.get(at + 1) == Some(&b'/') => {
                while at < bytes.len() && bytes[at] != b'\n' {
                    out[at] = b' ';
                    at += 1;
                }
            }
            b'/' if bytes.get(at + 1) == Some(&b'*') => {
                // A block comment ends at the `*/` that matches its opening, and one inside it opens another:
                //  the depth is kept, so the end of the inner comment is no end of the outer one.
                out[at] = b' ';
                out[at + 1] = b' ';
                at += 2;
                let mut depth: usize = 1;
                while at < bytes.len() {
                    if bytes[at] == b'/' && bytes.get(at + 1) == Some(&b'*') {
                        depth += 1;
                        out[at] = b' ';
                        if at + 1 < bytes.len() {
                            out[at + 1] = b' ';
                        }
                        at += 2;
                        continue;
                    }
                    if bytes[at] == b'*' && bytes.get(at + 1) == Some(&b'/') {
                        depth -= 1;
                        out[at] = b' ';
                        if at + 1 < bytes.len() {
                            out[at + 1] = b' ';
                        }
                        at += 2;
                        if depth == 0 {
                            break;
                        }
                        continue;
                    }
                    if bytes[at] != b'\n' {
                        out[at] = b' ';
                    }
                    at += 1;
                }
            }
            _ => at += 1,
        }
    }
    return String::from_utf8(out).expect("masking keeps the text's bytes");
}

/// The first offset of `needle` in `bytes` from `from`.
fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || from >= bytes.len() {
        return None;
    }
    let mut at: usize = from;
    while at + needle.len() <= bytes.len() {
        if &bytes[at..at + needle.len()] == needle {
            return Some(at);
        }
        at += 1;
    }
    return None;
}

/// One clause of a source: its first line,
/// its last line,
/// and the masked and raw text between them.
#[derive(Clone, Debug)]
struct Clause {
    first: usize,
    masked: String,
    raw: String,
}

/// What:
///  The clauses of a masked source,
/// split at every `;` outside parentheses and brackets.
/// Why:
///   A statement is the unit a chain of setters belongs to,
/// and a `;` inside a closure or a `vec![a; 3]` is no boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// splitAtSemicolonsOutsideBrackets(maskedLines)
/// ```
fn clauses(masked: &str, raw: &str) -> Vec<Clause> {
    // Every boundary is a boundary: one line may hold several statements, and reading only the first would put the rest of
    // the line in the wrong clause.
    let mut found: Vec<Clause> = Vec::new();
    let bytes: &[u8] = masked.as_bytes();
    let mut depth: i32 = 0;
    let mut from: usize = 0;
    let mut at: usize = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            // A block closes a statement that ends with its last expression and no `;`, so a `}` at the outside level ends
            // the clause too; a struct literal's `}` only ends it early, which costs nothing.
            b';' | b'}' if depth <= 0 => {
                found.push(clause_of(masked, raw, from, at + 1));
                from = at + 1;
            }
            _ => {}
        }
        at += 1;
    }
    if from < masked.len() {
        found.push(clause_of(masked, raw, from, masked.len()));
    }
    return found;
}

/// The clause between the byte offsets `from` and `to` of both listings,
/// with the lines it starts and ends on.
fn clause_of(masked: &str, raw: &str, from: usize, to: usize) -> Clause {
    let first: usize = masked[..from].matches('\n').count() + 1;
    return Clause { first, masked: masked[from..to].to_owned(), raw: raw.get(from..to).unwrap_or("").to_owned() };
}

/// What:
///  The program a clause names,
/// as the source spells it,
/// when the clause builds its own command.
/// Why:
///   `Command::new(<program>)` and `service_command()` are the two shapes this crate starts a child with;
/// anything else is a command this reader cannot see,
/// and the guard treats that as a start it must judge without a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const program = /Command::new\(([^()]*)\)/u.exec(clause)?.[1] ?? (/service_command\(\)/u.test(clause) ? 'service_command()' : null);
/// ```
fn program_of(raw: &str) -> Option<String> {
    if let Some(at) = raw.find("Command::new(") {
        let from: usize = at + "Command::new(".len();
        let bytes: &[u8] = raw.as_bytes();
        let mut depth: i32 = 1;
        let mut index: usize = from;
        let mut in_string: bool = false;
        while index < bytes.len() && depth > 0 {
            match bytes[index] {
                b'\\' if in_string => index += 2,
                b'"' => {
                    in_string = !in_string;
                    index += 1;
                }
                b'(' | b'[' if !in_string => {
                    depth += 1;
                    index += 1;
                }
                b')' | b']' if !in_string => {
                    depth -= 1;
                    index += 1;
                }
                _ => index += 1,
            }
        }
        let end: usize = index.saturating_sub(1).max(from);
        return Some(raw[from..end].trim().to_owned());
    }
    if raw.contains("service_command()") {
        return Some(String::from("service_command()"));
    }
    return None;
}

/// The program name a `Command::new` argument names,
/// without its quotes,
/// when it is one quoted text.
pub fn program_name(program: &str) -> &str {
    return program.trim().trim_matches('"');
}

/// What:
///  The identifier a terminal is called on,
/// when it is called on a bare binding,
/// so its clauses can be gathered.
/// Why:
///   A command is configured in clauses of its own before the clause that starts it
/// (`command.args(…).stderr(…); … command.spawn()`),
/// and the setters are where the streams are given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const receiver = /(?<![\w)\]}])([a-z_][a-z0-9_]*)\s*$/u.exec(clause.slice(0, at))?.[1] ?? null;
/// ```
fn receiver_of(masked: &str, at: usize) -> Option<String> {
    let before: &str = masked[..at.min(masked.len())].trim_end();
    let name: String = before.chars().rev().take_while(|letter| { return letter.is_alphanumeric() || *letter == '_'; }).collect::<Vec<char>>().into_iter().rev().collect();
    if name.is_empty() {
        return None;
    }
    let head: &str = &before[..before.len() - name.len()];
    let trimmed: &str = head.trim_end();
    // The name is a binding this clause configures only when it stands as an expression of its own: after `=`, `(`, `[`,
    // `,`, `;`, `{`, `}`, `&`, `return` or `let`, or at the start. `fixture.root.join(…)` and `answer.join()` name no binding
    // of the chain, and following them would read another clause's command as this clause's.
    let bare: bool = trimmed.is_empty()
        || trimmed.ends_with('=')
        || trimmed.ends_with('(')
        || trimmed.ends_with('[')
        || trimmed.ends_with(',')
        || trimmed.ends_with(';')
        || trimmed.ends_with(':')
        || trimmed.ends_with('{')
        || trimmed.ends_with('}')
        || trimmed.ends_with('&')
        || trimmed.ends_with("return")
        || trimmed.ends_with("let")
        || trimmed.ends_with("mut");
    if !bare {
        return None;
    }
    return Some(name);
}

/// What:
///  The binding whose clauses configure the command this start runs:
///  the receiver of a method chain (`command.spawn()`),
///  or the command `spawn_owned` is handed (`spawn_owned(&mut command)`).
/// Why:
///   The setters are in clauses of their own,
///  and a start whose command is a binding must be read with them;
///  without the name the reader cannot say what the child is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const followed = terminal.kind === 'spawn_owned' ? commandArgument(masked, at) : receiverOf(masked, at);
/// ```
fn binding_followed(masked: &str, terminal: &FoundTerminal) -> Option<String> {
    if terminal.kind == Terminal::Followed {
        // A followed call is made on the binding the chain configures (`command.status_retrying()`), so its clauses say what
        // the child is given and whether the words name the service.
        return receiver_of(masked, terminal.offset);
    }
    if terminal.kind != Terminal::SpawnOwned {
        return receiver_of(masked, terminal.offset);
    }
    let open_relative: usize = masked.get(terminal.offset..)?.find('(')?;
    let open: usize = terminal.offset + open_relative;
    let argument: &str = masked[open + 1..].trim_start();
    let rest: &str = argument.strip_prefix("&mut ").or_else(|| { return argument.strip_prefix('&'); }).unwrap_or(argument);
    let name: String = rest.chars().take_while(|letter| { return letter.is_alphanumeric() || *letter == '_'; }).collect();
    if name.is_empty() {
        return None;
    }
    return Some(name);
}

/// What:
///  Whether the clause runs the wrapper's service:
///  it names the service mode's words,
///  the support that builds a service start,
///  or the executable the service runs as.
/// Why:
///   A start of the service is what the guard's streams rule is about,
/// and the words are how the source says so in a start built through a shell or a tracer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runsService = /SERVICE_WORDS|EARLIER_SERVICE_WORDS|SERVICE_PROGRAM|service_executable\(\)|"service",\s*"run"/u.test(rawClause)
/// ```
fn runs_the_service(raw: &str) -> bool {
    for word in ["SERVICE_WORDS", "EARLIER_SERVICE_WORDS", "SERVICE_PROGRAM", "service_executable()", "service_command()"] {
        if raw.contains(word) {
            return true;
        }
    }
    let quoted: bool = raw.contains("\"service\"") && raw.contains("\"run\"");
    return quoted;
}


/// What:
///  The names of this crate's functions that start a child,
///  with what each gives that child,
///  read from the sources the caller hands in.
/// Why:
///   A list of terminal names misses the next name
/// (a review of 2026-11-14, finding 4: `.status_retrying()` starts a child and the reader passed it).
/// A function that returns `Child`, `Output` or `ExitStatus` has started a child,
/// and so has one that ends in a call to such a function,
///  which is what the loop below follows until nothing new is found.
///  The shape of the body says what the child is given:
/// a body that captures,
///  or that sets a standard error,
///  needs no setter at the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = new Map(); for (const fn of functions) if (returnsChild(fn)) names.set(fn.name, shapeOf(fn.body)); untilStable(() => alsoNamesThatCallThese())
/// ```
pub fn read_names(sources: &[String]) -> Names {
    // The functions are read once: a crate of thousands of functions is no place for a rescan per pass.
    let mut all: Vec<Function> = Vec::new();
    for source in sources {
        all.extend(functions(mask(source).as_str()));
    }
    let mut names: Names = Names { starts: BTreeMap::new(), commands: BTreeSet::new() };
    for function in &all {
        let returned: &str = return_type(function.signature.as_str());
        if mentions_a_child(returned) {
            names.starts.insert(function.name.clone(), shape_of_body(function.body.as_str()));
        }
        if mentions_identifier(returned, "Command") {
            let _added: bool = names.commands.insert(function.name.clone());
        }
    }
    return names;
}

/// What:
///  The masked body of the function `name` in `source`,
///  or nothing when the source does not define it with a body.
/// Why:
///  A rule about one launcher reads that launcher's body and nothing else,
/// so a setting beside it in the same file neither satisfies the rule nor hides a loss.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bodyOf = (source, name) => functions(mask(source)).find((fn) => fn.name === name)?.body
/// ```
pub fn function_body(source: &str, name: &str) -> Option<String> {
    return functions(mask(source).as_str()).into_iter().find(|function| { return function.name == name; }).map(|function| { return function.body; });
}

/// Every name `masked` calls at the outside level,
/// with the offset the name starts at.
/// A call inside an argument list is no method of the chain:
/// `Command::new("sh").arg(script.as_path())` names `as_path`, and `as_path` is no start.
fn called_names_at_top_level(masked: &str) -> Vec<(usize, String)> {
    return called_names_at_depth(masked, false);
}

/// The calls of `masked`,
/// each with how deep in brackets it sits,
/// or only those at the outside level.
fn called_names_at_depth(masked: &str, every_depth: bool) -> Vec<(usize, String)> {
    let bytes: &[u8] = masked.as_bytes();
    let mut found: Vec<(usize, String)> = Vec::new();
    let mut depth: i32 = 0;
    let mut start: usize = 0;
    while start < bytes.len() {
        match bytes[start] {
            b'(' => {
                let mut name_end: usize = start;
                while name_end > 0 && is_word_byte(bytes[name_end - 1]) {
                    name_end -= 1;
                }
                if name_end < start {
                    let name: &str = &masked[name_end..start];
                    let first: u8 = bytes[name_end];
                    if (first.is_ascii_alphabetic() || first == b'_') && (every_depth || depth == 0) {
                        found.push((name_end, name.to_owned()));
                    }
                }
                depth += 1;
            }
            // A block's braces are no brackets: a call in a function body is still at the outside level.
            b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            _ => {}
        }
        start += 1;
    }
    return found;
}

/// What:
///  The names this crate's sources give to a start and to a command,
///  and what each start gives its child.
/// Why:
///   A name alone is no proof:
/// `new`,
/// `wait`,
/// `kill` and `finish` are names of many types,
/// so a call counts as a start only when the function's return type says it hands back a started child,
///  and a method call counts only when the binding it is called on was built as a command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Names = { starts: Map<string, Shape>; commands: Set<string> }
/// ```
pub struct Names {
    /// The functions whose return type is a started child, with what each gives it.
    pub starts: BTreeMap<String, Shape>,
    /// The functions whose return type is a `Command`.
    pub commands: BTreeSet<String>,
}

/// The return type of a signature:
/// what stands after `->`,
/// or nothing when the function returns nothing.
fn return_type(signature: &str) -> &str {
    return match signature.find("->") {
        Some(at) => &signature[at + 2..],
        None => "",
    };
}

/// Whether a return type names a started child:
/// a `Child`,
/// an `Output` or an `ExitStatus`.
fn mentions_a_child(returned: &str) -> bool {
    for word in ["Child", "Output", "ExitStatus"] {
        if mentions_identifier(returned, word) {
            return true;
        }
    }
    return false;
}

/// One function of a masked source:
/// its name,
/// its signature,
/// and its body.
struct Function {
    name: String,
    signature: String,
    body: String,
}

/// Every function a masked source defines,
/// with the body's own text.
fn functions(masked: &str) -> Vec<Function> {
    let mut found: Vec<Function> = Vec::new();
    let bytes: &[u8] = masked.as_bytes();
    let mut at: usize = 0;
    while let Some(start) = find(bytes, at, b"fn ") {
        at = start + 3;
        let after: &str = &masked[at..];
        let name: String = after.trim_start().chars().take_while(|letter| { return letter.is_alphanumeric() || *letter == '_'; }).collect();
        if name.is_empty() {
            continue;
        }
        // The signature runs from the name to the body's `{`, and a declaration without a body ends at `;`.
        let name_at: usize = at + after.len() - after.trim_start().len();
        let Some(open) = masked[name_at..].find('{') else { continue };
        let Some(close) = masked[name_at..].find(';') else { continue };
        if close < open {
            continue;
        }
        let body_start: usize = name_at + open;
        let Some(body_end) = matching_brace(masked, body_start) else { continue };
        found.push(Function {
            name,
            signature: masked[name_at..body_start].to_owned(),
            body: masked[body_start + 1..body_end].to_owned(),
        });
        at = body_end;
    }
    return found;
}

/// The offset of the `}` that closes the `{` at `open`,
/// or nothing when the source does not close it.
fn matching_brace(masked: &str, open: usize) -> Option<usize> {
    let bytes: &[u8] = masked.as_bytes();
    let mut depth: i32 = 0;
    let mut at: usize = open;
    while at < bytes.len() {
        match bytes[at] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(at);
                }
            }
            _ => {}
        }
        at += 1;
    }
    return None;
}

/// What a body gives the child it starts.
fn shape_of_body(body: &str) -> Shape {
    return Shape {
        captures: body.contains(".output("),
        sets_error: body.contains(".stderr("),
        sets_all: body.contains(".stdin(") && body.contains(".stdout(") && body.contains(".stderr("),
    };
}

/// What:
///  Every start of a child in `raw`,
///  with the standard error it gives the child and the program it names.
/// Why:
///   This is the one place the guard learns a start,
/// so a form it cannot read is visible here and nowhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const starts = readStarts(source); for (const start of starts) judge(start);
/// ```
pub fn starts(raw: &str) -> Vec<Start> {
    return starts_following(raw, &read_names(&[String::from(raw)]));
}

/// What:
///  Every start of a child in `raw`,
///  following the functions of this crate that the caller names as starts.
/// Why:
///  A start may be a call to a helper the crate defines (`status_retrying`, `git`, `run`),
/// and the reader then needs the crate's own names to see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const starts = startsFollowing(text, crateStartNames(allSources))
/// ```
pub fn starts_following(raw: &str, names: &Names) -> Vec<Start> {
    let masked: String = mask(raw);
    let found: Vec<Clause> = clauses(masked.as_str(), raw);
    let mut all: Vec<Start> = Vec::new();
    for (index, clause) in found.iter().enumerate() {
        // The clause's own chain, following the binding a call is made on, is what says whether a method call is a
        // start at all and whether the child is given a standard error.
        let chain_for_the_walk: (String, String) = chain_of(found.as_slice(), index, clause, None);
        for terminal in terminals(clause, chain_for_the_walk.0.as_str(), names) {
            let (chain_masked, chain_raw): (String, String) = chain_of(found.as_slice(), index, clause, Some(&terminal));
            let helper: Shape = terminal.name.as_deref().and_then(|name| { return names.starts.get(name).copied(); }).unwrap_or(Shape { captures: false, sets_error: false, sets_all: false });
            // A setter is judged by what it is given: `.stderr(Stdio::inherit())` hands the child the run's own stream, and a
            //  chain that does that at any of the three is no isolation whatever else it sets.
            let hands_over_the_run_s_stream: bool = chain_masked.contains("Stdio::inherit");
            let standard_error: StandardError = if hands_over_the_run_s_stream {
                StandardError::Inherited
            } else if terminal.kind == Terminal::Output || chain_masked.contains(".stderr(") || helper.captures || helper.sets_error {
                StandardError::Given
            } else {
                StandardError::Inherited
            };
            let line: usize = clause.first + clause.masked[..terminal.offset].matches('\n').count();
            let serves: bool = runs_the_service(chain_raw.as_str());
            let anchor: String = raw.lines().nth(line.saturating_sub(1)).unwrap_or("").trim().to_owned();
            all.push(Start {
                line,
                terminal: terminal.kind,
                program: program_of(chain_raw.as_str()),
                anchor,
                standard_error,
                chain: chain_raw,
                runs_the_service: serves,
                followed: terminal.name.clone(),
            });
        }
    }
    return all;
}

/// What:
///  The masked and raw text of a clause together with the clauses of the binding its terminal is called on,
///  or with the clauses of the binding its first call names when no terminal is given.
/// Why:
///   A command is configured in clauses of its own before the clause that starts it,
/// so the setters are there and not where the call is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [masked, raw] = chainOf(found, index, clause, terminal)
/// ```
fn chain_of(found: &[Clause], index: usize, clause: &Clause, terminal: Option<&FoundTerminal>) -> (String, String) {
    let binding: Option<String> = match terminal {
        Some(found_terminal) => binding_followed(clause.masked.as_str(), found_terminal),
        None => first_called_binding(clause.masked.as_str()),
    };
    let Some(name) = binding else {
        return (clause.masked.clone(), clause.raw.clone());
    };
    let mut masked_pieces: Vec<String> = Vec::new();
    let mut raw_pieces: Vec<String> = Vec::new();
    for earlier in found[..index].iter().rev() {
        if mentions_identifier(earlier.masked.as_str(), name.as_str()) {
            masked_pieces.push(earlier.masked.clone());
            raw_pieces.push(earlier.raw.clone());
        }
        if binds_identifier(earlier.masked.as_str(), name.as_str()) {
            break;
        }
    }
    masked_pieces.reverse();
    raw_pieces.reverse();
    // The binding is a child when the first thing its own clause does is start it: `let child = command.spawn()`.
    // What the clause then calls is called on that child, and following it would read another clause's command.
    if let Some(earliest) = raw_pieces.first() {
        let masked_earliest: &str = masked_pieces.first().map(String::as_str).unwrap_or("");
        let starts_it: bool = called_names_at_top_level(masked_earliest).first().is_some_and(|(_, called)| {
            return STARTING_METHODS.contains(&called.as_str()) || called == "spawn_owned" || called == "exec";
        });
        let _ = earliest;
        if starts_it {
            return (clause.masked.clone(), clause.raw.clone());
        }
    }
    let mut masked: String = masked_pieces.join("\n");
    let mut raw: String = raw_pieces.join("\n");
    if !masked.is_empty() {
        masked.push('\n');
        raw.push('\n');
    }
    masked.push_str(clause.masked.as_str());
    raw.push_str(clause.raw.as_str());
    return (masked, raw);
}

/// The binding the first call in `masked` is made on,
/// when it is made on a bare binding,
/// or nothing.
fn first_called_binding(masked: &str) -> Option<String> {
    for (offset, _) in called_names_at_top_level(masked) {
        // A method's receiver is the text before its dot, which `receiver_of` reads from the dot's offset.
        let at: usize = if offset >= 1 && masked.as_bytes().get(offset - 1) == Some(&b'.') { offset - 1 } else { offset };
        if let Some(name) = receiver_of(masked, at) {
            return Some(name);
        }
    }
    return None;
}

/// A terminal call found in a clause:
/// what it is and where in the clause's text it starts.
struct FoundTerminal {
    kind: Terminal,
    offset: usize,
    /// The helper's name, when this terminal is a call to a function of this crate.
    name: Option<String>,
}

/// The methods that start a command,
/// read as raw terminals above:
/// a chain that reaches one has ended the command's life, and a method after it is called on the child.
const STARTING_METHODS: &[&str] = &["spawn", "status", "output"];

/// The methods that hand back a value of the command's own and end its chain:
/// a read of its arguments or environment is no command to configure further.
const COMMAND_GETTERS: &[&str] = &["get_args", "get_envs", "get_current_dir", "as_std", "into_std"];

/// The methods of a command that configure it and start nothing:
/// the standard library's `std::process::Command` setters and getters,
/// read from its documentation.
/// A chain that ends in any other method has started that command's child,
/// which is how this reader finds a start the crate names something else (`status_retrying`).
const CONFIGURING_METHODS: &[&str] = &[
    "arg", "args", "arg0", "env", "envs", "env_clear", "env_remove", "current_dir", "stdin", "stdout", "stderr", "process_group", "pre_exec", "before_exec", "uid", "gid", "groups",
    "kill_on_drop",
];

/// The terminals of a clause:
/// a raw start (`output`, `status`, `spawn`, `spawn_owned`) when the clause holds one,
/// and otherwise the method that ends a command this clause builds,
/// when that method is not one of the command's configuration calls.
fn terminals(clause: &Clause, chain_masked: &str, names: &Names) -> Vec<FoundTerminal> {
    let masked: &str = clause.masked.as_str();
    let mut found: Vec<FoundTerminal> = Vec::new();
    for word in [".output(", ".status(", ".spawn()", "spawn_owned("] {
        let mut from: usize = 0;
        while let Some(relative) = masked[from..].find(word) {
            let at: usize = from + relative;
            from = at + word.len();
            // `wait_with_output(` is a read of a started child, not a start.
            if word == ".output(" && masked[..at].ends_with("wait_with") {
                continue;
            }
            // A definition names the support, it does not start a child through it.
            if word == "spawn_owned(" && is_definition(masked, at) {
                continue;
            }
            found.push(FoundTerminal { kind: terminal_of(word), offset: at, name: None });
        }
    }
    if !found.is_empty() {
        found.sort_by_key(|terminal| { return terminal.offset; });
        return found;
    }
    // The chain is walked from its first call: a clause that starts with a command this crate builds, and whose methods
    // are that command's configuration calls until one is not, has started the command's child at that method. The walk
    // reads the chain, because the command is built in the binding's own clauses, and the terminal is looked for in this
    // clause so the finding still names the line it is on.
    let calls: Vec<(usize, String)> = called_names_at_top_level(chain_masked);
    let chain_bytes: &[u8] = chain_masked.as_bytes();
    let mut command: bool = false;
    let mut at_the_build: bool = true;
    for (offset, name) in calls.iter() {
        // A definition's own name is no call of the chain: `fn f() {` starts one.
        if is_definition(chain_masked, *offset) {
            continue;
        }
        let method_call: bool = *offset >= 1 && chain_bytes[*offset - 1] == b'.';
        if at_the_build {
            at_the_build = false;
            // A name this crate gives to a command *and* to a started child is ambiguous, and an ambiguous name builds no
            // command here: `quiet` is a command in one file and a child in another, and its call is no proof of either.
            let gives_a_command: bool = names.commands.contains(name.as_str()) && !names.starts.contains_key(name.as_str());
            let built_here: bool = name == "service_command" || (!method_call && gives_a_command);
            let new_of_command: bool = name == "new" && chain_masked[..*offset].ends_with("Command::");
            command = built_here || new_of_command || (method_call && gives_a_command);
            continue;
        }
        if !command || !method_call {
            command = false;
            continue;
        }
        // A method that starts the command, or that hands back a value of its own, ends its chain: what a later method is
        // called on is the child or the value, not the command.
        if STARTING_METHODS.contains(&name.as_str()) || COMMAND_GETTERS.contains(&name.as_str()) || name == "exec" {
            break;
        }
        if CONFIGURING_METHODS.contains(&name.as_str()) {
            continue;
        }
        let call: String = format!(".{name}(");
        let Some(in_clause) = masked.find(call.as_str()) else { break };
        found.push(FoundTerminal { kind: Terminal::Followed, offset: in_clause, name: Some(name.clone()) });
        break;
    }
    return found;
}

/// The terminal a word names.
fn terminal_of(word: &str) -> Terminal {
    return match word {
        ".output(" => Terminal::Output,
        ".status(" => Terminal::Status,
        ".spawn()" => Terminal::Spawn,
        _ => Terminal::SpawnOwned,
    };
}

/// What:
///  Whether the call at `at` is the name in a `fn` definition rather than a call.
/// Why:
///   `fn spawn_owned(` names the support and starts nothing,
///  and the head of a definition is the few characters before its name:
/// the name directly follows `fn`,
///  so the text before the call ends with `fn`.
///  Reading the whole line would refuse a call that happens to sit on a line whose start is a definition.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isDefinition = (masked, at) => /fn$/u.test(masked.slice(0, at).trimEnd())
/// ```
fn is_definition(masked: &str, at: usize) -> bool {
    let head: &str = &masked[..at.min(masked.len())];
    return head.trim_end().ends_with("fn");
}

/// Whether `masked` mentions `name` as one identifier.
fn mentions_identifier(masked: &str, name: &str) -> bool {
    let bytes: &[u8] = masked.as_bytes();
    let needle: &[u8] = name.as_bytes();
    let mut at: usize = 0;
    while let Some(relative) = find(bytes, at, needle) {
        let before_ok: bool = relative == 0 || !is_word_byte(bytes[relative - 1]);
        let after: usize = relative + needle.len();
        let after_ok: bool = after >= bytes.len() || !is_word_byte(bytes[after]);
        if before_ok && after_ok {
            return true;
        }
        at = relative + 1;
    }
    return false;
}

/// Whether a byte continues an identifier.
fn is_word_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'_';
}

/// Whether `masked` binds `name` with a `let` or assigns it.
fn binds_identifier(masked: &str, name: &str) -> bool {
    let let_word: String = format!("let {name}");
    let let_mut: String = format!("let mut {name}");
    if masked.contains(let_word.as_str()) || masked.contains(let_mut.as_str()) {
        return true;
    }
    let assigned: String = format!("{name} = ");
    return masked.contains(assigned.as_str());
}

/// What:
///  The reader's own controls:
///  the masking of strings and comments,
///  the clauses,
///  the binding a start follows,
///  and the four terminals with the program each start names.
/// Why:
///   A rule that cannot be shown to read the right thing is no rule:
/// each control below is one shape a source really holds,
/// and the last is the review's own file,
/// which is the start the guard used to pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// describe('starts', () => { it('reads a shell with nothing set', ...); it('follows a binding', ...) })
/// ```

#[test]
fn the_mask_hides_strings_characters_and_comments_and_keeps_each_line_s_length() {
    let source: &str = "let a = \"c.stderr(Stdio::inherit())\"; // a.spawn()\nlet b = 'x'; /* .output() */\n";
    let masked: String = mask(source);
    assert_eq!(masked.len(), source.len(), "masking keeps the lengths");
    assert_eq!(masked.lines().count(), source.lines().count(), "masking keeps the lines");
    assert!(!masked.contains("Stdio::inherit"), "{masked:?}");
    assert!(!masked.contains(".spawn()"), "{masked:?}");
    assert!(!masked.contains(".output()"), "{masked:?}");
    assert!(masked.contains("let a ="), "{masked:?}");
}

#[test]
fn one_line_of_two_statements_is_two_clauses_and_a_bracket_holds_no_boundary() {
    let found: Vec<Start> = starts("fn f() { let a = 1; let b = vec![1; 2]; let c = g(a; b); }");
    assert_eq!(found.len(), 0, "no start here");
    let starts_found = starts("fn f() { let a = Command::new(\"sh\").arg(\"-c\").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status(); let b = 1; }");
    assert_eq!(starts_found.len(), 1);
    assert_eq!(starts_found[0].standard_error, StandardError::Given);
    assert_eq!(starts_found[0].program.as_deref(), Some("\"sh\""));
}

#[test]
fn a_binding_s_clauses_are_read_with_the_start() {
    let source: &str = "fn f() {\n    let mut command: Command = Command::new(\"sleep\").arg(\"600\");\n    command.args([\"a\"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());\n    let child = command.spawn().expect(\"a child\");\n}\n";
    let found = starts(source);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].standard_error, StandardError::Given, "the setters are in the binding's own clause");
    assert_eq!(found[0].program.as_deref(), Some("\"sleep\""));
}

#[test]
fn the_command_a_support_is_handed_is_followed_as_a_receiver_is() {
    let source: &str = "fn f() {\n    let mut command: Command = Command::new(\"sleep\");\n    let child = spawn_owned(&mut command).expect(\"a step\");\n}\n";
    let found = starts(source);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].terminal, Terminal::SpawnOwned);
    assert_eq!(found[0].standard_error, StandardError::Inherited);
    assert_eq!(found[0].program.as_deref(), Some("\"sleep\""), "the command is the binding the support is handed");
}

#[test]
fn a_definition_a_thread_a_read_and_a_capture_are_no_inheriting_start() {
    // The definition of the support is no start, a thread takes an argument, `wait_with_output` reads a child already started,
    // and `output()` captures both streams by itself.
    let definition: Vec<Start> = starts("pub fn spawn_owned(command: &mut Command) -> std::io::Result<Child> { command.spawn() }");
    assert_eq!(definition.len(), 1, "the body's spawn is a start, the definition is not: {definition:?}");
    let thread: Vec<Start> = starts("fn f() { std::thread::scope(|scope| { let _ = scope.spawn(|| work()); }); }");
    assert_eq!(thread.len(), 0, "a thread is no child");
    let read: Vec<Start> = starts("fn f() { let out = command.wait_with_output(); }");
    assert_eq!(read.len(), 0, "a read is no start");
    let capture: Vec<Start> = starts("fn f() { let out = std::process::Command::new(\"git\").arg(\"status\").output(); }");
    assert_eq!(capture.len(), 1);
    assert_eq!(capture[0].standard_error, StandardError::Given, "`output()` captures both streams");
}

#[test]
fn the_service_support_is_read_as_a_start_of_the_service() {
    let source: &str = "fn f() { let mut command = super::support::service_command(); command.args([\"commit\"]); let child = command.spawn().expect(\"the service\"); }";
    let found = starts(source);
    assert_eq!(found.len(), 1);
    assert!(found[0].runs_the_service, "the support that builds a service start names it");
}
