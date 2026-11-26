//! What:
//!  Writing one entry of the service's log:
//!  the words,
//!  the priority,
//!  the fields a reader can filter the journal on,
//!  the sink the entry goes to,
//!  and the three macros every call site uses.
//! Why:
//!   The userinfo of every URL is masked before a line is cut or written,
//!   because a cut just before the `@` of `scheme://user:secret@host` would leave a password that no longer looks like
//!   userinfo.
//!  Nothing here waits:
//!  a service's standard error is the journal,
//!  a pipe or a FIFO,
//!  and a reader that stops reading holds `write_all` in the kernel for as long as it does not read
//!  (the audit of 0.6.1-alpha.14 held a landing 7.35 s that way).
//!  Whether this process is the resident service decides where an entry of the wrapper goes,
//!  and a control of the service's entries sees them on the thread that runs it.
//!  The text of a line is `service_timing_line_text.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! logEntry('warning', 'cannot listen', [['CLI_GIT_REQUEST_ID', id]])
//! ```
use crate::journald_native::{
    Delivery,
    MESSAGE_LIMIT,
    Priority,
    Sink,
    StreamWriter,
    cut,
    one_line,
};
use crate::lfs_output_mask::mask_url_secrets_text;
use crate::service_timing_line_text::{
    stamped_line,
    wall_millis,
};
use std::sync::OnceLock;

/// What:
///  Writes one line to the service's log.
/// Why:
///   Standard error is the service manager's journal;
/// one `write` carries the prefix,
/// the line,
/// its own time and the newline together,
/// and a
///  closed or broken stream is not the service's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.error(LOG_PREFIX + line + ` at_ms=${Date.now()}`)
/// ```
pub fn log(line: &str) {
    log_entry(Priority::Info, line, &[]);
}

/// What:
///  Whether this process is the resident service,
/// which names its release once at the start (`set_context`).
/// Why:
///   Code the service and a client run (the auto-push's fetch,
/// the shadow's `init`) records Git's words in the service's log in the
///  service and as a client entry in the wrapper;
/// a control that captures the service's entries counts as the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// contextWasSet
/// ```
pub fn in_service() -> bool {
    #[cfg(test)]
    if capture::active() {
        return true;
    }
    return CONTEXT.get().is_some();
}

/// What the service says of itself in every entry:
/// its release and,
/// when it serves one worktree,
/// its instance.
static CONTEXT: OnceLock<Vec<(&'static str, String)>> = OnceLock::new();

/// Where entries go,
/// decided at the first one.
static SINK: OnceLock<Sink> = OnceLock::new();

/// The sink of this process.
pub fn sink() -> &'static Sink {
    return SINK.get_or_init(Sink::from_environment);
}

