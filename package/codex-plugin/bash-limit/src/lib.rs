//! Shell quotation shared by the hook encoder and the host fixture that installs it.
//!
//! The host test writes a hook command line that runs this same binary,
//! so it quotes paths with the rule below instead of a second spelling of it.
//! Two spellings of one injection-safety policy drift silently,
//! and the fixture would then exercise an encoder that the hook rewrite never uses.

/// What:
/// `pub` exposes the function to the binary and to the integration tests;
/// `&str` borrows text and the returned `String` owns the quoted copy.
/// Why:
/// A path interpolated into shell source must reach the shell as one literal word,
/// whatever bytes it holds.
/// POSIX single quotes take everything literally except the single quote itself,
/// which closes the quoting,
/// so each one becomes a close,
/// an escaped quote,
/// and a reopen.
/// TypeScript equivalent:
/// `function shellQuote(value: string): string { /* escaped copy */ }`
pub fn shell_quote(value: &str) -> String {
    // What: `mut` permits appending to an owned buffer; `String::from` copies the literal.
    // Why: Build the escaped path in one linear pass.
    // TypeScript equivalent: `let output = "'";`
    let mut output = String::from("'");
    for character in value.chars() {
        if character == '\'' {
            output.push_str("'\\''");
        } else {
            output.push(character);
        }
    }
    output.push('\'');
    return output;
}
