# monochromatic-cargo-shadow

A shadow for `cargo`,
modeled on `pnpm-shadow`,
that keeps every `Cargo.lock` satisfying its manifests under the trust policy in `rust-catalog.toml`.
It also holds the repository's Rust catalog tools (`install`,
`update`),
which used to be the TypeScript package
 `package/dependency-catalog/cli-rust-update`.
Design and decisions:
[`doc/decision/rust-dependency-catalog.md`](../../../doc/decision/rust-dependency-catalog.md),
 [`doc/planning/cargo-shadow-update.md`](../../../doc/planning/cargo-shadow-update.md).

## Why it exists

The catalog's `>=` ranges are raised by `catalog:tighten` and rendered into the manifests by `mise run sync:files`;
nothing regenerated the lockfiles.
A lockfile its own manifest refuses is rewritten by the next plain cargo run,
through cargo's own resolver,
past the release-age and trust policy.
pnpm solves the same problem with `verifyDepsBeforeRun: install` (its default):
"Automatically runs install if `node_modules` is not up to date"
 ([pnpm settings][pnpm-build],
read 2026-11-03),
and with `frozenLockfile`,
which defaults to true in CI when a lockfile is present
 ([pnpm settings][pnpm-store],
read 2026-11-03).
The shadow does both for `Cargo.lock`.

## Two modes, one binary

Started as `cargo` (the toolchain plan links it as `.cache/cargo-shadow-bin/cargo`),
it is the shadow.
Started under any other name it is the catalog CLI:
`install`,
`update`,
`--help`,
`--version`.

The binary is named `cargo-shadow` (the owner kept the name,
matching `pnpm-shadow` and `mpm-shadow`),
so cargo itself also runs it as the subcommand `cargo shadow <command>` (cargo runs any `cargo-<name>` executable on PATH).
The shadow drops the leading `shadow` word,
so `cargo shadow install` and `cargo-shadow install` mean the same.
That is a feature,
not a hazard:
the plugin name is not a command the shadow itself recognizes,
so it passes through to the real cargo,
which finds the binary.

## What the shadow does before cargo

For a command that resolves dependencies it runs the install step for the Cargo workspace that holds the working directory
 (`-C <dir>` and `--manifest-path` move that place),
and then runs the real cargo with the arguments as typed.
The step edits no argument of its own accord;
the one place the shadow adds an option to your command is the `--locked` described below for an operational failure.
Outside a repository with a `rust-catalog.toml`,
and in a workspace without a `Cargo.lock`,
it does nothing.

The install step:

- A lockfile that satisfies its manifests is left byte for byte as it is.
- A stale lockfile is updated offline,
  keeping every locked release the manifests still allow (`cargo update --workspace --offline`,
  then `cargo metadata --locked --offline` confirms).
