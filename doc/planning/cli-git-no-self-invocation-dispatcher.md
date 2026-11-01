# cli-git: no self-invocation, phase 2: the commit hook dispatcher, and then all external hooks

This is the dispatcher section of the self-invocation ban, kept in its own file so merges stay clean.
The pointer file `doc/planning/cli-git-no-self-invocation.md` belongs to the lfs and push phase.
It records the decision, what was tried, what was measured, and the rollout order.

## The owner's words, in order

- "We should never invoke ourselves recursively. That's bad design. Audit, remove, and ban that design aggressively."
- "What the hell, we don't have to use any shell!"
- On running configured hooks inside Git: "Git runs your configured hooks directly - no it shouldn't do that. We don't
  need to reimplement git commit or use generated shell entries. There are better ways."
- On who needs support: "No, not dropping preparation hook support. I didn't ask you to do that. I only asked you to
  stop supporting OTHER PEOPLE's hooks."
- The settled scope, chosen label: "B: Only hooks we declare, natively".
  The wrapper supports no external hook command at all, anywhere.
  Every lifecycle stage keeps working because the stages are the wrapper's own policies and actions, declared in
  `cli-git.config.jsonc`; what an external hook did in this repository is ported in.

## What the dispatcher was

For every commit the wrapper wrote a `hooks` directory inside the transaction directory with three symbolic links named
`pre-commit`, `prepare-commit-msg` and `commit-msg`, each pointing at the wrapper's own executable, and ran the native
`git commit` in the shadow with `-c core.hooksPath=<tx>/hooks`.
Git found the link as the traditional hook of the event and started it, so Git started the wrapper.
`main.rs` read `argv[0]`, and `hook_dispatch.rs` recognized the entry by its name, took the hook lock, restored
`GIT_CONFIG_PARAMETERS`, exported the worktree root and a preparation lease, and ran
`git hook run --ignore-missing <event>` with a second `core.hooksPath`.
The `pre-commit` re-run after a replay and the post-landing `post-commit` used the same machinery.
It was a self-invocation by design, with its own lock, environment restoration and failure modes.

## What was tried and rejected

### Option i: the wrapper replays Git's message protocol

Measured on Git 2.56.0 and read in `builtin/commit.c` `prepare_to_commit` (v2.56.0).
`prepare-commit-msg` gets `COMMIT_EDITMSG` and a source argument that depends on the flag: `message` for `-m`, `-F`
and `--fixup`, `template` for a template, `commit <sha>` for `-c`, `-C` and `--amend`, `merge` and `squash` from the
state files, and `commit CHERRY_PICK_HEAD`; with an editor the file already holds the status comment block.
The editor runs between `prepare-commit-msg` and `commit-msg`, `--no-verify` skips only two of the three hooks, and
"nothing to commit" is decided between `pre-commit` and `prepare-commit-msg`.
Replaying that is reimplementing `git commit`.

### Option ii: a shell entry that execs `git hook run`

Rejected by the owner: no shell, no generated entries.

### Option iii: Git runs the configured hooks itself in the shadow, the wrapper holds a lock around the commit

Committed and measured (commits `7cd936b6bc` to `9b20e24e56` of this branch), then rejected by the owner.
It kept full message-protocol fidelity but held the hook lock across an open message editor, changed the hooks'
`GIT_WORK_TREE` and `GIT_CONFIG_PARAMETERS`, and still ran other people's programs.
Its measurement, for the record: a commit with one configured hook started one hook runner (four before) and synced 21
times (23 before).

## What was built

The wrapper runs no external hook program, and Git never starts the wrapper for a hook.

- Every real-Git start carries `-c core.hooksPath=/dev/null/cli-git-no-hooks` and `-c hook.<event>.enabled=false` for
  each of the 28 hook events of Git 2.56.0 (`HOOK_EVENTS` in `hooks_off.rs`; a control asks the host Git whether each
  name is an event, with a made-up name as the positive control).
  `pre-push` is left on: the push phase's own Git LFS upload step is a configured `pre-push` hook, and an event-level
  switch would turn it off.
  That phase should switch user hooks off by name from the snapshot it already reads (see "Not done").
- The shadow's `config` switches the same events off after its include, and a guard test asserts each event and the
  order.
