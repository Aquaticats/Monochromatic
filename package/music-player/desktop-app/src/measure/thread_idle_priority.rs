//! How a sweep thread drops itself to the machine's idle scheduling tier,
//!  written once per platform.
//!
//! What:
//! `lower_current_thread_to_idle` has one definition per target,
//!  and each one asks the operating system,
//!  in that system's own terms,
//!  to run the CALLING thread only when nothing more important wants the CPU.
//! Why:
//! the sweep decodes whole files back-to-back,
//!  so its threads must never compete with the realtime audio thread or the window;
//! the four answers (POSIX scheduling classes,
//!  macOS quality of service,
//!  Windows thread priority,
//!  a no-op elsewhere) share one call site in `measure.rs`,
//!  and each is best-effort:
//! a refused request only means the sweep keeps running at normal priority.

/// What:
///      `#[cfg(target_os = "linux")] pub(super) fn lower_current_thread_to_idle()`.
///  Move the
///           CALLING thread into the Linux `SCHED_IDLE` scheduling class.
///  The `#[cfg(...)]`
///           attribute compiles this version ONLY on Linux.
/// Why:
///       The sweep decodes whole files back-to-back (CPU-bound).
///  `SCHED_IDLE` threads
///           run only when no normal-priority thread wants the CPU,
///  on ANY core,
///  so the
///           sweep never competes with the realtime audio thread,
///  the UI,
///  or other
///           applications.
///  It still finishes when the machine is otherwise idle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: JS runtimes expose no per-thread scheduling class
/// ```
#[cfg(target_os = "linux")]
pub(super) fn lower_current_thread_to_idle() {
    // What:     `let param: libc::sched_param = unsafe { std::mem::zeroed() };`. The scheduler
    //           parameter struct with every byte zero; `SCHED_IDLE` ignores the priority, so a
    //           priority of 0 is the only valid value. `libc::` names the raw C library bindings.
    // Why:      `sched_setscheduler` requires a `*const sched_param` argument. The struct literal
    //           `libc::sched_param { sched_priority: 0 }` does not compile on musl, where the
    //           struct also carries the sporadic-server fields (`sched_ss_*`); zeroing the whole
    //           struct is valid on every libc because all of its fields are plain integers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const param = { sched_priority: 0 };
    // ```
    let param: libc::sched_param = unsafe { std::mem::zeroed() };
    // What:     `let result = unsafe { libc::syscall(libc::SYS_sched_setscheduler, 0, libc::SCHED_IDLE, &param) };`.
    //           Call the kernel's `sched_setscheduler` system call directly. `unsafe { ... }` is
    //           required for ANY raw FFI call. The first arg `0` means "the calling thread";
    //           `libc::SCHED_IDLE` selects the idle class; `&param` lends the struct as a raw
    //           pointer. Returns `0` on success, `-1` on error.
    // Why:      musl's `sched_setscheduler` wrapper is a stub that always fails with ENOSYS, because
    //           musl cannot give it the per-thread meaning POSIX leaves open. The Linux system call
    //           changes exactly the calling thread, and glibc's wrapper is a thin shell around the same
    //           call, so one direct call behaves the same on glibc and on Alpine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result = schedSetscheduler(0, SCHED_IDLE, param);
    // ```
    let result =
        unsafe { libc::syscall(libc::SYS_sched_setscheduler, 0, libc::SCHED_IDLE, &param) };
    // What:     `if result != 0 { eprintln!(...); }`. On failure, log and carry on.
    // Why:      Best-effort: a failure just means the sweep runs at normal priority, which
    //           is still correct, only less polite. Never abort the sweep.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (result !== 0) console.error("music-player: could not lower sweep thread");
    // ```
    if result != 0 {
        let error = std::io::Error::last_os_error();
        tracing::warn!(%error, "could not lower sweep thread to SCHED_IDLE; running at normal priority");
    }
}

