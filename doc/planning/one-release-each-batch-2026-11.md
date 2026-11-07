# One release of each published crate: execution plan for the 2026-11 batch

Status:
the batch finished at 2026-11-07T13:10:36Z (first publish 2026-11-07T11:43:35Z);
the release record in the runbook lists what was dispatched and published,
and its "Steps after the batch" entry lists the lock move,
the live wrapper check,
the 14 yanks and the first daily run
(2026-11-07T08:20:23-05:00 to 2026-11-07T08:33:17-05:00).
Written before that as a plan only.
Written 2026-11-07T04:36:34-05:00 and checked against `main` at `7956f6c4c9` (the checks ran while `main` moved from
`17eff86b35` through `d62451d591`;
a diff of every `Cargo.toml`,
`Cargo.lock`,
`.github/workflows/*`,
`package/crate-release` and `package/git-policy/cli.fuzz` between `17eff86b35` and `7956f6c4c9` is empty).
A crates.io publish cannot be undone,
and a yank only hides a version,
so every step that publishes names its crate,
its versions and what must already be on crates.io.

## What the owner decided

- 2026-11-05,
  verbatim:
  "One release each.
  And we publish less to crates.io .
  That fact is given."
  Each of our published crates gets one new release.
- 2026-11-04,
  verbatim (`doc/planning/crate-release-yank-after-publish.md`):
  the crates except `forbidden-strings` and
  `forbidden-regex` carry the README notice "EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE" (landed in `1694ce1816`,
  2026-11-04T18:53:33-05:00),
  and the older releases are yanked right after the new ones.
- 2026-11-07,
  verbatim:
  "My thinking is to deploy all releases on pnpr,
  but only publish rc and stable releases on
  crates.io".
  Picked "Label decides":
  `-alpha.N` and `-beta.N` go to pnpr only;
  `-rc.N` and plain numbers go to pnpr and crates.io.
  This batch is plain numbers,
  so it goes to crates.io and the GitHub releases.
  pnpr does not serve crates yet,
  so nothing goes there for now.
- 2026-11-07:
  the one release goes out "After the test gap is fixed";
  that gap is fixed and merged.
- 2026-11-07,
  "Right after the release":
  the seven tools move from the `cargo:` install route to the `archive` route.
  That is a later step;
  this plan only keeps it possible (see "What this batch must leave possible").
- Also in force:
  "I'm authorizing you to bump and publish any updates this session."
  "Mutation runs don't block publish and release."
  "Ignore Win and mac until I say so."

## What crates.io holds now

Read from the public API (`https://crates.io/api/v1/crates/<crate>` and
`https://crates.io/api/v1/crates?user_id=415138`,
the account `Aquaticat`) at about 2026-11-07T04:00:00-05:00,
with a generic user agent.
The account owns 16 crates,
132 versions in all,
16 of them unyanked:
exactly one per crate,
its newest.
Four crates of this repository are not on crates.io and are not part of this batch:
`monochromatic-lint-closure`,
`wg-quicker-exempt`,
`truepeak-core`,
`monochromatic-codex-plugin-bash-limit`
(the API answered 404 for each).

For each crate:
newest version on crates.io and when it was created (UTC),
the `Cargo.toml` version on `main`
(equal in every case),
and whether the README on `main` carries the notice.
The notice landed 2026-11-04T23:53:33Z,
after every one of these uploads (the latest is `monochromatic-cargo-shadow`
0.1.3 at 2026-11-04T21:59:45Z),
so no published README shows it yet.

- `monochromatic-jsonc-edit`:
  0.1.2,
  2026-10-31T04:36:13Z;
  notice present.
- `monochromatic-xml-edit`:
  0.1.0,
  2026-10-31T05:01:26Z;
  notice present.
- `monochromatic-browserslist`:
  0.1.0,
  2026-10-31T05:12:51Z;
  notice present.
- `monochromatic-dependency-catalog`:
  0.2.0,
  2026-11-04T18:58:16Z;
  notice present.
- `monochromatic-catalog-specifier`:
  0.1.0,
  2026-11-01T04:50:19Z;
  notice present.
- `monochromatic-file-enforcer`:
  0.1.1,
  2026-10-31T05:28:39Z;
  notice present.
- `monochromatic-lint`:
  0.5.0,
  2026-11-01T13:39:33Z;
  notice present.
- `monochromatic-mpm-shadow`:
  0.2.5,
  2026-11-03T12:28:18Z;
  notice present.
- `monochromatic-cargo-shadow`:
  0.1.3,
  2026-11-04T21:59:45Z;
  notice present.
- `monochromatic-pnpm-shadow`:
  0.1.1,
  2026-11-01T15:17:07Z;
  notice present.
- `monochromatic-search-shadow`:
  0.2.1,
  2026-10-31T05:50:33Z;
  notice present.
- `monochromatic-nested-wayland-session`:
  0.6.2,
  2026-10-25T23:38:41Z;
  notice present.
- `monochromatic-file-enforcer-cli`:
  0.1.2,
  2026-11-04T20:41:44Z;
  notice present.
- `monochromatic-git-policy-cli`:
  0.5.34,
  2026-11-04T20:27:47Z;
  notice present.
- `forbidden-regex`:
  0.1.1,
  2026-09-17T03:32:14Z;
  exempt,
  no notice (0.1.0 is yanked).
- `forbidden-strings`:
  0.6.1,
  2026-11-01T12:33:50Z;
  exempt,
  no notice (the 16 older versions are all yanked).

Every one of the 14 crates with the notice has `readme = "README.md"` and `README.md` in its `include` list,
so the
notice ships in the packed crate.
`forbidden-regex` and `forbidden-strings` have no notice,
by the owner's rule.
Their older versions are already yanked (all but the newest of each);
that is a fact about crates.io now,
not a step of this plan.

## The dependency graph and the publish order it forces

Read with `cargo metadata --no-deps --offline` on every `Cargo.toml` of the repository (path dependencies with their
`=` requirements;
the dev-dependency and target tables hold none).
No crate outside the 16 depends on one of them except the sidecar crates named under "Pins that move".

- No internal dependency:
  `monochromatic-jsonc-edit`,
  `monochromatic-xml-edit`,
  `monochromatic-browserslist`,
  `monochromatic-dependency-catalog`,
  `monochromatic-search-shadow`,
  `monochromatic-nested-wayland-session`,
  `forbidden-regex`.
- Depend on `monochromatic-jsonc-edit` only:
  `monochromatic-catalog-specifier`,
  `monochromatic-file-enforcer`,
  `monochromatic-lint`,
  `monochromatic-mpm-shadow`.
- Depends on `monochromatic-dependency-catalog` only:
  `monochromatic-cargo-shadow`.
- Depends on `forbidden-regex` only (normal and build dependency):
  `forbidden-strings`.
- Depends on `monochromatic-catalog-specifier` only:
  `monochromatic-pnpm-shadow`.
- Depends on four:
  `monochromatic-file-enforcer-cli` uses `monochromatic-file-enforcer`,
  `monochromatic-dependency-catalog`,
  `monochromatic-browserslist` and `monochromatic-xml-edit`
  (it reaches `monochromatic-jsonc-edit` through `monochromatic-file-enforcer`).
- Depends on four:
  `monochromatic-git-policy-cli` uses `monochromatic-jsonc-edit`,
  `monochromatic-catalog-specifier`,
  `monochromatic-lint` and `forbidden-strings`.

crates.io refuses a crate whose dependency it does not hold,
and `cargo publish --dry-run` does not catch that
(`doc/planning/crates-io-footprint.md`,
"Research results"),
so a dependency must be on crates.io before its dependents.
That gives three waves,
with the optional exempt crates in brackets:

- Wave 0:
  `monochromatic-jsonc-edit`,
  `monochromatic-dependency-catalog`,
  `monochromatic-browserslist`,
  `monochromatic-xml-edit`,
  `monochromatic-search-shadow`,
  `monochromatic-nested-wayland-session`,
  [`forbidden-regex`].
- Wave 1:
  `monochromatic-catalog-specifier`,
  `monochromatic-file-enforcer`,
  `monochromatic-lint`,
  `monochromatic-mpm-shadow`,
  `monochromatic-cargo-shadow`,
  [`forbidden-strings`].
- Wave 2:
  `monochromatic-pnpm-shadow`,
  `monochromatic-file-enforcer-cli`,
  `monochromatic-git-policy-cli`.

`cargo-publish.yml` orders only some of this inside one run:
`fe-publish-crate` needs `je-publish-crate`;
`fec-publish-crate` needs the jobs of jsonc-edit,
xml-edit,
dependency-catalog,
browserslist and file-enforcer;
`gpc-publish-crate` polls the crates.io index for the
`forbidden-strings` version it requires for up to thirty minutes.
The blocks of `monochromatic-lint`,
`monochromatic-mpm-shadow`,
`monochromatic-catalog-specifier`,
`monochromatic-pnpm-shadow` and `monochromatic-cargo-shadow` have no such wait,
and `gpc-publish-crate` does not wait for
`monochromatic-jsonc-edit`,
`monochromatic-catalog-specifier` or `monochromatic-lint`.
So the waves are enforced by this plan's order of dispatches,
not by the workflow.

## Pins that move when a dependency gets its new version

Tool-measured for `monochromatic-jsonc-edit`:
in a throwaway worktree of `17eff86b35` its `version` was changed to
0.1.3 and nothing else;
`file-enforcer check` then reported rule `rust-catalog` drift in exactly these eight manifests
(the repository rule writes each internal path dependency's `version` as `=<target package.version>`,
`doc/decision/rust-dependency-catalog.md`),
plus one unrelated,
pre-existing local drift in
`.config/mise/conf.d/10-bin.local.toml`:

- `package/catalog-specifier/core-rust/Cargo.toml`
- `package/file-enforcer/core-rust/Cargo.toml`
- `package/git-policy/cli/Cargo.toml`
- `package/git-policy/cli.fuzz/Cargo.toml`
- `package/jsonc-edit/core-rust.fuzz/Cargo.toml` (multi-line `[dependencies.monochromatic-jsonc-edit]` table)
- `package/linter/monochromatic-lint/Cargo.toml`
- `package/linter/monochromatic-lint.fuzz/Cargo.toml`
- `package/mpm-shadow/cli/Cargo.toml`

For the other crates the pin sets below come from the `cargo metadata` graph and the same rule,
not from a second
`file-enforcer check` (`file-enforcer write` was never run):

- `monochromatic-xml-edit`:
  `package/file-enforcer/cli/Cargo.toml`,
  and `package/xml-edit/core-rust.fuzz/Cargo.toml`
  (multi-line table).
- `monochromatic-browserslist`:
  `package/file-enforcer/cli/Cargo.toml`.
- `monochromatic-dependency-catalog`:
  `package/cargo-shadow/cli/Cargo.toml`,
  `package/file-enforcer/cli/Cargo.toml`.
- `monochromatic-catalog-specifier`:
  `package/git-policy/cli/Cargo.toml`,
  `package/pnpm-shadow/cli/Cargo.toml`.
- `monochromatic-file-enforcer`:
  `package/file-enforcer/cli/Cargo.toml`.
- `monochromatic-lint`:
  `package/git-policy/cli/Cargo.toml`,
  `package/git-policy/cli.fuzz/Cargo.toml`,
  `package/linter/monochromatic-lint.fuzz/Cargo.toml`.
- `forbidden-regex` (only if released):
  `package/forbidden-strings/cli/Cargo.toml` (two lines,
  `[dependencies]` and
  `[build-dependencies]`),
  `package/forbidden-strings/core.bench/Cargo.toml`,
  `package/forbidden-strings/core.fuzz/Cargo.toml` (multi-line table).
- `forbidden-strings` (only if released):
  `package/git-policy/cli/Cargo.toml`,
  `package/forbidden-strings/cli.fuzz/Cargo.toml`,
  `package/forbidden-strings/core.bench/Cargo.toml`.
- `monochromatic-git-policy-cli`:
  `package/git-policy/cli.fuzz/Cargo.toml`.
- `monochromatic-nested-wayland-session`,
  `monochromatic-search-shadow`,
  `monochromatic-mpm-shadow`,
  `monochromatic-cargo-shadow`,
  `monochromatic-pnpm-shadow`,
  `monochromatic-file-enforcer-cli`:
  no manifest pins them.

Each manifest above is its own Cargo workspace with a tracked `Cargo.lock` that records the version of every path
member,
so a version change also changes the lockfiles that name the crate.
From a scan of every tracked `Cargo.lock`,
the lockfiles to refresh per crate are the crate's own directory and:

- `monochromatic-jsonc-edit`:
  `catalog-specifier/core-rust`,
  `file-enforcer/cli`,
  `file-enforcer/core-rust`,
  `git-policy/cli`,
  `git-policy/cli.fuzz`,
  `jsonc-edit/core-rust.fuzz`,
  `linter/monochromatic-lint`,
  `linter/monochromatic-lint.fuzz`,
  `mpm-shadow/cli`,
  `pnpm-shadow/cli` (all under `package/`).
