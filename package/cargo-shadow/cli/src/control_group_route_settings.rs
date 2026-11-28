//! What:
//! The settings one discovery of a route is made with:
//! the two programs it starts,
//! where the cgroup v2 hierarchy is mounted,
//! the slice the scopes are made in,
//! the variables the ask of the manager sees,
//! and the bound the ask is given.
//! Why:
//! Split out of `control_group_route_discovery.rs`,
//! which holds the discovery itself,
//! to keep that file under the code-line limit.
//! A test plants a host through these settings,
//! so the host under the fixture keeps its own manager.
//! The type is re-exported from `control_group_route_discovery`,
//! so its path there is unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control_group_route_settings.ts: interface RouteSettings, its defaults
//! ```

/// What:
/// `use std::path::PathBuf;` imports the owned path type,
/// whose sibling `&Path` is a borrowed view that owns nothing.
/// Why:
/// The mount point,
/// the file that names this process's own control group,
/// and the two program names are owned by the settings,
/// so they outlive the call that reads them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is a plain string in TS
/// ```
use std::path::PathBuf;
/// What:
/// `use std::time::Duration;` imports a span of time.
/// Its siblings are `Instant`,
/// a reading of the monotonic clock,
/// and `SystemTime`,
/// the wall clock,
/// which can jump.
/// Why:
/// The bound the ask of the manager is given is a span.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds
/// ```
use std::time::Duration;

/// What:
/// `use crate::control_group_route_discovery::{CGROUP_ROOT, DISCOVERY_BOUND, OWN_CGROUP_FILE};` imports the mount point of the
/// unified hierarchy,
/// the bound of one discovery,
/// and the file that names this process's own control group.
/// Why:
/// The ordinary settings name exactly those,
/// so the value a host is discovered with is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CGROUP_ROOT, DISCOVERY_BOUND, OWN_CGROUP_FILE } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::{
    CGROUP_ROOT,
    DISCOVERY_BOUND,
    OWN_CGROUP_FILE,
};

/// What the discovery of the route uses,
/// so a test can make the host look like one that lacks a capability.
#[derive(Clone, Debug)]
pub(crate) struct RouteSettings {
    /// The program that makes the scope.
    pub(crate) systemd_run: String,
    /// The program that asks the manager where the slice is.
    pub(crate) systemctl: String,
    /// Where the cgroup v2 hierarchy is mounted.
    pub(crate) cgroup_root: PathBuf,
    /// The slice the scopes are made in.
    pub(crate) slice: String,
    /// Variables set (`Some`) or removed (`None`) for the program that asks the manager.
    pub(crate) environment: Vec<(String, Option<String>)>,
    /// The arguments of the question to the manager when a test replaces the ordinary ones (`None` is the ordinary question).
    pub(crate) question: Option<Vec<String>>,
    /// How long the question may take.
    pub(crate) discovery_bound: Duration,
    /// The file that names this process's own control group.
    pub(crate) own_cgroup_file: PathBuf,
}

/// The settings of an ordinary host.
impl Default for RouteSettings {
    /// What:
    /// `fn default()` is the ordinary settings:
    /// `systemd-run`,
    /// `systemctl`,
    /// `/sys/fs/cgroup` and `app.slice`.
    /// Why:
    /// `app.slice` is where the user manager puts application scopes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const defaults: RouteSettings = { ... }
    /// ```
    fn default() -> RouteSettings {
        return RouteSettings {
            systemd_run: "systemd-run".to_string(),
            systemctl: "systemctl".to_string(),
            cgroup_root: PathBuf::from(CGROUP_ROOT),
            slice: "app.slice".to_string(),
            environment: Vec::new(),
            question: None,
            discovery_bound: DISCOVERY_BOUND,
            own_cgroup_file: PathBuf::from(OWN_CGROUP_FILE),
        };
    }
}
