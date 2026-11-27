//! What:
//!  The facts one coverage comparison reads,
//!  and the stable name the check carries in a report line.
//! Why:
//!  One value is the whole comparison's input,
//!  so a control builds the healthy set and changes one field to see exactly
//!  one finding,
//!  and the name lets a report and a control point at the same check.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CoverageFacts = { decision; tree; configPath; config; storePath };
//! ```

/// The coverage the project decided,
/// with the recursive ancestor,
/// the template and the exclusions.
use super::decided_coverage::CoverageDecision;
/// The dataset tree one machine measured.
use super::measured_dataset_tree::MeasuredTree;
/// A configuration text's reading,
/// and the reason a reading failed.
use super::sanoid_conf_reading::{
    ConfigRefusal,
    ReadConfig,
};
/// The check's stable name in a report line.
pub const COVERAGE_CHECK_NAME: &str = "sanoid-coverage";

/// What:
///  Every fact the coverage comparison reads,
///  gathered once.
/// Why:
///  One value is the whole comparison's input,
///  so a control builds the healthy set and changes one field to see exactly
///  one finding,
///  and a guest arm gathers the same facts from a real pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CoverageFacts = { decision; tree; configPath; config: Result<ReadConfig, ConfigRefusal>; storePath };
/// ```
pub struct CoverageFacts {
    /// The coverage the project decided.
    pub decision: CoverageDecision,
    /// The dataset tree the machine measured.
    pub tree: MeasuredTree,
    /// The configuration file's path,
    /// which every correction names.
    pub config_path: String,
    /// The configuration's reading,
    /// or the reason it could not be read.
    pub config: Result<ReadConfig, ConfigRefusal>,
    /// The store folder the design keeps in home and snapshotted.
    pub store_path: String,
}