- `monochromatic-xml-edit`:
  `file-enforcer/cli`,
  `xml-edit/core-rust.fuzz`.
- `monochromatic-browserslist`:
  `file-enforcer/cli`.
- `monochromatic-dependency-catalog`:
  `cargo-shadow/cli`,
  `file-enforcer/cli`.
- `monochromatic-catalog-specifier`:
  `git-policy/cli`,
  `git-policy/cli.fuzz`,
  `pnpm-shadow/cli`.
- `monochromatic-file-enforcer`:
  `file-enforcer/cli`.
- `monochromatic-lint`:
  `git-policy/cli`,
  `git-policy/cli.fuzz`,
  `linter/monochromatic-lint.fuzz`.
- `monochromatic-git-policy-cli`:
  `git-policy/cli.fuzz`.
- `forbidden-regex`:
  `forbidden-strings/cli`,
  `forbidden-strings/cli.fuzz`,
  `forbidden-strings/core.bench`,
  `forbidden-strings/core.fuzz`,
  `git-policy/cli`,
  `git-policy/cli.fuzz`.
- `forbidden-strings`:
  `forbidden-strings/cli.fuzz`,
  `forbidden-strings/core.bench`,
  `git-policy/cli`,
  `git-policy/cli.fuzz`.

Measured consequence of a missed pin:
with `monochromatic-jsonc-edit` at 0.1.3 and the pin in
`package/git-policy/cli.fuzz/Cargo.toml` left at `=0.1.2`,
`cargo update --workspace --offline` in
`package/git-policy/cli.fuzz` failed with "failed to select a version for the requirement
`monochromatic-jsonc-edit = "=0.1.2"` ... candidate versions found which didn't match:
0.1.3".
So the pins must move before any lockfile is refreshed,
in every dependent,
sidecars included.

## How each crate is released today

`package/crate-release/cli/releasable-crates.json` lists four crates:

- `forbidden-strings` (`pinFiles`:
  the wrapper,
  `cli.fuzz`,
  `core.bench`;
  `lockDirs`:
  `git-policy/cli.fuzz`).
- `monochromatic-cargo-shadow` (no pins,
  no lock directories).
- `monochromatic-git-policy-cli` (`pinFiles` and `lockDirs`:
  `git-policy/cli.fuzz`).
- `monochromatic-mpm-shadow` (no pins,
  no lock directories).

Each has a `crate` choice in `.github/workflows/crate-release.yml` and `release` / `release:wait` tasks in its package
`mise.toml` (the wrapper's are `native:release` and `native:release:wait`).
`crate-release.yml` has run for real on 2026-11-04 (`gh run list --workflow=crate-release.yml`:
the four newest
listed runs,
2026-11-04T19:47:40Z to 2026-11-04T21:56:42Z,
concluded `success`).
It moves only the crate's own version and the pins named in `pinFiles`;
it does not move the pins of the crate's
dependencies,
so the dependencies' new versions must already be pinned on `main` before it runs.

The other twelve have no registry entry,
no `crate-release.yml` choice and no `release` task:
`monochromatic-jsonc-edit`,
`monochromatic-xml-edit`,
`monochromatic-browserslist`,
`monochromatic-dependency-catalog`,
`monochromatic-catalog-specifier`,
`monochromatic-file-enforcer`,
`monochromatic-lint`,
`monochromatic-search-shadow`,
`monochromatic-nested-wayland-session`,
`monochromatic-pnpm-shadow`,
`monochromatic-file-enforcer-cli`,
`forbidden-regex`.
They are released by a version bump on `main`:
a push that changes the crate's `Cargo.toml` version starts
`cargo-publish.yml`,
and a `workflow_dispatch` with `crate=<name>` publishes whatever version `main` holds.
To go through `crate-release.yml` each would need (`package/crate-release/cli/README.md`,
"The registry"):
a registry entry,
a name in the `crate` choices of `crate-release.yml` (a unit test compares them with the registry
and with `cargo-publish.yml`),
and `release` / `release:wait` tasks like those of `package/forbidden-strings/cli`.

### What `cargo-publish.yml` does

- It has a `detect` and a `publish-crate` job for each of the 16 crates,
  and a `crate` choice for each of the 16.
- Its `push` trigger lists the 16 `Cargo.toml` paths;
  a push publishes every crate whose `version` line differs between
  the pushed head and the previous head,
  all in one run,
  in parallel except for the `needs` named above.
- A push made with the repository's `GITHUB_TOKEN` (the bump of `crate-release.yml`) starts no workflow,
  which is why
  `crate-release.yml` dispatches `cargo-publish.yml` itself.
- A version already on crates.io:
  the publish step of every block except `forbidden-strings` checks
  `https://crates.io/api/v1/crates/<crate>/<version>` first and prints "already on crates.io;
  skipping publish".
  The `forbidden-strings` publish step has no skip:
  only its push path checks the index,
  so a dispatch for a version
  that is already there fails at `cargo publish` (the README's recovery:
  `gh run rerun <run id> --failed` after the
  upload succeeded).
- The tools build and release binaries:
  `forbidden-strings` and `monochromatic-lint` for eight targets including macOS
  and Windows;
  `monochromatic-search-shadow`,
  `monochromatic-mpm-shadow`,
  `monochromatic-pnpm-shadow`,
  `monochromatic-nested-wayland-session`,
  `monochromatic-file-enforcer-cli`,
  `monochromatic-cargo-shadow` and
  `monochromatic-git-policy-cli` for the two Linux gnu targets.
  The seven libraries have no binary and no GitHub release.
- Both workflows use `concurrency` with `cancel-in-progress: false` and no `queue` key.
  GitHub's documentation ("Control the concurrency of workflows and jobs",
  read 2026-11-07):
  by default at most one
  run is pending,
  and "any existing `pending` job or workflow in the same concurrency group will be canceled and the
  new queued job or workflow will take its place".
  So a second dispatch queued behind a running publish waits,
  and a third dispatch silently cancels the second.
  The runbook therefore never starts the next dispatch until the previous run has ended.

## Do `forbidden-strings` and `forbidden-regex` need a release in this batch?

Facts only;
the owner decides (see "Open choices").
The published `.crate` files were downloaded from `https://static.crates.io/crates/<crate>/<crate>-<version>.crate`
and compared with `main`.

- No dependent needs a new version of either.
  `monochromatic-git-policy-cli` pins `forbidden-strings =0.6.1` and `forbidden-strings` pins `forbidden-regex =0.1.1`;
  both versions stay unyanked because the owner exempted these two crates from the yank.
- `forbidden-strings` 0.6.1 (built from `55f3a1634e`):
  no difference in `src` or `build.rs`,
  except two new TypeScript
  files in the crate directory (`src/mise.port-betterleaks.ts`,
  `src/port-betterleaks-relaxations.ts`);
  the README differs (renamed command word and settings-file notes,
  a disk-weight line);
  the manifest differs in two catalog floors (`ignore` 0.4.33 to 0.4.34,
  `memchr` 2.8.2 to 2.8.3).
  Commits to its directory since:
  `31f3223d25`,
  `0c840d0e17`,
  `afb215d030`,
  `24eda66cef`,
  `a683ef5986`,
  `581190711e`.
- `forbidden-regex` 0.1.1 (built from `43404709a8`,
  2026-09-16T23:31:03-04:00):
  47 files under `src` differ;
  85 changed lines are not comments,
  and they are renames of shadowed variables,
  test constants,
  and
  `Node::Repeat { node: body, ... }` patterns;
  no changed line declares a `pub` item.
  The README differs.
  The published `Cargo.toml.orig` points `homepage` at `.../tree/main/package/rust-module/forbidden-regex`,
  a path that
  does not exist on `main` any more (the crate lives at `package/forbidden-strings/core`).
  The manifest also moved to workspace-inherited dependencies with the same floors in the catalog.
- Neither README has the notice,
  by the owner's rule.

## Versions

Plain numbers (the batch counts as stable under the label rule),
next patch each,
nothing forcing more:
exact `=` pins make semver compatibility between our own crates irrelevant,
and no step of the tooling requires a minor.

- Wave 0:
  `monochromatic-jsonc-edit` 0.1.2 to 0.1.3;
  `monochromatic-dependency-catalog` 0.2.0 to 0.2.1;
  `monochromatic-browserslist` 0.1.0 to 0.1.1;
  `monochromatic-xml-edit` 0.1.0 to 0.1.1;
  `monochromatic-search-shadow` 0.2.1 to 0.2.2;
  `monochromatic-nested-wayland-session` 0.6.2 to 0.6.3;
  [`forbidden-regex` 0.1.1 to 0.1.2].
- Wave 1:
  `monochromatic-catalog-specifier` 0.1.0 to 0.1.1;
  `monochromatic-file-enforcer` 0.1.1 to 0.1.2;
  `monochromatic-lint` 0.5.0 to 0.5.1;
  `monochromatic-mpm-shadow` 0.2.5 to 0.2.6;
  `monochromatic-cargo-shadow` 0.1.3 to 0.1.4;
  [`forbidden-strings` 0.6.1 to 0.6.2].
- Wave 2:
  `monochromatic-pnpm-shadow` 0.1.1 to 0.1.2;
  `monochromatic-file-enforcer-cli` 0.1.2 to 0.1.3;
  `monochromatic-git-policy-cli` 0.5.34 to 0.5.35.

Flag for the owner:
the 0.5.34 manifest (`0b5abc6fcb`) names the binary `cli-git-native` (and a second binary
`cli-git-service`);
`main` names one binary,
`monochromatic-git-policy-cli`.
So 0.5.35 renames the installed program and changes what the binstall archive holds
(`bin-dir = "{ name }-{ version }-{ target }/{ bin }{ binary-ext }"`).
Under 0.x semver that is a minor step,
0.6.0.
The plan uses 0.5.35 under the rule "next patch unless something forces more";
nothing in the tooling forces 0.6.0,
and
the old command word keeps working with a notice,
but the choice is listed under "Open choices".

## Known hazards

- `package/git-policy/cli.fuzz` does not compile on `main`.
  Evidence:
  `cargo check --lib --offline --locked` there (throwaway worktree,
  capped) ended with
  `error[E0432]: unresolved import git_policy_cli::owner_lock_record` at `src/owners.rs:13`;
  `git grep owner_lock_record -- package/git-policy` finds no module of that name in the library
  (only `src/owners.rs` and `bin/planted-table.ts` in the fuzz crate,
  and merge-corpus fixtures).
  It does not block the release:
  the release command runs `cargo check` only in the crate directory
  (`monochromatic-git-policy-cli`,
  `--bin monochromatic-git-policy-cli`) and `cargo update --workspace` in the lock
  directories,
  which resolves and compiles nothing;
  `cargo update --workspace --offline` in `cli.fuzz` on the unchanged
  tree exited 0 ("Locking 0 packages").
  It is still a `lockDirs` entry and a pin file of the wrapper and of `forbidden-strings`,
  so its manifest must
  satisfy the pins (see the measured failure above).
  Repairing the import is separate work and does not gate this batch.
- Pins that the release tool could not edit (fixed on `main`,
  see "What exists now" under "Preparatory steps").
  `package/crate-release/cli/src/manifest-edit.ts` matched a dependency only as an inline table on one line
  (`name = { ..., version = "=x" }`);
  three sidecars use a multi-line `[dependencies.<crate>]` table:
  `package/jsonc-edit/core-rust.fuzz`,
  `package/xml-edit/core-rust.fuzz`,
  `package/forbidden-strings/core.fuzz`.
  The tool now moves both forms.
- Missing wait on dependencies.
  A dependent dispatched before its dependency is in the crates.io index fails at `cargo package` (verification builds
  against crates.io).
  Recovery is to wait for the index and dispatch again;
  nothing is half-published.
- Dispatch queue.
  A third queued run replaces the pending second (see "What `cargo-publish.yml` does").
  Never dispatch before the previous run ended.
- `monochromatic-nested-wayland-session` may not build on the floating nightly.
  `d1a9c90cc2` records in `doc/handover/cli-git-rust-implementation.md` that `zbus` 5.19.0 fails on
  `nightly-2026-11-01` (`E0391: cycle detected ... Child needs async drop`) and compiles on `nightly-2026-10-28`;
  `package/nested-wayland-session/cli/Cargo.lock` locks `zbus` 5.19.0 (no other publishable crate does).
  `cargo-publish.yml` installs `rustup toolchain install nightly`,
  the floating channel,
  so the result on the dispatch
  day is unknown.
  The crate is a leaf with no dependent,
  so it goes last in wave 0 and a failure blocks nothing else;
  run its dry run first (`dry-run=true` also builds every target for this crate).
