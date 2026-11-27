//! What:
//!  Everything of a shadow that no commit decides:
//!  the skeleton,
//!  the config,
//!  the alternates and the links,
//!  made here so a commit's own creation and the pool's fill make the same entries.
//! Why:
//!   These are the stretches a pool member holds,
//!  and the two callers of them must make the same entries,
//!  so every step of them is named and failed in one place.
//!  A step is made of the facts the template holds
//!  (`shadow_create_template.rs`)
//!  and of the skeleton,
//! links and private storage it writes,
//!  which is why it sits after those files and not beside the facts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await createShadowTemplate({ context, template, shadowPath, clock });
//! ```

/// The build phase's stretches,
/// which the creation marks as it goes.
use crate::commit_build_part_durations::BuildPartClock;
/// The `git init` a reftable shadow is made by.
use crate::shadow_create_init::init_with_git;
/// The private mode of the shadow's own directory and files.
use crate::shadow_create_private_storage::{
    overwrite_private,
    set_private_directory_mode,
};
/// The facts the entries are made from.
use crate::shadow_create_template::ShadowTemplate;
/// The entry links.
use crate::shadow_links::link_shadow_entries;
/// The in-process entries of an empty bare repository.
use crate::shadow_skeleton::create_shadow_skeleton;
/// Process context and the failure of a step.
use crate::transaction_git::{
    GitContext,
    GitFailure,
};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Turn the result of a filesystem step on the shadow into a Git failure that names the verb and the shadow:
///  `<verb> <shadow> failed: <error>`.
///  `std::io::Result<T>` is "the value,
/// or an input/output error".
/// Why:
///   Several steps of the creation report their failure this way,
/// and a named function stands where a closure that built
///  that text stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stepFailure<T>(attempt: () => T, verb: string, shadow: string): T; // throws GitFailure(`${verb} ${shadow} failed: ${error}`)
/// ```
pub(crate) fn step_failure<T>(
    result: std::io::Result<T>,
    verb: &str,
    shadow: &Path,
) -> Result<T, GitFailure> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(GitFailure(format!(
            "{verb} {} failed: {error}",
            shadow.display()
        ))),
    };
}

/// The same for a step that is named by a phrase and not by a verb and the shadow:
/// `<phrase> failed: <error>`.
fn named_step_failure<T>(
    result: std::io::Result<T>,
    phrase: &str,
) -> Result<T, GitFailure> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(GitFailure(format!("{phrase} failed: {error}"))),
    };
}

/// What:
///  Creates everything of a shadow that does not depend on the commit or its branch:
/// the skeleton,
/// the config,
/// the alternates and the links.
/// Why:
///   These are the stretches a pool member holds,
/// and the two callers of them (a commit's own creation and the pool's fill) must make
///  the same entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowTemplate({ context, template, shadowPath, clock });
/// ```
pub fn create_shadow_template(
    context: &GitContext,
    template: &ShadowTemplate,
    shadow: &Path,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    if template.ref_format == crate::transaction_journal::RefFormat::Reftable {
        // The reftable backend's tables are Git's to create, so this one case still starts `git init`.
        init_with_git(context, template, shadow)?;
    } else {
        step_failure(create_shadow_skeleton(shadow), "creating", shadow)?;
    }
    step_failure(set_private_directory_mode(shadow), "protecting", shadow)?;
    clock.next("shadow config");
    named_step_failure(
        overwrite_private(
            shadow.join("config").as_path(),
            template.config_bytes().as_slice(),
        ),
        "writing the shadow config",
    )?;
    named_step_failure(
        overwrite_private(
            shadow
                .join("objects")
                .join("info")
                .join("alternates")
                .as_path(),
            template.alternates_bytes().as_slice(),
        ),
        "writing the shadow alternates",
    )?;
    clock.next("shadow links");
    named_step_failure(
        link_shadow_entries(
            shadow,
            template.common_dir.as_path(),
            template.git_dir.as_path(),
        ),
        "linking the shadow entries",
    )?;
    return Ok(());
}
