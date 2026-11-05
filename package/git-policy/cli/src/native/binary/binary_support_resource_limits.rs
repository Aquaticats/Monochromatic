//! What:
//!  The limits a binary-level control puts on a fixture service process, and how it applies them.
//! Why:
//!  `binary_support.rs` held every helper of the binary-level controls in one file; the limits need nothing from the other helpers, so they live in a file of their own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const launch = { limit: "OpenFiles" };
//! ```

use std::ffi::OsString;

/// A resource limit a service starts with, set between `fork` and `exec` with no privilege.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitKind {
    /// Open descriptors (`RLIMIT_NOFILE`).
    OpenFiles,
    /// Processes of the real user (`RLIMIT_NPROC`).
    Processes,
    /// Address space (`RLIMIT_AS`).
    AddressSpace,
    /// Size of a core file (`RLIMIT_CORE`); zero writes none.
    CoreFile,
}

impl LimitKind {
    /// Sets this limit on the calling process; it is async-signal-safe, so a `pre_exec` closure may call it.
    pub fn apply(self, soft: u64, hard: u64) -> std::io::Result<()> {
        let cap: libc::rlimit = libc::rlimit { rlim_cur: soft, rlim_max: hard };
        let resource = match self {
            LimitKind::OpenFiles => libc::RLIMIT_NOFILE,
            LimitKind::Processes => libc::RLIMIT_NPROC,
            LimitKind::AddressSpace => libc::RLIMIT_AS,
            LimitKind::CoreFile => libc::RLIMIT_CORE,
        };
        // SAFETY: `setrlimit` reads the one `rlimit` this function owns.
        if unsafe { libc::setrlimit(resource, &raw const cap) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        return Ok(());
    }

    /// The hard limit of this process, which a child inherits and which an unprivileged process can lower but never raise.
    pub fn current_hard(self) -> u64 {
        let mut held: libc::rlimit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        let resource = match self {
            LimitKind::OpenFiles => libc::RLIMIT_NOFILE,
            LimitKind::Processes => libc::RLIMIT_NPROC,
            LimitKind::AddressSpace => libc::RLIMIT_AS,
            LimitKind::CoreFile => libc::RLIMIT_CORE,
        };
        // SAFETY: `getrlimit` writes the one `rlimit` this function owns.
        let read: libc::c_int = unsafe { libc::getrlimit(resource, &raw mut held) };
        assert_eq!(read, 0, "getrlimit failed: {}", std::io::Error::last_os_error());
        return held.rlim_max;
    }

    /// Changes the soft limit of the running process `pid` of the same user, as the end of the pressure that caused it
    ///  would: the hard limit stays, so raising the soft limit up to it needs no privilege.
    #[cfg(target_os = "linux")]
    pub fn lift_for(self, pid: u32, soft: u64, hard: u64) -> std::io::Result<()> {
        let cap: libc::rlimit = libc::rlimit { rlim_cur: soft, rlim_max: hard };
        let resource = match self {
            LimitKind::OpenFiles => libc::RLIMIT_NOFILE,
            LimitKind::Processes => libc::RLIMIT_NPROC,
            LimitKind::AddressSpace => libc::RLIMIT_AS,
            LimitKind::CoreFile => libc::RLIMIT_CORE,
        };
        // SAFETY: `prlimit` reads the one `rlimit` this function owns and is given no old-value pointer.
        if unsafe { libc::prlimit(i32::try_from(pid).expect("a process ID fits"), resource, &raw const cap, std::ptr::null_mut()) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        return Ok(());
    }
}

/// How a fixture starts its commit services: a program in front of the service (a tracer), and limits on the service and
///  everything it starts. The default is the plain service of production.
#[derive(Clone, Debug, Default)]
pub struct ServiceLaunch {
    /// The program and arguments placed in front of the service executable; empty starts the service itself.
    pub prefix: Vec<OsString>,
    /// Soft and hard limits applied to the service before it runs.
    pub limits: Vec<(LimitKind, u64, u64)>,
    /// Whether the service and everything it starts may run on one processor only.
    pub single_cpu: bool,
    /// Whether a service that ends or never listens is the control's subject, not a failure of the fixture.
    pub may_fail_to_start: bool,
    /// Whether the service is started with the command word an earlier release wrote in its unit files,
    ///  as a helper that an older program started is.
    pub earlier_command_word: bool,
}

/// Restricts the calling process, and so everything it starts, to the one processor it may use first.
///  It is async-signal-safe, so a `pre_exec` closure may call it.
#[cfg(target_os = "linux")]
pub fn restrict_to_one_cpu() -> std::io::Result<()> {
    // SAFETY: `cpu_set_t` is plain data, and a zeroed set is the empty one.
    let mut allowed: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    // SAFETY: `sched_getaffinity` writes the set it is given, whose size is passed.
    if unsafe { libc::sched_getaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &raw mut allowed) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let first: usize = (0..libc::CPU_SETSIZE as usize)
        .find(|cpu| { return unsafe { libc::CPU_ISSET(*cpu, &allowed) }; })
        .ok_or_else(|| { return std::io::Error::other("no allowed processor"); })?;
    // SAFETY: as above.
    let mut one: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    // SAFETY: `first` is below `CPU_SETSIZE`, so the set holds the bit.
    unsafe { libc::CPU_SET(first, &mut one) };
    // SAFETY: `sched_setaffinity` reads the set it is given, whose size is passed.
    if unsafe { libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &raw const one) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}
