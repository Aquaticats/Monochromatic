//! What:
//!  Which packages the plans select,
//!  the severity each rule was selected with,
//!  and the allow-list file each of them names.
//! Why:
//!  A package is checked once however many of its files select the rules,
//!  and the first selecting file decides each rule's severity and what `allow` is relative to.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Selection = { folder?: Severity; folderAllow?: Path; order?: Severity; orderAllow?: Path };
//! function selectPackages(plans: FilePlan[], notes: string[]): Map<Path, Selection>;
//! ```

/// Import the severity a rule was selected with,
///  the run's plan and root rules,
///  the manifest lookup,
///  and the one spelling of a listed path.
use crate::{
    diagnostic::Severity,
    markdown_lfs_target::lexical_normal,
    run_paths::Language,
    run_plan::{
        FilePlan,
        RootRules,
    },
    rust_manifest::discover_manifest,
};
/// Import an ordered map and native paths.
use std::{
    collections::BTreeMap,
    path::PathBuf,
};

/// What:
///  The severities selected for one package,
///  and the allow-list file each selected rule names.
/// Why:
///  The first selecting file of the package decides each rule's severity,
///  and its configuration decides what `allow` is relative to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Selection = { folder?: Severity; folderAllow?: Path; order?: Severity; orderAllow?: Path };
/// ```
#[derive(Clone, Debug, Default)]
pub(crate) struct Selection {
    /// Severity of the folder rule.
    pub(crate) folder: Option<Severity>,
    /// The allow-list file the folder rule tolerates findings with.
    pub(crate) folder_allow: Option<PathBuf>,
    /// Severity of the file rule.
    pub(crate) order: Option<Severity>,
    /// The allow-list file the file rule tolerates findings with.
    pub(crate) order_allow: Option<PathBuf>,
}

/// What:
///  Group the selecting plans by owning package.
/// Why:
///  A package is checked once however many of its files select the rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selectPackages(plans: FilePlan[], notes: string[]): Map<Path, Selection>;
/// ```
pub(crate) fn select_packages(
    plans: &[FilePlan],
    notes: &mut Vec<String>,
) -> BTreeMap<PathBuf, Selection> {
    let mut chosen: BTreeMap<PathBuf, Selection> = BTreeMap::<PathBuf, Selection>::new();
    for plan in plans {
        if plan.language != Language::Rust {
            continue;
        }
        let RootRules::Rust {
            settings,
        } = &plan.root
        else {
            continue;
        };
        if settings.folder_uses_links.is_none() && settings.file_order_in_folder.is_none() {
            continue;
        }
        let manifest: PathBuf = match discover_manifest(&plan.absolute) {
            Ok(found) => found,
            Err(error) => {
                notes.push(format!(
                    "{}: not checked by the layout rules: {error}",
                    plan.display
                ));
                continue;
            }
        };
        let slot: &mut Selection = chosen.entry(manifest).or_default();
        if slot.folder.is_none()
            && let Some(rule) = &settings.folder_uses_links
        {
            slot.folder = Some(rule.severity);
            slot.folder_allow = rule.allow.as_ref().map(|path| {
                return lexical_normal(&plan.config.base.join(path));
            });
        }
        if slot.order.is_none()
            && let Some(rule) = &settings.file_order_in_folder
        {
            slot.order = Some(rule.severity);
            slot.order_allow = rule.allow.as_ref().map(|path| {
                return lexical_normal(&plan.config.base.join(path));
            });
        }
    }
    return chosen;
}
