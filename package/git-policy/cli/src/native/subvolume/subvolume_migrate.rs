//! What:
//!  Convert a main checkout into a btrfs subvolume at the same path,
//! in process,
//! resumably.
//! Why:
//!  The steps are those of the proven script (`doc/handover/cli-git-subvolume-migration-script.md`),
//! without any
//!  program started:
//! create `<root>.subvol-new` as a subvolume;
//! create the linked worktrees inside it as nested
//!  subvolumes;
//! copy the tree with cloned data;
//! repeat the copy as a delta;
//! hold `.git/index.lock`,
//! run the last
//!  delta and swap the two directories in one atomic step;
//! rename the old tree aside;
//! carry over files newer in the
//!  old tree;
//! rewrite the Git index stat fields;
//! verify.
//! Each step is recorded in the journal (`subvolume_journal.rs`),
//!  so a rerun after a kill at any point resumes:
//! a finished step is skipped and an unfinished one is repeated,
//! and all
//!  of them are idempotent.
//! The old tree is never deleted here.
//!
//! The options a caller passes and the verdict a finished run returns are `subvolume_migrate_options.rs`,
//! the steps that change the checkout are `subvolume_migrate_steps.rs`,
//! and the lines a person reads are `subvolume_migrate_words.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = await migrate({ platform, root, options, interrupt, say });
//! ```

pub use super::subvolume_migrate_options::{
    MigrateOptions,
    Verdict,
};
use super::subvolume_migrate_steps::{
    create_subvolumes,
    exchange_trees,
    finish,
    is_subvolume,
};
use super::subvolume_migrate_words::{
    announce,
    closing,
};
use crate::subvolume_arguments::MIGRATE_RISK_NOTICE;
use crate::subvolume_error::{
    Interrupt,
    MigrateError,
};
use crate::subvolume_platform::Platform;
use crate::subvolume_preflight::{
    Refusal,
    layout_of,
    migrate_refusals,
};
use crate::subvolume_run::Run;
use std::path::Path;
use std::time::Instant;

pub fn migrate(
    platform: &dyn Platform,
    root: &Path,
    options: MigrateOptions,
    interrupt: &Interrupt,
    say: &mut dyn FnMut(&str),
) -> Result<Verdict, MigrateError> {
    let Some(layout) = layout_of(root) else {
        return Err(MigrateError::Journal(format!(
            "{} has no parent directory to hold the new subvolume",
            root.display()
        )));
    };
    let mut run: Run<'_> = Run::open(platform, &layout, interrupt, say)?;
    if is_subvolume(platform, root)
        && (run.journal.is_done("verify") || !run.journal.is_persisted())
    {
        run.line(
            format!(
                "{} is already a btrfs subvolume; nothing to do.",
                root.display()
            )
            .as_str(),
        );
        if run.journal.is_done("verify") && layout.old_path.exists() {
            run.line(format!("The original tree is still at {}; delete it with `git monochromatic-git-policy-cli migrate-to-subvolume --remove-old`.", layout.old_path.display()).as_str());
        }
        return Ok(Verdict::AlreadyDone);
    }
    let refusals: Vec<Refusal> = migrate_refusals(platform, &layout, &run.journal);
    if !refusals.is_empty() {
        return Ok(Verdict::Refused(refusals));
    }
    // The accepted risk of another program changing the same files is said once, before the first step that changes the
    // checkout; a refusal above and a preview change nothing, so neither says it.
    if !options.dry_run {
        run.line(MIGRATE_RISK_NOTICE);
    }
    announce(&mut run, options);
    if options.dry_run {
        run.line("plan: create the new subvolume and the nested ones; copy with cloned data; repeat as a delta; hold .git/index.lock, run the last delta, exchange the directories atomically; carry over newer files; rewrite the index stat fields; verify.");
        run.line(
            "dry run complete; nothing was changed. Run the command without --dry-run to migrate.",
        );
        return Ok(Verdict::DryRun);
    }
    let started: Instant = Instant::now();
    create_subvolumes(&mut run, options)?;
    if !run.journal.is_done("copy") {
        run.journal.mark("copy", "started")?;
        run.pass(
            "copy",
            layout.root.as_path(),
            layout.new_path.as_path(),
            false,
            false,
        )?;
        run.checkpoint("after-copy")?;
        run.journal.mark("copy", "done")?;
    }
    if !run.journal.is_done("delta1") && !run.journal.has_begun("exchange") {
        run.pass(
            "first delta",
            layout.root.as_path(),
            layout.new_path.as_path(),
            true,
            false,
        )?;
        run.journal.mark("delta1", "done")?;
    }
    run.checkpoint("after-delta1")?;
    exchange_trees(&mut run)?;
    finish(&mut run, options)?;
    run.line(format!("migration took {:.2} s", started.elapsed().as_secs_f64()).as_str());
    closing(&mut run);
    return Ok(Verdict::Migrated);
}

/// Migration controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_migrate_tests.rs"]
mod tests;
