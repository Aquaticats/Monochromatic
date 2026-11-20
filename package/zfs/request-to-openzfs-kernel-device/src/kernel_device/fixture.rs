//! What:
//!  The disposable guest a live test runs in:
//! the environment that names it,
//!  the
//! OpenZFS userspace it carries,
//! the account that holds the proposed grants,
//! and the
//!  encrypted file-backed pools the root-side tests build.
//! Why:
//!  A live request opens `/dev/zfs`,
//! so on this host it would reach the host's
//!  pool,
//! which no test may change.
//! The fixture therefore refuses to run anywhere
//!  but the guest the recipe names,
//! and it says by name why it skipped.
//! Every
//!  pool it builds is a 1 GiB image inside the guest,
//! destroyed when the test
//!  ends,
//! so the guest's own disks are the only thing a run touches.

// The codec list the creation-time properties travel in.
use crate::nvlist::NvList;
// The environment the fixture reads, and the process it starts.
use std::env;
// The one-argument type the environment lookup returns.
use std::ffi::OsString;
// The path types the fixture root and the program paths are.
use std::path::{Path, PathBuf};
// The process the fixture runs its userspace through.
use std::process::{Command, Output};

/// The variable naming the guest a live test may run in.
pub(crate) const GUEST_VARIABLE: &str = "MONOCHROMATIC_ZFS_FIXTURE_GUEST";
/// The variable naming the directory that holds the OpenZFS 2.4.4 userspace.
pub(crate) const SBIN_VARIABLE: &str = "MONOCHROMATIC_ZFS_FIXTURE_SBIN";
/// The variable naming the directory the pools and mountpoints live under.
pub(crate) const ROOT_VARIABLE: &str = "MONOCHROMATIC_ZFS_FIXTURE_ROOT";
/// The variable naming the account that holds the proposed grants.
pub(crate) const ACCOUNT_VARIABLE: &str = "MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT";
/// The variable naming the pool the delegated-account tests use.
pub(crate) const ACCOUNT_POOL_VARIABLE: &str = "MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT_POOL";
/// The variable naming the unpacked OpenZFS release tree,
/// whose headers carry
/// the request numbers.
pub(crate) const SOURCE_VARIABLE: &str = "MONOCHROMATIC_ZFS_FIXTURE_SOURCE";
/// The design's OpenZFS release,
/// which the guest's userspace and module must both report.
pub(crate) const OPENZFS_RELEASE: &str = "2.4.4-1";

/// The guest fixture a live test runs in.
pub(crate) struct Fixture {
    /// The directory holding `zfs` and `zpool` of OpenZFS 2.4.4.
    pub(crate) sbin: PathBuf,
    /// The directory the pools and mountpoints live under.
    pub(crate) root: PathBuf,
    /// The account the grants are on.
    pub(crate) account: String,
    /// The pool the delegated-account tests use.
    pub(crate) account_pool: String,
}

/// The methods of `Fixture`.
impl Fixture {
    /// The fixture the environment names,
    /// or `None` on a host run,
    /// where no live request may be made.
    ///
    /// # Panics
    ///
    /// When the environment names a guest that is not this machine,
    /// or names it
    /// without the rest of the fixture's variables:
    /// a run that was meant to be live
    /// must not look like a passing test that skipped its assertions.
    pub(crate) fn from_environment(test: &str) -> Option<Fixture> {
        return resolve(test, env::var_os(GUEST_VARIABLE), hostname(), |name| {
            return env::var_os(name);
        });
    }

    /// The unpacked OpenZFS release tree the request numbers come from.
    ///
    /// # Panics
    ///
    /// When the variable is unset inside the named guest,
    /// because the request-number
    /// assertions would then be silently absent.
    pub(crate) fn source_tree(&self, test: &str) -> PathBuf {
        return env::var_os(SOURCE_VARIABLE).map_or_else(
            || panic!("{test}: {SOURCE_VARIABLE} is unset, so the release headers are absent."),
            PathBuf::from,
        );
    }