- Windows and macOS legs.
  `forbidden-strings` and `monochromatic-lint` build eight targets including macOS and Windows with `fail-fast: false`,
  and `ml-create-release` needs `ml-build-binary` as a whole,
  so a failing macOS or Windows leg skips the GitHub
  release of `monochromatic-lint` (the owner said to ignore these two platforms;
  the workflow does not).
  Both previous releases (`forbidden-strings-v0.6.1`,
  `monochromatic-lint-v0.5.0`) hold all eight assets,
  so the legs
  passed on 2026-11-01.
  `ml-create-release` and the `forbidden-strings` `create-release` do not wait for the crate publish,
  so a release can
  exist (immutable) for a version whose crate upload failed;
  a re-dispatch then fails at `gh release create` on the
  existing tag after the crate uploaded.
- The seven tools named in `toolchain.lock.json` by exact version (`monochromatic-lint` 0.5.0,
  `monochromatic-git-policy-cli` 0.5.34,
  `monochromatic-cargo-shadow` 0.1.3,
  `monochromatic-pnpm-shadow` 0.1.1,
  `monochromatic-search-shadow` 0.2.1,
  `monochromatic-file-enforcer-cli` 0.1.2,
  `monochromatic-nested-wayland-session` 0.6.2,
  all `cargo-binstall`,
  no checksums) cannot be installed fresh once the
  version is yanked (`doc/planning/crate-release-yank-after-publish.md`,
  "What yanking does to installs").
  The yank right after the batch therefore opens a window in which a fresh install of the current lock fails
  (other worktrees,
  CI on an older commit);
  see "Open choices".
- Rate limits.
  crates.io,
  "Publishing Rate Limits" (read 2026-11-07):
  new versions of existing crates,
  a burst of 30,
  then one a
  minute.
  The batch is 14 versions (16 with the exempt pair),
  inside the burst,
  if nothing else is published in the hour.
  Yanks:
  a burst of 100,
  then one a minute (`doc/planning/crate-release-yank-after-publish.md`,
  "Rate limits met").
- A defective new version cannot be replaced.
  A second release would break "one release each";
  ask the owner before publishing any fix.

## What this batch must leave possible

For the later move of the seven tools to the `archive` route,
as `mpm-shadow` already is:

- Each tool's new GitHub release must exist with the Linux assets (`<crate>-<version>-x86_64-unknown-linux-gnu.tar.gz`
  and the aarch64 one,
  tag `<crate>-v<version>`);
  `release:wait` and `release:wait:commit` check these by default.
  Releases are immutable;
  none is deleted or replaced.
- `monochromatic-nested-wayland-session` also ships a checksums file;
  the lock needs a SHA-256 for every archive
  and records them from the downloaded files.
- The wrapper's 0.5.35 archive holds `monochromatic-git-policy-cli`,
  not `cli-git-native`;
  the lock's `bins` and `options.postinstall.link.sourceBin`,
  the root `mise.toml` task and the link folder change with
  the lock (`doc/planning/git-policy-cli-program-rename.md`,
  "Edits that wait for the release (step 3)").
- The yank of the seven locked versions waits on the lock move or accepts the window (see "Open choices").
- `monochromatic-mpm-shadow` already installs from its archive;
  yanking its 0.2.5 on crates.io does not affect it.

## The owner's answers at 2026-11-07

Picked in the question tool,
verbatim labels:

- The two exempt crates:
  "Both".
  `forbidden-regex` 0.1.1 to 0.1.2 joins wave 0 and `forbidden-strings` 0.6.1 to 0.6.2 joins wave 1;
  both stay exempt from the notice and from the yank,
  and the wrapper's pin moves to `forbidden-strings` 0.6.2.
- Yank timing:
  "All after the lock moves".
  No old version is yanked until the toolchain lock points at the new versions.
- Wrapper version:
  "0.6.0 (Recommended)".
  The rename breaks every user of the old names,
  so the middle number moves.
- Route for the twelve unregistered crates:
  "Register all twelve".
  Before the batch,
  each of the twelve gets a registry entry in `package/crate-release/cli/releasable-crates.json`,
  a `crate` choice in `crate-release.yml`,
  and package `release` and `release:wait` tasks,
  and the release tool learns to move the pins in the three multi-line `[dependencies.X]` sidecar manifests
  (`jsonc-edit/core-rust.fuzz`,
  `xml-edit/core-rust.fuzz`,
  `forbidden-strings/core.fuzz`).
  Every crate of the batch then goes through `crate-release.yml`,
  one run at a time.

The sections below keep the options as they were offered.

## Open choices for the owner

### Whether the two exempt crates get a release

- A.
  Neither.
  Pros:
  nothing depends on a new version;
  fewest crates.io versions,
  as "we publish less" asks;
  no Windows or macOS
  build of `forbidden-strings` to run;
  no pin changes in the wrapper,
  `cli.fuzz`,
  `core.bench`.
  Cons:
  "each of our published crates" is read as fewer than all;
  the stale `homepage` of `forbidden-regex` stays on
  crates.io;
  the `forbidden-strings` README renames stay unpublished.
- B.
  Only `forbidden-regex` 0.1.2.
  Pros:
  fixes the dead `homepage` and `repository` path on crates.io.
  Cons:
  its dependents then need new pins (`forbidden-strings`,
  `core.bench`,
  `core.fuzz`),
  and `forbidden-strings`
  would be out of step unless released too;
  a behaviour-free refactor ships.
