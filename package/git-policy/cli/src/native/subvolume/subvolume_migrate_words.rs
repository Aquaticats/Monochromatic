//! What:
//!  The lines a migration says to the person:
//! the facts the run begins with,
//! and the closing text of a finished migration with everything left to do.
//! Why:
//!  A migration keeps the old tree and leaves open sessions in it,
//! so what it says is part of the command's answer and not a log;
//! the steps themselves are `subvolume_migrate_steps.rs`,
//! the options they read are `subvolume_migrate_options.rs`,
//! and the command that calls both is `subvolume_migrate.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! announce(run, options); /* the steps */ closing(run);
//! ```

use super::subvolume_migrate_options::MigrateOptions;
use crate::process_use::{
    UseTarget,
    processes_using,
};
/// A name another program chose,
/// shown so no byte of it can act on the line it lands in,
/// and the parentheses that line puts around it.
use crate::shown_so_it_cannot_rewrite_the_line::{
    TheWrappingAroundTheName,
    shown_so_it_cannot_rewrite_the_line,
};
use crate::subvolume_index_files::linked_worktrees_inside;
use crate::subvolume_preflight::Layout;
use crate::subvolume_run::Run;
use std::path::PathBuf;

/// What:
///  One process that keeps its working directory in the tree the exchange replaces,
///  as the line naming it:
///  its number and the program it runs,
///  the program shown so no byte of it can act on the line.
/// Why:
///  A program sets its own name to any bytes it likes (`/proc/<pid>/comm`),
///  and this line is the person's pointer to what must be restarted:
///  an escape in the name would move the cursor or erase the line,
///  and a `)` would end the parentheses that say whose name it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (pid, command) => `  pid ${pid} (${shownSoItCannotRewriteTheLine(command ?? 'name unreadable', TheWrappingAroundTheName.Parentheses)})`;
/// ```
pub(crate) fn a_process_left_in_the_old_tree_line(
    pid: i64,
    command: Option<&str>,
) -> String {
    return format!(
        "  pid {pid} ({})",
        shown_so_it_cannot_rewrite_the_line(
            command.unwrap_or("name unreadable"),
            TheWrappingAroundTheName::Parentheses
        )
    );
}

/// What:
///  Print the facts the preview and the run begin with:
/// the mount,
/// the option,
/// the processes that will stay behind.
/// Why:
///   A process whose working directory is inside the checkout keeps writing into the old tree after the exchange.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// announce(run, options);
/// ```
pub(crate) fn announce(
    run: &mut Run<'_>,
    options: MigrateOptions,
) {
    let root: PathBuf = run.layout.root.clone();
    let removal: bool = run.platform.removal_allowed(root.as_path());
    run.line(
        format!(
            "checkout {}; new subvolume {}; old tree will be {}",
            root.display(),
            run.layout.new_path.display(),
            run.layout.old_path.display()
        )
        .as_str(),
    );
    run.line(format!("user_subvol_rm_allowed: {}", if removal { "active" } else { "absent (a subvolume is then removed by emptying it and rmdir; see git monochromatic-git-policy-cli doctor)" }).as_str());
    let real: PathBuf = std::fs::canonicalize(root.as_path()).unwrap_or(root);
    let scan = processes_using(&UseTarget::Inside(real.as_path()));
    if scan.users.is_empty() {
        run.line("no other process has its working directory inside the checkout");
    } else {
        run.line(format!("WARNING: {} process(es) have a working directory inside the checkout and will stay in the OLD tree after the exchange; restart them (or `cd \"$PWD\"` in a shell) afterwards:", scan.users.len()).as_str());
        for user in &scan.users {
            run.line(
                a_process_left_in_the_old_tree_line(user.pid, user.command.as_deref()).as_str(),
            );
        }
    }
    let nested: usize = if options.nested_worktrees {
        linked_worktrees_inside(run.layout.root.as_path()).len()
    } else {
        0
    };
    run.line(
        format!("linked worktrees inside the checkout that become nested subvolumes: {nested}")
            .as_str(),
    );
}

/// What:
///  The closing text of a finished migration.
/// Why:
///   It states what is left to do and every way to undo or finish:
/// restart sessions,
/// check Git,
/// delete the old tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// closing(run);
/// ```
pub(crate) fn closing(run: &mut Run<'_>) {
    let layout: &Layout = run.layout;
    let (root, old_path) = (
        layout.root.display().to_string(),
        layout.old_path.display().to_string(),
    );
    run.line(format!("{root} is now a btrfs subvolume (root inode 256).").as_str());
    run.line(format!("The original tree is kept at {old_path}. Check the result with `git status` and `git fsck`, then delete it with `git monochromatic-git-policy-cli migrate-to-subvolume --remove-old` (or undo everything with `--rollback`).").as_str());
    run.line("Shells, editors and agent sessions that had their working directory inside the checkout still use the old tree; restart them, or run `cd \"$PWD\"` in a shell.");
    if !run.unreadable.is_empty() {
        let count: usize = run.unreadable.len();
        let list: String = run.layout.unreadable_path.display().to_string();
        run.line(format!("{count} file(s) this user may not read were not copied and exist only in the old tree; they are listed in {list}. Copy them with the needed privilege before deleting the old tree.").as_str());
    }
}

/// Controls of the lines a migration says stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_migrate_words_tests.rs"]
mod tests;