    /// Run the fixture's `zfs` program and return its standard output.
    ///
    /// # Panics
    ///
    /// When the program fails,
    /// with its standard error in the message,
    /// because a
    /// fixture step that did not happen makes the assertions meaningless.
    pub(crate) fn zfs(&self, arguments: &[&str]) -> String {
        return self.program("zfs", arguments);
    }

    /// Run the fixture's `zpool` program and return its standard output.
    pub(crate) fn zpool(&self, arguments: &[&str]) -> String {
        return self.program("zpool", arguments);
    }

    /// Whether the fixture's `zfs` program accepts `arguments`,
    /// asked without
    /// panicking,
    /// which a test needs when the interesting answer is "no such dataset".
    pub(crate) fn zfs_succeeds(&self, arguments: &[&str]) -> bool {
        return Command::new(self.sbin.join("zfs"))
            .args(arguments)
            .output()
            .is_ok_and(|output| {
                return output.status.success();
            });
    }

    /// Run one of the fixture's OpenZFS programs.
    fn program(&self, program: &str, arguments: &[&str]) -> String {
        let path: PathBuf = self.sbin.join(program);
        let output: Output = Command::new(&path)
            .args(arguments)
            .output()
            .expect("the fixture's OpenZFS userspace runs");
        assert!(
            output.status.success(),
            "{} {} failed: {}",
            path.display(),
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        return String::from_utf8_lossy(&output.stdout).into_owned();
    }

    /// The value and source of each named property of `dataset`,
    /// as `zfs get`
    /// prints them.
    pub(crate) fn property_rows(
        &self,
        dataset: &str,
        properties: &str,
    ) -> Vec<(String, String, String)> {
        let printed: String = self.zfs(&[
            "get",
            "-H",
            "-p",
            "-o",
            "property,value,source",
            properties,
            dataset,
        ]);
        let mut rows: Vec<(String, String, String)> = Vec::new();
        for line in printed.lines() {
            let mut columns: std::str::Split<'_, char> = line.split('\t');
            let (Some(name), Some(value), Some(source)) =
                (columns.next(), columns.next(), columns.next())
            else {
                continue;
            };
            rows.push((name.to_string(), value.to_string(), source.to_string()));
        }
        return rows;
    }

    /// The GUID and creation transaction group of every object `selection`
    /// names,
    /// as `zfs list` prints them.
    pub(crate) fn identity_rows(&self, selection: &str, dataset: &str) -> Vec<(String, u64, u64)> {
        let printed: String = self.zfs(&[
            "list",
            "-H",
            "-p",
            "-o",
            "name,guid,createtxg",
            "-t",
            selection,
            "-r",
            dataset,
        ]);
        let mut rows: Vec<(String, u64, u64)> = Vec::new();
        for line in printed.lines() {
            let mut columns: std::str::Split<'_, char> = line.split('\t');
            let (Some(name), Some(guid), Some(txg)) =
                (columns.next(), columns.next(), columns.next())
            else {
                continue;
            };
            let (Ok(parsed_guid), Ok(parsed_txg)) = (guid.parse::<u64>(), txg.parse::<u64>())
            else {
                continue;
            };
            rows.push((name.to_string(), parsed_guid, parsed_txg));
        }
        return rows;
    }

    /// The uid and gid of `name`,
    /// read from the guest's own account database,
    /// so no
    /// test hardcodes one.
    pub(crate) fn account_ids(&self) -> (u32, u32) {
        let printed: String = std::fs::read_to_string("/etc/passwd").expect("/etc/passwd is read");
        for line in printed.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() > 3 && fields[0] == self.account {
                let uid: u32 = fields[2].parse().expect("a passwd entry has a numeric uid");
                let gid: u32 = fields[3].parse().expect("a passwd entry has a numeric gid");
                return (uid, gid);
            }
        }
        panic!(
            "the fixture account '{}' is not in /etc/passwd",
            self.account
        );
    }

