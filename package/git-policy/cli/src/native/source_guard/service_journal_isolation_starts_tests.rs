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
                out[at] = b' ';
                out[at + 1] = b' ';
                at += 2;
                while at + 1 < bytes.len() && !(bytes[at] == b'*' && bytes[at + 1] == b'/') {
                    if bytes[at] != b'\n' {
                        out[at] = b' ';
                    }
                    at += 1;
                }
                if at + 1 < bytes.len() {
                    out[at] = b' ';
                    out[at + 1] = b' ';
                    at += 2;
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
            b'\'' if bytes.get(at + 2) == Some(&b'\'') => {
                out[at] = b' ';
                out[at + 1] = b' ';
                out[at + 2] = b' ';
                at += 3;
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
    last: usize,
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
    let masked_lines: Vec<&str> = masked.lines().collect();
    let raw_lines: Vec<&str> = raw.lines().collect();
    let mut found: Vec<Clause> = Vec::new();
    let mut depth: i32 = 0;
    let mut first: Option<usize> = None;
    let mut buffer: Vec<usize> = Vec::new();
    for (index, line) in masked_lines.iter().enumerate() {
        let number: usize = index + 1;
        if first.is_none() {
            first = Some(number);
        }
        buffer.push(index);
        let mut ended: bool = false;
        for byte in line.bytes() {
            match byte {
                b'(' | b'[' => depth += 1,
                b')' | b']' => depth -= 1,
                b';' if depth <= 0 => {
                    ended = true;
                    break;
                }
                // A block closes a statement that ends with its last expression and no `;`, so a `}` at the outside
                // level ends the clause too; a struct literal's `}` only ends it early, which costs nothing.
                b'}' if depth <= 0 => {
                    ended = true;
                    break;
                }
                _ => {}
            }
        }
        if ended {
            found.push(clause(first.unwrap_or(number), number, &masked_lines, &raw_lines, buffer.as_slice()));
            buffer.clear();
            first = None;
        }
    }
    if !buffer.is_empty() {
        found.push(clause(first.unwrap_or(1), masked_lines.len(), &masked_lines, &raw_lines, buffer.as_slice()));
    }
    return found;
}

/// The clause made of `lines`, which are indices into both listings.
fn clause(first: usize, last: usize, masked_lines: &[&str], raw_lines: &[&str], lines: &[usize]) -> Clause {
    let mut masked: Vec<&str> = Vec::new();
    let mut raw: Vec<&str> = Vec::new();
    for index in lines {
        masked.push(masked_lines.get(*index).copied().unwrap_or(""));
        raw.push(raw_lines.get(*index).copied().unwrap_or(""));
    }
    return Clause { first, last, masked: masked.join("\n"), raw: raw.join("\n") };
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
    let before: &str = masked[..at].trim_end();
    let name: String = before.chars().rev().take_while(|letter| { return letter.is_alphanumeric() || *letter == '_'; }).collect::<Vec<char>>().into_iter().rev().collect();
    if name.is_empty() {
        return None;
    }
    let head: &str = &before[..before.len() - name.len()];
    let trimmed: &str = head.trim_end();
    if trimmed.ends_with(')') || trimmed.ends_with(']') || trimmed.ends_with('}') || trimmed.ends_with('&') {
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
    if terminal.kind != Terminal::SpawnOwned {
        return receiver_of(masked, terminal.offset);
    }
    let open_relative: usize = masked[terminal.offset..].find('(')?;
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
    let masked: String = mask(raw);
    let found: Vec<Clause> = clauses(masked.as_str(), raw);
    let mut all: Vec<Start> = Vec::new();
    for (index, clause) in found.iter().enumerate() {
        for terminal in terminals(clause.masked.as_str()) {
            let mut chain: Clause = clause.clone();
            let mut chain_raw: String = clause.raw.clone();
            let mut chain_masked: String = clause.masked.clone();
            if let Some(name) = binding_followed(clause.masked.as_str(), &terminal) {
                let word: String = name.clone();
                let mut masked_pieces: Vec<String> = Vec::new();
                let mut raw_pieces: Vec<String> = Vec::new();
                for earlier in found[..index].iter().rev() {
                    let mentions: bool = mentions_identifier(earlier.masked.as_str(), word.as_str());
                    if mentions {
                        masked_pieces.push(earlier.masked.clone());
                        raw_pieces.push(earlier.raw.clone());
                    }
                    if binds_identifier(earlier.masked.as_str(), word.as_str()) {
                        break;
                    }
                }
                masked_pieces.reverse();
                raw_pieces.reverse();
                masked_pieces.push(chain_masked);
                raw_pieces.push(chain_raw);
                chain_masked = masked_pieces.join("\n");
                chain_raw = raw_pieces.join("\n");
                chain.first = found[..index].iter().rev().find(|earlier| { return mentions_identifier(earlier.masked.as_str(), word.as_str()); }).map_or(clause.first, |earlier| { return earlier.first; });
                chain.last = clause.last;
            }
            let standard_error: StandardError = if terminal.kind == Terminal::Output || chain_masked.contains(".stderr(") {
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
            });
            let _ = &mut chain;
        }
    }
    return all;
}

/// A terminal call found in a clause:
/// what it is and where in the clause's text it starts.
struct FoundTerminal {
    kind: Terminal,
    offset: usize,
}

/// The terminals of a masked clause,
/// in the order they appear.
fn terminals(masked: &str) -> Vec<FoundTerminal> {
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
            found.push(FoundTerminal { kind: terminal_of(word), offset: at });
        }
    }
    found.sort_by_key(|terminal| { return terminal.offset; });
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

/// Whether the call at `at` is the name in a `fn` definition rather than a call.
fn is_definition(masked: &str, at: usize) -> bool {
    let head: &str = &masked[..at];
    let Some(line_start) = head.rfind('\n') else {
        return definition_head(head);
    };
    return definition_head(&head[line_start + 1..]);
}

/// Whether the text before a call reads `fn …` or `pub fn …`.
fn definition_head(head: &str) -> bool {
    let trimmed: &str = head.trim_start();
    return trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("pub(crate) fn ") || trimmed.starts_with("const fn ");
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
