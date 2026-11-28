//! What:
//!  The handler's installed halves,
//!  its configuration,
//!  the two root-owned folders,
//!  and ZED's wiring,
//!  each read as one look at one path.
//! Why:
//!  The route's privileged half is one root-owned executable,
//!  a zedlet link to it,
//!  a root-owned configuration and two root-owned `/run` folders,
//!  and a ZFS event daemon somebody enabled,
//!  so a machine whose install is incomplete fails only when a clone is made
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  `doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
//!  choice 3).
//!  Presence,
//!  kind,
//!  owner and mode are four answers a person acts on differently,
//!  and a look this account was refused proves nothing,
//!  so one look answers one `PathState` and each check turns that into its own
//!  finding.
//!  Nothing here asks the service manager anything:
//!  `doctor` neither starts nor signals a unit,
//!  so whether ZED is active is not a fact this report can have.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const handler = theHandlerPathsRead(places); // { installation, configuration, folders, wiring }
//! ```

/// The look at one path,
/// which every handler check's owner and mode come through.
use super::one_look_at_a_path::the_path_facts;
/// The places every one of these paths is named by.
use super::the_route_places::TheRoutePlaces;
/// The unknown a read that failed answers.
use super::the_unknown_a_failed_read_answers::the_unknown_of;
/// One check's stable name,
/// and the report line a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// What one look at a path answered,
/// and the path a finding names beside it.
use crate::doctor_zfs_route::handler_checks::{
    PathFacts,
    PathState,
};
/// The handler's configuration file and whether its text parsed.
use crate::doctor_zfs_route::handler_checks_configuration::HandlerConfigurationFacts;
/// The readiness and results folders.
use crate::doctor_zfs_route::handler_checks_folder_modes::FolderFacts;
/// The handler's two installed files.
use crate::doctor_zfs_route::handler_checks_installation::HandlerInstallationFacts;
/// ZED's unit,
/// its enablement and the other zedlets an enablement runs.
use crate::doctor_zfs_route::handler_checks_zed_wiring::HandlerWiringFacts;
/// A borrowed path,
/// and the owned one a fact keeps.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The handler's facts,
///  each read from the place the build installs it at.
/// Why:
///  The installation,
///  the configuration,
///  the two folders and ZED's wiring are one group of path looks,
///  so one value carries them to the report and no check reads another
///  version of a place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheHandlerPaths = { installation; configuration; folders; wiring };
/// ```
pub(crate) struct TheHandlerPaths {
    /// The installed executable and zedlet.
    pub installation: HandlerInstallationFacts,
    /// The configuration file.
    pub configuration: HandlerConfigurationFacts,
    /// The readiness and results folders.
    pub folders: FolderFacts,
    /// ZED's unit and enablement beside the other zedlets.
    pub wiring: HandlerWiringFacts,
}

/// What:
///  The handler's installation,
///  configuration,
///  folders and wiring,
///  as the places they are installed at answer.
/// Why:
///  A report that reads one place twice could describe two machines,
///  so every path look of this group is made here once,
///  and the unknowns a failed look answers go to one sink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handler = the_handler_paths_read({ places, unread });
/// ```
pub(crate) fn the_handler_paths_read(
    places: &TheRoutePlaces,
    unread: &mut Vec<CheckReport>,
) -> TheHandlerPaths {
    return TheHandlerPaths {
        installation: HandlerInstallationFacts {
            executable: the_path_facts(places.executable.as_path()),
            zedlet: the_path_facts(places.zedlet.as_path()),
        },
        configuration: HandlerConfigurationFacts {
            file: the_path_facts(places.configuration.as_path()),
            // What:     The configuration's text is parsed by the one read that names the route's layout, and a
            //           text that did not parse stopped the gather before this file ran.
            // Why:      A second parse here would read one file twice and could disagree with the first.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // parseError: undefined,
            // ```
            parse_error: None,
        },
        folders: FolderFacts {
            readiness: the_path_facts(places.readiness.as_path()),
            results: the_path_facts(places.results.as_path()),
        },
        wiring: the_wiring_read(places, unread),
    };
}

/// What:
///  ZED's own unit,
///  whether a wants link enables it,
///  and every other zedlet the installed folder holds.
/// Why:
///  A unit installed and never enabled starts nothing,
///  and an enablement starts every other zedlet beside the route's own,
///  which is a person's decision before the link is made
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "ZFS reconciliation and mount safety").
///  A folder of zedlets this run could not list leaves the sentence that names
///  them unproved,
///  so the wiring answers unknown rather than "none besides the route's own".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wiring = theWiringRead(places, unread); // { zedUnit, zedEnabled, otherZedlets }
/// ```
fn the_wiring_read(
    places: &TheRoutePlaces,
    unread: &mut Vec<CheckReport>,
) -> HandlerWiringFacts {
    let unit: PathFacts = the_installed_unit(places);
    return match the_wiring_answers(places) {
        Ok((enabled, zedlets)) => HandlerWiringFacts {
            zed_unit: unit,
            zed_enabled: enabled,
            other_zedlets: zedlets,
        },
        Err(why) => {
            unread.push(the_unknown_of(Check::ZedWiring, why.as_str()));
            HandlerWiringFacts {
                zed_unit: unit,
                zed_enabled: false,
                other_zedlets: Vec::new(),
            }
        }
    };
}

