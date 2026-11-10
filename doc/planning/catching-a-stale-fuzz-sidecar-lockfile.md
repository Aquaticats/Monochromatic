# Catching a stale fuzz sidecar lockfile

Status:
proposed 2026-11-09;
option A built 2026-11-10 on `ci/git-policy-cli-fuzz-sidecar-compile-workflow`,
after the owner chose "Workflow only" in the question tool.
Options G,
B,
C,
D and E stay unbuilt.
The owner did not answer whether `lint:sidecar-locks` should go,
so it stays where it is,
and `toml-edit-fuzz.yml` and `logger-fuzz.yml` gained no lock check of their own.

## Purpose

`package/git-policy/cli.fuzz` is an independent Cargo workspace
that links `package/git-policy/cli` by path.
Its `Cargo.lock` therefore records the wrapper's whole dependency graph
at the versions the wrapper locked,
so any dependency the wrapper gains,
drops or moves makes that lock stale.
A stale lock makes cargo refuse the sidecar's build,
and then its fuzz targets,
its generator controls and its smoke campaign all stop compiling.

The same missing trigger loses a second,
larger failure:
the sidecar's sources call the wrapper's API,
so a wrapper change to a public type breaks the sidecar without touching its lock.
Both failures were measured on 2026-11-09 at `main` `33562f4ff7`.

## What exists today

- `//package/git-policy/cli:lint:sidecar-locks`
  (`package/git-policy/cli/bin/native-sidecar-locks.ts`)
  reads both resolutions through `cargo metadata --locked --offline`
  and fails on cargo's refusal of a sidecar lock
  and on any package the wrapper locks that the sidecar locks at another version.
  It is a `lint:*` child of the wrapper package,
  so the wrapper's `lint` runs it,
  and the wrapper's container gate runs it before it builds anything.
  Measured cost:
  2.84 s with a warm registry and build cache.
- `//package/git-policy/cli.fuzz:lint:types` runs
  `cargo check --all-targets --locked --offline`
  and Clippy with `-D warnings`,
  so cargo's own refusal of a stale lock fails the sidecar's build too.
  It is the sidecar's `lint`,
  which the root `lint` fanout reaches.
  Measured cost:
  5.2 s for the check and 13.1 s for Clippy when warm,
  and 152 s for the check from an empty `target/` on four CPUs,
  where it first failed.
- `//package/git-policy/cli.fuzz:lock:members` records moved versions again
  with `cargo update --offline --workspace`;
  `lock:subject` seeds the sidecar's lock from the wrapper's before that.
  Neither is a hand edit.

## The gap

Nothing runs either check when the wrapper moves.
No workflow runs the wrapper's or the sidecar's lint;
the two workflows that name `package/git-policy/cli`
(`cargo-publish.yml` and `npm-release.yml`) are release plumbing.
No Git hook runs lint on push,
and merges happen locally with auto-push.
The root `lint` fanout is the only path,
and it runs only when someone starts it.

Measured on 2026-11-09 at `main` `33562f4ff7`:

- The wrapper's lock check passed and cargo accepted the sidecar's lock,
  so the lock itself was current.
- The sidecar did not compile against the wrapper's sources:
  four errors in `src/batch.rs`,
  `src/batch_tests.rs` and `src/controls.rs`.
- The wrapper changes responsible landed after the sidecar's last source change:
  `6bd37751ab` (2026-11-08T23:26:11-05:00) added `BatchReply::TooLarge`,
  `de74356200` (2026-11-09T03:03:32-05:00) made `shipped_checks` take `ToolCacheDirectory`,
  and `61b25ef6d6` (2026-11-09T06:24:39-05:00) added three `RepositoryFacts` methods.
  The wrapper's own `lint` reads the sidecar's lock,
  never its sources,
  so each of those could land with the sidecar's build broken.

## Options

### A. A path-filtered workflow that runs the checks

Pros:

- Runs without anyone remembering,
  on pull request,
  merge queue and push to `main`.
