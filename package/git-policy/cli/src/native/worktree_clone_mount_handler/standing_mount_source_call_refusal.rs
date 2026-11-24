//! What:
//!  The seccomp filter a control installs so the mount-source call answers nothing,
//!  and the statement a run makes about the seccomp profile of its containers.
//! Why:
//!  The sweep reads a standing mount's source with `statmount(2)` and falls back to the live mount table where that call
//!  answers nothing.
//!  A kernel before 6.13 and a container whose seccomp profile refuses the call both take that fallback,
//!  and the gate's containers are the second kind unless the gate passes a profile of its own
//!  (`bin/native-gate-syscall-profile.ts`).
//!  A control cannot make the kernel lack the call,
//!  so the filter here makes the call answer `ENOSYS` for one thread,
//!  which proves the fallback wherever the suite runs,
//!  including a container whose own profile already refuses the call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (kernelsAnswer().kind === 'no-answer') controlDidNotRun(); else assertTheSweepsAnswer();
//! ```

/// The variable a run sets to state that its containers' seccomp profile allows the mount-source call.
///
/// Why:
///  A control that needs the call must fail where the run says its containers allow it,
///  because a profile the gate passes and podman does not apply would otherwise leave the call unread and the run green.
pub(crate) const ALLOWED_IN_CONTAINERS_VARIABLE: &str = "CLI_GIT_STATMOUNT_ALLOWED_IN_CONTAINERS";

/// What:
///  Whether this run states that its containers allow the call,
///  printing one line naming the control when it states no such thing.
/// Why:
///  A control that quietly returned where the call answered nothing would pass a run in which it never read the kernel,
///  and the run's verdict would then say nothing about the call;
///  the line it prints is what a person reads in the log of a run that did not prove the call,
///  as the btrfs and ZFS absence lines report theirs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theCallMustAnswer(name: string, why: string): boolean {
///   if (theRunStatesItsContainersAllowTheCall()) return true; // the caller fails
///   console.error(`STATMOUNT CONTROL NOT RUN: ${name} (${why})`);
///   return false; // the caller returns
/// }
/// ```
pub(crate) fn the_call_must_answer(name: &str, why: &str) -> bool {
    // What: the run's statement, read as present and non-empty; an absent or empty variable states nothing.
    let allowed: bool = std::env::var_os(ALLOWED_IN_CONTAINERS_VARIABLE).is_some_and(|value| { return !value.is_empty(); });
    if allowed {
        return true;
    }
    eprintln!("STATMOUNT CONTROL NOT RUN: {name} (this run does not state that its containers allow statmount(2), and the call answered nothing: {why})");
    return false;
}

/// The errno a filter answers with,
/// which is the one an absent system call's own answer carries,
/// so the sweep cannot tell the two apart and must not.
const NOT_IMPLEMENTED: u32 = libc::ENOSYS as u32;

/// What:
///  Install a seccomp filter on this thread that answers `ENOSYS` for each call number in `numbers` and allows every other
///  call.
/// Why:
///  This is how a control makes the mount-source call answer nothing on a kernel that has it,
///  which is what a container's seccomp profile does to the sweep,
///  and every other call of the run keeps working,
///  so a control can still mount,
///  read and assert around it.
/// Gotcha:
///  The filter belongs to the thread that installs it and to that thread's children.
///  A test body runs on a thread of its own,
///  so nothing else in the test binary is confined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseTheCalls(numbers: number[]): void; // throws when the filter is not installed
/// ```
pub(crate) fn refuse_the_calls(numbers: &[libc::c_long]) {
    // What: the filter's instructions, being one "load the call number" step, one "is it this number" pair per number, and
    // one "allow" step at the end.
    let mut program: Vec<libc::sock_filter> = Vec::with_capacity(numbers.len() * 2 + 1);
    // What: `BPF_LD | BPF_W | BPF_ABS` with `k` zero loads the 32-bit word at offset zero of the kernel's `seccomp_data`,
    // which is the call number.
    program.push(libc::sock_filter { code: (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16, jt: 0, jf: 0, k: 0 });
    for number in numbers {
        // What: `BPF_JMP | BPF_JEQ | BPF_K` jumps zero steps when the call number equals `k`, so the next instruction runs,
        // and one step, skipping it, when the number is another one.
        program.push(libc::sock_filter { code: (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16, jt: 0, jf: 1, k: *number as u32 });
        // What: `BPF_RET | BPF_K` ends the filter with the errno the kernel reports for this call.
        program.push(libc::sock_filter { code: (libc::BPF_RET | libc::BPF_K) as u16, jt: 0, jf: 0, k: libc::SECCOMP_RET_ERRNO | NOT_IMPLEMENTED });
    }
    // What: the same return, allowing every call this filter did not match.
    program.push(libc::sock_filter { code: (libc::BPF_RET | libc::BPF_K) as u16, jt: 0, jf: 0, k: libc::SECCOMP_RET_ALLOW });
    // What: the filter as the kernel takes it, being its length and a pointer to its instructions.
    let mut filter: libc::sock_fprog = libc::sock_fprog { len: program.len() as u16, filter: program.as_mut_ptr() };
    // SAFETY: `PR_SET_NO_NEW_PRIVS` takes four integers, and this call passes the one it reads and three zeroes; it needs no
    // pointer and changes nothing this control reads.
    let no_new_privileges: libc::c_int = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    assert_eq!(no_new_privileges, 0, "the thread may install a filter: {}", std::io::Error::last_os_error());
    // SAFETY: the kernel copies the instructions before this call returns, so `program` may end with this function; `filter`
    // is a live structure whose only pointer is into `program`, and the mode names an installed filter as the third
    // argument.
    let installed: libc::c_int = unsafe { libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &mut filter as *mut libc::sock_fprog) };
    assert_eq!(installed, 0, "the filter is installed: {}", std::io::Error::last_os_error());
}
