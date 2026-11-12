# The discarding-commands guard: what is left

Branch: `fix/git-policy-cli-discarding-commands-guard-every-file-they-overwrite-or-delete`.
The guard now decides what a form writes from Git's own answer about the written paths and the
worktree's untracked and ignored entries, saves every entry in the way, and lets the form run.
`SPEC.md` in `package/git-policy/cli` says what it does, by form and by kind of entry, and records
the measurements it rests on (the reading, its cost, and the reset-mode measurement).

The two audits this work answers, on the host that wrote them:

- `${HOME}/temp/agent/kimi-trials/kimi-audit-alpha-12-discarding-commands/report.md`
- `${HOME}/temp/agent/sonnet-audits/alpha-12-merge-discarding-commands-lfs-and-package-repair/report.md`

## Open

One discarding form still destroys a file with no refusal and no copy:
`git read-tree -m -u HEAD <tree>` (and every `read-tree` form with `-u`) overwrites an ignored file
the target tree tracks, in native Git and through the wrapper, at exit 0. An untracked file in the
same place is refused by Git itself, so the ignored file is the whole of it.

Why it is not fixed here: a `read-tree` writes the worktree paths of the tree it names and, with
`-m`, the paths where the named trees differ, and no written-path reading is made for it
(`command/command_written_paths.rs`). Answering it needs a diff-shaped written-path reading,
which this run did not have the evidence to shape as carefully as the tree-shaped one.

A second item, not this branch's: one unit control of `main` hangs on this host,
`merge::merge_isolation_recovery::tests::a_side_ref_another_command_repointed_is_not_deleted`,
inside the wrapper's read of another process's `/proc/<pid>/fdinfo/<n>` (the inotify record of a
desktop file watcher). The coordinating session names that defect and the run that fixes it.
