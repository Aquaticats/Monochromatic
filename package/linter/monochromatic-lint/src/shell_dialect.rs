//! What:
//!  The shell dialects the workflow rules understand,
//!  the constructs each rule bans,
//!  and the reading of a `shell:` value.
//! Why:
//!  One table decides which construct is checked in which dialect,
//!  so a PowerShell backtick is not mistaken for command substitution.
//!  The dialect operator facts come from the fish language manual,
//!  the PowerShell `about_Operators` and `about_Pipeline_Chain_Operators` pages
//!  and the `cmd` command reference;
//!  doc/planning/workflow-shell-line-limit.md cites them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ShellDialect = 'posix' | 'fish' | 'powershell' | 'cmd';
//! type Construct = 'pipe' | 'and' | 'or' | 'pipeStderr' | 'semicolon' | 'dollarParen' | 'backtick' | 'ampersand';
//! ```

/// What:
///  The four syntax families a workflow step can be written in.
/// Why:
///  bash,
///  sh,
///  dash,
///  ash,
///  ksh and zsh share one operator grammar,
///  so each does not need its own variant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShellDialect = 'posix' | 'fish' | 'powershell' | 'cmd';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellDialect {
    /// bash,
    ///  sh,
    ///  dash,
    ///  ash,
    ///  ksh and zsh.
    Posix,
    /// fish.
    Fish,
    /// pwsh and Windows PowerShell.
    PowerShell,
    /// Windows `cmd.exe`.
    Cmd,
}

/// What:
///  `impl ShellDialect { ... }` attaches methods to the enum,
///  like methods on a TS class.
/// Why:
///  The virtual file name suffix and the dialect convert into each other in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ShellDialect { suffix(): string; static fromFilename(name: string): ShellDialect | undefined }
/// ```
impl ShellDialect {
    /// The virtual file suffix that configuration patterns select,
    ///  including the dot.
    pub fn suffix(self) -> &'static str {
        match self {
            ShellDialect::Posix => return ".sh",
            ShellDialect::Fish => return ".fish",
            ShellDialect::PowerShell => return ".ps1",
            ShellDialect::Cmd => return ".cmd",
        }
    }

    /// Read the dialect back from a virtual file name such as `ci.yml/3.ps1`.
    pub fn from_filename(name: &str) -> Option<ShellDialect> {
        // `Some(...)` is the present variant of an optional value, like returning a value instead of `undefined`.
        if name.ends_with(".sh") {
            return Some(ShellDialect::Posix);
        }
        if name.ends_with(".fish") {
            return Some(ShellDialect::Fish);
        }
        if name.ends_with(".ps1") {
            return Some(ShellDialect::PowerShell);
        }
        if name.ends_with(".cmd") {
            return Some(ShellDialect::Cmd);
        }
        // `None` is the absent variant, like returning `undefined`.
        return None;
    }
}

/// What:
///  The constructs the `shell/no-*` rules ban.
/// Why:
///  One variant per rule keeps each rule's occurrence count separate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Construct = 'pipe' | 'and' | 'or' | 'pipeStderr' | 'semicolon' | 'dollarParen' | 'backtick' | 'ampersand';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Construct {
    /// `|`.
    Pipe,
    /// `&&`.
    AndChain,
    /// `||`.
    OrChain,
    /// `|&`.
    PipeStderr,
    /// `;`.
    Semicolon,
    /// `$(...)`.
    DollarParen,
    /// A backtick command substitution.
    Backtick,
    /// A single `&`.
    Ampersand,
}

/// Every construct in the order the rules are registered.
pub const CONSTRUCTS: [Construct; 8] = [
    Construct::Pipe,
    Construct::AndChain,
    Construct::OrChain,
    Construct::PipeStderr,
    Construct::Semicolon,
    Construct::DollarParen,
    Construct::Backtick,
    Construct::Ampersand,
];

/// Rule identifiers and wording of each construct,
///  and the dialects that give it a command-composing meaning.
impl Construct {
    /// The rule identifier as written in configuration and carried in a finding's `code`.
    pub fn rule_id(self) -> &'static str {
        match self {
            Construct::Pipe => return "shell/no-pipe",
            Construct::AndChain => return "shell/no-and-chain",
            Construct::OrChain => return "shell/no-or-chain",
            Construct::PipeStderr => return "shell/no-pipe-stderr",
            Construct::Semicolon => return "shell/no-semicolon",
            Construct::DollarParen => return "shell/no-dollar-paren",
            Construct::Backtick => return "shell/no-backtick",
            Construct::Ampersand => return "shell/no-ampersand",
        }
    }

    /// The token as the author wrote it,
    ///  for the finding's message.
    pub fn token(self) -> &'static str {
        match self {
            Construct::Pipe => return "|",
            Construct::AndChain => return "&&",
            Construct::OrChain => return "||",
            Construct::PipeStderr => return "|&",
            Construct::Semicolon => return ";",
            Construct::DollarParen => return "$(",
            Construct::Backtick => return "`",
            Construct::Ampersand => return "&",
        }
    }

    /// Whether the dialect gives this construct a meaning that composes commands.
    ///  fish has no backtick substitution,
    ///  PowerShell uses the backtick as its escape and has no `|&`,
    ///  and `cmd` has no `;`,
    ///  `$(` or backtick.
    pub fn applies_to(self, dialect: ShellDialect) -> bool {
        match dialect {
            ShellDialect::Posix => return true,
            ShellDialect::Fish => return self != Construct::Backtick,
            ShellDialect::PowerShell => {
                return self != Construct::Backtick && self != Construct::PipeStderr;
            }
            ShellDialect::Cmd => {
                return matches!(
                    self,
                    Construct::Pipe
                        | Construct::AndChain
                        | Construct::OrChain
                        | Construct::Ampersand
                );
            }
        }
    }
}

