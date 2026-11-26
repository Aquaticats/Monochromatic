//! What:
//!  The measurement-only fences of a measurement build:
//! option C (refuse the rung whenever a conversion could apply,
//! with no Git run at all),
//!  option D (Git writes every tracked path,
//! no attribute read),
//! and the switch that picks one of them.
//! Why:
//!  A report of options has to time the options,
//! and neither needs a second behaviour in shipped code:
//! the feature is off in every
//!  shipped build,
//! so the release binary holds no switch at all,
//! and a route no shipped code takes can never be turned on by a
//!  setting.
//! The fence lives beside its measurement routes,
//! so `worktree_clone_conversion_fence.rs` stays under the line limit and no
//!  route of this file is ever compiled into a release.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fence = fenceOfTheMeasurementRoute(source); // C, D, or undefined
//! ```

use crate::worktree_clean::{
    StatEntry,
    stored_stats,
};
use crate::worktree_clone_conversion_fence::{
    Fence,
    refuse_untracked_attributes,
};
use crate::worktree_clone_conversion_rules::{
    ConversionRules,
    conversion_rules_from_config,
};
use crate::worktree_snapshot_source::SnapshotSource;

/// The configuration entries the fence reads its settings from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configOf = (source) => source.config.entries();
/// ```
fn config_of(source: &SnapshotSource) -> &[crate::config_snapshot::ConfigEntry] {
    return source.config.entries();
}

/// What:
///  Option C measured with the same binary:
/// the rung declines whenever any conversion source could apply,
/// and carries every path when none could,
///  so neither answer costs a Git run.
/// Why:
///   The whole point of the option is that it reads a few small files and nothing else.
/// It exists only in a measurement build (`measurement-routes`),
/// because a route no shipped code takes is a defect waiting for a setting to turn it on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function declineWhenConversionCouldApply(source, entries): Fence; // throws the reason
/// ```
#[cfg(feature = "measurement-routes")]
fn decline_when_conversion_could_apply(
    source: &SnapshotSource,
    entries: &[StatEntry],
) -> Result<Fence, String> {
    let rules: ConversionRules = conversion_rules_from_config(config_of(source));
    let filter_setting: bool = config_of(source)
        .iter()
        .any(|entry| return entry.key.starts_with("filter."));
    let shared_attributes: bool = source.common_dir.join("info/attributes").exists();
    let named_attributes: bool = config_of(source)
        .iter()
        .any(|entry| return entry.key == "core.attributesfile");
    if rules.auto_crlf != crate::worktree_clone_conversion_rules::AutoCrlf::False
        || filter_setting
        || shared_attributes
        || named_attributes
        || !source.clean.attribute_blobs.is_empty()
    {
        return Err(String::from(
            "the source has a conversion setting or an attributes file, and this build declines the rung rather than reading its paths (option C)",
        ));
    }
    return Ok(Fence {
        git_writes: Vec::new(),
        carried: entries.len(),
    });
}

/// What:
///  Option D measured with the same binary:
/// Git writes every tracked path,
/// and no attribute is read.
/// Why:
///   The option's shape is exactly this,
/// so its cost is the clone plus a checkout of the whole tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const allPaths = (entries) => ({ gitWrites: entries.map((entry) => entry.path), carried: 0 });
/// ```
#[cfg(feature = "measurement-routes")]
fn all_paths_through_git(entries: &[StatEntry]) -> Fence {
    let mut git_writes: Vec<Vec<u8>> = Vec::with_capacity(entries.len());
    for entry in entries {
        git_writes.push(entry.path.clone());
    }
    return Fence {
        git_writes,
        carried: 0,
    };
}

/// What:
///  The option C or D fence of a measurement build,
///  when the switch names one:
/// option C refuses the rung whenever anything could convert,
///  and option D hands Git every tracked path.
/// Why:
///   A report of options has to time the options,
/// and neither needs a second behaviour in shipped code:
/// the feature is
///  off in every shipped build,
/// so the release binary holds no switch at all,
/// and a route no shipped code takes can never be turned on by a
///  setting.
/// Both decide before any copy,
/// which is what their cost is:
/// no Git read of the source's paths beyond nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fenceOfTheMeasurementRoute(source): Fence | undefined;
/// ```
#[cfg(feature = "measurement-routes")]
pub fn fence_of_the_measurement_route(source: &SnapshotSource) -> Option<Result<Fence, String>> {
    let route: String = std::env::var("MONOCHROMATIC_CLONE_FENCE").unwrap_or_default();
    if route != "decline" && route != "all" {
        return None;
    }
    let refused: Result<(), String> = refuse_untracked_attributes(source);
    let Some(entries) = stored_stats(
        source.clean.index_bytes.as_slice(),
        source.clean.hash_length,
    ) else {
        return Some(Err(String::from("the source's index changed shape")));
    };
    return Some(match route.as_str() {
        "all" => refused.and(Ok(all_paths_through_git(entries.as_slice()))),
        _ => refused
            .and_then(|()| return decline_when_conversion_could_apply(source, entries.as_slice())),
    });
}
