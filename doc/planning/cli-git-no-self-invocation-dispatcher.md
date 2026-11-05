# cli-git: no self-invocation, phase 2: the commit hook dispatcher, and then all external hooks

Note 2026-11-04T22:23:58-05:00: the program `cli-git-native` is now `monochromatic-git-policy-cli`,
the command word `git cli-git` is now `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` is now `monochromatic-git-policy-cli.config.jsonc`,
and the folders, logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
This record keeps the names of its time.

This is the dispatcher section of the self-invocation ban, kept in its own file so merges stay clean.
The pointer file `doc/planning/cli-git-no-self-invocation.md` belongs to the lfs and push phase.
It records the decision, what was tried, what was measured, and what is still open.

## The owner's words, in order

- "We should never invoke ourselves recursively. That's bad design. Audit, remove, and ban that design aggressively."
- We don't have to use any shell.
- On running configured hooks inside Git: "Git runs your configured hooks directly - no it shouldn't do that. We don't
  need to reimplement git commit or use generated shell entries. There are better ways."
- "No, not dropping preparation hook support. I didn't ask you to do that. I only asked you to stop supporting OTHER
  PEOPLE's hooks."
- The settled scope: the wrapper supports no external hook command at all, anywhere.
- On a generic declared-action facility (`postCommit.actions`, built and then deleted): "I already said we don't allow
  3p code and custom actions".
  The wrapper executes no program named by any configuration: not Git config, not `cli-git.config.jsonc`, not the git
  directory, not the environment.

## What the dispatcher was

For every commit the wrapper wrote a `hooks` directory inside the transaction directory with three symbolic links named
`pre-commit`, `prepare-commit-msg` and `commit-msg`, each pointing at the wrapper's own executable, and ran the native
`git commit` in the shadow with `-c core.hooksPath=<tx>/hooks`.
Git found the link as the traditional hook of the event and started it, so Git started the wrapper.
`main.rs` read `argv[0]`, and `hook_dispatch.rs` recognized the entry by its name, took the hook lock, restored
`GIT_CONFIG_PARAMETERS`, exported the worktree root and a preparation lease, and ran
`git hook run --ignore-missing <event>` with a second `core.hooksPath`.
The `pre-commit` re-run after a replay and the post-landing `post-commit` used the same machinery.

## What was tried and rejected

- **The wrapper replays Git's message protocol** (read in `builtin/commit.c` `prepare_to_commit`, v2.56.0):
  `prepare-commit-msg` gets a source argument that depends on the flag (`message`, `template`, `commit <sha>`, `merge`,
  `squash`), the editor runs between it and `commit-msg`, `--no-verify` skips two of three hooks, and "nothing to
  commit" is decided between `pre-commit` and `prepare-commit-msg`. That is reimplementing `git commit`.
- **A shell entry that execs `git hook run`**: rejected by the owner (no shell, no generated entries).
- **Git runs the configured hooks in the shadow under a lock the wrapper holds around the commit**: committed and
  measured, then rejected by the owner. It held the lock across an open message editor and still ran other people's
  programs. Measured: a commit with one configured hook started one hook runner (four before) and synced 21 times (23).
- **A `postCommit.actions` facility in `cli-git.config.jsonc`** (an argument array the wrapper runs after a landing):
  built, tested and then deleted on the owner's instruction, as a custom-action path.

## What exists now

The wrapper runs no external hook program and no program any configuration names, and Git never starts the wrapper.

- Every real-Git start carries `-c core.hooksPath=/dev/null/cli-git-no-hooks` and `-c hook.<event>.enabled=false` for
  each of the 28 hook events of Git 2.56.0 (`HOOK_EVENTS` in `hooks_off.rs`).
  The options follow every global option of the caller and precede the subcommand, so a caller's `-c` or
  `--config-env` cannot switch a hook or the hooks path back on (probed: a caller's
  `-c core.hooksPath=X -c hook.post-commit.enabled=true -c hook.reference-transaction.enabled=true commit` runs
  nothing).
  Controls: the host Git lists each name as a hook event (with a made-up name as the positive control), lists a
  configured hook of every event as disabled with the options and enabled without them, and, when the `githooks(5)` page
  is installed, every documented event is in the list.
- `pre-push` is switched off like the other 27 events: the LFS branch removed the wrapper's injected LFS hook, so no
  exception remains (the temporary by-name switch for a configured `pre-push` hook was removed with it).
- The shadow's `config` switches the same events off after its include (a guard test asserts each event and the order).
- The native commit is `git <kept options> --git-dir=<shadow> -c core.hooksPath=<absent> commit ...`, with no
  `--work-tree` and no lease, lock or plan.
- A repository that configures `hook.<name>.event` gets a `configuration-warning` (code `external-hooks-unsupported`)
  that names the keys, the events and the ways forward and says the wrapper runs no program a configuration names:
  on every commit and on every non-dry-run push (the two commands that already hold the configuration snapshot, so no
  extra Git start), and `git cli-git doctor` lists the hooks and exits 1.
  Other forwarded commands print nothing, because reading the snapshot would cost a Git start.
  `git hook run <event>` is forwarded unchanged; with every event switched off it runs nothing and exits 0 quietly, so
  an alias that relies on it passes silently. Refusing `git hook run` is the alternative; not done, an open choice.
- `hooks.concurrentCommits` still loads and is ignored, with a note in doctor and the docs.
- A `postCommit` key in `cli-git.config.jsonc` is a configuration error: "Configuration key postCommit is not
  supported: cli-git runs no program named by any configuration ...".