- Precedent exists:
  `.github/workflows/toml-edit-fuzz.yml` and `.github/workflows/logger-fuzz.yml`
  already run a subject's and its fuzz sidecar's `lint:types` on a path-filtered change.
- Calls the two tasks that exist,
  so no check code is new.
- The filter can name the wrapper,
  the sidecar and the six crates the sidecar links by path,
  so a linked crate's source change fires it as well.

Cons:

- CI minutes:
  the lock check is seconds,
  the sidecar's compile is minutes.
- Needs network for `fetch:rust-dependencies`,
  because both checks run `--offline`.
- Reports after the push that landed it,
  unless the branch is reviewed through a pull request.

### B. A root `lint:sidecar-locks` fanout over every package with a sidecar crate

Pros:

- One command covers every sidecar crate;
  there are 19 `.fuzz` directories today,
  not all of them Rust crates.
- No CI minutes and no network beyond the local registry.

Cons:

- Still on demand,
  so the reported failure stays possible.
- Needs a subject-to-sidecar mapping that does not exist yet,
  since each sidecar names its subject by path in its own manifest.
- Wider than this task.

### C. A pre-push hook that runs the lock check

Pros:

- Fires at the only automatic local step,
  in about three seconds.
- Catches the drift before the merge lands.

Cons:

- No hook mechanism exists:
  there is no `core.hooksPath`,
  and the mise `enter` hook is commented out.
- A failing hook stops a push,
  which the owner's auto-push flow would feel on every branch.
- It is installed per clone,
  so other agents' clones behave differently.

### D. Repair the lock inside the sidecar's lint

Pros:

- One command both fixes and checks,
  and it reuses `lock:members`.

Cons:

- A check that writes the working tree hides the drift from the diff a reviewer reads.
- A fanout would mutate the tree while other work runs in it.

### E. One workspace for the wrapper and its sidecar

Pros:

- Removes the second lockfile and the whole failure class with it.

Cons:

- cargo-fuzz expects its own workspace and lock,
  and every other sidecar keeps its own.
- Far larger than the check,
  for one sidecar.

### F. Keep the status quo

Pros:

- Nothing to build.

Cons:

- The observed failure stays:
  the check exists but runs only when someone remembers.

### G. Make the wrapper's `lint` compile the sidecar

Pros:

- Runs on the branch that changed the wrapper,
  at the lint step its agent already runs,
  so both the lock and the source lag are caught before the merge.
- Continues the owner's own decision:
  the wrapper's `lint` already reads the sidecar's lock.
- No CI minutes and no network beyond the local registry.

Cons:

- The wrapper's `lint` becomes minutes slower from an empty `target/`,
  and seconds when warm.
- A wrapper-only change then fails on a sidecar file,
  so its author must repair the sidecar in the same commit.

Ranking:
G beats A because it reports on the branch that changed the wrapper,
before the merge,
and costs no CI minutes.
A beats C because it needs no per-clone setup and reports instead of stopping a push.
C beats B because it fires without anyone remembering to start the fanout.
B beats D because it reports drift and leaves the tree alone,
so the lock diff stays visible.
D beats E because it reuses the existing lock and task,
while E restructures the workspace for one sidecar.
E beats F because F keeps the failure that was measured.

## Recommendation

G,
narrowly:
the wrapper's `lint` gains one task that runs
`mise run //package/git-policy/cli.fuzz:lint:types`,
next to the existing `lint:sidecar-locks`.
The sidecar's `lint:types` compiles the targets against the wrapper with
`--locked --offline`,
so it fails on a stale lock and on a wrapper API change alike,
and it is the same task the root `lint` fanout already runs.
The wrapper's own `lint:sidecar-locks` stays,
because it fails in seconds and names the repair task.

The repair stays `mise run //package/git-policy/cli.fuzz:lock:members`,
never a hand edit.

If the owner also wants coverage after a push,
option A's workflow is the automatic backstop,
using the filter and steps under `## Path filter for option A`.
It is not part of this recommendation,
because G reports before the merge and costs no CI minutes.

