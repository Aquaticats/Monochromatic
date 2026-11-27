//! What:
//!  The ZFS event daemon's unit file and its enablement,
//!  and the other zedlets an enablement would also run.
//! Why:
//!  The handler's zedlet runs only inside ZED,
//!  so a route that needs an event and a daemon nobody enabled fails only when
//!  a clone is made,
//!  and a person turning ZED on must first see the other zedlets it would
//!  start.
//!  Whether the unit is active is not asked of the service manager,
//!  because `doctor` neither starts nor signals anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkZedWiring(facts); // pass | finding('unit'|'enablement') | unknown
//! ```

/// One check's answer,
/// and the three constructors every check builds it from.
use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
/// What one look at a path answered.
use super::handler_checks_paths_and_modes::{
    PathFacts,
    PathState,
};
//region ZED and the service wiring

/// What:
///  The ZFS event daemon's unit file and enablement link,
///  and the other zedlets an enablement of ZED would also run.
/// Why:
///  The handler's zedlet runs only inside ZED,
///  so ZED's own unit and its enablement are the route's wiring,
///  and a person turning ZED on must first see the other zedlets it would
///  start,
///  including syslog,
///  notification,
///  enclosure and scrub effects
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "What doctor checks and prints").
///  Whether the unit is active is not asked of the service manager,
///  because `doctor` neither starts nor signals anything,
///  so that state is named as unread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HandlerWiringFacts = { zed_unit: PathFacts; zed_enabled: boolean; other_zedlets: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandlerWiringFacts {
    /// The ZFS event daemon's unit file.
    pub zed_unit: PathFacts,
    /// Whether a wants link enables it.
    pub zed_enabled: bool,
    /// Every other zedlet the installed directories hold.
    pub other_zedlets: Vec<String>,
}

/// What:
///  Whether ZED is installed and enabled,
///  and which other zedlets an enablement would start.
/// Why:
///  A route that needs an event and a daemon nobody enabled fails only when a
///  clone is made,
///  and the other zedlets are a person's decision before that enablement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_zed_wiring(facts); // pass | finding('unit'|'enablement') | unknown
/// ```
pub fn check_zed_wiring(facts: &HandlerWiringFacts) -> CheckOutcome {
    let path: String = facts.zed_unit.path.display().to_string();
    return match &facts.zed_unit.state {
        PathState::Unreadable {
            why,
        } => unknown(format!(
            "{path} could not be looked at ({why}), so ZED's installation is unproved"
        )),
        PathState::Absent => finding(
            "unit",
            format!("{path} is not installed, so no ZFS event reaches the handler's zedlet"),
        ),
        PathState::WrongKind {
            found,
        } => finding("unit", format!("{path} is not a unit file ({found})")),
        PathState::File {
            ..
        }
        | PathState::Directory {
            ..
        } => {
            if !facts.zed_enabled {
                return finding(
                    "enablement",
                    format!(
                        "{path} is installed and no wants link enables it, so the route's zedlet never runs; enabling it would also run the other zedlets this report lists: {}",
                        others(&facts.other_zedlets)
                    ),
                );
            }
            pass(format!(
                "{path} is installed and enabled; whether it is active is not asked of the service manager, and the other zedlets an enablement runs are: {}",
                others(&facts.other_zedlets)
            ))
        }
    };
}

/// The other zedlets in one sentence,
/// or the fact that there are none.
fn others(zedlets: &[String]) -> String {
    if zedlets.is_empty() {
        return String::from("none besides the route's own");
    }
    return zedlets.join(", ");
}

//endregion