- Removed with the hooks: the replay `pre-commit` re-run, the hook-change bookkeeping (`HookChanges`,
  `describe_hook_changes`), `no_verify` tracking, the preparation lease, the hook lock, the plan and `hooks/plan.json`,
  `defines_named_hook`, and the post-landing hook run.

## core.fsmonitor

Measured on Git 2.56.0 (`status` with a `core.fsmonitor` program): the program runs with `core.hooksPath` pointing at a
missing directory and with `-c hook.fsmonitor-watchman.enabled=false`, so the event switch does not reach it.
The owner decided the wrapper does not stop it ("To stop the fsmonitor program - we don't need to."), so `core.fsmonitor`
is left as configured, like the other programs Git starts for its own features, and the built-in monitor daemon keeps
working.
A first attempt switched it off with `-c core.fsmonitor=false` and reported it as a configured hook; both were removed.

## Deleted, with line counts at the base of this work (`5561c8b7cb`)

Whole files, 3409 lines:
`hook_dispatch.rs` 286 and its tests 255, `hook_dispatch_entries.rs` 162 and its tests 149, `hook_dispatch_plan.rs` 604
and its tests 457, `post_commit_hook.rs` 267 and its tests 442, `binary_hook_tests.rs` 306,
`binary_hook_shim_tests.rs` 481.
The generic action facility, built after the base and deleted: `post_commit_action.rs` 239 and its tests 132,
`post_commit_template.rs` 208 and its tests 76, `config_post_commit.rs` 193 and its tests 90: 938 lines.
Added and kept: `external_hooks.rs`, `doctor_config.rs`, `lexical_path.rs`, their test files, and
`binary_external_hook_tests.rs`.

## Counts

`binary_budget_tests.rs`, strace over the whole process tree, Git 2.56.0.

- A commit with no hooks, an amend and a warm commit: unchanged (19 wrapper syncs for the no-hook commit).
- A commit in a repository that configures a `post-commit` hook: 0 hook runners, 19 syncs, every Git start equal to the
  commit with no hooks.
  The original dispatcher started 4 hook runners and synced 23 times for one configured hook; the lock-around-commit
  design started 1 and synced 21 times.
- No count rose anywhere.

## Guards

`hook_self_invocation_tests.rs` scans `main.rs` for a start-name read and eight commit-path sources for an entry writer,
Git's hook runner, a program named by a literal (a shell is one) and a transaction hooks path.
Its self-test now plants one source per forbidden token and requires each to be flagged in code and ignored in a
comment.
Shown failing on the real sources (before the action facility was deleted): a start-name read planted in `main.rs`
gave `main.rs contains args_os().next()`; `["hook", "run", "post-commit"]` planted in a commit-path source gave
`contains "hook", "run"`; one event switch flipped in `shadow_config.rs` gave `applypatch-msg is not switched off`;
each file was restored and its diff was empty afterwards.
`binary_external_hook_tests.rs` proves the behavior with real commits: no configured hook runs on commit or checkout, a
caller's own `-c` cannot bring one back or redirect the shadow with `core.worktree`, a global-file hook is reported, the
`postCommit` key is an error, and a wrapper link named like a former entry behaves as `git`.

## Open: what replaces `hook.standards-review-shadow`

This repository's one configured hook is machine-local Git config (no installer exists in the repository):
event `post-commit`, command
`systemd-run --user --unit=standards-review-shadow-$(git rev-parse --short HEAD) --collect
--working-directory=$(git rev-parse --show-toplevel) --setenv=PATH=$PATH --setenv=HOME=$HOME mise run shadow-review`.
It starts a detached systemd user unit per landing, so the landing never waits.
The `shadow-review` task in `mise.toml` reads the landed commit (`git show`, `git diff`), writes the diff and a candidate
excerpt under `${HOME}/temp/agent/shadow-review`, sends them (24000 characters at most) to the LLM gateway
(`https://api.llmgateway.io/v1/decisions`, key from `SHADOW_REVIEW_LLMGATEWAY_API_KEY`) unless
`SHADOW_REVIEW_ALLOW=1`, and appends an entry to `doc/audit/standards-review-gate-shadow.md` as an uncommitted change
that a later commit carries.
Its findings are required by nothing and checked by nothing.
Nothing on this branch depends on it; since the wrapper now switches the hook off, the review stops running until the
owner decides what replaces it.
The runbook `doc/runbook/standards-review-gate.md` says so.

## What else relied on external hooks

- This repository: only `hook.standards-review-shadow`. No `hook.pre-commit.*`, `prepare-commit-msg` or `commit-msg`
  configuration exists in the repository, its mise tasks or its packages, and no file sets `hooks.concurrentCommits`.
- The container end-to-end suite under `package/git-policy/cli/e2e` uses configured hooks as its instrument and now
  observes nothing; see "Not done".

## Not done

- The end-to-end suite port (editor as the hold and kill instrument; deleting the scenarios that tested only hooks).
- `guarded_paths` plumbing in `landing_index.rs`, `landing_attempt.rs` and `commit_landing_loop.rs` (always empty now)
  and its tests: left, because the landing lane pins it with its own tests.
- The planted-control table `bin/native-planted-controls-entry.ts` runs only in the container gate.
- Windows: the crate does not build for Windows, so the removed `<event>.exe` naming had no gate.
