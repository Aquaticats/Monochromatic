//! Observed-state output for the nested-Wayland boundary test.
//!
//! When `FM_STICKY_STATE_PATH` is set,
//!  the controller mirrors a shallow JSON snapshot into that
//! file after every model mutation and every scroll change,
//!  using the exact key set the Electron
//! prototype's main process writes (`activePath`,
//!  `columnCount`,
//!  `overlapCount`,
//!  `paneCount`,
//! `ready`,
//!  `rootPinned`,
//!  `scrolledDown`,
//!  `scrollTopPx`),
//!  so both apps' boundary tests share their
//! assertions.
//!  Writes are atomic (unique temp file plus rename) so a poller never reads a
//! truncated JSON document.

/// What:
///  imports filesystem write/rename helpers.
/// Why:
///  the state file is written to a unique temp path and renamed into place.
use std::fs;
/// What:
///  imports the process-id accessor.
/// Why:
///  the temp filename embeds the pid so concurrent apps never collide.
use std::process;
/// What:
///  imports the wall-clock type used in the temp filename.
/// Why:
///  a timestamp keeps successive temp names unique within one process.
use std::time::{SystemTime, UNIX_EPOCH};

/// What:
///  imports the JSON object and value types.
/// Why:
///  the snapshot is a shallow scalar object built explicitly (the `json!` macro expands to a
///      disallowed `unwrap`,
///  so the map is assembled by hand).
use serde_json::{Map, Value};

/// What:
///  imports the pure band math and its placement snapshot type.
/// Why:
///  pinning and overlap facts are recomputed from placements plus scroll,
///  never from widgets.
use crate::band::{self, Placement};
/// What:
///  imports the state-path env-var name.
/// Why:
///  output is enabled only when a boundary test provides a destination.
use crate::constants::STATE_PATH_ENV;

/// What:
///  everything the observed snapshot is computed from.
/// Why:
///  one plain value decouples state output from the controller's interior mutability.
pub(crate) struct ObservedInputs {
    /// Focused pane's location path,
    ///  or empty when none is focused.
    pub(crate) active_path: String,
    /// Number of columns spanned.
    pub(crate) column_count: usize,
    /// Live pane count.
    pub(crate) pane_count: usize,
    /// Latest placement snapshot.
    pub(crate) placements: Vec<Placement>,
    /// Whether the top-level window has mapped;
    ///  boundary tests must not send keys before this,
    /// because a keystroke delivered before the surface exists is silently dropped.
    pub(crate) ready: bool,
    /// Current app vertical scroll offset.
    pub(crate) scroll: f64,
}

/// What:
///  mirror `inputs` into the boundary-test state file when one is configured.
/// Why:
///  called after every reconcile and scroll change;
///  a missing env var makes this a no-op so
///      normal runs never touch the filesystem.
pub(crate) fn write_observed_state(inputs: &ObservedInputs) {
    let Some(path) = std::env::var_os(STATE_PATH_ENV) else {
        return;
    };
    let mut snapshot = Map::new();
    snapshot.insert("activePath".into(), Value::from(inputs.active_path.clone()));
    snapshot.insert("columnCount".into(), Value::from(inputs.column_count));
    snapshot.insert(
        "overlapCount".into(),
        Value::from(band::overlap_count(&inputs.placements, inputs.scroll)),
    );
    snapshot.insert("paneCount".into(), Value::from(inputs.pane_count));
    snapshot.insert("ready".into(), Value::from(inputs.ready));
    snapshot.insert(
        "rootPinned".into(),
        Value::from(band::root_pinned(&inputs.placements, inputs.scroll)),
    );
    snapshot.insert("scrolledDown".into(), Value::from(inputs.scroll > 0.0));
    snapshot.insert("scrollTopPx".into(), Value::from(inputs.scroll.round() as i64));
    let snapshot_value = Value::Object(snapshot);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| return elapsed.as_nanos())
        .unwrap_or(0);
    let temp = {
        let mut temp = path.clone();
        temp.push(format!(".{}.{}.tmp", process::id(), stamp));
        temp
    };
    let body = format!("{snapshot_value:#}\n");
    if let Err(error) = fs::write(&temp, body) {
        tracing::error!(%error, "failed to write observed-state temp file");
        return;
    }
    if let Err(error) = fs::rename(&temp, &path) {
        tracing::error!(%error, "failed to move observed-state file into place");
    }
}
