//! What:
//!  The dataset whose mount point holds the store,
//!  and the finding when the recursive coverage does not reach it.
//! Why:
//!  The design keeps the pnpm store in home and snapshotted,
//!  so a store held by another dataset is a drift no INI line or property
//!  ends,
//!  and a store no measured mount point holds is a fact this build could not
//!  read rather than a pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! storeHolder(facts, findings); // Ok('tank/home') | Err('no measured dataset ...')
//! ```

/// The facts the comparison reads.
use super::coverage_facts::CoverageFacts;
/// One finding,
/// and the field its report line carries.
use super::coverage_finding::{
    CoverageFinding,
    FIELD_STORE_COVERAGE,
};
/// Whether one dataset lies under another.
use super::measured_dataset_tree::is_under;
/// What:
///  The dataset whose mount point holds the store,
///  and the finding when recursive coverage does not reach it.
/// Why:
///  The design keeps the pnpm store in home and snapshotted,
///  so a store held by another dataset is a drift no INI line or property
///  ends,
///  and a store no measured mount point holds is a fact this build could not
///  read rather than a pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// storeHolder(facts, config, findings); // Ok('tank/home') | Err('no measured dataset ...')
/// ```
pub(super) fn store_holder<'a>(
    facts: &'a CoverageFacts,
    findings: &mut Vec<CoverageFinding>,
) -> Result<&'a str, String> {
    let store: &str = facts.store_path.as_str();
    let Some(holder) = facts.tree.holding_dataset(store) else {
        return Err(format!(
            "no measured dataset's mount point covers the store at {store}, so this build cannot prove the store is snapshotted"
        ));
    };
    let root: &str = facts.decision.recursive_root.as_str();
    let covered: bool =
        (holder == root || is_under(holder, root)) && !facts.decision.excludes(holder);
    if !covered {
        let detail: String = format!(
            "the store at {store} is held by {holder}, which the recursive section of {root} does not reach, so the store is not snapshotted"
        );
        findings.push(CoverageFinding::new(FIELD_STORE_COVERAGE, detail, None));
    }
    return Ok(holder);
}
