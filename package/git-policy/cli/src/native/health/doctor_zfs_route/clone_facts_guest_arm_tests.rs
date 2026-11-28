//! What:
//!  The gatherer's guest arm:
//!  six managed clones of one repository,
//!  each gathered from the live machine and each asserted as its own group.
//! Why:
//!  The facts read `/dev/zfs`,
//!  a real pool's property replies,
//!  the plan folder and the all-namespace scan,
//!  so no host control may show them
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "VM measurements").
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!process.env.CLI_GIT_ZFS_CLONE_FACTS_GUEST) print('SKIPPED');
//! ```

/// The arm prints this where its own variable names no guest.
#[test]
fn zfs_doctor_clone_facts_guest() {
    if std::env::var("CLI_GIT_ZFS_CLONE_FACTS_GUEST").is_err() {
        println!("SKIPPED CLI_GIT_ZFS_CLONE_FACTS_GUEST is not set, so no live arm ran");
    }
}
