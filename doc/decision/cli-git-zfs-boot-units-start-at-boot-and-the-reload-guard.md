# The ZFS boot units start at boot by a cited exception, and the reload guard matches only a manager reload

Status: accepted by the owner on 2026-11-25, answering the two `source_guard` findings of the round that built plan step 19's
boot units ahead (`feat/git-policy-cli-zfs-step-19-migration-entry-and-units-ahead`).

## What conflicted

The owner's rule of 2026-11-07, which `source_guard/no_start_at_login_tests.rs` enforces:
"Were you implying our wrapper or something in it auto starts itself on system startup or user login?
It should never do that! Follow Bazel."
Then: "Systemd units are fine."
The guard refuses every `[Install]` and `WantedBy=` line in a unit text the release writes,
and every release line containing `Reload`.

Step 19's accepted design (`doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`, with the owner's
2026-11-24 answers in `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`)
gives the root boot gate `reconcile-git-worktree-clone-mounts.service`, and the nested-dataset units,
a `WantedBy=multi-user.target` line,
and the accepted lost-event design (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`) gives the gate an
`ExecReload=` line.
That design never named the 2026-11-07 rule.

## Decision

- "Cited exception for boot units (Recommended)":
  the guard allows `[Install]` with `WantedBy=multi-user.target` only in the root boot gate's text and the nested-dataset
  units' text, which the one-time ZFS setup installs, citing step 19's design;
  it keeps refusing them everywhere else, and nothing of the wrapper starts at login.
  The commit service still starts only when a command needs it.
  The owner chose this over installing the gate without `[Install]`, which would leave every clone worktree an empty folder
  after a reboot until a command ran.
- "Narrow the guard (Recommended)":
  the guard keeps refusing `daemon-reload` and the manager's `Reload` and `DaemonReload` calls,
  whose cost it names (the manager holds its own event loop while it re-reads every unit),
  and stops matching a unit text's own `ExecReload=` line, which reloads only that unit;
  a control shows a planted daemon reload still fails it.
  The owner chose this over dropping `ExecReload=` from the gate, which the lost-event design needs so its printed lever
  re-runs the gate's pass instead of restarting the unit.