- The native commit is `git <kept options> --git-dir=<shadow> -c core.hooksPath=<absent> commit ...`, with no
  `--work-tree` (the shadow's config names it) and no lease, no lock, no plan.
- A repository that configures hooks gets one `configuration-warning` (code `external-hooks-unsupported`) per commit,
  from the snapshot the commit already holds (no process start), and `git cli-git doctor` lists the hooks and exits 1.
  Both name the `hook.<name>` keys, the events and the ways forward.
- `hooks.concurrentCommits` still loads and is ignored, with a note in doctor and the docs.
- `postCommit.actions` in `cli-git.config.jsonc` declares what runs after a landing: `name`, `argv` array, optional
  `timeoutSeconds`.
  Arguments are literal text with the fields `{head}`, `{shortHead}` (12 characters), `{topLevel}` and `{env:NAME}`.
  The wrapper runs each action as a bounded direct child after disposal and maintenance and never fails the commit.
- Removed with the hooks: the replay `pre-commit` re-run, the hook-change bookkeeping (`HookChanges`,
  `describe_hook_changes`), `no_verify` tracking, the preparation lease, the hook lock and `hooks/plan.json`.
  The tearing table lost its three `hooks/plan.json` cases per phase.

## Removed and added

Whole files removed, with their line counts at the base of this work (`5561c8b7cb`):
`hook_dispatch.rs` (286) and its tests (255), `hook_dispatch_entries.rs` (162) and its tests (149),
`hook_dispatch_plan.rs` (604) and its tests (457), `post_commit_hook.rs` (267) and its tests (442),
`binary_hook_tests.rs` (306), `binary_hook_shim_tests.rs` (481): 3409 lines.
The `argv[0]` branch of `main.rs`, `PreparationHooks`, the preparation lease functions of `transaction_lease.rs`, and
the lease checks of `wrapped_command.rs` and `commit_command.rs` are gone too.
Whole files added: `external_hooks.rs` (201), `doctor_config.rs` (128), `config_post_commit.rs` (193),
`post_commit_template.rs` (208), `post_commit_action.rs` (239), `lexical_path.rs` (50), their test sidecars, and
`binary_external_hook_tests.rs` (242).

## Counts

`binary_budget_tests.rs`, strace over the whole process tree, Git 2.56.0.

- The commit with no hooks and the amend and warm commits: unchanged (19 wrapper syncs for the no-hook commit).
- The one-hook commit: the original dispatcher started 4 `hook run` processes, 3 more wrapper processes, and synced 23
  times; option iii started 1 and synced 21; now it is a commit with one declared action: 0 hook runners, 1 `show`
  (the author lookup for the hook is gone), the action's own program (`touch`) as the one extra start, and 19 syncs,
  the same as the commit with no hooks.
- No count rose anywhere.

## Guard

`hook_self_invocation_tests.rs` scans `main.rs` for a start-name read and eight commit-path sources for an entry
writer, Git's hook runner (`"hook", "run"`, `hook run`), a shell and a transaction hooks path.
It was shown failing on the real sources: a start-name read planted in `main.rs` gave `main.rs contains
args_os().next()`; the pair `["hook", "run", "post-commit"]` planted in `post_commit_action.rs` gave
`post_commit_action.rs contains "hook", "run"`; one event switch flipped in `shadow_config.rs` gave
`applypatch-msg is not switched off`; each file was restored and its `git diff` was empty afterwards.
`binary_external_hook_tests.rs` proves the behavior with a real commit.

## Rollout order

A wrapper that does not know `postCommit` refuses the whole configuration file ("Unknown configuration key"), so the
repository's `cli-git.config.jsonc` must not name the action before the new wrapper is installed.

1. Release the wrapper and install it.
   From this moment the local Git config hook `hook.standards-review-shadow` is switched off (and reported), so the
   review does not run until step 2: commits are not blocked.
2. In the same sitting add the declaration to the repository's `cli-git.config.jsonc` (below) and commit it.
3. Remove the local hook so the warning goes away:
   `git config --unset-all hook.standards-review-shadow.event` and `git config --unset-all
   hook.standards-review-shadow.command`.

The declaration, to add after the `policies` object:

```jsonc
// cli-git.config.jsonc
"postCommit": {
  "actions": [
    {
      "name": "standards-review-shadow",
      "argv": [
        "systemd-run",
        "--user",
        "--unit=standards-review-shadow-{shortHead}",
        "--collect",
        "--working-directory={topLevel}",
        "--setenv=PATH={env:PATH}",
        "--setenv=HOME={env:HOME}",
        "mise",
        "run",
        "shadow-review"
      ]
    }
  ]
}
```

The unit name uses 12 characters of the commit where the shell command used `git rev-parse --short HEAD`
(usually 7), which keeps two landings from sharing a unit name.
The local hook lived only in machine-local Git config, so there is no installer in the repository to change; the
runbook `doc/runbook/standards-review-gate.md` now describes the declaration.

## What else relied on external hooks

- This repository: only `hook.standards-review-shadow`, ported above.
  No `hook.pre-commit.*`, `prepare-commit-msg` or `commit-msg` configuration exists in the repository, its mise tasks
  or its packages, and no file sets `hooks.concurrentCommits`, so no preparation-hook behavior needed porting.
- The container end-to-end suite under `package/git-policy/cli/e2e` uses configured hooks as its instrument: hook
  program fixtures (`hook-program-fixture.ts`), baseline hook scenarios (`scenario-baseline-hook-fixture.ts`), the
  `commit-msg-hook` and `post-commit-once` invariants, kill-in-hook scenarios and the observation of `post-commit`
  runs.
  They cannot observe anything now.
  They were not rewritten: they need a decision on a replacement instrument (the editor for the kill points, a declared
  action for the post-commit observation).
- `package/git-policy/cli.fuzz` builds its expected configuration from the parser's types; its generator now sets the
  `hooks.present` flag.
- Handover and research documents that describe hooks (`doc/handover/cli-git-*.md`, `doc/research/...`) are history and
  were not edited.

## Not done

- `pre-push`: user hooks on pushes still run until the push phase (owned by the lfs agent) disables them by name from
  its snapshot and moves the LFS upload out of Git's hook mechanism.
- `landing_index.rs` still carries the `guarded_paths` guard that hook-staged paths used; the commit path passes none.
  It can be removed with its tests in a follow-up.
- Files over 300 lines that this change touched and did not create: `commit_preparation.rs`, `commit_conclusion.rs`,
  `config_schema.rs`, `config_parse.rs`, `doctor.rs` and test sidecars beside them; all were over before.
- The planted-control table `bin/native-planted-controls-entry.ts` was updated for the new guards and each `from`
  text was checked to occur once, but it runs only in the container gate.
- Windows: the crate does not build for Windows, so the removed `<event>.exe` naming had no gate.
