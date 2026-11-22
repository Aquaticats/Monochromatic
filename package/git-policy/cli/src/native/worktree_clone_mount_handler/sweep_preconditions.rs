//! What:
//!  What the sweep and the boot reconciliation check before any attach:
//!  this boot's identifier,
//!  the start probe's record for it,
//!  the pool the pinned parent lives in,
//!  and the checkout the walk starts from.
//! Why:
//!  A failed prerequisite authorizes no attachment
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit"),
//!  and every process that makes a raw request reads the start probe's record
//!  first,
//!  because an arm that could not run keeps the managed route blocked
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  Q4).
//!  The readiness record is a cache and not authority,
//!  so these are read from the live kernel and the live folders each run,
//!  and each answer is one of the record's own three states.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = bootFacts(configuration, reader, recordPath); // { boot, poolName, poolState, checkout, checkoutState }
//! ```

use super::sweep_checkout_mount::{CheckoutMount, CheckoutMounts};
use super::sweep_configuration::SweepConfiguration;
use super::sweep_dataset_reader::DatasetReader;
use crate::start_probe::probe_fingerprint::{FingerprintPaths, ProbeFingerprint};
use crate::start_probe::probe_record::{ABSENT, RECORD_KIND, RECORD_VERSION};
use crate::start_probe::{RECORD_DIRECTORY, RESULT_NAME};
use crate::worktree_clone_mount::clone_slot_walk_refusals::WalkRefusalReason;
use crate::worktree_clone_mount::hold_the_worktrees_parent;
use crate::worktree_clone_mount_protocol::{ReadyState, READINESS_DIRECTORY};
use std::path::{Path, PathBuf};

/// The kernel file that names this boot.
pub const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";

/// The start probe's record,
/// which every run that makes a raw request reads first.
pub fn probe_record_path() -> PathBuf {
    return Path::new(RECORD_DIRECTORY).join(RESULT_NAME);
}

/// What:
///  The record's verdict about the managed route,
///  as the record itself words it.
/// Why:
///   Only a passed probe enables a managed attach,
///  and a record that names another boot or another verdict is a reason to
///  attach nothing rather than a reason to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProbeStanding = { kind: 'passed' } | { kind: 'blocked'; verdict; step };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbeStanding {
    /// The probe passed for this boot.
    Passed,
    /// The record says the managed route is blocked,
    /// and names the verdict and the step it stopped at.
    Blocked {
        /// The verdict word the record carries.
        verdict: String,
        /// The step the record names,
        /// or `none`.
        step: String,
    },
}

/// What:
///  Why a run attaches nothing:
///  the record is unreadable,
///  names another boot,
///  or blocks the route.
/// Why:
///  A person has to tell "the probe never wrote a record here" from "the probe
///  ran and its `fsmount` arm failed",
///  because the first is a boot unit that did not run and the second is a kernel
///  the route cannot attach on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProbeRefusal = { kind: 'missing' | 'unreadable' | 'another-boot' | 'blocked' | 'boot-id'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbeRefusal {
    /// The boot identifier could not be read.
    BootId {
        /// What the read answered.
        reason: String,
    },
    /// The record is not there.
    Missing {
        /// The path that was read.
        path: PathBuf,
    },
    /// The record could not be read.
    Unreadable {
        /// The path that was read.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The record holds no line this route reads,
    /// or names another boot.
    AnotherBoot {
        /// The path that was read.
        path: PathBuf,
        /// The boot the record names,
        /// where it names one.
        named: Option<String>,
        /// This boot's identifier.
        boot: String,
    },
    /// This machine's own kernel release or boot identity could not be read,
    /// so no record can be judged against it.
    Running {
        /// What the read answered.
        reason: String,
    },
    /// The record's verdict does not enable a managed attach.
    Blocked {
        /// The path that was read.
        path: PathBuf,
        /// The verdict word.
        verdict: String,
        /// The step the verdict names.
        step: String,
    },
    /// The record is no `v1` `probe-result` of this build's shape.
    NotARecord {
        /// The path that was read.
        path: PathBuf,
        /// What about the record is not the expected shape.
        said: String,
    },
    /// One fingerprint line names another value than the running one,
    /// so the record is stale.
    Stale {
        /// The path that was read.
        path: PathBuf,
        /// The line that differs.
        line: &'static str,
        /// The value the record holds.
        named: String,
        /// The value the running kernel or module reports.
        running: String,
    },
    /// The pool is not ready.
    PoolNotReady {
        /// The pool's name.
        pool: String,
        /// The state the read answered.
        state: ReadyState,
    },
    /// The checkout is not ready.
    CheckoutNotReady {
        /// The checkout's path.
        checkout: PathBuf,
        /// The state the walk answered.
        state: ReadyState,
        /// What the walk answered,
        /// where it refused.
        said: String,
    },
}

