//! Application logging:
//!  which records are written,
//!  how helix-lsp's records are labelled,
//!  and a
//! writer that never makes a logging thread wait for the output.
//!
//! By default the log holds warnings and errors.
//!  The standard `RUST_LOG` variable adds detail with
//! `tracing-subscriber`'s directive syntax,
//!  for example `RUST_LOG=ide_app=debug`.
//!  Whatever it says,
//! helix-lsp stays at warnings unless `RUST_LOG` names a `helix_lsp` target itself,
//!  because
//! helix-lsp writes every protocol message in full at INFO.

/// Queue and writer thread between the subscriber and the output.
pub mod background;

/// The `log` logger that re-labels helix-lsp's records for healthy servers.
pub mod relabel;

/// Each language server's last standard-error lines,
///  for the language worker's records of a
/// server that ended unexpectedly or failed to start;
///  never for a user-facing note.
pub mod stderr_tail;

/// What:
///  `EnvFilter` decides which records are written from directives such as `ide_app=debug`;
///       `MakeWriter` is how the subscriber obtains an output for each record.
/// Why:
///  The filter and the output are what callers choose;
///  everything else is fixed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { EnvFilter, MakeWriter } from 'tracing-subscriber';
/// ```
use tracing_subscriber::{EnvFilter, fmt::MakeWriter};

/// What:
///  `LevelFilter` is a level threshold:
///  OFF,
///  ERROR,
///  WARN,
///  INFO,
///  DEBUG,
///  TRACE,
///  ordered from
///       least to most detail.
/// Why:
///  A bare level in `RUST_LOG` is read to keep helix-lsp from showing more than it asks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LevelFilter = 'off' | 'error' | 'warn' | 'info' | 'debug' | 'trace';
/// ```
use tracing::level_filters::LevelFilter;

/// The process's base level:
///  warnings and errors.
pub const BASE_DIRECTIVE: &str = "warn";

/// What:
///  helix-lsp's directive for a given override:
///  warnings,
///  or the override's own bare level
///       when that shows less (`RUST_LOG=error` means errors only,
///  helix-lsp included).
///       `parse::<LevelFilter>()` reads a level name and fails for anything else,
///  such as a target.
/// Why:
///  helix-lsp's directive names a target,
///  so it would otherwise outrank a less detailed bare level.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function helixDirective(requested?: string): string {
///   const level = requested?.split(',').map(parseLevel).filter(Boolean).at(-1);
///   return level !== undefined && level < WARN ? `helix_lsp=${level}` : HELIX_LOG_DIRECTIVE;
/// }
/// ```
fn helix_directive(requested: Option<&str>) -> String {
    let mut global: Option<LevelFilter> = None;
    if let Some(text) = requested {
        for part in text.split(',') {
            // The last bare level wins, as it does in the filter itself.
            if let Ok(level) = part.trim().parse::<LevelFilter>() {
                global = Some(level);
            }
        }
    }
    return match global {
        Some(level) if level < LevelFilter::WARN => format!("helix_lsp={level}"),
        _ => crate::language::HELIX_LOG_DIRECTIVE.to_string(),
    };
}

/// What:
///  The directive text for a filter:
///  the base level,
///  then helix-lsp's directive,
///  then the
///       caller's `defaults`,
///  then the `requested` override.
///  `Option<&str>` is borrowed text or nothing.
/// Why:
///  When two directives name the same target,
///  `tracing-subscriber` keeps the later one,
///  so this
///      order lets `RUST_LOG` replace any earlier choice,
///  including helix-lsp's,
///  but only by naming it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directives(defaults: string, requested?: string): string {
///   return ['warn', helixDirective(requested), defaults, requested].filter(Boolean).join(',');
/// }
/// ```
pub fn directives(defaults: &str, requested: Option<&str>) -> String {
    // An override is kept only when it has text after `str::trim` removes surrounding spaces.
    let mut wanted: Option<&str> = None;
    if let Some(text) = requested.map(str::trim)
        && !text.is_empty()
    {
        wanted = Some(text);
    }
    // `Vec<String>` is a growable list of owned texts (sibling: `Vec<&str>`, which cannot hold the
    // computed helix-lsp directive).
    let mut parts: Vec<String> = vec![BASE_DIRECTIVE.to_string(), helix_directive(wanted)];
    if !defaults.trim().is_empty() {
        parts.push(defaults.trim().to_string());
    }
    // `if let Some(x)` runs the block only when an override is present.
    if let Some(text) = wanted {
        parts.push(text.to_string());
    }
    return parts.join(",");
}

/// What:
///  The filter for this process:
///  `defaults` plus the `RUST_LOG` override,
///  when it is set.
///       Directives `RUST_LOG` gets wrong are reported on standard error and skipped.
/// Why:
///  The application passes no defaults (warnings only);
///  test tools pass the detail they read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filter(defaults: string): EnvFilter { return EnvFilter.parseLossy(directives(defaults, process.env.RUST_LOG)); }
/// ```
pub fn filter(defaults: &str) -> EnvFilter {
    // What: `var` returns `Err` when the variable is unset or not valid text; `.ok()` turns that
    //       into "nothing".
    // Why: A missing or unreadable override leaves the defaults in force.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const requested = process.env.RUST_LOG;
    // ```
    let requested = std::env::var(EnvFilter::DEFAULT_ENV).ok();
    // `as_deref` lends the owned text inside the `Option` as `&str`.
    return EnvFilter::builder().parse_lossy(directives(defaults, requested.as_deref()));
}

/// What:
///  Install the process-wide subscriber with `filter` and `writer`,
///  then the helix-lsp bridge.
///       `ansi` chooses colored levels;
///  `None` keeps the library's choice,
///  which honors `NO_COLOR`.
///       `W: for<'writer> MakeWriter<'writer>` accepts any output the subscriber can write through.
/// Why:
///  The application,
///  the inspection tool,
///  and the tests that read the log install the same
///      pipeline,
///  so a test sees what a user would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function install(filter: EnvFilter, writer: MakeWriter, ansi?: boolean): void
/// ```
pub fn install<W>(filter: EnvFilter, writer: W, ansi: Option<bool>) -> anyhow::Result<()>
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer);
    // What: `finish` builds the subscriber; the two arms differ only in whether colors are set.
    // Why: Without an explicit choice the library decides from `NO_COLOR`, as before this module.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const subscriber = ansi === undefined ? builder.finish() : builder.withAnsi(ansi).finish();
    // ```
    let installed = match ansi {
        Some(colors) => tracing::subscriber::set_global_default(builder.with_ansi(colors).finish()),
        None => tracing::subscriber::set_global_default(builder.finish()),
    };
    // `map_err` replaces the library's error with one that names the operation.
    if let Err(error) = installed {
        return Err(anyhow::anyhow!(
            "Cannot install the application log: {error}"
        ));
    }
    relabel::install()?;
    return Ok(());
}

/// Directive order and the override rules,
///  without installing anything.
#[cfg(test)]
#[path = "logging_tests.rs"]
mod tests;