/// What:
///  Whether a wants link enables ZED's unit,
///  and every other zedlet the installed folder holds,
///  or the reason one of the two looks could not answer.
/// Why:
///  The two answers are one fact of the report,
///  so a failed look of either makes the one check unknown rather than half a
///  sentence that reads as a whole one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = theWiringAnswers(places); // [true, ['zfs_scrub_finish']] | throws
/// ```
fn the_wiring_answers(places: &TheRoutePlaces) -> Result<(bool, Vec<String>), String> {
    // What:     `?` hands the enablement look's own refusal to the caller, so one failed look answers the check once.
    // Why:      A wiring sentence built on one of two looks would report a machine nobody read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const enabled = theEnablementRead(places);
    // ```
    let enabled: bool = the_enablement_read(places)?;
    // What:     `?` hands the folder listing's refusal on the same way, so a folder nobody listed is no list of
    //           "none besides the route's own".
    // Why:      The sentence a person reads names the other zedlets an enablement would start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const others = theOtherZedlets(places);
    // ```
    let others: Vec<String> = the_other_zedlets(places)?;
    return Ok((enabled, others));
}

/// What:
///  The first place ZED's own unit stands at,
///  or the first place when it stands at none.
/// Why:
///  A source build and a package put the same unit in different folders,
///  so the places are asked in the release's own order,
///  and a machine with no unit answers the first place as absent,
///  which is the file a person installs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unit = theInstalledUnit(places); // { path, state }
/// ```
fn the_installed_unit(places: &TheRoutePlaces) -> PathFacts {
    let mut first: Option<PathFacts> = None;
    for place in &places.zed_units {
        let facts: PathFacts = the_path_facts(place.as_path());
        if !matches!(facts.state, PathState::Absent) {
            return facts;
        }
        if first.is_none() {
            first = Some(facts);
        }
    }
    // What:     `unwrap_or_else` builds the empty answer for a places value that names no unit place at all, so the
    //           check names its finding rather than reading no path.
    // Why:      The release's own places always name three, and a control states its own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return first ?? { path: '', state: { kind: 'absent' } };
    // ```
    let fallback: PathFacts = PathFacts {
        path: PathBuf::new(),
        state: PathState::Absent,
    };
    return first.unwrap_or(fallback);
}

/// What:
///  Whether a link enabling ZED's unit stands in one of the places a manager
///  writes it.
/// Why:
///  Enabling a unit is one link,
///  and a machine whose link is there needs no root command,
///  so the answer is a look at each place rather than a question to the
///  manager.
///  A look that failed for any reason but absence proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const enabled = theEnablementRead(places); // true | false
/// ```
fn the_enablement_read(places: &TheRoutePlaces) -> Result<bool, String> {
    for place in &places.zed_enablements {
        match std::fs::symlink_metadata(place.as_path()) {
            Ok(_) => return Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "whether a link enables ZED's unit could not be read at {} ({error}), so the enablement is unproved",
                    place.display()
                ));
            }
        }
    }
    return Ok(false);
}

/// What:
///  Every other zedlet the installed folder holds,
///  by name,
///  in the order the folder gives them.
/// Why:
///  Enabling ZED starts every zedlet of the folder,
///  and a report that named only the route's own would hide the others from
///  the person deciding,
///  so each name is listed beside the route's own zedlet's when it is not that
///  file.
///  A folder that is not there holds no zedlet,
///  and one that could not be listed proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const zedlets = theOtherZedlets(places); // ['zfs_scrub_finish', …]
/// ```
fn the_other_zedlets(places: &TheRoutePlaces) -> Result<Vec<String>, String> {
    let folder: &Path = places.zedlets_folder.as_path();
    let installed: &Path = places.zedlet.as_path();
    let mut others: Vec<String> = Vec::new();
    let entries: std::fs::ReadDir = match std::fs::read_dir(folder) {
        Ok(listed) => listed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(others),
        Err(error) => {
            return Err(format!(
                "the zedlets folder in {} could not be listed ({error}), so the other zedlets an enablement would start are unproved",
                folder.display()
            ));
        }
    };
    for listed in entries {
        let one_entry: std::fs::DirEntry = listed.map_err(|error| {
            return format!(
                "the zedlets folder in {} could not be listed ({error}), so the other zedlets an enablement would start are unproved",
                folder.display()
            );
        })?;
        if one_entry.path() == installed {
            continue;
        }
        others.push(one_entry.file_name().to_string_lossy().into_owned());
    }
    return Ok(others);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_handler_paths_read_tests.rs"]
mod tests;
