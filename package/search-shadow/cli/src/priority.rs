//! What:
//! The only place in the crate that talks to the operating system's disk-priority system call.
//! It puts the running process in the "best-effort" disk class at its lowest level (7 of 0 to 7).
//! Why:
//! The real tool inherits the priority through the process replacement,
//! so a broad search yields the disk to the desktop.
//! Keeping the single `unsafe` call in this one tiny function means nothing else in the crate
//! ever needs to think about it.
//! Gotcha:
//! This is FFI (calling a C function from Rust).
//! TypeScript has no analogue:
//! Node cannot set a process's disk priority at all,
//! so the pseudocode below is only a picture of the intent and not a translation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // priority.ts
//! export function lowerDiskPriority(): boolean { return nativeIoprioSet(WHO_PROCESS, 0, (BEST_EFFORT << 13) | 7) === 0; }
//! ```

/// What:
/// `const IOPRIO_WHO_PROCESS: libc::c_long = 1;` is a constant whose type `libc::c_long` is C's `long`
/// integer (64 bits on this machine;
/// the siblings `i32` / `i64` are what Rust picks for you elsewhere).
/// The value 1 is the Linux code meaning "the next number names a process".
/// Why:
/// The system call takes C `long` arguments,
/// so the constants carry that exact type and need no casts.
/// It is `c_long` and not `i64` because the C ABI says `long`,
/// whatever width the platform gives it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IOPRIO_WHO_PROCESS = 1;
/// ```
const IOPRIO_WHO_PROCESS: libc::c_long = 1;

/// The process id that means "the calling process" in the system call.
const THIS_PROCESS: libc::c_long = 0;

/// The number of bits the class is shifted left inside the combined priority value (a Linux ABI constant).
const IOPRIO_CLASS_SHIFT: libc::c_long = 13;

/// The code for the best-effort class (class 2;
/// 1 is real-time and 3 is idle).
const IOPRIO_CLASS_BEST_EFFORT: libc::c_long = 2;

/// The lowest level inside the best-effort class (level 0 is the highest and level 7 is the lowest).
const LOWEST_BEST_EFFORT_LEVEL: libc::c_long = 7;

/// What:
/// `pub fn lower_disk_priority() -> bool` declares a public function with no inputs that returns true when the
/// system accepted the new priority.
/// Why:
/// The shim calls it once just before replacing itself with the real tool.
/// A false answer (an old kernel or a sandbox that forbids the call) is not fatal:
/// the search simply runs at normal priority.
/// Gotcha:
/// The priority is honored only under the BFQ and mq-deadline disk schedulers.
/// Under kyber the call succeeds and has no effect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lowerDiskPriority(): boolean { ... }
/// ```
pub fn lower_disk_priority() -> bool {
    // What:     `(CLASS << SHIFT) | LEVEL` packs the class and the level into one number: `<<` shifts the class bits
    // left by 13 places and `|` merges in the level in the low bits (both are the same operators as in
    // TypeScript, here on a 64-bit integer).
    // Why:      This is the format the system call expects for "class 2, level 7".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const priorityValue = (IOPRIO_CLASS_BEST_EFFORT << IOPRIO_CLASS_SHIFT) | LOWEST_BEST_EFFORT_LEVEL;
    // ```
    let priority_value: libc::c_long = (IOPRIO_CLASS_BEST_EFFORT << IOPRIO_CLASS_SHIFT) | LOWEST_BEST_EFFORT_LEVEL;

    // What:     `unsafe { ... }` is a block inside which the compiler stops checking memory safety for the calls it
    // contains, because it cannot see inside C code. `libc::syscall(libc::SYS_ioprio_set, a, b, c)` asks the
    // Linux kernel to run its `ioprio_set` system call (`SYS_ioprio_set` is that call's number) with
    // arguments "the target is a process", "this process" and the packed priority. It returns 0 on success
    // and -1 on failure.
    // Why:      This is the one and only way to set a disk priority from a program without starting `ionice`.
    // Gotcha:   `unsafe` does not mean the call is dangerous here; it means the compiler cannot prove it safe. The
    // arguments are plain integers, nothing is read or written through a pointer, and a failed call only
    // returns -1. SAFETY: all three arguments are valid constants for this system call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result = nativeSyscall('ioprio_set', IOPRIO_WHO_PROCESS, THIS_PROCESS, priorityValue);
    // ```
    let result: libc::c_long = unsafe {
        libc::syscall(libc::SYS_ioprio_set, IOPRIO_WHO_PROCESS, THIS_PROCESS, priority_value)
    };

    // What:     `result == 0` is true when the kernel accepted the new priority.
    // Why:      Report success to the caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return result === 0;
    // ```
    return result == 0;
}
