//! What:
//!  The JSON-lines printer and the conversion of clap's help,
//! version and parse errors into events.
//! Why:
//!  Every line standard output receives is one JSON object with a per-process sequence number,
//! including the lines
//!  clap would normally print as prose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class LinePrinter implements EventSink { sequence = 0; emit(event) { stdout.write(JSON.stringify(...) + '\n'); } }
//! ```

/// Imports `ErrorKind` from `clap::error`.
use clap::error::ErrorKind;
/// Imports `Event`,
/// `EventSink` from `monochromatic_file_enforcer::event`.
use monochromatic_file_enforcer::event::{Event, EventSink};
/// Imports `Write` from `std::io`.
use std::io::Write;

/// What:
///  Writes events to standard output as numbered JSON lines.
///  `u64` is an unsigned 64-bit integer (siblings:
/// `u32`,
/// `usize`).
/// Why:
///  One counter per process gives consumers a total order across help,
/// drift and result lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LinePrinter { sequence: number }
/// ```
#[derive(Debug, Default)]
pub struct LinePrinter {
    /// Number of the next line,
    /// counted per process from 0.
    sequence: u64,
}

/// What:
///   How the printer receives an event:
/// it prints one JSON line to standard output with the next sequence number.
/// Why:
///   Standard output is JSON lines only,
/// so every event goes through this one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// emit(event: Event): void { console.log(JSON.stringify(event.toJson(this.sequence++))); }
/// ```
impl EventSink for LinePrinter {
    /// What:
    ///  Number the event,
    /// render it,
    /// and write it to standard output;
    /// a closed pipe is ignored.
    /// Why:
    ///  A reader that stops early (`| head`) must not turn into a panic.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// emit(event: Event): void
    /// ```
    fn emit(&mut self, event: Event) {
        let line = event.to_line(self.sequence);
        self.sequence += 1;
        let mut stdout = std::io::stdout().lock();
        let _ = stdout.write_all(line.as_bytes());
        let _ = stdout.flush();
    }
}

/// What:
///  Convert a clap error into an event and the exit code it implies.
///  `(Event, i32)` is a pair (TS:
/// `[Event, number]`).
/// Why:
///  `--help` and `--version` are successes (exit 0,
/// event types `help` and `version`);
/// anything else is a usage
///  error (exit 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function eventForClapError(error: ClapError): [Event, number]
/// ```
pub fn event_for_clap_error(error: &clap::Error) -> (Event, i32) {
    let text = error.render().to_string();
    let trimmed = text.trim_end().to_string();
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            let code = if error.kind() == ErrorKind::DisplayHelp { 0 } else { 2 };
            return (Event::new("help", "help", &trimmed), code);
        }
        ErrorKind::DisplayVersion => return (Event::new("version", "version", &trimmed), 0),
        _ => {
            let mut event = Event::new("error", "usage-error", &trimmed);
            event.hint = Some("run `file-enforcer --help` for the commands and options".to_string());
            return (event, 2);
        }
    }
}
