//! What:
//!  The three settings the skip line prints,
//!  as this host answers them.
//! Why:
//!  A host cannot lose the check without a printed reason,
//!  and the values are what a person needs to see the cause:
//!  `user.max_user_namespaces` bounds how many namespaces a host gives,
//!  `kernel.unprivileged_userns_clone` is the distribution's own switch,
//!  and `kernel.apparmor_restrict_unprivileged_userns` is the one a
//!  distribution's default profile sets
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "Its skip").
//!  A setting the running kernel does not have prints `absent`,
//!  never nothing,
//! so a missing line cannot pass for a satisfied one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settings = { maxUserNamespaces: read('/proc/sys/user/max_user_namespaces') ?? 'absent', ... };
//! ```

/// The path that names how many user namespaces this host gives.
pub const MAX_USER_NAMESPACES: &str = "/proc/sys/user/max_user_namespaces";
/// The path a distribution patches in to disable them wholesale.
pub const UNPRIVILEGED_USERS_CLONE: &str = "/proc/sys/kernel/unprivileged_userns_clone";
/// The path a distribution's AppArmor profile sets.
pub const APPARMOR_RESTRICTION: &str = "/proc/sys/kernel/apparmor_restrict_unprivileged_userns";
/// The entry whose absence means the kernel was built without user namespaces.
pub const USER_NAMESPACE_ENTRY: &str = "/proc/self/ns/user";
/// The word a setting the running kernel does not have prints.
pub const ABSENT: &str = "absent";

/// What:
///  The three values the skip line prints,
///  each read once.
/// Why:
///  The line names the settings,
///  and `doctor` prints the same ones from a fresh run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Settings = { maxUserNamespaces: string; unprivilegedUsernsClone: string; apparmorRestriction: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Settings {
    /// How many user namespaces this host gives.
    pub max_user_namespaces: String,
    /// The distribution's own switch,
    /// where it has one.
    pub unprivileged_userns_clone: String,
    /// The AppArmor restriction,
    /// where the module is.
    pub apparmor_restriction: String,
}

/// One setting's value,
/// trimmed,
/// or the word for a setting the kernel does not have.
fn value_of(path: &str) -> String {
    return match std::fs::read_to_string(path) {
        Ok(text) => String::from(text.trim()),
        Err(_missing) => String::from(ABSENT),
    };
}

/// What:
///  The three values as this host answers them.
pub fn read() -> Settings {
    return Settings {
        max_user_namespaces: value_of(MAX_USER_NAMESPACES),
        unprivileged_userns_clone: value_of(UNPRIVILEGED_USERS_CLONE),
        apparmor_restriction: value_of(APPARMOR_RESTRICTION),
    };
}

/// What:
///  Whether this kernel has the user-namespace entry at all.
/// Why:
///  The design reads it before the setup,
///  so `EINVAL` from `unshare(CLONE_NEWUSER)` is a host without the option
///  rather than the companion's own malformed request.
pub fn user_namespace_entry_exists() -> bool {
    return std::fs::symlink_metadata(USER_NAMESPACE_ENTRY).is_ok();
}

impl Settings {
    /// Every setting as one printed line,
    /// in the order the design names them.
    pub fn line(&self) -> String {
        return format!(
            "user.max_user_namespaces={}, kernel.unprivileged_userns_clone={}, kernel.apparmor_restrict_unprivileged_userns={}",
            self.max_user_namespaces, self.unprivileged_userns_clone, self.apparmor_restriction
        );
    }
}