- Every registry version the update newly locked is checked against the release-age and trust policy,
  and so is every version already in the lockfile that the policy covers:
  all of them when `trustLockfile = false` (pnpm's default),
  the ones new since the base revision when `trustLockfile = true` (the repository's setting).
  This runs on every local (not frozen) install,
  as pnpm's lockfile verification runs on every install that is not frozen
  (`pnpm11/installing/deps-installer/src/install/mutateModules.ts:314-343`).
- A release the policy refuses is set aside:
  the lockfile is changed to the newest older release that passes and fits the manifests (`cargo update --package <name>@<version> --precise <older>`),
  one line names both releases,
  and cargo runs.
  This is what pnpm does with `minimumReleaseAge` (its resolver picks the newest release older than the cutoff;
  `resolving/npm-resolver/src/pickMatchingVersion.ts`, `pickRespectingMinReleaseAge`),
  and the owner's decision of 2026-11-03 for a committed lockfile:
  a merge that brings `serde 1.0.230` published three hours ago under a one day limit gives `serde 1.0.229` in `Cargo.lock`,
  and a trust downgrade gives the earlier release with the earlier trust level.
  The refused release never runs; the older copy does.
  Under `trustLockfile = true` this fires for a lockfile change that the base revision does not hold yet
  (an uncommitted change, or a base set behind the merge with `CARGO_SHADOW_TRUST_BASE`);
  once a merge is committed, `HEAD` already holds it and the local install sets nothing back.
  Whether local runs should compare with something older than `HEAD` is an open question to the owner.
  Also open: under `trustLockfile = true` the local install applies the release age to the versions new since the base,
  while the frozen check applies trust only (the amendment of 2026-10-29 spoke of trust).
- If no older release passes and fits,
  the refusal stands:
  nothing is left changed and your cargo command is not run.
  That is the only case in which the step stops the command because of the policy.
- In CI (`CI` set to anything but empty,
  `0` or `false`),
  or when the user's own command carries `--locked` or `--frozen`,
  the install is frozen:
  a stale lockfile fails the step naming it,
  nothing is written and nothing is set aside.
  That is `pnpm install --frozen-lockfile`.
  In CI the check also drops `--offline`,
  because a fresh runner has no cached registry index.
- When the step fails for an operational reason (no network,
  a registry lookup that fails,
  a cargo error,
  an unreadable catalog),
  or a frozen step refuses,
  it prints a warning naming what failed and why,
  and then runs your cargo command with `--locked` added (before any `--`) unless you already passed `--locked` or `--frozen`;
  the warning says so.
  This is pnpm's behavior:
  for `verifyDepsBeforeRun: install` pnpm says "If that install fails,
  for example because the store is read-only or there is no network,
  pnpm prints a warning and runs the command anyway" ([pnpm settings][pnpm-build],
  read 2026-11-03).
  pnpm can do that because `pnpm run` never rewrites the lockfile;
  cargo does rewrite a stale lock,
  so the faithful form of "run it anyway" is `--locked`:
  cargo runs when the lock is current and stops with its own lock error when the lock would change.
  `--locked` keeps cargo from rewriting the lock;
  it does not check publisher trust,
  which is why a local policy refusal stops the command instead.

### The fast path

The check is on every cargo call,
so it resolves nothing when nothing changed.
After a pass,
the step records the size and modification time of the lockfile and of every manifest `cargo metadata` lists as part of the workspace
 (members and path dependencies) in `.cache/cargo-shadow/stamps/`.
That stamp certifies one thing:
the lockfile satisfies its manifests.
It says nothing about the policy,
so a call takes the fast path only when the stamp is fresh and the policy check of the same mode,
the same settings and the same base revision has passed for this exact lockfile text
 (a remembered pass under `.cache/cargo-shadow/verified/`).
Otherwise the policy check runs again and only the staleness check (a cargo call) is skipped.
A missing stamp,
an unreadable file or any difference runs the authoritative check again.
Limits:
a change to `.cargo/config.toml` `[patch]` tables or to a manifest outside the recorded list is not noticed until a recorded file changes,
and a base revision spelled as a moving reference (`HEAD`, `origin/main`) is keyed by its spelling,
not by the commit it names,
so the pass is reused after the reference moves.

Measured on 2026-11-03 in a throwaway worktree (release build,
up-to-date workspaces,
alternating arms,
20 to 30 runs each,
a second direct arm as the same-binary control):

- `cargo metadata --offline --format-version 1` in `package/pnpm-shadow/cli`:
  direct 33.4 ms (sd 1.7) and 33.1 ms (sd 1.4),
  through the shadow 34.4 ms (sd 1.6).
  The shadow added about 1.2 ms.
- The same in `package/music-player/android-app/rust` (the largest lockfile):
  direct 114.9 ms (sd 5.5) and 114.7 ms (sd 5.5),
  through the shadow 117.1 ms (sd 7.9).
  The shadow added about 2.3 ms,
  inside the run-to-run spread of the control arms.
- `cargo metadata --no-deps` (not a resolving command,
  so the shadow only starts and execs):
  about 0.7 to 1.1 ms added to 14.5 ms.
- What the stamp avoids:
  `cargo update --workspace --locked --offline` took 72 ms (sd 2.3) in `package/pnpm-shadow/cli` and 221 ms (sd 5.6) in the android-app workspace,
  on every call.

## Which commands trigger the install step

Read from `cargo --list` and `cargo --help` of cargo 1.101.0-nightly (3960fa69b 2026-10-12),
then measured:
in a workspace whose lock lacked a dependency its manifest declares,
each of these rewrote the lock,
and these did not.

Install step first (they resolve dependencies),
with aliases:

- `build` (`b`),
  `check` (`c`),
  `clippy`,
  `doc` (`d`),
  `run` (`r`),
  `test` (`t`),
  `bench`,
  `fix`,
  `fetch`,
  `metadata`,
  `tree`,
  `vendor`,
  `rustc`,
  `rustdoc`,
  `package`,
  `publish`,
  `info`.
- Measured to rewrite the lock:
  `check`,
  `build`,
  `metadata`,
  `tree`,
  `fetch`,
  `clippy`,
  `doc`,
  `vendor`,
  `package`,
  `rustc`,
  `rustdoc`,
  `test`,
  `bench`,
  `info`,
  `nextest`,
  `dylint`,
  `ndk`.
  `run` and `fix` stopped on their own errors in the fixture (no binary target,
  nothing to fix) before a write could be observed;
  they run the same build path.
- Installed cargo plugins that compile the workspace:
  `nextest`,
  `dylint`,
  `miri`,
  `mutants`,
  `mutants-parallel`,
  `mutants-gate-slot`,
  `fuzz`,
  `ndk`,
  `ndk-test`,
  `bloat`.
- `metadata --no-deps` resolves nothing (measured:
  lock kept),
  so it passes through.

Answered as the pnpm commands they stand for:
`update` and `generate-lockfile` (next section).

Warn after cargo,
never block:
`add`,
`remove` (`rm`).
After a successful run the shadow reads the nearest `Cargo.toml` (or the `--manifest-path` one) and warns on standard error about each dependency that is not `workspace = true`,
`path` or `git`,
 the way `pnpm-shadow` warns about specifiers that are not `catalog:`.
The warning names `mise run sync:files`,
which adopts the literal into `rust-catalog.toml`.

Straight to cargo,
nothing done (measured to keep the lock,
or no workspace involved):
`fmt`,
`clean`,
`locate-project`,
`pkgid`,
`read-manifest`,
`verify-project`,
`config`,
`new`,
`init`,
`search`,
`login`,
`logout`,
`owner`,
`yank`,
`install`,
`uninstall`,
`report`,
`version`,
`help`,
`about`,
`binstall`,
`deny`,
and every unknown word (including aliases from `.cargo/config.toml`).
A help or version word before `--` (`-h`,
`--help`,
`-V`,
`--version`) cancels everything,
and everything after `--` belongs to the program cargo runs.
The command word is found past a leading `+toolchain` and past the value-taking global options `-C <dir>`,
`--color <when>`,
`--config <kv>`,
`-Z <flag>` and `--explain <code>`.
After a successful `cargo --version` or `cargo --help` the shadow prints one line naming itself and the real cargo,
the way `pnpm-shadow` does.

## Mapping `cargo update` to `pnpm update`

The owner's rule (2026-11-03):
`cargo update` under the shadow behaves like `pnpm update`,
pnpm's own policy-aware command
 ([pnpm update][pnpm-update],
[pnpm install][pnpm-install],
read 2026-11-03);
cargo's resolver never chooses freely.
Each flag maps to the nearest pnpm behavior:

- `cargo update` (no names) is `pnpm update`:
  the full catalog update of this workspace.
  Each crate root is re-resolved with `cargo -Zunstable-options generate-lockfile --publish-time <now minus minimumReleaseAge>`
   (only `generate-lockfile` accepts `--publish-time`),
  crates exempt from the release age move to their newest release,
   every newly locked version is checked,
  and a version the policy refuses is set aside for the newest older release in the same caret class that passes
   (`cargo update --package <name>@<version> --precise <candidate>`,
  in at most five rounds).
  Everything is kept only when nothing is refused;
  otherwise every lockfile is restored.
- `cargo update <name>` or `-p <name>` is `pnpm update <name>`:
  cargo runs the command as typed,
  then the same checks and set-aside apply.
- `--recursive` is `pnpm update <name> --depth Infinity`:
  cargo's resolver picks which dependencies move,
  then the policy decides.
- `--precise <version>` is `pnpm update <name>@<version>`:
  still subject to the policy,
  so a refusal fails and nothing is set aside in its place.
  pnpm refuses it for a package that is not a direct dependency (`ERR_PNPM_UPDATE_VERSION_ON_INDIRECT_DEP`),
  and so does the shadow.
- `--breaking` is `pnpm update --latest`:
  cargo's own breaking update runs
  (it rewrites the requirements in the manifests as well as the lockfile,
  so the manifests are part of the transaction:
  a refusal or a dry run puts them back with the lockfiles,
  while a manifest another process changed meanwhile is left alone with a warning),
  then the policy decides,
  and then the same thing pnpm does with a `catalog:` dependency happens.
  In pnpm 12.16.1 the resolver collects the updated range of every catalog dependency of an update that saves
   (`pnpm11/installing/deps-resolver/src/index.ts:280-322`,
  `getUpdatedCatalogs`;
  `wantedDepShouldUpdateCatalog.ts:3-5`,
  false under `--no-save`)
   and writes it to the catalog entry in `pnpm-workspace.yaml`,
  leaving `catalog:` in `package.json`.
  Here every default-catalog `>=` floor in `rust-catalog.toml` whose crate's highest locked version rose is raised to that version,
   and `file-enforcer write` (the `sync:files` task) renders the manifests.
  A plain `cargo update` is `pnpm update --no-save`:
  the lockfile moves,
  the catalog does not.
- `-n`,
  `--dry-run`:
  pnpm update has no dry run.
  The nearest is this tool's own:
  run,
  report,
  restore every lockfile.
- `-w`,
  `--workspace` is `pnpm install`:
  the install step.
- `--locked`,
  `--frozen` is `pnpm install --frozen-lockfile`:
  check,
  fail naming the lock,
  change nothing.
- `-m`,
  `--manifest-path` is pnpm's `--dir`:
  it selects the workspace.
- `--offline`,
  `--ignore-rust-version`,
  `--config`,
  `-Z`,
  `--color`,
  `-q`,
  `-v`:
  cargo-only options,
  forwarded to every cargo call the procedure makes.
  The policy checks still ask crates.io,
  so `--offline` makes cargo offline,
  not the checks.
  Global options are read before the command word as well as after it
  (`cargo --locked update` is the frozen check, `cargo --offline update` forwards `--offline`),
  and `-m <path>` selects the workspace like `--manifest-path`.
  An option that has no mapping is refused by name instead of being dropped from a full update or passed on to a targeted one,
  and `--package=<name>`, `--package <name>` and `-p<name>` all name the package.
- `cargo generate-lockfile` is `pnpm install --lockfile-only`:
  with a lockfile present it is the install step (locked versions kept,
  not re-resolved from scratch);
   without one it is the full update,
  which creates the lockfile under the policy.

## The catalog CLI

```sh
# package/cargo-shadow/cli/README.md
cargo-shadow install                       # every first-party Cargo.lock; frozen in CI
cargo-shadow install --frozen-lockfile     # fail naming the stale roots, write nothing
cargo-shadow install --no-frozen-lockfile  # update stale lockfiles even in CI
cargo-shadow install --dry-run             # report which lockfiles are stale, then restore every Cargo.lock
cargo-shadow update                        # pnpm update for every first-party Cargo.lock under the policy
cargo-shadow update --dry-run              # report what would change, then restore every Cargo.lock
```

The commands that were the mise tasks `rust:install`,
`rust:update` and `rust:verify` are `install` and `update`;
the third task is folded in.
There is no `--reconcile` and no `verify` command,
because pnpm has neither.
pnpm 12.16.1 checks every entry already in the lockfile against `minimumReleaseAge` and `trustPolicy` whenever it installs,
frozen or not,
 unless `trustLockfile` is set,
and remembers a passing lockfile by its hash
 (`pnpm11/installing/deps-installer/src/install/mutateModules.ts:314-343`,
`verifyLockfileResolutionsCache.ts`;
 [pnpm dependency resolution settings][pnpm-resolution]).
`cargo-shadow install --frozen-lockfile` follows both settings of `trustLockfile`,
which `rust-catalog.toml` carries with pnpm's name (default false):

- `trustLockfile = true`,
  the repository's setting by the owner's decision of 2026-10-29 (`doc/decision/rust-dependency-catalog.md`:
  the owner never accepts pull requests that change a lockfile,
  so committed lockfiles are part of the trusted base):
  only the registry versions that no `Cargo.lock` held at the base revision are checked,
  for trust.
  The base is the environment variable `CARGO_SHADOW_TRUST_BASE`:
  CI sets the pushed range's previous commit or the pull request's base,
  and without it the base is `HEAD`,
  so a developer's uncommitted lockfile changes are the ones checked.
  A lockfile that did not exist at the base counts as new in full.
  This is the base check the former `rust:verify --base` made,
  now inside the frozen install.
- `trustLockfile = false`,
  pnpm's default:
  every registry version of every lockfile is verified against the release age and the trust policy,
  and a lockfile that passed is remembered under `.cache/cargo-shadow/verified/` by a SHA-256 of its text and the policy settings.

A non-frozen install and `cargo update` check every version they newly resolve while choosing it,
whatever `trustLockfile` says,
as pnpm's resolver does.
The crates.io histories come from the database dump when more than a hundred crates are uncached (one streaming pass),
and from the API otherwise.
The dump reader refuses a table that is cut off inside a quoted field,
has text after a closing quote or a quote inside a bare field,
has a row of another width than its header,
or has a field, row or crate history over the bounds in `csv.rs` and `dump.rs`,
and nothing is cached from a table it refused.
The bounds are chosen ceilings, not measured on a real dump.
Only crates.io is supported:
a package from any other registry (`registry+` or `sparse+` with another URL) is refused by name,
because every lookup asks crates.io and would approve it by an unrelated release of the same name and version.
The owner list of a crate is asked whenever it can change a trust verdict (an earlier release or the checked one ranks as a first-time publisher without it).
A full update whose strict release-age cutoff cannot resolve (a manifest needs a young release of a crate in `minimumReleaseAgeExclude`)
resolves without the cutoff and then holds every crate that is not exempt to it, setting its young releases aside or refusing.
After a floor raise the order is the pnpm one:
edit the catalog (`cargo-shadow update` and `mise run catalog:tighten`),
 render the manifests (`mise run sync:files`),
then any cargo command,
or `cargo-shadow install`,
makes the lockfiles satisfy them.

## Toolchain and registry settings

The step's own cargo calls run the first `cargo` on PATH that is not the shadow,
under the toolchain `toolchain.lock.json` pins (`.tools.rust.version`).
The pin wins over an inherited `RUSTUP_TOOLCHAIN` (a warning says so),
so every lockfile the step writes has one format.
The user's own cargo command is not changed:
the shadow starts the PATH candidate under its own name,
so rustup's `+toolchain`,
`RUSTUP_TOOLCHAIN` and `rust-toolchain` files select the toolchain as before.
The search skips every candidate that resolves to the shadow,
and a loop guard bound to the process stops a launcher that leads back to it.

Settings (`minimumReleaseAge`,
`minimumReleaseAgeExclude`,
`minimumReleaseAgeStrict`,
`trustPolicy`,
`trustPolicyExclude`,
`trustPolicyIgnoreAfter` and the prune flags) live in `rust-catalog.toml`.
The trust levels are heuristics from crates.io data:
Trusted Publishing,
then a publisher who published an earlier release or is a listed individual owner,
then any other account.
Version histories are cached under `.cache/cargo-shadow/crates-io/` (gitignored);
the API is called at most once per 1.1 seconds,
and an HTTP 429 is retried after its `Retry-After`.

## Limits

- A workspace that has no lockfile is passed through,
  and cargo creates one with its own resolver;
  `cargo generate-lockfile` through the shadow creates it under the policy instead.
- Linux only,
  by the owner's word ("Ignore Win and mac until I say so."):
  the crate does not compile elsewhere,
  because the journal reads `/proc/<pid>` to tell a killed run from a running one.
  A reused process id reads as alive, which only delays a recovery.
- Lockfile writes are lock-free,
  as the owner asked ("never discard user data",
  "use as few locks as possible").
  The pinned nightly has no `--lockfile-path` (checked 2026-11-03),
  so cargo writes the live lockfile.
  A restore replaces a lockfile only if it still holds what this run's cargo wrote,
  through a sibling file and an atomic rename;
  a lockfile another process wrote meanwhile,
  and a root this run never wrote,
  is left alone with a warning.
  Each lockfile is recorded as this run's output the moment its cargo call returned,
  so a failure halfway never guesses.
  The snapshots are journaled under `.cache/cargo-shadow/journal/`,
  and a journal that cannot be started stops the run before cargo runs;
  if a run is killed,
  the next run keeps the leftover lockfile when its new versions pass the policy,
  and otherwise saves it as `Cargo.lock.refused` (or the first free `Cargo.lock.refused.N`,
  so an earlier copy is never replaced) before putting the snapshot back,
  after reading the lockfile again right before the replacement.
  The journal is deleted only when every root it lists was settled;
  an unreadable journal or lockfile and a failed step keep it.
  The window between the last read and the rename is not locked.
- Until the checks pass,
  cargo's output is in the live lockfile,
  so a plain cargo that is not the shadow can build on it.
  A second shadow call never takes that lockfile for approved:
  only a remembered pass of the exact text approves it,
  so the second call checks it itself.
  A private lockfile that the checks approve before it is published is not possible with the pinned nightly.
- The step needs the network only for the policy checks of newly locked versions and for a frozen install's verification of a lockfile it has not seen;
  the lock check and the update run offline (the check drops `--offline` in CI,
  where no registry index is cached).

## Containers and the tasks that run cargo outside the shadow

The shadow is on PATH only where the toolchain plan's links are,
in the mise environment on a development machine.
Containers do not have it:
the podman builds of `package/ide/app`,
`package/music-player/desktop-app` (and its Alpine and Windows GNU variants) and `package/terminal/app` run the image's own cargo,
 so a lockfile is not checked there and a stale one would be rewritten by cargo.
Run a cargo command on the host first,
or `cargo-shadow install`,
before building in a container.
Release workflows that build with `cargo build --locked` fail on a stale lockfile instead.

## Development

```sh
# package/cargo-shadow/cli/README.md
mise run //package/cargo-shadow/cli:test
mise run //package/cargo-shadow/cli:lint:clippy
```

Unit tests sit beside the modules as `*_tests.rs`;
`tests/shim.rs` starts the built binary as `cargo` against a fake cargo.
One install case runs the real cargo against a path-only workspace and skips with a message when no cargo is on PATH.

[pnpm-build]: https://pnpm.io/settings/build
[pnpm-store]: https://pnpm.io/settings/store
[pnpm-resolution]: https://pnpm.io/settings/dependency-resolution
[pnpm-update]: https://pnpm.io/cli/update
[pnpm-install]: https://pnpm.io/cli/install
