//! What:
//!  What a service preparation reads once,
//!  before it stages anything:
//!  the layout of the command's invocation,
//!  the unmerged paths the live index held,
//!  the selection the command line makes,
//!  the branch it commits on,
//!  the capture of the repository,
//!  the command's pathspecs,
//!  the scratch directory and the two index files,
//!  and the pathspec file the private index reads.
//! Why:
//!   Every later step of the preparation reads these and none of them changes after the
//!  capture,
//!  so one function that reads them,
//!  and one record that carries them,
//!  keep `service_prepare.rs` to the steps that do the work;
//!  the warnings of a first preparation are made here because the layout is what they read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const setup = readTheSetup(invocation, inputs); // the layout, the selection, the capture, the scratch, the two indexes
//! ```

use crate::capture_invocation::{
    InvocationCapture,
    base_revision,
    capture_invocation_base,
    capture_invocation_layout,
};
use crate::commit_index::list_unmerged_index_paths;
use crate::commit_prepare_part_durations::PreparePartClock;
use crate::commit_transaction::{
    CommitInvocation,
    Selection,
    TransactionStop,
    decide_selection,
    failed,
};
use crate::external_hooks::warn_external_hooks;
use crate::private_index_request::CaptureIndexes;
use crate::ref_name::RefName;
use crate::service_prepare_changes_config_and_pathspec_steps::{
    config_needs_native,
    the_pathspec_file,
    zero_object,
};
use crate::service_prepare_ready_shapes::Scratch;
use crate::standard_error_descriptor::StandardErrorDescriptor;
use crate::traditional_hook_warning::warn_traditional_hooks;
use crate::transaction_git::GitContext;
use crate::transaction_journal::{
    Base,
    Conclusion,
    SymbolicHead,
    TransactionMode,
};
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The values one preparation reads before it stages anything,
/// and from which every later step of the preparation works.
/// Why:
///   The selection,
/// the capture and the scratch directory are read once,
///  so a step that came later cannot read a different answer than the one the person's
///  command line gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Preparation = { capture, unmergedBefore, selection, reference, detached, named, unborn, configNative, base, announcedBase, pathspecs, amends, scratch, indexes, pathspecFile };
/// ```
pub(crate) struct Preparation {
    /// The layout the invocation was captured with,
    /// and the base the branch names,
    /// with the index the commit copies from.
    pub(crate) capture: InvocationCapture,
    /// The paths the live index held unmerged,
    /// when the commit may take them,
    /// so the warning can name the ones it takes.
    pub(crate) unmerged_before: Vec<Vec<u8>>,
    /// The selection the command line makes.
    pub(crate) selection: Selection,
    /// The branch reference the commit is made on,
    /// or `HEAD` for a detached head.
    pub(crate) reference: RefName,
    /// Whether the head is detached.
    pub(crate) detached: bool,
    /// Whether the command names paths rather than committing the index.
    pub(crate) named: bool,
    /// Whether the branch has no commit yet.
    pub(crate) unborn: bool,
    /// Whether the configuration asks for native Git,
    /// read from the snapshot the layout already holds.
    pub(crate) config_native: bool,
    /// The commit the branch names,
    /// or the empty tree when it has none.
    pub(crate) base: String,
    /// The base as the person is told it.
    pub(crate) announced_base: String,
    /// The pathspecs the command line names,
    /// as bytes,
    /// in the order the command wrote them.
    pub(crate) pathspecs: Vec<OsString>,
    /// Whether the commit amends the head.
    pub(crate) amends: bool,
    /// The commit's own scratch directory,
    /// which the sweep of the repository scratch directory removes after a kill.
    pub(crate) scratch: Scratch,
    /// The two index files the commit is prepared through,
    /// and the object store they read.
    pub(crate) indexes: CaptureIndexes,
    /// The pathspec file the selection reads,
    /// when the command gave one.
    pub(crate) pathspec_file: Option<PathBuf>,
}