impl ProbeRefusal {
    /// The refusal in words,
    /// naming the file,
    /// the verdict and every way forward.
    pub fn message(&self) -> String {
        let cause: String = match self {
            ProbeRefusal::BootId { reason } => format!("this boot's identifier could not be read from {BOOT_ID_PATH} ({reason})"),
            ProbeRefusal::Missing { path } => format!("the start probe's record {path:?} is not there, and a boot whose probe never wrote one attaches nothing"),
            ProbeRefusal::Unreadable { path, reason } => format!("the start probe's record {path:?} could not be read ({reason})"),
            ProbeRefusal::AnotherBoot { path, named, boot } => format!(
                "the start probe's record {path:?} names the boot {} and this boot is {boot}, so the record is another boot's",
                named.as_deref().unwrap_or("no boot at all")
            ),
            ProbeRefusal::Blocked { path, verdict, step } => {
                format!("the start probe's record {path:?} carries the verdict `{verdict}` and names the step `{step}`, so the managed route is blocked for this boot")
            }
            ProbeRefusal::Running { reason } => format!("this machine's own kernel release or boot identity could not be read ({reason}), so no probe record can be judged current"),
            ProbeRefusal::NotARecord { path, said } => format!("the start probe's record {path:?} is no record this route reads: {said}"),
            ProbeRefusal::Stale { path, line, named, running } => format!(
                "the start probe's record {path:?} names `{line} {named}` and this machine reports `{line} {running}`, so the record is stale: it judged another kernel, module or boot"
            ),
            ProbeRefusal::PoolNotReady { pool, state } => format!("the pool `{pool}` is `{}` and only a ready pool admits an attach", state.word()),
            ProbeRefusal::CheckoutNotReady { checkout, state, said } => format!("the checkout {} is `{}` and only a ready checkout admits an attach: {said}", checkout.display(), state.word()),
        };
        return format!(
            "{cause}. Nothing was enumerated, attached, unmounted or recorded, and every clone's mount intent stands untouched. Ways forward: run `mount-git-worktree-clone-datasets start-probe` and read the record it writes, load the pool's key and mount the checkout, or read {READINESS_DIRECTORY} and the handler's journal; the sweep runs again on a ZED event, the checkout provider's post-key-load hook, or a caller's sweep nudge."
        );
    }
}

/// What:
///  This boot's identifier,
///  as the kernel's own file spells it.
/// Why:
///  The readiness record and the probe's record are both compared with this
///  boot,
///  so a record another boot wrote is stale rather than ready.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const boot = readFileSync('/proc/sys/kernel/random/boot_id').trim();
/// ```
///
/// # Errors
/// The kernel file could not be read.
pub fn boot_identifier() -> Result<String, ProbeRefusal> {
    let text: String = std::fs::read_to_string(BOOT_ID_PATH).map_err(|error| return ProbeRefusal::BootId { reason: error.to_string() })?;
    return Ok(text.trim().to_string());
}

/// What:
///  One line of a record,
///  by its first word.
/// Why:
///  Both records this step reads are `key value` lines,
///  and reading one field rather than parsing the whole file keeps a record
///  another step owns from being decoded twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const field = lineOf(record, 'verdict'); // 'passed' | ...
/// ```
fn line_of(record: &str, key: &str) -> Option<String> {
    let prefix: String = format!("{key} ");
    return record.lines().find(|line| return line.starts_with(prefix.as_str())).map(|line| return String::from(line[prefix.len()..].trim()));
}