/// Names the release and instance every later entry carries as `CLI_GIT_RELEASE` and `CLI_GIT_INSTANCE`;
/// the first call wins.
pub fn set_context(
    release: &str,
    instance: Option<&str>,
) {
    let mut fields: Vec<(&'static str, String)> = vec![("CLI_GIT_RELEASE", release.to_string())];
    if let Some(name) = instance {
        fields.push(("CLI_GIT_INSTANCE", name.to_string()));
    }
    let _first = CONTEXT.set(fields);
}

/// What:
///  Writes one entry of a priority,
/// with fields a reader can filter the journal on,
/// and gives where it went.
/// Why:
///   The userinfo of every URL is masked (a push or a review reports what Git and a server said),
/// the line on standard error
///  is one line,
///  then it states its own time (`stamped_line`);
/// a native entry also carries the time as `CLI_GIT_AT_MS`,
/// the priority
///  and the context.
/// Nothing here waits:
/// see `journald_native`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// logEntry('warning', 'cannot listen', [['CLI_GIT_REQUEST_ID', id]])
/// ```
pub fn log_entry(
    priority: Priority,
    line: &str,
    fields: &[(&str, String)],
) -> Delivery {
    let at_ms: u128 = wall_millis();
    // Masked before it is cut: a cut that falls just before the `@` of `scheme://user:secret@host` would leave a password
    //  that no longer looks like userinfo.
    let masked: String = mask_url_secrets_text(line);
    let mut all: Vec<(&str, String)> = Vec::new();
    if let Some(context) = CONTEXT.get() {
        for (name, value) in context.iter() {
            all.push((*name, value.clone()));
        }
    }
    all.push(("CLI_GIT_AT_MS", at_ms.to_string()));
    for (name, value) in fields {
        all.push((*name, mask_url_secrets_text(value.as_str())));
    }
    // A native entry carries the text as written (a newline in the length-prefixed form); the line on standard error is one line.
    let message: String = format!("{} at_ms={at_ms}", cut(masked.as_str(), MESSAGE_LIMIT));
    #[cfg(test)]
    if capture::record(priority, message.as_str(), all.as_slice()) {
        return Delivery::Stderr;
    }
    let text: String = stamped_line(one_line(masked.as_str(), MESSAGE_LIMIT).as_str(), at_ms);
    // On the journal's stream, and everywhere in the service, a line that finds no room is dropped, never waited for: a
    //  service's standard error is the journal, a pipe or a FIFO, and a reader that stops reading holds `write_all` in the
    //  kernel for as long as it does not read (the audit of 0.6.1-alpha.14 held a landing 7.35 s that way, and the journal's
    //  own disk was the busy one on 2026-11-12). The service's entries travel to the journal as datagrams as well, so what a
    //  FIFO drops is what the stream would have dropped anyway (`journald_native.rs`).
    if sink().on_stream() || in_service() {
        let mut never_waits: StreamWriter = StreamWriter::stderr();
        return sink().deliver(
            priority,
            message.as_str(),
            all.as_slice(),
            text.as_str(),
            &mut never_waits,
        );
    }
    // The wrapper's own standard error, where a person reads it, is written by a thread of its own, for the same reason: a
    //  command that waited for its own log's reader was held 7.35 seconds by a full FIFO on 2026-11-12
    //  (`service_log_lines_off_the_calling_thread.rs`). A line the queue cannot hold is written here instead, because the
    //  wrapper's own stream, unlike a landing's, is the one worth waiting for.
    return if crate::service_log_lines_off_the_calling_thread::hand(text.as_str()) {
        Delivery::Stderr
    } else {
        sink().deliver(
            priority,
            message.as_str(),
            all.as_slice(),
            text.as_str(),
            &mut std::io::stderr(),
        )
    };
}

/// What:
///  Writes one formatted line at the `warning` priority,
/// with its time.
/// Why:
///   Something did not go as intended and the service went on;
/// `journalctl --priority=warning` shows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceWarn(`the review of ${oid} was skipped: ${reason}`)
/// ```
#[macro_export]
macro_rules! service_warn {
    ($($argument:tt)*) => {
        {
            let _delivered = $crate::service_timing::log_entry($crate::journald_native::Priority::Warning, format!($($argument)*).as_str(), &[]);
        }
    };
}

/// What:
///  Writes one formatted line at the `err` priority,
/// with its time.
/// Why:
///   Something the service needed failed;
/// `journalctl --priority=err` shows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceError(`cannot listen: ${error}`)
/// ```
#[macro_export]
macro_rules! service_error {
    ($($argument:tt)*) => {
        {
            let _delivered = $crate::service_timing::log_entry($crate::journald_native::Priority::Error, format!($($argument)*).as_str(), &[]);
        }
    };
}

/// What:
///  Writes one formatted line to the service's log,
/// with its time.
/// Why:
///   Every line the service writes states its own time (`stamped_line`);
/// the macro is the one way to write a line with arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceLog(`cannot take the index lock ${path}: ${error}`)
/// ```
#[macro_export]
macro_rules! service_log {
    ($($argument:tt)*) => {
        $crate::service_timing::log(format!($($argument)*).as_str())
    };
}

/// What a control sees of the entries the code under test writes,
/// on the thread that runs it.
#[cfg(test)]
#[path = "service_timing_capture_test_support.rs"]
pub mod capture;
