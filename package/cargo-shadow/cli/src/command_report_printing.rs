//! What:
//! What the catalog commands print:
//! the findings of a refused step,
//! the report of an install,
//! and the report of an update.
//! Why:
//! `commands.rs` holds the setup every catalog command shares and the one command that runs them;
//! the printing was split out of it to keep each file under the code-line limit.
//! The items are re-exported from `commands`,
//! so their paths there are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // command-report-printing.ts
//! export function printFindings(heading: string, problems: string[]): void
//! export function printInstallReport(report: InstallReport, dryRun: boolean, sink: Writer): void
//! export function printUpdateReport(report: UpdateReport, dryRun: boolean, sink: Writer): void
//! ```

/// What:
/// `use std::io::Write;` imports `Write` from the standard library so this file can name it unqualified.
/// Why:
/// `Write` is used by `print_install_report`,
/// `print_update_report`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Write } from 'std/io';
/// ```
use std::io::Write;

/// What:
/// `use crate::install::InstallReport;` imports `InstallReport` from another module of this crate so this file can name it unqualified.
/// Why:
/// `InstallReport` is used by `print_install_report`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { InstallReport } from 'crate/install';
/// ```
use crate::install::InstallReport;
/// What:
/// `use crate::update::UpdateReport;` imports `UpdateReport` from another module of this crate so this file can name it unqualified.
/// Why:
/// `UpdateReport` is used by `print_update_report`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateReport } from 'crate/update';
/// ```
use crate::update::UpdateReport;
/// Prints findings under a heading,
/// with the pointer to the settings.
pub fn print_findings(
    heading: &str,
    problems: &[String],
) {
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      The findings go to standard error under a heading that names the refused step, one bullet per finding.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    eprintln!("{heading}: {} finding(s).", problems.len());
    for problem in problems {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("- {problem}");
    }
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("Settings live in rust-catalog.toml; see doc/decision/rust-dependency-catalog.md.");
}

/// Prints an install report.
pub fn print_install_report(
    report: &InstallReport,
    dry_run: bool,
    sink: &mut dyn Write,
) {
    // Macro `writeln!`: builds or prints text (or a list) from a template.
    // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
    // Why:      The report goes to a writer the caller supplies; a failed write to it is not a reason to fail a finished install, so the results are dropped.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // void expr;   // or try { expr } catch {}
    // ```
    let _ = writeln!(
        sink,
        "{} lockfiles, {} did not satisfy their manifests{}",
        report.crate_dirs.len(),
        report.stale.len(),
        if report.stale.is_empty() { "." } else { ":" }
    );
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      The report is only printed, so its fields are walked by reference.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    for crate_dir in &report.stale {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "  {crate_dir}");
    }
    if !report.added.is_empty() {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(
            sink,
            "{} versions no lockfile held before:",
            report.added.len()
        );
    }
    // Borrow `&report.added`: lends the value without copying or giving it away.
    for added in &report.added {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "  {} {}", added.name, added.version);
    }
    // Borrow `&report.warnings`: lends the value without copying or giving it away.
    for warning in &report.warnings {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Warning: {warning}");
    }
    if dry_run {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Dry run: every Cargo.lock was restored.");
    } else if report.written {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(
            sink,
            "Every Cargo.lock satisfies its manifests. Review the diff."
        );
    }
}

/// Prints an update report.
pub fn print_update_report(
    report: &UpdateReport,
    dry_run: bool,
    sink: &mut dyn Write,
) {
    // Macro `writeln!`: builds or prints text (or a list) from a template.
    // `let _ =`: discards the result on purpose.
    let _ = writeln!(
        sink,
        "{} lockfiles, {} newly locked versions:",
        report.crate_dirs.len(),
        report.added.len()
    );
    // Borrow `&report.added`: lends the value without copying or giving it away.
    for added in &report.added {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "  {} {}", added.name, added.version);
    }
    // Borrow `&report.fell_back`: lends the value without copying or giving it away.
    for crate_dir in &report.fell_back {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(
            sink,
            "Resolved {crate_dir} without the release-age cutoff (minimumReleaseAgeStrict = false, or crates exempt through minimumReleaseAgeExclude); crates that are not exempt are checked against it afterwards."
        );
    }
    // Borrow `&report.warnings`: lends the value without copying or giving it away.
    for warning in &report.warnings {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Warning: {warning}");
    }
    // Borrow `&report.pruned`: lends the value without copying or giving it away.
    for selector in &report.pruned {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Pruned {selector} from rust-catalog.toml.");
    }
    if dry_run {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Dry run: every Cargo.lock was restored.");
    } else if report.written {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(
            sink,
            "Kept the new lockfiles. Next: `mise run catalog:tighten`, then `mise run sync:files`, then `cargo-shadow install`."
        );
    }
}