/// What:
///  Whether the start probe's record enables a managed attach for this boot.
/// Why:
///  The record is the only evidence that this boot's kernel passed the
///  probe's arms,
///  and its verdict is written by the mode itself rather than inferred.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = probeStanding(record, boot); // passed | blocked | another boot
/// ```
///
/// # Errors
/// The record names another boot,
/// holds no verdict,
/// or blocks the route.
pub fn probe_standing(record: &str, boot: &str, path: &Path) -> Result<ProbeStanding, ProbeRefusal> {
    let named: Option<String> = line_of(record, "boot");
    if named.as_deref() != Some(boot) {
        return Err(ProbeRefusal::AnotherBoot { path: path.to_path_buf(), named, boot: String::from(boot) });
    }
    let verdict: String = line_of(record, "verdict").ok_or_else(|| return ProbeRefusal::Unreadable { path: path.to_path_buf(), reason: String::from("the record holds no verdict line") })?;
    if verdict == "passed" {
        return Ok(ProbeStanding::Passed);
    }
    let step: String = line_of(record, "verdict-step").unwrap_or_else(|| return String::from("none"));
    return Err(ProbeRefusal::Blocked { path: path.to_path_buf(), verdict, step });
}

/// What:
///  Read the start probe's record and its standing for this boot.
/// Why:
///  Every run that makes a raw request reads it first,
///  so one function is the read and the decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = readProbeStanding(path, boot);
/// ```
///
/// # Errors
/// The record is absent or unreadable,
/// names another boot,
/// or blocks the route.
pub fn read_probe_standing(path: &Path) -> Result<ProbeStanding, ProbeRefusal> {
    let boot: String = boot_identifier()?;
    let record: String = std::fs::read_to_string(path).map_err(|error| {
        return match error.kind() {
            std::io::ErrorKind::NotFound => ProbeRefusal::Missing { path: path.to_path_buf() },
            _ => ProbeRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() },
        };
    })?;
    let running: ProbeFingerprint = ProbeFingerprint::read(&FingerprintPaths::live()).map_err(|refusal| return ProbeRefusal::Running { reason: refusal.message() })?;
    let standing: ProbeStanding = probe_standing(record.as_str(), boot.as_str(), path)?;
    probe_fingerprint_standing(record.as_str(), &running, path)?;
    return Ok(standing);
}

/// What:
///  Whether the record is this build's `v1` `probe-result` and pins the running
///  kernel,
///  module and boot.
/// Why:
///  The SPEC's own rule is that "a record whose fingerprint differs from the
///  running kernel,
///  module or boot is stale"
///  and that "`passed` enables managed attaches for this boot and module
///  identity"
///  (`package/git-policy/cli/SPEC.md`,
///  "The start probe").
///  Comparing the boot and the verdict alone would accept a record written
///  before a module reload within one boot,
///  so the four fingerprint lines are compared with the live values and the
///  file's version and kind are required as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// probeFingerprintStanding(record, runningFingerprint, path);
/// ```
///
/// # Errors
/// The file is no `v1` `probe-result`,
/// or one fingerprint line names another value than the running one.
pub fn probe_fingerprint_standing(record: &str, running: &ProbeFingerprint, path: &Path) -> Result<(), ProbeRefusal> {
    let first: Option<&str> = record.lines().next();
    if first != Some(RECORD_VERSION) {
        return Err(ProbeRefusal::NotARecord { path: path.to_path_buf(), said: format!("its first line is `{}` rather than `{RECORD_VERSION}`", first.unwrap_or("nothing")) });
    }
    let kind: Option<String> = line_of(record, "kind");
    if kind.as_deref() != Some(RECORD_KIND) {
        return Err(ProbeRefusal::NotARecord { path: path.to_path_buf(), said: format!("its kind is `{}` rather than `{RECORD_KIND}`", kind.as_deref().unwrap_or("nothing")) });
    }
    for (line, running_value) in [
        ("kernel", running.kernel_release.as_str()),
        ("module", running.module_version.as_deref().unwrap_or(ABSENT)),
        ("srcversion", running.module_srcversion.as_deref().unwrap_or(ABSENT)),
        ("build-id", running.module_build_id.as_deref().unwrap_or(ABSENT)),
    ] {
        let named: String = line_of(record, line).ok_or_else(|| return ProbeRefusal::NotARecord { path: path.to_path_buf(), said: format!("it holds no `{line}` line") })?;
        if named != running_value {
            return Err(ProbeRefusal::Stale { path: path.to_path_buf(), line, named, running: String::from(running_value) });
        }
    }
    return Ok(());
}

