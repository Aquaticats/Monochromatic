//! What:
//!  What the machine must answer before any dataset is judged:
//!  the device the route reads the pool through,
//!  the loaded module's layout against the request contract,
//!  and the pool the checkout dataset names.
//! Why:
//!  Every other pool read stands on the device,
//!  a layout that does not match disables every raw request,
//!  reads included,
//!  and a pool name in a configuration can outlive the pool,
//!  so each of the three is a check a person reads before anything below it,
//!  and the device path and the pool's name travel with the answer so a
//!  finding names what to check.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const device = checkDevice({ path: '/dev/zfs', state }); // pass | finding | unknown
//! const abi = checkAbi({ kind: 'refused', words }); // finding: module-layout
//! const pool = checkPool({ pool, reading }); // pass | finding('pool') | unknown
//! ```

/// One check's answer,
/// and the three constructors every check builds it from.
use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
/// What a read of the pool itself answered.
use super::zfs_checks_reading::Reading;
/// The device path one finding names.
use std::path::PathBuf;
//region `/dev/zfs`

/// What:
///  What the device path said when the caller looked at it.
/// Why:
///  A device that is not there,
///  a device this account may not open and a device whose state could not be
///  read send a person to three different places,
///  so the three states are kept apart before any judgement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeviceState = { kind: 'opens' } | { kind: 'denied'; why } | { kind: 'absent' } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeviceState {
    /// The caller opened the device.
    Opens,
    /// The path is there and the open refused.
    Denied {
        /// What the open answered.
        why: String,
    },
    /// The path is not there.
    Absent,
    /// The path's own state could not be read.
    Unreadable {
        /// What the look answered.
        why: String,
    },
}

/// What:
///  The device the route reads the pool through,
///  and what the caller's look at it answered.
/// Why:
///  One value carries the path so a finding names the device a person must
///  check,
///  rather than a word like "the device".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeviceFacts = { path: string; state: DeviceState };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceFacts {
    /// The device path,
    /// which is `/dev/zfs` for this route.
    pub path: PathBuf,
    /// What the look and the open answered.
    pub state: DeviceState,
}

/// What:
///  Whether the route's device is there and this account may open it.
/// Why:
///  Every other pool read stands on this one,
///  and the design's preflight asks for the account's own access rather than
///  root's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkDevice({ path: '/dev/zfs', state }); // pass | finding | unknown
/// ```
pub fn check_device(facts: &DeviceFacts) -> CheckOutcome {
    let path: String = facts.path.display().to_string();
    return match &facts.state {
        DeviceState::Opens => pass(format!("{path} is there and this account opened it")),
        DeviceState::Absent => finding(
            "device",
            format!("{path} is not there, so no pool read can run on this machine"),
        ),
        DeviceState::Denied {
            why,
        } => finding(
            "open",
            format!(
                "{path} is there and this account's open refused ({why}); the route needs the account to open it, not only root"
            ),
        ),
        DeviceState::Unreadable {
            why,
        } => unknown(format!(
            "{path} could not be looked at ({why}), so its presence and the account's open are unproved"
        )),
    };
}

//endregion

//region The ABI verdict

/// What:
///  What the start-time layout gate answered,
///  in the three states a report keeps apart.
/// Why:
///  The gate's own two outcomes are a compatible layout and a refusal,
///  and a refusal at the device or identity step proves nothing about the
///  layout,
///  so that case is unknown here rather than a second finding about a device
///  the device check already names
///  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AbiFacts = { kind: 'compatible'; module?: string; details: string[] } | { kind: 'refused'; words } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbiFacts {
    /// The loaded module's declaration matches the request contract.
    Compatible {
        /// The module's release text,
        /// which is reported and never compared with a rule.
        module: Option<String>,
        /// The files and sizes the gate compared.
        details: Vec<String>,
    },
    /// The layout comparison refused.
    Refused {
        /// The refusal in the words the gate prints.
        words: String,
    },
    /// The gate could not compare the layout.
    Unreadable {
        /// Why no comparison happened.
        why: String,
    },
}

/// What:
///  Whether the loaded module's layout matches the contract this build writes
///  requests from.
/// Why:
///  A mismatch disables every raw request,
///  reads included,
///  so a person must see it before anything else about the pool,
///  and the release text is reported beside the verdict because nothing
///  compares it with a rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkAbi({ kind: 'refused', words }); // finding: module-layout
/// ```
pub fn check_abi(facts: &AbiFacts) -> CheckOutcome {
    return match facts {
        AbiFacts::Compatible {
            module,
            details,
        } => {
            let release: String = module
                .as_ref()
                .map_or(String::from("no release text"), |text| {
                    return format!("release {text}");
                });
            pass(format!(
                "the loaded module's layout matches this build's request contract ({release}; {})",
                details.join(", ")
            ))
        }
        AbiFacts::Refused {
            words,
        } => finding(
            "module-layout",
            format!(
                "{words}; every raw request stays disabled, reads included, and no flag or environment turns this into a pass"
            ),
        ),
        AbiFacts::Unreadable {
            why,
        } => unknown(format!(
            "the layout gate ran no comparison ({why}), so the module's declaration is unproved"
        )),
    };
}

//endregion

//region The pool

/// What:
///  The pool the checkout dataset names,
///  and what a read of the pool itself answered.
/// Why:
///  A pool name in a configuration can outlive the pool,
///  and the design's preflight asks whether the pool is there before any
///  dataset below it is judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PoolFacts = { pool: string; reading: Reading<number> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolFacts {
    /// The pool's name.
    pub pool: String,
    /// The pool's own identity,
    /// or why it was not read.
    pub reading: Reading<u64>,
}

/// What:
///  Whether the pool named by the checkout dataset answers.
/// Why:
///  A missing pool and an unreadable one are different for a person:
///  the first says an import or a name is wrong,
///  the second that this run could not prove the pool either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkPool({ pool, reading }); // pass | finding('pool') | unknown
/// ```
pub fn check_pool(facts: &PoolFacts) -> CheckOutcome {
    let pool: &str = facts.pool.as_str();
    return match &facts.reading {
        Reading::Read(guid) => pass(format!("pool {pool} answers with GUID {guid}")),
        Reading::Missing {
            why,
        } => finding(
            "pool",
            format!(
                "pool {pool} is not there ({why}); import it or correct the checkout dataset's name"
            ),
        ),
        Reading::Unreadable {
            why,
        } => unknown(format!(
            "pool {pool} could not be read ({why}), so its presence is unproved"
        )),
    };
}

//endregion