/// What:
///      `#[cfg(target_os = "macos")] pub(super) fn lower_current_thread_to_idle()`.
///  The macOS
///           version:
///  drop the CALLING thread into the BACKGROUND Quality of Service (QoS)
///           class.
///  `#[cfg(target_os = "macos")]` compiles it only on macOS.
/// Why:
///       macOS schedules threads by QoS class,
///  not the POSIX scheduling classes Linux
///           uses,
///  so the Linux SCHED_IDLE call does not exist here.
///  `QOS_CLASS_BACKGROUND`
///           is the lowest tier:
///  the sweep's CPU-bound decoding yields to the realtime audio
///           thread,
///  the UI,
///  and foreground apps,
///  the same intent as the Linux path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: JS runtimes expose no per-thread QoS class
/// ```
#[cfg(target_os = "macos")]
pub(super) fn lower_current_thread_to_idle() {
    // What:     `let result = unsafe { libc::pthread_set_qos_class_self_np(libc::qos_class_t::QOS_CLASS_BACKGROUND, 0) };`.
    //           Call the Apple-specific libc function that sets the CURRENT thread's QoS
    //           class. `unsafe { ... }` is required for ANY raw FFI call.
    //           `libc::qos_class_t::QOS_CLASS_BACKGROUND` is a VARIANT of the `qos_class_t`
    //           C enum and is the lowest QoS tier; the second argument is a relative-priority
    //           offset WITHIN that class (`0` = no offset). Returns `0` on success, a nonzero
    //           errno on failure.
    // Why:      Actually lower this thread's scheduling tier.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result = pthreadSetQosClassSelfNp(QOS_CLASS_BACKGROUND, 0);
    // ```
    let result =
        unsafe { libc::pthread_set_qos_class_self_np(libc::qos_class_t::QOS_CLASS_BACKGROUND, 0) };
    // What:     `if result != 0 { eprintln!(...); }`. On failure, log and carry on.
    // Why:      Best-effort, exactly like the Linux path: a failure just means the sweep
    //           runs at normal priority, which is still correct.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (result !== 0) console.error("music-player: could not lower sweep thread");
    // ```
    if result != 0 {
        tracing::warn!(
            "could not lower sweep thread to background QoS; running at normal priority"
        );
    }
}

/// What:
///      `#[cfg(windows)] pub(super) fn lower_current_thread_to_idle()`.
///  The Windows version:
///  set
///           the CALLING thread to the IDLE priority level.
///  `#[cfg(windows)]` compiles it
///           only on Windows.
/// Why:
///       Windows schedules by per-thread priority level,
///  not POSIX classes or QoS.
///           `THREAD_PRIORITY_IDLE` is the lowest level:
///  the sweep runs only when no
///           higher-priority thread wants the CPU,
///  the same intent as Linux SCHED_IDLE and
///           macOS background QoS.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: JS runtimes expose no per-thread priority level
/// ```
#[cfg(windows)]
pub(super) fn lower_current_thread_to_idle() {
    /// What:
    ///      `use windows::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_IDLE};`.
    ///           Import the Win32 thread-priority bindings.
    ///  A function-local `use` keeps these
    ///           Windows-only names out of the module's top scope.
    /// Why:
    ///       Name the three Win32 items the call below needs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// import { GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_IDLE } from "windows";
    /// ```
    use windows::Win32::System::Threading::{
        GetCurrentThread,
        SetThreadPriority,
        THREAD_PRIORITY_IDLE,
    };
    // What:     `let handle = unsafe { GetCurrentThread() };`. Get a PSEUDO-HANDLE to the
    //           current thread: a special constant that always means "this thread" and never
    //           needs closing. `unsafe` because it is a raw Win32 FFI call.
    // Why:      `SetThreadPriority` needs a thread handle to act on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const handle = getCurrentThread();
    // ```
    let handle = unsafe { GetCurrentThread() };
    // What:     `let result = unsafe { SetThreadPriority(handle, THREAD_PRIORITY_IDLE) };`.
    //           Set this thread's priority to the idle level. In the high-level `windows`
    //           crate this returns `windows::core::Result<()>`.
    // Why:      Actually lower this thread's priority.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result = setThreadPriority(handle, THREAD_PRIORITY_IDLE);
    // ```
    let result = unsafe { SetThreadPriority(handle, THREAD_PRIORITY_IDLE) };
    // What:     `if result.is_err() { eprintln!(...); }`. On failure, log and carry on.
    // Why:      Best-effort, exactly like the other platforms.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!ok) console.error("music-player: could not lower sweep thread");
    // ```
    if result.is_err() {
        tracing::warn!("could not lower sweep thread to idle priority; running at normal priority");
    }
}

/// What:
///      `#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))] pub(super) fn lower_current_thread_to_idle() {}`.
///           The no-op fallback compiled on every OTHER target (for example the BSDs) where
///           no scheduling call is wired up.
///  Empty body `{}`.
/// Why:
///       Keep the workers portable:
///  the call site stays the same and simply does nothing
///           where we have not implemented a scheduling tweak.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lowerCurrentThreadToIdle() {} // fallback no-op
/// ```
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub(super) fn lower_current_thread_to_idle() {}