- C.
  Both (`forbidden-regex` 0.1.2 in wave 0,
  `forbidden-strings` 0.6.2 in wave 1,
  with the wrapper's pin moving to it).
  Pros:
  matches "each of our published crates" literally;
  the `forbidden-strings` README renames and the
  `forbidden-regex` metadata fix go out.
  Cons:
  two more versions and a `forbidden-strings` binary matrix with Windows and macOS;
  the `forbidden-strings`
  dispatch has no skip for an existing version,
  so a failed run is re-run,
  never re-dispatched.
- Ranking:
  A > C > B.
  A beats C because no dependent needs either release and the owner's stated direction is to publish less;
  C beats B because B leaves `forbidden-strings` pinned to a regex crate with a new version nobody uses,
  while C at least keeps the pair and its dependents consistent and meets the literal "each".

### When the seven locked tool versions are yanked

- A.
  Yank everything right after the batch,
  as the owner's rule says.
  Pros:
  matches the rule;
  one pass.
  Cons:
  a fresh install of the current `toolchain.lock.json` fails until the lock names new versions.
- B.
  Yank the seven libraries and `monochromatic-mpm-shadow` right after the batch;
  yank the seven locked tools in the
  same session once the lock names the new versions (or the `archive` route).
  Pros:
  nothing that a lock names is hidden while the lock still names it;
  the rule still holds inside the session.
  Cons:
  seven yanks wait on lock work,
  so "right after" is not literal.
- C.
  Move the lock first,
  then yank all.
  Pros:
  no window.
  Cons:
  the lock work (rename of the program,
  `archive` route) is large and the yank waits on it.
- Ranking:
  B > A > C.
  B beats A because it removes the install failure while keeping the yank inside the same work session;
  A beats C because the owner's rule says "right after",
  and C could delay it for the duration of the route change.

### Wrapper version 0.5.35 or 0.6.0

- A.
  0.5.35:
  the plan's default;
  the old command word still works.
- B.
  0.6.0:
  marks the program rename as the breaking step it is under 0.x semver.
- Ranking:
  A > B.
  A beats B because exact pins make the number irrelevant to resolution,
  the owner's rule is the next patch unless
  forced,
  and 0.6.0 would be a number chosen for readers who are told not to use the crate at all.

### How the twelve unregistered crates are released

- A.
  As they are:
  a bump commit per wave,
  published by the push trigger of `cargo-publish.yml` (this plan's runbook).
  Pros:
  no change to the release tooling;
  it is the path these crates have used so far.
  Cons:
  no CI check before the bump lands on `main`,
  no dry run of the publish with the bump in place,
  no signed
  commit by the workflow;
  the three multi-line sidecar pins are handled by `file-enforcer write`.
- B.
  Register all twelve first (entry,
  `crate` choice,
  two tasks each) and convert the three multi-line pins.
  Pros:
  one release path,
  with `--dry-run` and the check on a runner.
  Cons:
  a large preparatory change for a one-time batch,
  a registry unit test to satisfy,
  and a pin file that the tool
  cannot edit unless converted.
- C.
  Register only the five tools that publish binaries (`monochromatic-search-shadow`,
  `monochromatic-nested-wayland-session`,
  `monochromatic-pnpm-shadow`,
  `monochromatic-file-enforcer-cli`,
  `monochromatic-lint`).
  Pros:
  the crates with release assets get the checked path;
  no multi-line pins involved.
  Cons:
  still new entries and tasks;
  the libraries stay manual.
- Ranking:
  A > C > B.
  A beats C because the batch is one-time and A needs no tooling change;
  C beats B because it covers the crates whose
  releases matter for the archive route without touching the three multi-line pins.

## Preparatory steps

Separate work,
not part of this plan's commit,
and done before the first dispatch:

1.  The owner's answers are recorded under "The owner's answers at 2026-11-07":
    the exempt pair is released,
    the wrapper goes to 0.6.0,
    the yank waits until the lock moves,
    and all twelve unregistered crates go through `crate-release.yml`.
    The runbook below was written for route A (a bump commit per wave);
    the registered route replaces its "Per wave" procedure and its steps 3,
    4 and 6 with one dispatch per crate
    (see "What exists now"),
    and the version steps follow the owner's answers.
2.  Confirm `gh auth status` works and that Cargo's credentials file holds a token with the yank scope
    (the owner logged in with publish-new,
    trusted-publishing and yank scopes on 2026-11-04).
    Check only that `cargo yank --help` runs and that the file exists;
    never read or print the token.
3.  Run `file-enforcer check` on a clean `main` worktree and confirm its only drift is the local
    `.config/mise/conf.d/10-bin.local.toml`;
    any other drift is fixed first,
    so a release commit shows only its own changes.
4.  Done:
    the registry entries,
    the `crate` choices of `crate-release.yml`,
    the package tasks and the multi-line pin edit (see "What exists now").
5.  Optional,
    not blocking:
    repair the `owner_lock_record` import of `package/git-policy/cli.fuzz/src/owners.rs`.

### What exists now

Written 2026-11-07T05:19:18-05:00,
on the branch `feat/crate-release-register-every-published-crate`.

- `package/crate-release/cli/releasable-crates.json` lists all sixteen crates.
  The twelve new entries hold `crateDir`,
  `scope`,
  `pinFiles` (every manifest that pins the crate exactly,
  sidecars included),
  `checkBins` (empty for the libraries and `forbidden-regex`,
  the real `[[bin]]` name for the five tools) and
  `lockDirs` (every directory whose `Cargo.lock` holds the crate as a path package,
  except the crate's own).
  The pin and lock sets were found by scanning every tracked `Cargo.toml` and `Cargo.lock` and agree with the sets
  under "Pins that move when a dependency gets its new version".
- `crate-release.yml` offers all sixteen crates in the registry's order;
  `cargo-publish.yml` already offered all sixteen.
  The release job installs the Wayland build libraries for `monochromatic-nested-wayland-session` only,
  because the release command builds the crate on the runner and `cargo-publish.yml` installs the same set for it.
  This was missing before and would have failed that crate's release at `cargo check`.
- Every new crate's package has `release` and `release:wait` tasks
  (`mise run //package/<path>:release -- [--dry-run] <from> <to> "<note>"`).
  A library has no GitHub release,
  so its `release:wait` ends with `PUBLISH CHECK FAILED: missing release assets: ...` even when the crate is on
  crates.io;
  read its `crates.io <version>: <time>` line as the result.
- The release tool moves a pin written as a `[dependencies.<crate>]`,
  `[dev-dependencies.<crate>]` or `[build-dependencies.<crate>]` table as well as the inline form.
  Applying the tool's own edit to all sixteen crates at once (each one patch up) left `file-enforcer check` with only
  the local `10-bin.local.toml` drift;
  the same check reports drift when one table pin is put back (positive control).
- The release commit now includes the `Cargo.lock` of every lock directory.
  Before,
  only the lockfiles beside the crate and beside the pin files were committed,
  so a directory that reaches the crate through another crate (for example `package/file-enforcer/cli` for
  `monochromatic-jsonc-edit`) was refreshed on the runner and then left out of the commit.
- The existing `forbidden-strings` entry listed one lock directory (`git-policy/cli.fuzz`);
  the scan finds four (`forbidden-strings/cli.fuzz`,
  `forbidden-strings/core.bench`,
  `git-policy/cli`,
  `git-policy/cli.fuzz`),
  and the entry now lists all four.
  Its 0.6.1 commit `55f3a1634e` changed no lockfile beside the `git-policy/cli`,
  `cli.fuzz` or `core.bench` pins.
  The entries of `monochromatic-cargo-shadow`,
  `monochromatic-git-policy-cli` and `monochromatic-mpm-shadow` already match the scan.
- Proven on a runner without publishing:
  the dry run `https://github.com/Aquaticats/Monochromatic/actions/runs/42363884326` of
  `monochromatic-jsonc-edit` 0.1.2 to 0.1.3 (`mise run //package/jsonc-edit/core-rust:release -- --dry-run ...`,
  merged at `a6104c0f64`) concluded `success` in 51 seconds.
  Its log shows the crate manifest and all eight pin files edited,
  among them the multi-line table of
  `package/jsonc-edit/core-rust.fuzz`,
  then ten `cargo update --workspace` steps (one per lock directory),
  the publish dry run,
  and `RELEASE COMMITTED` at 2026-11-07T10:26:13Z;
  the push and publish jobs were skipped,
  as a dry run requires.
  The dry run cannot show two things:
  the push step (the watch set of `monochromatic-jsonc-edit` is the `cargo metadata` closure of ten lock directories,
  which reaches most Rust code of the repository,
  so its real release is the one most likely to be refused for a moved
  `main` and need a second dispatch;
  schedule it while no other Rust commits land),
  and the Wayland library step of `monochromatic-nested-wayland-session` (its own dry run,
  the first step of the runbook,
  is the proof).
- The "Per wave" procedure and steps 3,
  4 and 6 of the runbook describe route A and must be rewritten by the
  coordinator as one dispatch per crate in wave order (dry run,
  real run,
  `release:wait`,
  for the libraries read the
  `crates.io` line because their `release:wait` exits 1 on the missing release assets).
  The wave-0 crates have no dependency among them;
  a dependent's dispatch must wait until its dependencies are in the crates.io index,
  and `release:wait` ends only after the publish run,
  so waiting for it is the order guard.

## Runbook

Rules for every step:
one dispatch at a time;
wait for its run to end before the next;
the next wave starts only when
every crate of the previous wave is on crates.io.
Use `cargo yank` and the commands below with the generic user agent `crate-release-plan-check` for reads.
Nothing here puts a token on a command line,
in an environment variable or in output.

### Per wave: the bump commit for the unregistered crates

For wave N,
on a clean checkout of `main`,
in one commit per wave,
with the wave's crates only:

1.  Edit the `version` line of each wave crate's `Cargo.toml` to its new version.
2.  Run `file-enforcer write` from the repository root;
    it moves every dependent's pin (sidecars included).
    Run `file-enforcer check`;
    it must report no drift other than the local `10-bin.local.toml`.
3.  In each lockfile directory listed under "Pins that move" for the wave's crates,
    run
    `cargo update --workspace`;
    then `cargo metadata --locked --format-version 1 --manifest-path <dir>/Cargo.toml`
    must succeed in every one.
    A "failed to select a version" error means a pin was missed:
    stop and fix the pin.
4.  Commit with explicit pathspecs naming every changed `Cargo.toml` and `Cargo.lock`
    (message:
    `chore(*): release wave N crates: <list of crate and version>`).
    The push that follows starts `cargo-publish.yml`,
    which publishes each crate whose version line changed.
5.  Find the run:
    `gh run list --workflow=cargo-publish.yml --event push --limit 5`,
    take the run whose `headSha` is the
    commit;
    `gh run watch <run id> --exit-status`.
6.  For each crate of the wave,
    confirm it landed:
    `curl -sS -A 'crate-release-plan-check' https://crates.io/api/v1/crates/<crate>/<version>` returns the version with
    `"yanked":false`.
    For a tool also confirm the GitHub release:
    `gh release view <crate>-v<version> --json assets`
    lists the Linux tarballs (or run
    `mise run //package/crate-release/cli:release:wait:commit -- --crate-name <crate> <sha> <version>`,
    which does both
    and exits 0 only when all is in place;
    it fails for the libraries,
    which have no GitHub release).

If a crate of the wave failed:
do not start the next wave.
Re-dispatch just that crate:
`gh workflow run cargo-publish.yml --ref main -f crate=<crate> -f dry-run=false`
(a version already on crates.io is skipped;
the exceptions are `forbidden-strings`,
which is re-run with
`gh run rerun <run id> --failed`).
A failure that is not a missing dependency in the index is a defect in that crate:
report it;
do not publish a fix.

### Steps

1.  Preparatory steps above.
2.  Dry run of the leaf that is most likely to break:
    `gh workflow run cargo-publish.yml --ref main
    -f crate=monochromatic-nested-wayland-session -f dry-run=true`,
    watch it (`gh run list --workflow=cargo-publish.yml
    --limit 1`,
    then `gh run watch <id> --exit-status`).
    It builds and verifies every target on the floating nightly.
    If it fails on `zbus` (`E0391`),
    report to the owner and drop the crate from the batch for now;
    nothing else depends on it.
3.  Wave 0 bump commit for `monochromatic-jsonc-edit` 0.1.3,
    `monochromatic-dependency-catalog` 0.2.1,
    `monochromatic-browserslist` 0.1.1,
    `monochromatic-xml-edit` 0.1.1,
    `monochromatic-search-shadow` 0.2.2 and
    `monochromatic-nested-wayland-session` 0.6.3,
    with the per-wave procedure.
    Wait for all six on crates.io,
    and for the GitHub releases of the two tools.
4.  Wave 1,
    manual part:
    bump commit for `monochromatic-catalog-specifier` 0.1.1,
    `monochromatic-file-enforcer` 0.1.2
    and `monochromatic-lint` 0.5.1.
    Wait for the three on crates.io and for the `monochromatic-lint-v0.5.1` release.
5.  Wave 1,
    registered crates,
    one after the other.
    Each dispatch first as a dry run,
    followed with `gh run watch <run URL> --exit-status`:
    - `mise run //package/mpm-shadow/cli:release -- --dry-run 0.2.5 0.2.6 "<one-line note>"`
    - then for real:
      `mise run //package/mpm-shadow/cli:release -- 0.2.5 0.2.6 "<one-line note>"`
      and `mise run //package/mpm-shadow/cli:release:wait -- <run URL or id> 0.2.6`.
    - `mise run //package/cargo-shadow/cli:release -- --dry-run 0.1.3 0.1.4 "<note>"`,
      then without `--dry-run`,
      then `mise run //package/cargo-shadow/cli:release:wait -- <run URL or id> 0.1.4`.
    The wave-0 commit already pinned the new `monochromatic-jsonc-edit` and `monochromatic-dependency-catalog` in these
    two manifests;
      if a dry run fails with "failed to select a version" or "no matching package",
      stop.
6.  Wave 2,
    manual part:
    bump commit for `monochromatic-pnpm-shadow` 0.1.2 and `monochromatic-file-enforcer-cli` 0.1.3.
    Wait for both on crates.io and for their two GitHub releases.
7.  Wave 2,
    the wrapper last:
    `mise run //package/git-policy/cli:native:release -- --dry-run 0.5.34 0.5.35 "<note>"`,
    watch it,
    then without
    `--dry-run`,
    then `mise run //package/git-policy/cli:native:release:wait -- <run URL or id> 0.5.35`.
    The `gpc-publish-crate` job polls for the required `forbidden-strings` version,
    which stays 0.6.1.
    The wave-0 and wave-1 commits already pinned the new `monochromatic-jsonc-edit`,
    `monochromatic-catalog-specifier`
    and `monochromatic-lint`.
8.  Verify the whole batch in one pass:
    for each of the 14 crates the newest unyanked version on crates.io is the
    new one,
    its GitHub release exists where it should,
    and `git -C <repo> status` shows no stray change.

If a registered release fails before its push step,
`main` is as it was:
fix the cause and dispatch the same arguments.
After the push,
follow `package/crate-release/cli/README.md` ("How a release runs"):
start the publish again with
`gh workflow run cargo-publish.yml --ref main -f crate=<crate> -f dry-run=false` when nothing was uploaded,
and
`gh run rerun <run id> --failed` once the crate upload succeeded.
A refused push for a moved `main` that names changed watched paths means another commit touched the crate's closure:
dispatch again.

### Yank step (after step 8, per the chosen option)

For each of these 14 crates,
yank exactly the one older unyanked version (every crate has one):
`monochromatic-jsonc-edit` 0.1.2,
`monochromatic-dependency-catalog` 0.2.0,
`monochromatic-browserslist` 0.1.0,
`monochromatic-xml-edit` 0.1.0,
`monochromatic-search-shadow` 0.2.1,
`monochromatic-nested-wayland-session` 0.6.2,
`monochromatic-catalog-specifier` 0.1.0,
`monochromatic-file-enforcer` 0.1.1,
`monochromatic-lint` 0.5.0,
`monochromatic-mpm-shadow` 0.2.5,
`monochromatic-cargo-shadow` 0.1.3,
`monochromatic-pnpm-shadow` 0.1.1,
`monochromatic-file-enforcer-cli` 0.1.2,
`monochromatic-git-policy-cli` 0.5.34.
Neither `forbidden-strings` nor `forbidden-regex` is yanked.

Command,
one at a time,
from a directory outside every Cargo workspace (for example a scratch directory):
`cargo yank --version <version> <crate>`.
`cargo yank` reads the credentials file itself;
never pass `--token`,
never set a token variable.
Undo with `cargo yank --undo --version <version> <crate>`.
Bare `cargo` on this machine is `cargo-shadow`;
that script called bare `cargo yank` through it,
and `cargo yank --help` passes through it from a directory with no `Cargo.lock` (checked 2026-11-07).
The one-time yank of 2026-11-04 (`yank-older-releases.ts` in the agent scratch directory `issue-6261`) ran one at a
time with retry on 429;
the same retry applies (a minute per yank after a burst,
and an empty-body 429 from the cache in front
of the registry).
After each yank confirm with the API that `yanked` is true for that version and false for the new one.

Under option B of "When the seven locked tool versions are yanked":
yank the seven libraries and
`monochromatic-mpm-shadow` first (`monochromatic-jsonc-edit`,
`monochromatic-dependency-catalog`,
`monochromatic-browserslist`,
`monochromatic-xml-edit`,
`monochromatic-catalog-specifier`,
`monochromatic-file-enforcer`,
`monochromatic-mpm-shadow`),
and the seven locked tools (`monochromatic-lint`,
`monochromatic-git-policy-cli`,
`monochromatic-cargo-shadow`,
`monochromatic-pnpm-shadow`,
`monochromatic-search-shadow`,
`monochromatic-file-enforcer-cli`,
`monochromatic-nested-wayland-session`) once the lock no longer names their old
versions.

### Release record

One entry per crate,
as run through `crate-release.yml`;
the dry run of `monochromatic-nested-wayland-session` 0.6.2 to 0.6.3
(`https://github.com/Aquaticats/Monochromatic/actions/runs/42368894213`,
`success`,
2026-11-07)
proved the Wayland library step.

- `monochromatic-jsonc-edit` 0.1.2 to 0.1.3:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369224009`,
  bump commit `33d20fa630`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369300426`,
  crates.io `2026-11-07T11:43:35.824945Z`,
  `yanked` false.
  The first dispatch pushed without a refusal.
  `release:wait` ended with the expected library line `PUBLISH CHECK FAILED: missing release assets`.

- `monochromatic-dependency-catalog` 0.2.0 to 0.2.1:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369457223`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369535765`
  (built from `a8923376a2`),
  crates.io `2026-11-07T11:47:06.445839Z`,
  `yanked` false.

- `monochromatic-browserslist` 0.1.0 to 0.1.1:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369723100`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369851512`
  (built from `4f224afa10`),
  crates.io `2026-11-07T11:52:01.185390Z`,
  `yanked` false.

- `monochromatic-xml-edit` 0.1.0 to 0.1.1:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42370054129`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42370120031`
  (built from `d6dec5bcac`),
  crates.io `2026-11-07T11:55:20.267501Z`,
  `yanked` false.

- `monochromatic-search-shadow` 0.2.1 to 0.2.2:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42370262671`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42370330409`
  (built from `b3a85f3e8d`),
  crates.io `2026-11-07T11:58:24.564425Z`,
  `yanked` false.
  GitHub release `monochromatic-search-shadow-v0.2.2` holds both Linux tarballs and the checksums file.

- `forbidden-regex` 0.1.1 to 0.1.2:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42370496745`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42370600358`
  (built from `3374eccf8a`),
  crates.io `2026-11-07T12:02:25.921188Z`,
  `yanked` false.
  Exempt crate:
  no README notice;
  note carries the homepage path fix.

- `monochromatic-nested-wayland-session` 0.6.2 to 0.6.3:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42370762186`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42370971740`
  (built from `bc56e10ce7`),
  crates.io `2026-11-07T12:08:46.831419Z`,
  `yanked` false.
  GitHub release `monochromatic-nested-wayland-session-v0.6.3` holds both Linux tarballs and the checksums file.
  Wave 0 is complete on crates.io.

- `monochromatic-catalog-specifier` 0.1.0 to 0.1.1:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42371369818`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42371457006`
  (built from `f6239c698a`),
  crates.io `2026-11-07T12:13:51.127758Z`,
  `yanked` false.

- `monochromatic-file-enforcer` 0.1.1 to 0.1.2:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42371620801`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42371722087`
  (built from `f86710023d`),
  crates.io `2026-11-07T12:17:42.698494Z`,
  `yanked` false.
  One dispatch was refused before any run by the origin-holds-main check (another session's local commit was not yet pushed);
  dispatched again after it was pushed.

- `monochromatic-lint` 0.5.0 to 0.5.1:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42371874158`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42372124153`
  (built from `f9a51d4f37`),
  crates.io `2026-11-07T12:24:28.368166Z`,
  `yanked` false.
  GitHub release `monochromatic-lint-v0.5.1` holds all eight target assets.

- `monochromatic-mpm-shadow` 0.2.5 to 0.2.6:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42372933230`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42373065104`
  (built from `9efecbcd39`),
  crates.io `2026-11-07T12:35:39.343192Z`,
  `yanked` false.
  GitHub release `monochromatic-mpm-shadow-v0.2.6` holds both Linux tarballs and the checksums file.

- `monochromatic-cargo-shadow` 0.1.3 to 0.1.4:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42373293103`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42373384574`
  (built from `be098a3f79`),
  crates.io `2026-11-07T12:40:03.706516Z`,
  `yanked` false.
  GitHub release `monochromatic-cargo-shadow-v0.1.4` holds both Linux tarballs and the checksums file.

- `forbidden-strings` 0.6.1 to 0.6.2:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42373581094`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42373820150`
  (built from `f707a66cda`),
  crates.io `2026-11-07T12:48:15.573379Z`,
  `yanked` false.
  Exempt crate:
  no README notice.
  GitHub release `forbidden-strings-v0.6.2` holds all eight target assets.
  Wave 1 is complete on crates.io.

- `monochromatic-pnpm-shadow` 0.1.1 to 0.1.2:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42374326862`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42374408890`
  (built from `0172aface6`),
  crates.io `2026-11-07T12:54:08.727833Z`,
  `yanked` false.
  GitHub release `monochromatic-pnpm-shadow-v0.1.2` holds both Linux tarballs and the checksums file.

- `monochromatic-file-enforcer-cli` 0.1.2 to 0.1.3:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42374580075`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42374713783`
  (built from `9d94aab55c`),
  crates.io `2026-11-07T12:58:55.297025Z`,
  `yanked` false.
  GitHub release `monochromatic-file-enforcer-cli-v0.1.3` holds both Linux tarballs and the checksums file.
  Wave 2 (before the wrapper) is complete.

- `monochromatic-git-policy-cli` 0.5.34 to 0.6.0:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42374936121`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42375444159`
  (built from `3cd939e8e9`),
  crates.io `2026-11-07T13:10:36.327011Z`,
  `yanked` false.
  Released after "go wrapper".
  GitHub release `monochromatic-git-policy-cli-v0.6.0` holds both Linux tarballs.
  Nothing was yanked;
  the batch is complete.

- Lock move (2026-11-07T08:24:40-05:00,
  commit `c048f3a566`,
  after the merge `ea2f72c3c1` of `feat/toolchain-always-latest-landing`):
  the seven locked tools moved from the `cargo-binstall` route to the `archive` route of their GitHub releases,
  at 0.5.1,
  0.6.0,
  0.1.4,
  0.1.2,
  0.2.2,
  0.1.3 and 0.6.3 (crate order of "Yank step");
  the yank still waits for the live check below.

### Steps after the batch

Run 2026-11-07T08:20:23-05:00 to 2026-11-07T08:33:17-05:00 on the main checkout,
in the order of the owner's decisions.

- Merge (2026-11-07T08:20:23-05:00,
  0.3 s):
  `feat/toolchain-always-latest-landing` into `main` with `--no-ff` as `ea2f72c3c1`
  (`6e17549cc4` the plan tracks latest,
  `7f1c244e4d` the `bincode-1` catalog);
  no conflict.
- Archive route (`c048f3a566`,
  2026-11-07T08:24:40-05:00):
  in `toolchain.plan.jsonc` the seven tools take
  `"request": "github:Aquaticats/Monochromatic@latest"`,
  `"route": "archive"` and `options.tagPrefix` `monochromatic-<crate>-v`,
  as `mpm-shadow` does;
  the ids keep their `cargo:` spelling because `.github/workflows/catalog-specifiers.yml` and
  `.github/actions/repo-tools-setup/action.yml` read the lock by them.
  The wrapper's `bins` and `postinstall.link.sourceBin` became `monochromatic-git-policy-cli` (the link folder `.cache/cli-git-bin` stays).
  `mise run toolchain:update` wrote the lock (8 tools moved:
  the seven plus `mpm-shadow` 0.2.5 to 0.2.6 by `@latest`);
  every Linux archive SHA-256 in it equals the digest GitHub reports for that release asset.
  The first update attempt (08:21:17,
  69 s) wrote nothing because one lookup failed (`conda:bubblewrap`,
  HTTP 503 from
  `api.anaconda.org`,
  an outage that ended about a minute later);
  the second (08:23:43,
  38 s) wrote the lock.
  The machine-global plan (outside the repository) got the same change for its `cargo:monochromatic-git-policy-cli` row,
  without `linkDir` (the link stays in `~/.local/bin`),
  and `toolchain:update:global -- cargo:monochromatic-git-policy-cli`
  moved that row alone,
  0.5.34 to 0.6.0.
- Install:
  `mise run toolchain:install` 31.7 s (19 installed;
  the six other repository tools in `cargo-monochromatic-*`
  directories with a `.mpm-shadow-installed` marker,
  no compile) and `toolchain:install:global` 17.3 s (12 installed).
  The rehearsal's global install took 14 min 34 s because it compiled the wrapper.
- Wrapper 0.6.0 live:
  `git --version` 12 ms and prints `monochromatic-git-policy-cli 0.6.0, shadowing git at /usr/bin/git`;
  `git monochromatic-git-policy-cli doctor` 25 ms.
  The commit `5fa212ddac` (this record's lock-move entry) landed through the new unit
  `monochromatic-git-policy-cli@fc85fdf7a58f866b7f7a84b44fe0b69a` (release 0.6.0,
  protocol 7) in 2.2 s
  (`reachMs` 645 for the first start,
  `answerWaitMs` 51) and was pushed in 2.0 s.
  The old unit `cli-git@fc85fdf7a58f866b7f7a84b44fe0b69a` of the 0.5.34 service was not stopped:
  `package/git-policy/cli/README.md` says the 0.6.0 release "neither stops it nor removes the old units"
  (it keeps running from the 0.5.34 install folder until it exits idle after three hours or someone stops it).
- Yank (2026-11-07T08:28:45-05:00 to 08:29:19-05:00):
  the 14 older versions of "Yank step",
  one at a time with
  `cargo yank --version <v> <crate>` from `/var/home/user/temp/agent/post-release/yank`;
  after each one the crates.io API showed the old
  version `yanked` true and the new one false;
  no 429.
  Afterwards the unyanked versions are the new one of each of the 14,
  and `forbidden-regex` 0.1.2 and 0.1.1,
  `forbidden-strings` 0.6.2 and 0.6.1 (exempt).
- Daily timer:
  `mise run toolchain:timer:install` at 2026-11-07T08:29:48-05:00 enabled `toolchain-daily-update.timer`
  (next run 2026-11-08T00:07:05-05:00).
  First run,
  `systemctl --user start --no-block toolchain-daily-update.service`:
  08:30:04 to 08:33:17 (3 min 12.6 s wall,
  1 min 15.3 s CPU,
  1.4 G memory peak),
  result `success`;
  see `doc/planning/toolchain-always-latest.md`,
  "First run".

### Release record: `monochromatic-mpm-shadow` 0.2.7-alpha.1, the first release held back from crates.io (2026-11-07T10:06:13-05:00)

The owner's rule,
verbatim:
"My thinking is to deploy all releases on pnpr,
but only publish rc and stable releases on crates.io";
"Label decides";
"I'm authorizing you to bump and publish any updates this session."
This is the first release run under the label gate (merge `4fd9c70572`,
2026-11-07T09:59:47-05:00),
so its version ends in `-alpha.1` and goes to the GitHub release (and so to the toolchain lock) but not to crates.io.

- Proof runs before it,
  both on the merged gate:
  publish dry run for 0.2.6 (`https://github.com/Aquaticats/Monochromatic/actions/runs/42384236171`,
  `success`;
  decision `label stable, goes to crates.io: true`,
  the package build and the dry run ran) and
  release dry run (`https://github.com/Aquaticats/Monochromatic/actions/runs/42384566055`,
  `success`;
  no publish dry run step).
- Release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42384727529`
  (`workflow_dispatch`,
  2026-11-07T10:06:13-05:00 to 10:07:37-05:00,
  `success`),
  bump commit `ec2e7dbeb1`
  (2026-11-07T10:07:26-05:00),
  version 0.2.6 to 0.2.7-alpha.1.
- Publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42384840600`
  (built from `ec2e7dbeb1`,
  2026-11-07T10:07:34-05:00 to 10:10:13-05:00,
  `success`).
  The `ms-detect` job printed `package/mpm-shadow/cli/Cargo.toml holds version '0.2.7-alpha.1': label alpha, goes to crates.io: false`.
  The crates.io steps of `ms-publish-crate` (package build,
  provenance attestation,
  Trusted Publishing login,
  dry run,
  publish) were skipped;
  the job itself ran,
  so `ms-create-release` was not skipped.
  Every other crate's `-publish-crate`,
  `-build-binary` and `-create-release` job was skipped.
- crates.io does not hold it:
  the public API
  `https://crates.io/api/v1/crates/monochromatic-mpm-shadow/0.2.7-alpha.1` answered HTTP 404
  (`crate ... does not have a version 0.2.7-alpha.1`,
  user agent `crate-release-plan-check`,
  no token),
  and the crate's newest listed version is 0.2.6.
  Nothing was yanked,
  because nothing went to crates.io.
- GitHub release `monochromatic-mpm-shadow-v0.2.7-alpha.1` (published 2026-11-07T10:10:11-05:00):
  `isPrerelease` false;
  assets are the aarch64 and x86_64 Linux tarballs and
  `monochromatic-mpm-shadow-0.2.7-alpha.1-sha256sums.txt`.
  (The release is not marked a pre-release,
  so `@latest` resolves to it;
  that is what moved the lock.)
- Lock move (commit `fb7e05141c`,
  2026-11-07T10:19:16-05:00):
  `mise run toolchain:update` moved `mpm-shadow` 0.2.6 to 0.2.7-alpha.1 and nothing else (`llama.cpp` stays pinned);
  the two archive SHA-256 values in the lock equal the release's checksums file.
  `mise run toolchain:install` fetched and verified the archive,
  removed the older 0.2.6 copy,
  and reported no other tool changed;
  `mpm-shadow --version` prints `mpm-shadow 0.2.7-alpha.1`.
- Copy check of the fix this release carries (links of the source checkout survive an install in a copy):
  main's four link folders (`.cache/monochromatic-git-policy-cli-bin`,
  `mpm-shadow-bin`,
  `cargo-shadow-bin`,
  `pnpm-shadow-bin`)
  were recorded (links,
  targets,
  modification times) before a plain `git worktree add --detach <path> main`,
  after it,
  after `mise run toolchain:install` inside the copy,
  and after `git worktree remove --force`;
  all four records were byte-identical to the first.
  The copy's own links pointed at the copy's own `.cache/toolchain-launchers`.
  A first attempt put the worktree under `~/temp/agent`,
  which is on another filesystem,
  so the wrapper copied the ignored files
  instead of making a snapshot (tens of gigabytes staged);
  it was stopped by PID after about seven minutes
  and its worktree and staging folder were removed.
  Main's links were identical after that attempt as well.
  The second worktree,
  under `.claude/worktrees` on main's filesystem,
  was a snapshot (removal took 16.4 s).
  Evidence files:
  `~/temp/agent/release-0-2-7-alpha-1/links-before.txt`,
  `links-after.txt`,
  `links-after-install.txt`,
  `links-after-remove.txt`.

### Release record: `monochromatic-mpm-shadow` 0.2.7-alpha.2, the install read-back (2026-11-07T11:19:01-05:00)

The owner's rules,
verbatim:
"My thinking is to deploy all releases on pnpr,
but only publish rc and stable releases on crates.io";
"Label decides";
"I'm authorizing you to bump and publish any updates this session."
The second run under the label gate,
again an alpha:
it goes to the GitHub release and the toolchain lock,
not to crates.io.

- The change it carries (branch `fix/mpm-shadow-cli-install-reobserves-and-plan-fields`,
  test commit `496b2e4bbf` 2026-11-07T11:14:10-05:00,
  fix commit `ed9e78b79a` 2026-11-07T11:18:09-05:00,
  merge `fc27f0bbc4` 2026-11-07T11:18:21-05:00):
  after `pnpm add --global`,
  `install` reads the package back from the `pnpm list --global --depth=0 --json` listing that
  `list` uses,
  and a package that is absent or listed at another version fails the install,
  naming the package,
  the locked version and what pnpm lists.
  The test commit failed on the three new read-back tests before the fix (the positive control among them) and
  all of the package's tests pass after it.
- Release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42390673663`
  (`workflow_dispatch`,
  2026-11-07T11:19:01-05:00 to 11:20:47-05:00,
  `success`),
  bump commit `b6ddfcfe7e`
  (2026-11-07T11:20:35-05:00),
  version 0.2.7-alpha.1 to 0.2.7-alpha.2.
  The dispatch printed no refusal,
  so origin held every local commit.
- Publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42390823090`
  (built from `caf8b050b4`,
  a merge of origin's main that holds the bump commit,
  2026-11-07T11:20:44-05:00 to 11:23:23-05:00,
  `success`).
  The `ms-detect` job printed `package/mpm-shadow/cli/Cargo.toml holds version '0.2.7-alpha.2': label alpha, goes to crates.io: false`.
  In `ms-publish-crate` the steps "Build .crate package",
  "Attest .crate provenance",
  "Authenticate with crates.io",
  "Publish (dry run)" and "Publish" were `skipped`,
  and the job itself ran,
  so `ms-create-release` was not skipped.
- crates.io does not hold it:
  `https://crates.io/api/v1/crates/monochromatic-mpm-shadow/0.2.7-alpha.2` answered HTTP 404 (user agent
  `crate-release-plan-check`,
  no token),
  and the crate's newest version is still 0.2.6.
  The wait task said `crates.io 0.2.7-alpha.2: skipped, the alpha label is not published to crates.io`.
- GitHub release `monochromatic-mpm-shadow-v0.2.7-alpha.2` (published 2026-11-07T11:23:21-05:00):
  `isPrerelease` false,
  `isDraft` false;
  assets are the aarch64 and x86_64 Linux tarballs and
  `monochromatic-mpm-shadow-0.2.7-alpha.2-sha256sums.txt`.
- Lock move (commit `d48fb5cc36`,
  2026-11-07T11:25:37-05:00):
  `mise run toolchain:update` moved `mpm-shadow` 0.2.7-alpha.1 to
  0.2.7-alpha.2 and nothing else (`llama.cpp` stays pinned);
  the two archive SHA-256 values in the lock equal the release's
  checksums file.
  `mise run toolchain:install` fetched and verified the archive,
  removed the older copy,
  and reported no failure;
  `mpm-shadow --version` prints `mpm-shadow 0.2.7-alpha.2`.
- Machine-global toolchain,
  repaired by one `mise run toolchain:install:global` run with that binary
  (`toolchain install: 11 installed, 51 unchanged, 0 failed`):
  before,
  `mise run toolchain:list:global` printed `toolchain list: 11 of 62 tools drifted from toolchain.lock.json`,
  all eleven `npm:` tools observed `(absent)` (`@kilocode/cli`,
  `command-code`,
  `agent-browser`,
  `@openai/codex`,
  `ccusage`,
  `cf`,
  `@alibaba-group/open-code-review`,
  `@github/copilot`,
  `@opencode/cli`,
  `@deepseek-ai/dsh`,
  `@namespacelabs/cli`);
  after,
  `toolchain list: 62 tools in sync with toolchain.lock.json`,
  each at its locked version.
  A binary built from main before the release printed the same eleven rows as the installed 0.2.7-alpha.1.
  Evidence files:
  `~/temp/agent/mpm-shadow-alpha2/list-global-before-alpha1.txt`,
  `list-global-before-main-binary.txt`,
  `list-global-after.txt`,
  `install-global.log`,
  `groups-before.txt` and `groups-after.txt`.
- The `tagPrefix` rejection the fix branch's agent reported ("not a known field" for
  `tools.cargo:monochromatic-git-policy-cli.options.tagPrefix`) does not come from main.
  A debug build of main at `5e4e8170ec` read the real global plan and lock (`list --global` printed every row),
  `decode_options` lists `tagPrefix` (added by `eb202865fc`,
  2026-10-31T03:40:22-04:00),
  and that message is exactly what the allow-list prints for a key it lacks,
  so the binary that printed it predates the option (`monochromatic-mpm-shadow` 0.2.0 and earlier;
  0.2.1 has it).
  The global plan entry is the documented shape for a repository release archive
  (`cargo:` is only the id the entry kept when its route moved to `archive`),
  so neither the plan nor the decoder was changed;
  `an_entry_with_a_cargo_id_on_the_archive_route_takes_a_tag_prefix_and_a_postinstall_link` pins it.
  Which checkout produced the old binary was not found.

### Release record: `monochromatic-cargo-shadow` 0.1.5-alpha.1, the scope keeps its caller's limits (2026-11-07T11:55:19-05:00)

The owner's rules,
verbatim:
"My thinking is to deploy all releases on pnpr,
but only publish rc and stable releases on crates.io";
"Label decides";
"I'm authorizing you to bump and publish any updates this session."
The third run under the label gate,
again an alpha:
it goes to the GitHub release and the toolchain lock,
not to crates.io.

- The change it carries (branch `fix/cargo-shadow-cli-scope-keeps-caller-limits`,
  test commit `3537a1ae63` 2026-11-07T11:43:56-05:00,
  fix commit `0033e7c879` 2026-11-07T11:51:26-05:00,
  docs commit `645c8ae5c7` 2026-11-07T11:51:57-05:00,
  merge `b1954d75ae` 2026-11-07T11:52:45-05:00):
  an audit of every `systemd-run` builder found that `control_group.rs` started each bounded run in a scope under the
  manager's `app.slice` with no limits,
  so a build inside a capped `systemd-run --user --scope` ran uncapped (rule RXI).
  The route now reads `memory.max`,
  `cpu.max`,
  `pids.max` and the `default` line of `io.weight` on the caller's cgroup and on
  each ancestor up to,
  and not including,
  the deepest ancestor it shares with the slice,
  and the launcher passes the tightest
  finite value of each as `--property=MemoryMax=`,
  `CPUQuota=`,
  `TasksMax=` and `IOWeight=`.
  Absent limits stay absent.
  The code is a port of the four limits of `package/wg/cli-quicker-exempt/src/inherited_limits.rs`
  (that crate is a binary without a library,
  and each crate stands on its own),
  in `src/inherited_limits.rs`.
  Contract clause:
  `doc/planning/cargo-shadow-guarantees.md`,
  "The spawn route",
  item 4.
- Red first:
  at the test commit ten of the twelve new tests failed (the two that passed are the absent-limits controls);
  after the fix all twelve pass and the release gate ran 664 tests,
  664 passed,
  0 failed,
  18 skipped;
  `lint` and `lint:clippy` with warnings denied are clean.
  `systemd-run` refuses `CPUQuotaPerSecUSec=` as a property ("Unknown assignment",
  probed),
  so the quota is passed as
  `CPUQuota=<percent with two decimals>%`,
  rounded down.
- Host proof,
  run inside `systemd-run --user --scope -p MemoryMax=1G -p CPUQuota=100% -p TasksMax=64 -p IOWeight=5`.
  `cargo --version` and `cargo metadata --no-deps` never start a bounded run (the shadow passes them through),
  so the probe is `cargo metadata` in a scratch git repository with a `rust-catalog.toml`,
  whose install step starts
  `git var GIT_CONFIG_SYSTEM` in a scope of its own;
  a scratch `git` on `PATH` read its own cgroup's four files and then ran the real git.
  Before (binary built from the parent of the change):
  scope `app.slice/cargo-shadow-<token>.scope` read
  `memory.max=max`,
  `cpu.max=max 100000`,
  `pids.max=75462`,
  `io.weight=default 100`,
  against the caller's
  `1073741824`,
  `100000 100000`,
  `64`,
  `default 5`.
  After (binary built from the fix):
  the new scope read `memory.max=1073741824`,
  `cpu.max=100000 100000`,
  `pids.max=64`,
  `io.weight=default 5`.
- Release dry run `https://github.com/Aquaticats/Monochromatic/actions/runs/42393543893` (`success`;
  the push step skipped).
  Release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42393703717`
  (`workflow_dispatch`,
  2026-11-07T11:55:19-05:00 to 11:56:36-05:00,
  `success`),
  bump commit `65ff580b45`
  (2026-11-07T11:56:25-05:00),
  version 0.1.4 to 0.1.5-alpha.1.
  The dispatch printed no refusal,
  so origin held every local commit.
- Publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42393804185`
  (built from `65ff580b45`,
  2026-11-07T11:56:34-05:00 to 11:58:54-05:00,
  `success`).
  The `cgs-detect` job printed `package/cargo-shadow/cli/Cargo.toml holds version '0.1.5-alpha.1': label alpha, goes to crates.io: false`.
  In `cgs-publish-crate` the steps "Build .crate package",
  "Attest .crate provenance",
  "Authenticate with crates.io",
  "Publish (dry run)" and "Publish" were `skipped`,
  and `cgs-build-binary` (both targets) and `cgs-create-release` succeeded.
- crates.io does not hold it:
  `https://crates.io/api/v1/crates/monochromatic-cargo-shadow/0.1.5-alpha.1` answered HTTP 404 (user agent
  `crate-release-plan-check`,
  no token),
  and the crate's newest version is still 0.1.4.
  The wait task said `crates.io 0.1.5-alpha.1: skipped, the alpha label is not published to crates.io`.
- GitHub release `monochromatic-cargo-shadow-v0.1.5-alpha.1` (published 2026-11-07T11:58:51-05:00):
  `isPrerelease` false,
  `isDraft` false;
  assets are the aarch64 and x86_64 Linux tarballs and
  `monochromatic-cargo-shadow-0.1.5-alpha.1-sha256sums.txt`.
- Lock move (commit `c6177b875b`,
  2026-11-07T12:01:43-05:00):
  `mise run toolchain:update` moved `cargo:monochromatic-cargo-shadow`
  0.1.4 to 0.1.5-alpha.1 and nothing else (`llama.cpp` stays pinned);
  the two archive SHA-256 values in the lock equal the release's
  checksums file.
  `mise run toolchain:install` reported `1 installed, 65 unchanged, 0 failed`;
  `.cache/cargo-shadow-bin/cargo --version` prints `cargo-shadow 0.1.5-alpha.1, shadowing cargo`.
- Installed binary:
  the same probe through `.cache/cargo-shadow-bin` in a caller scope with the same four limits made the
  scope `app.slice/cargo-shadow-<token>.scope` read `memory.max=1073741824`,
  `cpu.max=100000 100000`,
  `pids.max=64`,
  `io.weight=default 5`;
  the control through the pre-change binary,
  rerun in the same session,
  read `max`,
  `max 100000`,
  `75462`,
  `default 100`.
  Evidence files:
  `~/temp/agent/cargo-shadow-limits/evidence/before-run.log`,
  `after-run.keep`,
  `installed-run.log`,
  `before-run2.log`,
  `red.log`,
  `gate.log`,
  `clippy.log`,
  `prove.sh` and `git` (the probe).
- Round 21 (`feat/cargo-shadow-check-to-start-race`,
  unmerged):
  `git merge-tree --write-tree` of this branch against it
  reported no conflict,
  and its tests build `Route` only as the same struct literals this change already updated or with
  `..RouteSettings::default()`.

### Release record: `monochromatic-lint` 0.5.2-alpha.1, the package passes its own gates (2026-11-07T12:21:14-05:00)

The owner's rules,
verbatim:
"My thinking is to deploy all releases on pnpr,
but only publish rc and stable releases on crates.io";
"Label decides";
"I'm authorizing you to bump and publish any updates this session.";
"Mutation runs don't block publish and release."
The third run under the label gate,
again an alpha:
it goes to the GitHub release and the toolchain lock,
not to crates.io.

- What it carries (branch `fix/monochromatic-lint-package-gates-clean`,
  merge `b75b3755ea` 2026-11-07T12:20:34-05:00):
  the layout rules and the group folder checks added since 0.5.1,
  and these gate fixes.
  Formatting `5deefd74ba` (2026-11-07T11:57:31-05:00):
  `format:rust` on the files that failed `lint:rustfmt`;
  the nextest test list before and after is identical and `lint` passes;
  the changes are line wrapping,
  trailing commas,
  a moved `mod` and `use` order,
  and a `;` after `return` in four test files.
  Mutation scopes `14744ea632` (2026-11-07T12:00:26-05:00).
  README line breaks `0e272fd324` (2026-11-07T12:01:19-05:00),
  whitespace only:
  identical after collapsing whitespace.
  Test sidecar `80e791cb38` (2026-11-07T12:13:08-05:00),
  avoidable closures and tuple field docs `121630f4e8` (2026-11-07T12:13:18-05:00),
  rustdoc line breaks `525dca3f93` (2026-11-07T12:13:34-05:00),
  comment text only.
- Mutation scopes:
  `mutation:coverage` failed before the change with every mutant of the `layout_*` modules outside every scope,
  and also with the workflow and shell modules,
  `rust_manifest.rs` and `rust_file_engine_syntax.rs` outside every scope.
  The new scopes are `--layout` (`mutation:layout`,
  `mutation:list:layout`) for `src/layout_*.rs` against the `layout_` tests,
  and `--workflow-shell` (`mutation:workflow-shell`,
  `mutation:list:workflow-shell`) for `src/workflow_*.rs` and `src/shell_*.rs`.
  The manifest lookup and the syntax router joined `--semantic`.
  `mutation:coverage` then exited 0 with the union equal to the unscoped listing and nothing outside or unknown.
  No mutation campaign ran;
  the coordinator runs them.
- Gates on the branch (nothing in `package/linter` changed by the second merge of main):
  `test` 869 tests run,
  869 passed,
  2 skipped by `#[ignore]`
  (`panic_hook_child`,
  which only runs as the child of the panic-hook control,
  and `host_real_run_over_wg_quicker_exempt`,
  which `test:host` runs);
  `test:layout` 137 of 137;
  `lint`,
  `lint:types` and `lint:rustfmt` exit 0;
  `mise run lint:monochromatic-lint -- package/linter/monochromatic-lint` printed no finding,
  with the `rust/avoidable-closure` check running (a first run that skipped it for a removed `cargo-shadow` launcher was not counted).
- Release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42395888611`
  (`workflow_dispatch`,
  2026-11-07T12:21:14-05:00 to 12:23:11-05:00,
  `success`),
  bump commit `7b5438db2f`
  (2026-11-07T12:23:01-05:00),
  version 0.5.1 to 0.5.2-alpha.1.
- Publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42396050013`
  (built from `7b5438db2f`,
  2026-11-07T12:23:09-05:00 to 12:32:17-05:00,
  `success`).
  The `ml-detect` job printed `package/linter/monochromatic-lint/Cargo.toml holds version '0.5.2-alpha.1': label alpha, goes to crates.io: false`.
  In `ml-publish-crate` the steps "Build .crate package (verifies by compile)",
  "Attest .crate provenance",
  "Authenticate with crates.io (Trusted Publishing)",
  "Publish (dry run)" and "Publish (skips if this version is already on crates.io)"
  were `skipped`,
  and the job itself ran,
  so `ml-create-release` was not skipped;
  every `ml-build-binary` job succeeded.
- crates.io does not hold it:
  `https://crates.io/api/v1/crates/monochromatic-lint/0.5.2-alpha.1` answered HTTP 404
  (`crate ... does not have a version 0.5.2-alpha.1`,
  user agent `crate-release-plan-check`,
  no token),
  and the crate's newest version is still 0.5.1.
  The wait task said `crates.io 0.5.2-alpha.1: skipped, the alpha label is not published to crates.io`.
- GitHub release `monochromatic-lint-v0.5.2-alpha.1` (published 2026-11-07T12:32:14-05:00):
  `isPrerelease` false,
  `isDraft` false.
  The release holds no checksums file;
  the asset digests GitHub reports for the four Linux archives equal the lock's values.
- Lock move (commit `86ce4da394`,
  2026-11-07T12:35:04-05:00):
  `mise run toolchain:update` moved `cargo:monochromatic-lint` 0.5.1 to 0.5.2-alpha.1 and nothing else (1 tool moved);
  `mise run toolchain:install` installed it (`1 installed, 65 unchanged, 0 failed`) and `monochromatic-lint --version`
  prints `monochromatic-lint 0.5.2-alpha.1`.
- Whether the fixer's output changed:
  no,
  measured.
  The source diff from 0.5.1 changes no Markdown rule,
  but `Cargo.lock` moved `icu_segmenter`,
  `unicode-width`,
  `unicase`,
  `utf8_iter` and others,
  so the output was compared instead of inferred.
  A copy of every tracked Markdown and MDX file (3051 files and the root config) was fixed with `--fix` by the 0.5.1 binary,
  by a debug build of the branch,
  and by the installed 0.5.2-alpha.1 binary:
  `diff --recursive` of the three result trees was empty and the three finding logs were identical.
  The git wrapper's documentation reflow on `fix/git-policy-cli-passes-the-repository-linter` is therefore not affected by this release.
  Evidence files:
  `~/temp/agent/lint-gates/` (`corpusA`,
  `corpusB`,
  `corpusC`,
  `fixA.log`,
  `fixB.log`,
  `fixC.log`,
  `coverage-before.log`,
  `coverage-after.log`,
  `final-test.log`,
  `release-wait.log`).

### Release record: `monochromatic-mpm-shadow` 0.2.7-alpha.3 and 0.2.7-alpha.4 and `monochromatic-lint` 0.5.2-alpha.2, worktree launchers follow current (2026-11-07T13:32:40-05:00)

The owner's rules,
verbatim:
"Label decides";
"I'm authorizing you to bump and publish any updates this session."
Three runs under the label gate,
all alphas:
they go to GitHub releases and the toolchain lock,
not to crates.io.

- The defect (branch `fix/worktrees-use-their-own-locked-tools`;
  troubleshooting entry
  `doc/troubleshooting/mpm-shadow-launcher-names-a-deleted-release-folder-after-a-newer-install.md`):
  a launcher record names the archive folder below `current`,
  whose name carries the version,
  so installing a newer release in any tree
  deleted the folder and every other tree's launcher exited 127 (122 of 853 records dangled in a scan of this repository's checkouts).
  The brief's suspect,
  the inherited `PATH` or mise,
  was not the cause:
  `mise run` puts the tree's own `.cache/*-bin` first.
- Red first:
  test commits `982deb5c73` (2026-11-07T12:32:30-05:00,
  four red),
  `54f544709f` (2026-11-07T12:44:09-05:00,
  one red,
  the closure check
  that skipped with exit 0) and `51513f35aa` (2026-11-07T13:23:32-05:00,
  red for the record rewrite).
  Fix commits `477be995d3`,
  `14cd55b8d5` and `99dd1a03c0`;
  the mpm-shadow package ended at 589 of 589 tests,
  clippy with warnings denied clean;
  the monochromatic-lint focused tests passed and its container clippy passed,
  while its container test run stops at
  `config_pin::tests::the_hint_names_a_measured_newer_pin_and_nothing_else`,
  which fails alone as well and is in a module this change does not touch.
- Release runs:
  0.2.7-alpha.3 `https://github.com/Aquaticats/Monochromatic/actions/runs/42398834142` (publish `42398943635`),
  0.5.2-alpha.2 `https://github.com/Aquaticats/Monochromatic/actions/runs/42399661482` (publish `42399787455`),
  0.2.7-alpha.4 `https://github.com/Aquaticats/Monochromatic/actions/runs/42401355789` (publish `42401456839`);
  each dry run succeeded first;
  crates.io skipped each ("the alpha label is not published to crates.io").
- Lock moves:
  `ee2d870060` (mpm-shadow 0.2.7-alpha.2 to alpha.3 and monochromatic-lint 0.5.2-alpha.1 to alpha.2,
  two tools moved) and
  `5cd86b210f` (mpm-shadow alpha.3 to alpha.4,
  one tool moved);
  `toolchain:install` ended with `toolchain list: 66 tools in sync`.
- Proof at the user boundary (disposable worktree of the parent of `c6177b875b`,
  lock cargo-shadow 0.1.4,
  launcher record naming the deleted 0.1.4
  folder,
  launcher binary copied from the installed alpha.4,
  shell started in the main checkout):
  `cargo --version` printed the older-lock warning and the version lines with exit 0 (before:
  exit 127 `could not start ... 0.1.4 ... No such file`),
  and `mise run //package/mpm-shadow/cli:test` ended `581 tests run: 581 passed` with exit 0;
  the record had been rewritten to the 0.1.5-alpha.1 folder.
  With the alpha.3 launcher the same state stopped `mise run` at cargo-shadow's recursion guard,
  which the alpha.4 record rewrite fixed.
- Not fixed here:
  launcher binaries are hard links of the `mpm-shadow` that last ran `install` in each tree,
  so a tree that has not run
  `mise run toolchain:install` since alpha.4 still has an older launcher and fails until it does;
  174 records name another tree's lock file.

### Release record: `monochromatic-cargo-shadow` 0.1.5-alpha.2, round 21 starts a compiled real cargo through the file that was checked (2026-11-07T14:08:29-05:00)

The owner's rules,
verbatim:
"Label decides";
"I'm authorizing you to bump and publish any updates this session.";
"nothing should ever wait forever or indefinitely".
The fourth cargo-shadow-or-sibling alpha of the day under the label gate:
it goes to the GitHub release and the toolchain lock,
not to crates.io.

- The change it carries (branch `feat/cargo-shadow-check-to-start-race`,
  merged to main by `4c31f33751` 2026-11-07T14:01:31-05:00 after `eb2f651af5`):
  the chosen real cargo is opened once and a readable compiled (ELF) program is started through that open file,
  so a path made to name the shadow after the check cannot reach the start;
  scripts,
  unreadable executables and the install step's contained cargo keep the pathname with the check repeated.
  The details,
  the cross-vendor review and what stays open are in `doc/planning/cargo-shadow-update.md`,
  "Round 21 merge record (2026-11-07)",
  and the contract block "Round 21 additions" in `doc/planning/cargo-shadow-guarantees.md`.
- Gates before the merge (the final tree):
  test 679 run,
  679 passed,
  18 skipped;
  `lint` and `lint:clippy` with warnings denied exit 0.
  One earlier full run of the same tree failed two timing tests of `bounded_contain_tests`
  (a 3 second bound and a 1.5 second wait) while the host load average was 40 to 60:
  the whole early batch of contained starts took 5 to 6 seconds instead of 0.04 seconds,
  the same two tests passed alone three times at load 32 to 35,
  and the next full run passed.
  The cause measured is a slow scope start at suite start under that load;
  the tests' fixed bounds do not tolerate it and were not touched.
- Red first:
  the full gate found one red test the round caused (the new `spawned` trace event);
  the cross-vendor reviewer found one production defect no test reached
  (the exec site closed the descriptor before `exec`),
  an execute-only refusal and an inclusive process id bound;
  a second review round found the execute-only script case.
  Each has a red commit before its fix.
- Release dry run `https://github.com/Aquaticats/Monochromatic/actions/runs/42404208563` (`success`;
  the publish job skipped).
  Release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42404324402`
  (2026-11-07T19:03:27Z to 19:04:45Z),
  bump commit `20d7f38f67` (2026-11-07T19:04:33Z),
  version 0.1.5-alpha.1 to 0.1.5-alpha.2.
  Publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42404426395`
  (built from `20d7f38f67`,
  2026-11-07T19:04:42Z to 19:07:00Z,
  `success`);
  the wait task said `crates.io 0.1.5-alpha.2: skipped, the alpha label is not published to crates.io`,
  and `https://crates.io/api/v1/crates/monochromatic-cargo-shadow/0.1.5-alpha.2` answered HTTP 404.
- GitHub release `monochromatic-cargo-shadow-v0.1.5-alpha.2` (published 2026-11-07T19:06:57Z):
  `isPrerelease` false,
  `isDraft` false;
  the aarch64 and x86_64 Linux tarballs and the checksums file.
- Lock move (commit `62d3f69165`,
  2026-11-07T14:08:29-05:00):
  `mise run toolchain:update` moved `cargo:monochromatic-cargo-shadow` 0.1.5-alpha.1 to 0.1.5-alpha.2 and nothing else
  ("1 tools moved");
  the aarch64 and x86_64 SHA-256 values in the lock equal the release's checksums file.
  `mise run toolchain:install` reported `1 installed, 65 unchanged, 0 failed`.
- Installed binary:
  `.cache/cargo-shadow-bin/cargo --version` prints `cargo-shadow 0.1.5-alpha.2, shadowing cargo at .../.cache/toolchain-bin/cargo`;
  `cargo new` and `cargo build` of a scratch crate through that directory (capped scope) finished and
  the program printed `Hello, world!`.

### Release record: `monochromatic-git-policy-cli` 0.6.1-alpha.1, the earlier names fold and a no-checkout clone checks out (2026-11-07T20:02:01Z)

Released by the coordinator from main,
not waiting for the tokio branch:
the leftover-ref-lock cause the release had waited for was measured
(Git's own SIGTERM cleanup cut by a 5 s stop),
and the same race is in 0.6.0.

- Content:
  the fold of earlier `cli-git` names into the current ones (`baf4c039f2`),
  a checkout in a clone made with `--no-checkout` (merge `d2598b2842`),
  the held-settle message naming the earlier commit service's program,
  and the index-lock fallback.
- Gate:
  `native:test:container` on main `d2598b2842` in a throwaway worktree,
  unit `wrapper-alpha-gate-d2598b2842`:
  `Summary [ 976.843s] 5199 tests run: 5199 passed (4 slow), 8 skipped`.
  Between the gate and the dispatch,
  nothing under `package/git-policy/cli`,
  `rust-catalog.toml` or `Cargo.lock` changed;
  `.github/workflows/cargo-publish.yml` and the crates.io decision action did (the repository-wide lint pass),
  so the dry run went first.
- Dry run `https://github.com/Aquaticats/Monochromatic/actions/runs/42408755184`:
  `success`
  (release `success`,
  publish skipped).
  The first dispatch was refused locally:
  the note was 217 characters against the tool's 200.
- Release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42409052182`
  (2026-11-07T20:02:01Z to 20:05:36Z,
  `success`),
  bump commit `215d3dd3a1` (2026-11-07T20:05:23Z),
  0.6.0 to 0.6.1-alpha.1.
- Publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42409350167`
  (2026-11-07T20:05:33Z to 20:10:34Z,
  `success`);
  `native:release:wait` printed
  `crates.io 0.6.1-alpha.1: skipped, the alpha label is not published to crates.io (only a plain number and -rc.N are)`
  and the release assets for `aarch64` and `x86_64` Linux.
- Lock commit `4c06805b71`:
  `mise run toolchain:update` reported `1 tools moved`;
  the diff (read whole) changes only `cargo:monochromatic-git-policy-cli`
  (version,
  four artifact URLs,
  sha256 `04476eca...0deb` for `aarch64` and `12963df1...b35b` for `x86_64`) and the lock digest.
  `toolchain:install` reported `1 installed, 65 unchanged, 0 failed`.
- Installed binary:
  `.cache/toolchain-bin/monochromatic-git-policy-cli --version` prints
  `monochromatic-git-policy-cli 0.6.1-alpha.1, shadowing git at /usr/bin/git`;
  the lock commit itself went through it and printed "Updating the commit service...".
- The fold:
  the six earlier-name lines had already stopped before the install;
  `.git/cli-git` and the earlier logs were gone and `.git/monochromatic-git-policy-cli` last changed at
  2026-11-07T14:57:11-05:00,
  so a build of main's code (which has the fold) ran against the shared Git directory before 0.6.1-alpha.1 was
  installed;
  which program did it is not identified.
  Nothing was lost:
  0.5.34 wrote only the earlier names,
  and the current logs hold its records
  (257 commit-timing,
  22 audit,
  73 push-trigger,
  41 worktree-add-timing lines),
  matching the earlier files' sizes.

### Release record: the shadows never start a shadow, `monochromatic-search-shadow` 0.2.3-alpha.1, `monochromatic-pnpm-shadow` 0.1.3-alpha.1, `monochromatic-cargo-shadow` 0.1.5-alpha.3 and `monochromatic-mpm-shadow` 0.2.7-alpha.5 (2026-11-07T15:22:55-05:00)

The owner's rules,
verbatim:
"Label decides";
"I'm authorizing you to bump and publish any updates this session.";
"We should never invoke ourselves recursively. That's bad design. Audit, remove, and ban that design aggressively.";
"nothing should ever wait forever or indefinitely".
Four alphas under the label gate:
they go to the GitHub releases and the toolchain lock,
not to crates.io.

- The defect and the change (branch `fix/shadows-never-start-another-shadow`,
  merged to main by `ec70e4cf26` 2026-11-07T15:02:22-05:00;
  troubleshooting entry `doc/troubleshooting/search-shadow-copies-start-each-other-without-end.md`):
  two `search-shadow` copies at different paths chose each other as the real `grep` and replaced themselves with each other without end
  (measured 105 starts per second under `strace`, at a quarter of one CPU).
  Every `PATH` lookup of the four shadows skipped only its own path;
  now every shadow program carries an identity text in an ELF section,
  the lookup skips any candidate that carries it (through launcher records),
  the start goes through the file that was checked,
  and a shadow started by a shadow stops at once with one line and exit status 2.
  `mpm-shadow` launchers also refuse a record chain that leads back to a launcher.
- Red first:
  test commits `4b435e69f9` (search-shadow: three loops ended at the five second bound, two failures, two controls passed),
  `679d141568` (pnpm-shadow: four red),
  `6cebcb29ce` (cargo-shadow: four red) and `f9bafea350` (mpm-shadow: seven red,
  two of them launcher loops that ended at the bound).
  Fix commits `60c11621d4`,
  `505fcdc7dc`,
  `abb5b4210c`,
  `ec51606c45` and `99f7f2beba`.
- Gates on the merged tree:
  `lint`,
  `lint:clippy` with warnings denied and `test` for each crate:
  search-shadow 97 of 97,
  pnpm-shadow 97 of 97,
  cargo-shadow 708 of 708 (18 skipped),
  mpm-shadow 628 of 628.
  One earlier full cargo-shadow run failed
  `round_fourteen_tests::a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone`
  twice (it needs a freed time namespace number to be reused,
  and the allocator is shared by every session on the host);
  it passed in the later full run without a change to that code.
- Dry runs (each `success`,
  `publish` skipped):
  mpm-shadow `https://github.com/Aquaticats/Monochromatic/actions/runs/42409174548`,
  search-shadow `.../42409864215`,
  pnpm-shadow `.../42410075106`,
  cargo-shadow `.../42410286984`.
  Three dry runs started at once were cancelled by the workflow's one-run concurrency group (`.../42409146071`,
  `.../42409153052`,
  `.../42409159803`) and were started again one at a time.
- Release runs and publish runs (each `success`):
  mpm-shadow 0.2.7-alpha.4 to alpha.5 `.../42409491393` (bump `ffca464af8`),
  publish `.../42409620811`;
  search-shadow 0.2.2 to 0.2.3-alpha.1 `.../42409961703` (bump `b2b5a4f5bd`),
  publish `.../42410050963`;
  pnpm-shadow 0.1.2 to 0.1.3-alpha.1 `.../42410171012` (bump `1eb57682c6`),
  publish `.../42410263305`;
  cargo-shadow 0.1.5-alpha.2 to alpha.3 `.../42410399055` (bump `518be0a289`),
  publish `.../42410470730`.
  The wait task said for each `crates.io <version>: skipped, the alpha label is not published to crates.io`.
  GitHub releases `monochromatic-<crate>-v<version>` (published 2026-11-07T20:13:06Z for mpm-shadow,
  20:15:34Z for search-shadow,
  20:18:24Z for pnpm-shadow and
  20:21:17Z for cargo-shadow):
  `isPrerelease` false,
  `isDraft` false,
  with both Linux tarballs and the checksums file.
- Lock move (commit `2b9a1589ae`,
  2026-11-07T15:22:55-05:00):
  `mise run toolchain:update` moved the four tools and nothing else ("4 tools moved");
  every aarch64 and x86_64 SHA-256 value in the lock equals the release's checksums file.
  `mise run toolchain:install` reported `3 installed, 63 unchanged, 0 failed`
  (the mpm-shadow copy that ran the install was already the newest).
- Installed binaries:
  `.cache/search-shadow-bin/rg --version` and `grep --version`,
  `.cache/cargo-shadow-bin/cargo --version`,
  `.cache/pnpm-shadow-bin/pnpm --version` and `.cache/mpm-shadow-bin/mpm --version` print the real tool's text and one line naming the shadow at its new version,
  exit 0 (the first `mpm --version` call ran past a 20 second limit and was not reproduced:
  four later calls took 0.2 to 3 seconds,
  and a trace of one showed a single start of each program).
  The two-copy reproduction with the installed 0.2.3-alpha.1 program and a copy of it made another version
  ended by itself with exit 1 (grep, no match) after two starts (the shadow, then the real `grep` through the checked file),
  where 0.2.2 never ended.
  A shadow started by a script that runs another shadow stopped with exit status 2 and the one line.
- Not done here:
  the real cargo that `cargo-shadow`'s install step starts inside its contained runner gets no arrival marker
  (it keeps the vetting and a lookup that skips shadows by identity);
  a tree whose links point at an older shadow keeps the old behavior until `mise run toolchain:install` runs in it;
  a shared crate for the identity rule was not added (the broader option,
  left to the owner).

### Release record: `monochromatic-lint` 0.5.2-alpha.3, `--fix` changes only whitespace (2026-11-07T15:42:54-05:00)

The owner's rules,
verbatim:
"Label decides";
"I'm authorizing you to bump and publish any updates this session."
An alpha under the label gate:
it goes to the GitHub release and the toolchain lock,
not to crates.io.

- The change it carries (branch `fix/monochromatic-lint-fix-changes-only-whitespace`,
  merged to main by `2941e1e60e`):
  `--fix` no longer wraps a URL whose link text swallowed a code span backtick,
  no longer rewrites a pipe table,
  no longer rewrites a rustdoc comment inside a fenced listing
  and no longer adds `>` to a broken blockquote line.
  The open split of whitespace fixes from content fixes is
  `doc/planning/monochromatic-lint-fix-split-whitespace-and-content.md`.
- Gates on the merged tree:
  `lint` and `lint:types` (clippy with warnings denied) exit 0;
  the test run ended `871 tests run: 868 passed, 3 failed, 3 skipped`,
  the three failures being `config_pin::tests::the_hint_names_a_measured_newer_pin_and_nothing_else`,
  `config_pin::tests::unmeasured_pins_and_farther_lockfiles_produce_no_hint` and
  `binary::a_newer_repository_pin_adds_the_stale_binary_hint`,
  the pin-hint family the 0.5.2-alpha.2 record above already names as failing in a module this change does not touch
  (`numeric_triple` in `src/config_pin.rs` reads `0.5.2-alpha.3` as four parts and measures nothing).
- Release runs:
  dry run `https://github.com/Aquaticats/Monochromatic/actions/runs/42410774214` (success),
  release `https://github.com/Aquaticats/Monochromatic/actions/runs/42410912719`,
  publish `https://github.com/Aquaticats/Monochromatic/actions/runs/42411061993`;
  crates.io skipped it ("the alpha label is not published to crates.io"),
  and the GitHub release `monochromatic-lint-v0.5.2-alpha.3` holds the platform archives.
- Lock move:
  `4076b89351`,
  one tool moved by `mise run toolchain:update` (`cargo:monochromatic-lint` 0.5.2-alpha.2 to 0.5.2-alpha.3);
  `mise run toolchain:install` then installed it and `monochromatic-lint --version` printed `monochromatic-lint 0.5.2-alpha.3`.
- Proof at the user boundary:
  the installed 0.5.2-alpha.3 binary run with `--fix` on the five reproductions left the URL and the pipe table
  unchanged and still reported (`markdown/no-bare-urls`,
  `markdown/no-pipe-tables`),
  left the fenced Rust listing in Markdown unchanged and still reported,
  broke the outer doc comment of a Rust file and left its doc example alone,
  and broke the blockquote lines without adding a `>`.

## Evidence log

- crates.io API reads:
  `https://crates.io/api/v1/crates/<crate>` for the 20 candidate names,
  `https://crates.io/api/v1/users/Aquaticat` and `https://crates.io/api/v1/crates?user_id=415138&per_page=100`
  (16 crates),
  2026-11-07,
  user agent `crate-release-plan-check`.
- Graph:
  `cargo metadata --no-deps --offline --format-version 1 --manifest-path <each Cargo.toml>`,
  all 35 manifests.
- Lock scan:
  every tracked `Cargo.lock` for the 16 package names.
- Measured in a throwaway worktree of `17eff86b35` (removed):
  `cargo update --workspace --offline` in `cli.fuzz` on the
  unchanged tree (exit 0);
  the same after setting `jsonc-edit` to 0.1.3 without moving pins (the failure quoted above);
  `file-enforcer check` in that state (eight pin drifts);
  `cargo check --lib --offline --locked` in `cli.fuzz` (E0432).
  The edit to `package/jsonc-edit/core-rust/Cargo.toml` in the worktree was run only after the coordinator relayed the
  owner's permission for that one command,
  and the worktree was removed afterwards.
- Published `.crate` comparison:
  `forbidden-regex-0.1.1.crate` and `forbidden-strings-0.6.1.crate` against `main`.
- Files read:
  `doc/planning/crates-io-footprint.md`,
  `doc/planning/crate-release-yank-after-publish.md`,
  `doc/planning/git-policy-cli-program-rename.md`,
  `doc/decision/releases-run-in-ci.md`,
  `doc/runbook/publish-crate-first-time.md` (needed only for a crate never published;
  none is),
  `package/crate-release/cli/README.md`,
  `releasable-crates.json`,
  `.github/workflows/crate-release.yml`,
  `.github/workflows/cargo-publish.yml`.