/// What:
///  The pool one pinned parent lives in,
///  which is the first `/`-separated component of its name.
/// Why:
///  The readiness record names the pool beside the checkout,
///  and the pool is the parent's own first component rather than a second
///  configuration key that could disagree with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = parent.split('/')[0];
/// ```
pub fn pool_name_of(parent: &str) -> &str {
    return parent.split('/').next().unwrap_or(parent);
}

/// What:
///  Whether the pool is imported and its copy of the parent readable.
/// Why:
///  One identity read of the pinned parent answers both:
///  a pool that is not imported cannot answer it,
///  and a parent the pool no longer holds is not ready either.
///  A read the route cannot interpret is `unknown`,
///  which admits nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = poolState(reader, parent); // ready | not-ready | unknown
/// ```
pub fn pool_state(reader: &dyn DatasetReader, parent: &str) -> ReadyState {
    return match reader.identity(parent) {
        Ok(_identity) => ReadyState::Ready,
        // The kernel's own `ENOENT` is the one answer that says the pool or the parent is gone rather than unreadable,
        // so the state is read from the error number and never from the words the binding happened to print.
        Err(failure) if failure.errno == Some(libc::ENOENT) => ReadyState::NotReady,
        Err(_failure) => ReadyState::Unknown,
    };
}

/// What:
///  Whether the checkout is there,
///  its `.claude/worktrees` folder walkable,
///  and its own mount not shared.
/// Why:
///  The walk the attach starts with is the only proof that the checkout is
///  mounted and its parent folder reachable,
///  and the design's measured move refused `EINVAL` while the checkout was
///  shared and passed once only it was private
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Mounts held by other namespaces"),
///  so a shared checkout is an unverified immediate parent and admits nothing.
///  The state is the walk's own answer and the mount's own tag rather than a
///  path check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = checkoutState('/home/user/Monochromatic', liveCheckoutMounts); // ready | not-ready | unknown
/// ```
pub fn checkout_state(checkout: &Path, mounts: &dyn CheckoutMounts) -> (ReadyState, String) {
    if let Err(refusal) = hold_the_worktrees_parent(checkout) {
        let said: String = refusal.message();
        let state: ReadyState = match refusal.reason {
            // A checkout that is not there, or that is reached through a link which is not its own canonical path, is
            // not mounted as the provider promises; every other refusal is a state this run cannot interpret.
            WalkRefusalReason::Missing | WalkRefusalReason::NotTheCanonicalPath | WalkRefusalReason::Checkout | WalkRefusalReason::NotADirectory => ReadyState::NotReady,
            WalkRefusalReason::LinkInTheWay
            | WalkRefusalReason::BoundaryCrossed
            | WalkRefusalReason::NotAPlainName
            | WalkRefusalReason::Unreadable
            | WalkRefusalReason::UnreadableFields => ReadyState::Unknown,
        };
        return (state, said);
    }
    let mount: CheckoutMount = match mounts.mount_holding(checkout) {
        Ok(mount) => mount,
        Err(refusal) => return (ReadyState::Unknown, refusal.message()),
    };
    if mount.is_shared() {
        return (
            ReadyState::NotReady,
            format!(
                "the checkout's own mount is `{}`, so it is shared and a mount made on it is delivered to another namespace; this route requires a not-shared immediate parent",
                mount.shared.as_deref().unwrap_or("shared")
            ),
        );
    }
    return (ReadyState::Ready, String::new());
}