/// What:
///  What a `shell:` value means for the workflow rules.
/// Why:
///  An interpreter that is not a shell is out of scope,
///  but a name nobody listed must not silently escape the ban.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShellClass = { kind: 'dialect'; dialect: ShellDialect } | { kind: 'outOfScope' } | { kind: 'unknown'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShellClass {
    /// A shell the rules check.
    Dialect(
        /// The syntax family of the shell.
        ShellDialect,
    ),
    /// A known interpreter that is not a shell,
    ///  such as `node` or `python`.
    OutOfScope,
    /// A value the rules cannot classify.
    Unknown(
        /// Why the value cannot be classified.
        String,
    ),
}

/// Interpreter names that are known not to be shells.
const INTERPRETERS: [&str; 16] = [
    "node", "nodejs", "deno", "bun", "tsx", "ts-node", "perl", "ruby", "php", "lua", "julia",
    "rscript", "pypy", "pypy3", "python", "python3",
];

/// What:
///  Name the word that selects the interpreter in a `shell:` value.
/// Why:
///  `env bash {0}` and `/usr/bin/bash -e {0}` run bash,
///  and the first word alone would call the first one `env`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function interpreterWord(value: string): string | undefined;
/// ```
fn interpreter_word(value: &str) -> Option<String> {
    // A quoted first word may contain spaces, as in "C:\Program Files\PowerShell\7\pwsh.exe".
    let trimmed: &str = value.trim_start();
    let quote: char = trimmed.chars().next().unwrap_or(' ');
    if quote == '"' || quote == '\'' {
        let inner: &str = &trimmed[1..];
        let closed: &str = inner.split(quote).next().unwrap_or(inner);
        return word_stem(closed);
    }
    let mut words = value.split_whitespace();
    // `env` hands over to the next word that is neither an option nor a VAR=value assignment.
    let mut candidate: Option<&str> = words.next();
    if candidate == Some("env") {
        candidate = None;
        for word in words {
            if !word.starts_with('-') && !word.contains('=') {
                candidate = Some(word);
                break;
            }
        }
    }
    let word: &str = candidate?;
    return word_stem(word);
}

/// What:
///  The lowercase program name of a path without its directory and `.exe`.
/// Why:
///  `/usr/bin/bash` and `C:\Git\bin\bash.exe` both name bash,
///  and an empty word names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wordStem(word: string): string | undefined;
/// ```
fn word_stem(word: &str) -> Option<String> {
    let base: &str = word.rsplit(['/', '\\']).next().unwrap_or(word);
    let lowered: String = base.to_ascii_lowercase();
    let stem: &str = lowered.strip_suffix(".exe").unwrap_or(lowered.as_str());
    if stem.is_empty() {
        return None;
    }
    return Some(String::from(stem));
}

/// What:
///  Classify a `shell:` value.
/// Why:
///  A shell is checked,
///  a known interpreter is skipped,
///  and anything else is a processing failure so the ban fails closed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classifyShell(value: string): ShellClass;
/// ```
pub fn classify_shell(value: &str) -> ShellClass {
    if value.contains("${{") {
        return ShellClass::Unknown(String::from(
            "the shell is an expression, so its dialect cannot be known; write a literal shell name",
        ));
    }
    let Some(name): Option<String> = interpreter_word(value) else {
        return ShellClass::Unknown(String::from("the shell value is empty"));
    };
    match name.as_str() {
        "bash" | "sh" | "dash" | "ash" | "ksh" | "zsh" => {
            return ShellClass::Dialect(ShellDialect::Posix);
        }
        "fish" => return ShellClass::Dialect(ShellDialect::Fish),
        "pwsh" | "powershell" => return ShellClass::Dialect(ShellDialect::PowerShell),
        "cmd" => return ShellClass::Dialect(ShellDialect::Cmd),
        _ => {}
    }
    // Versioned names such as python3.12 are the same interpreter family.
    if INTERPRETERS.contains(&name.as_str()) || name.starts_with("python") {
        return ShellClass::OutOfScope;
    }
    return ShellClass::Unknown(format!(
        "the interpreter {name} is neither a listed shell nor a listed non-shell interpreter"
    ));
}

/// Dialect table and classification controls stay outside release artifacts.
#[cfg(test)]
#[path = "shell_dialect_tests.rs"]
mod tests;