## What option A built (2026-11-10T06:22:10-05:00)

One workflow,
`.github/workflows/git-policy-cli-fuzz.yml`,
triggered on `pull_request`,
`merge_group` and `push` to `main`,
path-filtered to:

- `package/git-policy/cli/**`
- `package/git-policy/cli.fuzz/**`
- `package/jsonc-edit/core-rust/**`
- `package/catalog-specifier/core-rust/**`
- `package/forbidden-strings/cli/**`
- `package/forbidden-strings/core/**`
- `package/linter/monochromatic-lint/**`
- `.github/workflows/git-policy-cli-fuzz.yml`

Those are the wrapper,
the sidecar and every crate the sidecar links by path;
`cargo metadata` on the sidecar lists exactly those seven packages.

Its steps:

- checkout,
  then the repository's task CLI setup with no workspace install,
  because the task CLI and its task files load node builtins and relative files only;
- the pinned Rust toolchain from `toolchain.lock.json`,
  because both cargo calls compile the wrapper and its sidecar;
- `cargo fetch --locked` for both manifests,
  because the type check runs `--offline`;
- `task-run //package/git-policy/cli.fuzz:lint:types`,
  which is the compile that would have failed on 2026-11-09.

The workflow does not run the smoke campaign,
the planted controls,
the mutation campaign,
or any other package's lint.

Three departures from the step list drafted under option A:

- The task runs through `task-run`,
  never `mise run`:
  no workflow invokes mise (`doc/decision/mise-free-ci.md`).
  The task is therefore also defined mise-free,
  in `package/git-policy/cli.fuzz/task.ts`,
  which mirrors the sidecar's `mise.toml` `lint:types` body.
  The package had no task file before,
  so the CI task CLI could not reach this sidecar's tasks at all.
- Only `lint:types` runs.
  `lint:sidecar-locks` stays in the wrapper's own `lint`,
  where it fails in seconds and names the repair task;
  the owner has not decided whether it goes.
- The merge-group scope step reuses
  `package/toml-edit/core.fuzz/src/ci-scope.ts`
  with `SCOPE_PACKAGE_PREFIXES`,
  `SCOPE_CONTRACT_PATHS` and `SCOPE_SUBJECT`,
  the reuse that program was written for
  (`doc/decision/toml-edit-ci-scope.md`),
  rather than a second copy of the decision.
  A shell scope step is not available here:
  a new workflow's shell steps allow two lines and no pipe,
  no `;`,
  no `$(` and no `||`
  (`.github/workflows/monochromatic-lint.config.jsonc`).

Measured on 2026-11-10 in a worktree of
`ci/git-policy-cli-fuzz-sidecar-compile-workflow`,
at 400 percent CPU quota on eight host CPUs:

- The check passes at that branch's `main`:
  `cargo check --all-targets --locked --offline` finished in 5 m 58 s from an empty target directory,
  and `cargo clippy --all-targets --locked --offline -- -D warnings` then took 5.09 s;
  a warm rerun took 1.07 s and 0.37 s.
  The CPU-time cost of the CI job is the same order,
  so budget minutes rather than seconds for it.
- The check fails on a sidecar that does not compile:
  one appended probe naming a wrapper function that does not exist made it stop with
  `error[E0425]: cannot find function `symbol_moved_by_a_later_wrapper_change` in crate `git_policy_cli``,
  `error: could not compile `git-policy-cli-fuzz` (lib test) due to 1 previous error`,
  and exit 1 from the task CLI.
  Removing the probe restored the passing run.

## Open questions for the owner

- Whether the workflow should be required on the merge queue,
  or advisory.
- Whether `lint:sidecar-locks` should go,
  now that the CI compile fails on a stale lock too.
- Whether option B should follow in the same change or later,
  so the other sidecars gain the same check.
- Whether `toml-edit-fuzz.yml` and `logger-fuzz.yml` should gain the lock check for their own sidecars,
  which have the same second lockfile.
