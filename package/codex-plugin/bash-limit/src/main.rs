//! Native Codex hook entry point;
//! stdout contains only the host's JSON protocol.

/// What:
/// `mod` loads sibling source files,
/// like TypeScript module imports.
/// Why:
/// Separate resource checks from JSON and shell encoding.
/// TypeScript equivalent:
/// `import * as limits from './limits';`
mod limits;
/// Load the sibling protocol implementation.
mod protocol;

/// What:
/// `use` imports a standard-library name;
/// `::` separates module names.
/// Why:
/// Report failures without an additional error-handling dependency.
/// TypeScript equivalent:
/// `import { Error } from 'node:io';`
use std::io;
/// Import the trait that adds byte-writing methods to stdout and stderr.
use std::io::Write;
/// Import owned paths,
/// unlike borrowed `Path` references.
use std::path::PathBuf;
/// Import the native process exit-status value.
use std::process::ExitCode;

/// What:
/// `fn` declares a function;
/// `Result<String>` carries text or an I/O error.
/// Why:
/// Keep failures out of the success JSON stream.
/// `String` owns text;
/// its sibling `&str` only borrows existing text.
/// TypeScript equivalent:
/// `function run(): string { /* may throw */ }`
fn run() -> io::Result<String> {
    // What: `mut` permits the argument iterator to advance.
    // Why: Consume the mode and parent path without a CLI framework.
    // TypeScript equivalent: `const args = process.argv.slice(2).values();`
    let mut arguments = std::env::args().skip(1);
    // What: `ok_or_else` converts a missing optional value into an error; `?` propagates it.
    // Why: Missing arguments must stop execution rather than choose an unrestricted default.
    // TypeScript equivalent: `const mode = args.next().value ?? throwError('...');`
    let mode = arguments.next().ok_or_else(|| {
        // Return the argument diagnostic from this zero-argument closure (`||`).
        return io::Error::other("Expected --hook or --initialize followed by the worker slice path.");
    })?;
    // Require the independently configured parent path.
    let parent_text = arguments.next().ok_or_else(|| {
        // Return a diagnostic naming the missing input.
        return io::Error::other("The worker slice path is required.");
    })?;
    if arguments.next().is_some() {
        // What: `Err` is the failure variant, paired with `Ok` for success.
        // Why: Reject accidental or unsupported arguments.
        // TypeScript equivalent: `throw new Error('Unexpected argument.');`
        return Err(io::Error::other("Unexpected argument after the worker slice path."));
    }
    // What: `PathBuf::from` owns the supplied path rather than borrowing its text.
    // Why: Filesystem helpers require a path value.
    // TypeScript equivalent: `const parent = parentText;`
    let parent = PathBuf::from(parent_text);
    if !parent.is_absolute() {
        // Refuse working-directory-dependent budget selection.
        return Err(io::Error::other("The worker slice path must be absolute."));
    }
    if mode == "--initialize" {
        // What: `&parent` lends the path without moving ownership; `?` propagates failure.
        // Why: Preparation and later hook calls use the same parent.
        // TypeScript equivalent: `limits.initialize(parent);`
        limits::initialize(&parent)?;
        // What: `Ok` returns successful output; `String::from` owns a copied literal.
        // Why: SessionStart needs valid empty JSON without per-command work.
        // TypeScript equivalent: `return '{}';`
        return Ok(String::from("{}"));
    }
    if mode != "--hook" {
        // Refuse an unknown operating mode.
        return Err(io::Error::other("Expected --hook or --initialize."));
    }
    // Read the host's complete JSON event; read errors propagate to the blocking exit.
    let input = io::read_to_string(io::stdin())?;
    // Borrow input and parent for the protocol consumer.
    return protocol::response(&input, &parent);
}

/// What:
/// `main` returns `ExitCode`,
/// the operating system's exit status.
/// Why:
/// Codex treats hook exit status 2 as a blocked call,
/// including encoding failures.
/// TypeScript equivalent:
/// `try { stdout.write(run()); } catch (error) { process.exitCode = 2; }`
fn main() -> ExitCode {
    // What: `match` separates the success and failure variants of `Result`.
    // Why: Rust reports ordinary failures as values instead of throwing exceptions.
    // TypeScript equivalent: `try { const output = run(); } catch (error) { ... }`
    let failure = match run() {
        // Handle successful protocol generation.
        Ok(output) => {
            // Borrow the encoded bytes; writing failure must still block the tool.
            match io::stdout().write_all(output.as_bytes()) {
                // Return the standard successful exit status after writing the response.
                Ok(()) => return ExitCode::SUCCESS,
                // Retain the actual output failure for stderr.
                Err(error) => error,
            }
        }
        // Preserve the original setup, input, or validation error.
        Err(error) => error,
    };
    // Own a diagnostic buffer so stderr remains independent of the JSON response.
    let mut diagnostic = String::from("Codex worker limits: ");
    // What: `to_string` formats the error into owned text; `&` borrows that text for appending.
    // Why: Preserve the actual cause rather than only an exit status.
    // TypeScript equivalent: `diagnostic += String(failure);`
    diagnostic.push_str(&failure.to_string());
    diagnostic.push('\n');
    // What: `let _` explicitly ignores a secondary stderr failure.
    // Why: A closed stderr must not change the blocking exit status.
    // TypeScript equivalent: `try { stderr.write(diagnostic); } catch { /* still exit 2 */ }`
    let _ = io::stderr().write_all(diagnostic.as_bytes());
    // Convert the required hook-blocking code to the native exit-status value.
    return ExitCode::from(2);
}
