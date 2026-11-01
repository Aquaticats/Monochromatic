//! Scheduling priority for the decode thread.
//!  On Linux, the engine thread asks rtkit (the system daemon PipeWire already uses)
//!  to raise its nice level. On other platforms the function does nothing.
//! Why:
//!  Under CPU load a normal-priority decode thread can get too little time,
//!  and the ring buffer runs dry. A modest nice raise keeps decoding ahead of the load.
//!  Real-time scheduling is not requested: it would let a runaway decode starve the desktop.
//!
//! Mental model for a TypeScript reader:
//!  there is no TS equivalent. This is a request to the OS scheduler,
//!  made over D-Bus, that changes how often the kernel picks this thread to run.

/// What:
///      `pub const DECODE_NICE_LEVEL: i32 = -10;`.
///  The nice level requested for the decode thread.
/// Why:
///       rtkit allows down to -15 on this host (`MinNiceLevel`); -10 gives the decoder about
///           nine times the share of a normal thread without taking everything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DECODE_NICE_LEVEL = -10;
/// ```
pub const DECODE_NICE_LEVEL: i32 = -10;

/// What:
///      `pub fn nice_level_is_allowed(nice: i32) -> bool`.
///  Whether a nice level is inside the range rtkit accepts (-15 to 0 on Linux).
/// Why:
///       Keeps the requested constant honest; a request outside the range would only fail at runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function niceLevelIsAllowed(nice: number): boolean { return nice >= -15 && nice <= 0; }
/// ```
pub fn nice_level_is_allowed(nice: i32) -> bool {
    return (-15..=0).contains(&nice);
}

#[cfg(target_os = "linux")]
/// What:
///      `pub fn raise_current_thread_for_decode() -> bool`.
///  Asks rtkit to set the calling thread's nice level to `DECODE_NICE_LEVEL`.
///  Returns whether rtkit accepted the request.
/// Why:
///       rtkit is the only unprivileged way to lower a nice value, since this user has no RLIMIT_NICE.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// raiseCurrentThreadForDecode(): boolean { ... }
/// ```
pub fn raise_current_thread_for_decode() -> bool {
    // What:     `let connection = match Connection::system() { ... };`. Open the system bus,
    //           where rtkit lives. A failure is logged and playback continues at normal priority.
    // Why:      Priority is an optimisation; missing it must never stop playback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const connection = await systemBus(); // on failure: log, return false
    // ```
    let connection = match zbus::blocking::Connection::system() {
        Ok(connection) => connection,
        Err(error) => {
            tracing::warn!(error = %error, "system bus unavailable; decode thread keeps normal priority");
            return false;
        }
    };
    let process = u64::from(std::process::id());
    // SAFETY: `gettid` takes no arguments and cannot fail.
    let thread = u64::from(unsafe { libc::gettid() } as u32);
    // What:     `connection.call_method(...MakeThreadHighPriorityWithPID...)`. Ask rtkit to
    //           set this thread's nice level. The WithPID variant is the one rtkit accepts
    //           for threads named by their kernel thread id.
    // Why:      The request is the same one PipeWire makes for its own threads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await rtkit.MakeThreadHighPriorityWithPID(pid, tid, DECODE_NICE_LEVEL);
    // ```
    let result = connection.call_method(
        Some("org.freedesktop.RealtimeKit1"),
        "/org/freedesktop/RealtimeKit1",
        Some("org.freedesktop.RealtimeKit1"),
        "MakeThreadHighPriorityWithPID",
        &(process, thread, DECODE_NICE_LEVEL),
    );
    if let Err(error) = result {
        tracing::warn!(error = %error, "rtkit refused the decode thread priority; keeping normal priority");
        return false;
    }
    tracing::info!(nice = DECODE_NICE_LEVEL, "decode thread raised by rtkit");
    return true;
}

#[cfg(not(target_os = "linux"))]
/// What:
///      `pub fn raise_current_thread_for_decode() -> bool`.
///  No-op on this platform: always returns `false`.
/// Why:
///       Only Linux has rtkit; macOS and Windows keep their own scheduling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// raiseCurrentThreadForDecode(): boolean { return false; }
/// ```
pub fn raise_current_thread_for_decode() -> bool {
    return false;
}

#[cfg(test)]
#[path = "thread_priority_tests.rs"]
mod tests;