/// What:
///  What one run knows about this boot before it attaches anything.
/// Why:
///  The readiness record's fields are these facts,
///  and the sweep's own admission is the same two states,
///  so one value carries them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BootFacts = { boot; poolName; poolState; checkout; checkoutState };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootFacts {
    /// This boot's identifier.
    pub boot: String,
    /// The pool's name.
    pub pool_name: String,
    /// Whether the pool is ready.
    pub pool_state: ReadyState,
    /// The checkout's path.
    pub checkout: PathBuf,
    /// Whether the checkout is ready.
    pub checkout_state: ReadyState,
    /// What the checkout's check answered,
    /// which the refusal quotes when the checkout is not ready.
    pub checkout_said: String,
}

impl BootFacts {
    /// What:
    ///  Refuse an attach unless both the pool and the checkout are ready.
    /// Why:
    ///  A failed prerequisite authorizes no attachment,
    ///  and `unknown` admits nothing.
    ///
    /// # Errors
    /// The first field that is not ready.
    pub fn admit(&self) -> Result<(), ProbeRefusal> {
        if self.pool_state != ReadyState::Ready {
            return Err(ProbeRefusal::PoolNotReady { pool: self.pool_name.clone(), state: self.pool_state });
        }
        if self.checkout_state != ReadyState::Ready {
            return Err(ProbeRefusal::CheckoutNotReady { checkout: self.checkout.clone(), state: self.checkout_state, said: self.checkout_said.clone() });
        }
        return Ok(());
    }
}

/// What:
///  What one run knows before it opens the device:
///  this boot,
///  the probe's answer for it,
///  and whether the checkout is walkable.
/// Why:
///  Every check here reads a file or walks a folder,
///  so a run that fails one of them makes no raw request at all,
///  and a boot with no probe record never opens the pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedBoot = { boot; checkout; checkoutState; standing };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedBoot {
    /// This boot's identifier.
    pub boot: String,
    /// The checkout's path.
    pub checkout: PathBuf,
    /// Whether the checkout is walkable.
    pub checkout_state: ReadyState,
    /// What the checkout's check answered.
    pub checkout_said: String,
    /// The probe's standing for this boot.
    pub standing: ProbeStanding,
}

/// What:
///  Read this boot's identifier,
///  the start probe's record for it,
///  and whether the checkout walks,
///  before anything is opened.
/// Why:
///  A boot with no passing probe record attaches nothing,
///  and that decision costs the pool nothing to make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const prepared = prepare(configuration, probeRecordPath);
/// ```
///
/// # Errors
/// The probe's record,
/// the boot identifier,
/// or nothing at all when every check passed.
pub fn prepare(configuration: &SweepConfiguration, probe_record: &Path, mounts: &dyn CheckoutMounts) -> Result<PreparedBoot, ProbeRefusal> {
    let standing: ProbeStanding = read_probe_standing(probe_record)?;
    let boot: String = boot_identifier()?;
    let (checkout_state, checkout_said): (ReadyState, String) = checkout_state(configuration.checkout.as_path(), mounts);
    return Ok(PreparedBoot { boot, checkout: configuration.checkout.clone(), checkout_state, checkout_said, standing });
}

/// What:
///  Fill in the pool's state and refuse the run when a prerequisite is not ready.
/// Why:
///  The pool is read through the device,
///  so this half is what needs the open handle,
///  and the checkout's own state joins it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = complete(prepared, reader, parent);
/// ```
///
/// # Errors
/// The pool or the checkout is not ready.
pub fn complete(prepared: PreparedBoot, reader: &dyn DatasetReader, parent: &str) -> Result<BootFacts, ProbeRefusal> {
    let facts: BootFacts = BootFacts {
        boot: prepared.boot,
        pool_name: String::from(pool_name_of(parent)),
        pool_state: pool_state(reader, parent),
        checkout: prepared.checkout,
        checkout_state: prepared.checkout_state,
        checkout_said: prepared.checkout_said,
    };
    facts.admit()?;
    return Ok(facts);
}



/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_preconditions_tests.rs"]
mod tests;
