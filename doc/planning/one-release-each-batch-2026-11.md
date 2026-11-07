# One release of each published crate: execution plan for the 2026-11 batch

Status:
the batch is running (first publish 2026-11-07T11:43:35Z);
the release record in the runbook lists what was dispatched and published (nothing is yanked).
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
- Pins that the release tool could not edit (fixed on `main`, see "What exists now" under "Preparatory steps").
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

Picked in the question tool, verbatim labels:

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
  (`jsonc-edit/core-rust.fuzz`, `xml-edit/core-rust.fuzz`, `forbidden-strings/core.fuzz`).
  Every crate of the batch then goes through `crate-release.yml`, one run at a time.

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
    the registered route replaces its "Per wave" procedure and its steps 3, 4 and 6 with one dispatch per crate
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
4.  Done: the registry entries,
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
  Before, only the lockfiles beside the crate and beside the pin files were committed,
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
  Its log shows the crate manifest and all eight pin files edited, among them the multi-line table of
  `package/jsonc-edit/core-rust.fuzz`,
  then ten `cargo update --workspace` steps (one per lock directory),
  the publish dry run, and `RELEASE COMMITTED` at 2026-11-07T10:26:13Z;
  the push and publish jobs were skipped, as a dry run requires.
  The dry run cannot show two things:
  the push step (the watch set of `monochromatic-jsonc-edit` is the `cargo metadata` closure of ten lock directories,
  which reaches most Rust code of the repository, so its real release is the one most likely to be refused for a moved
  `main` and need a second dispatch; schedule it while no other Rust commits land),
  and the Wayland library step of `monochromatic-nested-wayland-session` (its own dry run, the first step of the runbook,
  is the proof).
- The "Per wave" procedure and steps 3, 4 and 6 of the runbook describe route A and must be rewritten by the
  coordinator as one dispatch per crate in wave order (dry run, real run, `release:wait`, for the libraries read the
  `crates.io` line because their `release:wait` exits 1 on the missing release assets).
  The wave-0 crates have no dependency among them;
  a dependent's dispatch must wait until its dependencies are in the crates.io index,
  and `release:wait` ends only after the publish run, so waiting for it is the order guard.

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

One entry per crate, as run through `crate-release.yml`;
the dry run of `monochromatic-nested-wayland-session` 0.6.2 to 0.6.3
(`https://github.com/Aquaticats/Monochromatic/actions/runs/42368894213`, `success`, 2026-11-07)
proved the Wayland library step.

- `monochromatic-jsonc-edit` 0.1.2 to 0.1.3:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369224009`,
  bump commit `33d20fa630`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369300426`,
  crates.io `2026-11-07T11:43:35.824945Z`, `yanked` false.
  The first dispatch pushed without a refusal.
  `release:wait` ended with the expected library line `PUBLISH CHECK FAILED: missing release assets`.

- `monochromatic-dependency-catalog` 0.2.0 to 0.2.1:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369457223`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369535765`
  (built from `a8923376a2`),
  crates.io `2026-11-07T11:47:06.445839Z`, `yanked` false.

- `monochromatic-browserslist` 0.1.0 to 0.1.1:
  release run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369723100`,
  publish run `https://github.com/Aquaticats/Monochromatic/actions/runs/42369851512`
  (built from `4f224afa10`),
  crates.io `2026-11-07T11:52:01.185390Z`, `yanked` false.

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