    /// The current process's user id.
    pub(crate) fn current_uid() -> u32 {
        // SAFETY: `getuid` only reads the process's own credential, which the
        // kernel keeps for the life of the process; it takes no argument and
        // cannot fail.
        return unsafe { libc::getuid() };
    }
}

/// The kernel's own name for this machine,
/// which is the guest name mvm gives the VM.
///
/// The kernel keeps the authoritative copy in `/proc/sys/kernel/hostname`,
/// the same value
/// the `hostname` program prints and the recipe's guard compares;
/// `/etc/hostname` is only the
/// file a boot reads,
/// so a changed name would leave the two disagreeing.
pub(crate) fn hostname() -> String {
    let printed: String = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .expect("/proc/sys/kernel/hostname is read");
    return printed.trim().to_string();
}

/// The fixture `guest` names,
/// or `None` when no guest is named at all.
///
/// `this_machine` and `variable` are parameters rather than reads so the
/// three refusals can be
/// proven without a guest (see `fixture_tests.rs`).
pub(super) fn resolve(
    test: &str,
    named_guest: Option<OsString>,
    this_machine: String,
    variable: impl Fn(&str) -> Option<OsString>,
) -> Option<Fixture> {
    let Some(named) = named_guest else {
        println!(
            "SKIP {test}: {GUEST_VARIABLE} is unset, so this is not the mvm guest fixture; \
             live requests run only in the guest the package's e2e recipe creates."
        );
        return None;
    };
    let guest_name: String = named.to_string_lossy().into_owned();
    assert_eq!(
        this_machine, guest_name,
        "{test}: this machine is '{this_machine}' and {GUEST_VARIABLE} names '{guest_name}'. \
         A live request would change the pool of whichever machine runs it, so a run that names \
         a guest must be that guest; recreate it with the package's e2e recipe, or unset \
         {GUEST_VARIABLE} for a run that is not meant to be live."
    );
    let found_sbin: Option<OsString> = variable(SBIN_VARIABLE);
    let found_root: Option<OsString> = variable(ROOT_VARIABLE);
    let found_account: Option<OsString> = variable(ACCOUNT_VARIABLE);
    let found_account_pool: Option<OsString> = variable(ACCOUNT_POOL_VARIABLE);
    let missing: Vec<&str> = [
        (SBIN_VARIABLE, found_sbin.is_none()),
        (ROOT_VARIABLE, found_root.is_none()),
        (ACCOUNT_VARIABLE, found_account.is_none()),
        (ACCOUNT_POOL_VARIABLE, found_account_pool.is_none()),
    ]
    .into_iter()
    .filter_map(|(name, absent)| {
        return if absent { Some(name) } else { None };
    })
    .collect();
    assert!(
        missing.is_empty(),
        "{test}: this machine is the guest '{guest_name}' that {GUEST_VARIABLE} names, but {} is \
         unset, so the fixture would be incomplete and its assertions would not run. Set the \
         recipe's whole environment or unset {GUEST_VARIABLE}.",
        missing.join(", ")
    );
    let (Some(sbin_value), Some(root_value), Some(account_value), Some(account_pool_value)) =
        (found_sbin, found_root, found_account, found_account_pool)
    else {
        unreachable!("the unset variables were refused just above")
    };
    return Some(Fixture {
        sbin: PathBuf::from(sbin_value),
        root: PathBuf::from(root_value),
        account: account_value.to_string_lossy().into_owned(),
        account_pool: account_pool_value.to_string_lossy().into_owned(),
    });
}

/// Build a property list from pairs,
/// in the shape a creation-time `props` member takes.
pub(crate) fn properties(pairs: &[(&str, &str)]) -> NvList {
    let mut list: NvList = NvList::new();
    for (name, value) in pairs {
        list.add_str(*name, *value);
    }
    return list;
}

/// The directory a pool's image and key live in,
/// which is the pool's own folder under
/// the fixture root.
pub(crate) fn pool_folder(root: &Path, pool: &str) -> PathBuf {
    return root.join(pool);
}