/// What:
///  Reads everything one preparation reads before it stages anything.
/// Why:
///   The steps that follow work from the record this hands back,
/// and the seams of the preparation's own timing sit where the reads end,
///  so each stretch of the preparation is measured as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTheSetup(invocation: CommitInvocation, first: boolean, scratchName: string, standardInput?: Uint8Array): Preparation
/// ```
pub(crate) fn read_the_setup(
    invocation: &CommitInvocation<'_>,
    first: bool,
    scratch_name: &str,
    standard_input: Option<&[u8]>,
    part: &mut PreparePartClock,
) -> Result<Preparation, TransactionStop> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let region = invocation.region;
    let layout = capture_invocation_layout(context, cwd, region.amend)?;
    if first {
        // The descriptor itself, not the standard library's shared handle: a worker left at the ceiling holds no lock this
        //  command's own writes would wait for (`foundation/standard_error_descriptor.rs`).
        warn_traditional_hooks(layout.config_snapshot.as_ref(), StandardErrorDescriptor);
        warn_external_hooks(layout.config_snapshot.as_ref(), StandardErrorDescriptor);
    }
    part.next("unmergedBefore");
    // Named paths beside another session's unfinished merge or stopped cherry-pick land as an ordinary commit, one in conflict
    //  included: the owner's rule is "Committing a conflicted path is fine. Warn and commit anyway, even if conflict markers will
    //  be committed." The paths in conflict are listed here so the warning can name those the commit takes. `-a` stages the working
    //  copy of every tracked file, an unmerged one included, so it commits their markers too and is listed the same way.
    let names_paths: bool = !region.pathspecs.is_empty() || region.pathspec_from_file.is_some();
    let unmerged_before: Vec<Vec<u8>> = if region.all
        || (matches!(
            layout.conclusion,
            Conclusion::Merge | Conclusion::CherryPick
        ) && names_paths)
    {
        list_unmerged_index_paths(context, cwd, layout.real_index_path.as_path())?
    } else {
        Vec::new()
    };
    part.next("capture");
    let selection: Selection = decide_selection(region, layout.conclusion);
    let reference: RefName = match &layout.symbolic_head {
        SymbolicHead::Branch(name) => name.clone(),
        SymbolicHead::Detached => RefName::from_text("HEAD"),
    };
    let detached: bool = layout.symbolic_head == SymbolicHead::Detached;

    let named: bool = selection.mode == TransactionMode::ExplicitPath;
    let capture: InvocationCapture = capture_invocation_base(context, cwd, layout)?;
    let unborn: bool = matches!(capture.base, Base::Unborn);
    // Read from the snapshot the layout already holds, so nothing starts Git for the message or the identities before the
    //  cheap decisions above are made, and before a service is provisioned.
    let config_native: bool = config_needs_native(capture.layout.config_snapshot.as_ref());
    let base: String = base_revision(&capture);
    let announced_base: String = match &capture.base {
        Base::Commit(oid) => oid.clone(),
        Base::Unborn => zero_object(capture.layout.empty_tree_oid.as_str()),
    };
    let region_arguments: &[OsString] = &invocation.arguments[invocation.subcommand_index + 1..];
    let mut pathspecs: Vec<OsString> = Vec::new();
    for at in &region.pathspecs {
        pathspecs.push(region_arguments[*at].clone());
    }
    part.next("scratch");
    let amends: bool = region.amend;
    // The tree goes where the sweep of the repository scratch directory covers it, so a command killed at any instruction leaves
    //  nothing that only this process could have removed (`Scratch`).
    let scratch: Scratch = match Scratch::make(
        capture.layout.common_dir.as_path(),
        capture.layout.git_dir.as_path(),
        scratch_name,
    ) {
        Ok(made) => made,
        Err(error) => return Err(failed(format!("preparing the commit failed: {error}"))),
    };
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: scratch.0.join("captured.index"),
        commit_index: scratch.0.join("commit.index"),
        object_directory: capture.layout.object_directory.clone(),
    };
    let pathspec_file: Option<PathBuf> = the_pathspec_file(
        region,
        region_arguments,
        scratch.0.as_path(),
        cwd,
        standard_input,
    )?;
    // What: `Preparation { ... }` builds the record every later step of the preparation reads.
    // Why: the caller works from one answer, read once, rather than from locals it would have to thread itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { capture, unmergedBefore, selection, reference, detached, named, unborn, configNative, base, announcedBase, pathspecs, amends, scratch, indexes, pathspecFile };
    // ```
    return Ok(Preparation {
        capture,
        unmerged_before,
        selection,
        reference,
        detached,
        named,
        unborn,
        config_native,
        base,
        announced_base,
        pathspecs,
        amends,
        scratch,
        indexes,
        pathspec_file,
    });
}
