//! What:
//!  The fixture and the helpers the step's guest arm shares across its phases.
//! Why:
//!  Every phase makes a mount,
//! reads the table and says one line,
//!  so the pool,
//!  the checkout dataset and the table reading are stated once.
//!  A run on this machine prints one `SKIPPED` line and returns,
//!  so the recipe that pulls the log can tell an arm that ran from one that
//!  did not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = Fixture.fromEnvironment('the_propagation_guest_arm_tests');
//! ```

use std::path::PathBuf;
use std::process::Output;

/// The environment the arm reads,
/// so it never runs anywhere but the guest the driver made.
pub(crate) const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_PROPAGATION_GUEST";

/// The pool the arm's datasets live in.
pub(crate) const POOL_VARIABLE: &str = "CLI_GIT_ZFS_PROPAGATION_POOL";

/// The checkout path the arm mounts its dataset at.
pub(crate) const CHECKOUT_VARIABLE: &str = "CLI_GIT_ZFS_PROPAGATION_CHECKOUT";

/// The OpenZFS userspace the guest's commands need.
pub(crate) const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_PROPAGATION_PREFIX";

/// The pool a run uses when the driver names none,
/// which is the driver's own name.
const DEFAULT_POOL: &str = "propagation";

/// The checkout a run uses when the driver names none.
const DEFAULT_CHECKOUT: &str = "/root/propagation-checkout";

/// The OpenZFS prefix a run uses when the driver names none,
/// which is the prefix the guest's recipe installs.
const DEFAULT_PREFIX: &str = "/opt/openzfs-2.4.4";

/// What:
///  What one run of the arm reads and acts on:
///  the guest's name,
///  the pool,
///  the checkout path and the OpenZFS prefix.
/// Why:
///  Every phase needs the same four facts,
///  and the guest's name is what proves the arm runs nowhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fixture = { guest: string; pool: string; checkout: string; prefix: string };
/// ```
#[derive(Clone, Debug)]
pub(crate) struct Fixture {
    /// The guest's own `mvm` name.
    pub(crate) guest: String,
    /// The pool the arm's datasets live in.
    pub(crate) pool: String,
    /// The checkout path the arm mounts its dataset at.
    pub(crate) checkout: PathBuf,
    /// The OpenZFS prefix the guest's recipe installs.
    pub(crate) prefix: PathBuf,
}

impl Fixture {
    /// What:
    ///  The fixture this run's environment states,
    ///  or nothing on a machine that is not the guest.
    /// Why:
    ///  The arm makes a mount and reads a pool as root,
    ///  which no control on a person's machine may do,
    ///  so the arm returns with one `SKIPPED` line unless the guest's own
    ///  name is this machine's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fixture = Fixture.fromEnvironment('the_propagation_guest_arm_tests'); // undefined outside the guest
    /// ```
    pub(crate) fn from_environment(test: &str) -> Option<Self> {
        let named: std::ffi::OsString = match std::env::var_os(GUEST_VARIABLE) {
            Some(named) => named,
            None => {
                println!(
                    "SKIPPED {test}: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest"
                );
                return None;
            }
        };
        let guest: String = named.to_string_lossy().into_owned();
        let here: String = std::fs::read_to_string("/etc/hostname")
            .map(|text| return text.trim().to_string())
            .unwrap_or_default();
        if here != guest {
            println!(
                "SKIPPED {test}: {GUEST_VARIABLE} names the guest `{guest}` and this machine calls itself `{here}`"
            );
            return None;
        }
        return Some(Fixture {
            guest,
            pool: std::env::var(POOL_VARIABLE)
                .unwrap_or_else(|_error| return String::from(DEFAULT_POOL)),
            checkout: PathBuf::from(
                std::env::var(CHECKOUT_VARIABLE)
                    .unwrap_or_else(|_error| return String::from(DEFAULT_CHECKOUT)),
            ),
            prefix: PathBuf::from(
                std::env::var(PREFIX_VARIABLE)
                    .unwrap_or_else(|_error| return String::from(DEFAULT_PREFIX)),
            ),
        });
    }

    /// The `PATH` the guest's OpenZFS userspace needs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const searchPath = (): string => `${this.prefix}/sbin:${this.prefix}/bin:/sbin:/usr/sbin:/bin:/usr/bin`;
    /// ```
    pub(crate) fn search_path(&self) -> String {
        return format!(
            "{}:{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
            self.prefix.join("sbin").display(),
            self.prefix.join("bin").display()
        );
    }

    /// What:
    ///  One command,
    ///  its answer kept whichever way it went.
    /// Why:
    ///  A phase says what a command answered even when it refused,
    ///  because the arm's evidence is the lines it printed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const ask = (program: string, args: string[]): { ok: boolean; said: string } => ...;
    /// ```
    pub(crate) fn ask(
        &self,
        program: &str,
        arguments: &[&str],
    ) -> (bool, String) {
        let output: Result<Output, std::io::Error> = std::process::Command::new(program)
            .args(arguments)
            .env("PATH", self.search_path())
            .output();
        return match output {
            Ok(answer) => (
                answer.status.success(),
                format!(
                    "{}{}",
                    String::from_utf8_lossy(answer.stdout.as_slice()).trim(),
                    String::from_utf8_lossy(answer.stderr.as_slice()).trim()
                ),
            ),
            Err(error) => (false, format!("`{program}` could not start: {error}")),
        };
    }
}
