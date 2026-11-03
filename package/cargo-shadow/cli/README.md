# monochromatic-cargo-shadow

A shadow for `cargo`,
modeled on `pnpm-shadow`,
that keeps every `Cargo.lock` satisfying its manifests under the trust policy in `rust-catalog.toml`.
It also holds the repository's Rust catalog tools (`install`,
`update`,
`verify`),
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
`verify`,
`--help`,
`--version`.

The binary is named `cargo-shadow`,
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
and then runs the real cargo with the arguments exactly as typed.
It never adds `--locked`,
`--frozen`,
`--offline` or any other argument to the user's command.
Outside a repository with a `rust-catalog.toml`,
and in a workspace without a `Cargo.lock`,
it does nothing.

The install step:

- A lockfile that satisfies its manifests is left byte for byte as it is.
- A stale lockfile is updated offline,
  keeping every locked release the manifests still allow (`cargo update --workspace --offline`,
  then `cargo metadata --locked --offline` confirms).
  Any registry version no lockfile held before is checked against the release-age and trust policy;
  a refusal restores the lockfile and fails the command,
  naming the finding.
- In CI (`CI` set to anything but empty,
  `0` or `false`),
  or when the user's own command carries `--locked` or `--frozen`,
  a stale lockfile fails the command instead,
  naming it,
  and nothing is written.
  That is `pnpm install --frozen-lockfile`.
  In CI the check also drops `--offline`,
  because a fresh runner has no cached registry index.
- A failed or refused install step stops the cargo command (exit 1).
  This differs from pnpm on purpose:
  for `verifyDepsBeforeRun: install` pnpm says that if the install fails "pnpm prints a warning and runs the command anyway"
   ([pnpm settings][pnpm-build],
  read 2026-11-03).
  Running cargo after a refusal would let cargo's own resolver rewrite the lockfile past the policy,
  which is the incident this tool exists to close,
   so the shadow fails closed;
  the owner decides whether to follow pnpm's warn-and-run literally (see the report that came with this package).
  To run cargo past a broken step,
  start the real cargo by its path (the `cargo` under `.cache/toolchain-bin`,
  or `~/.cargo/bin/cargo`).

### The fast path

The check is on every cargo call,
so it resolves nothing when nothing changed.
After a pass,
the step records the size and modification time of the lockfile and of every manifest `cargo metadata` lists as part of the workspace
 (members and path dependencies) in `.cache/cargo-shadow/stamps/`.
A later call that finds all of them unchanged starts no cargo at all.
A missing stamp,
an unreadable file or any difference runs the authoritative check again.
Limits:
a change to `.cargo/config.toml` `[patch]` tables or to a manifest outside the recorded list is not noticed until a recorded file changes.

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
  cargo's own breaking update runs,
  then the policy decides.
  It edits manifests,
  which `sync:files` renders from the catalog;
  raise the catalog with `mise run catalog:tighten` afterwards.
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
cargo-shadow verify                        # registry versions locked since HEAD, against the trust policy
cargo-shadow verify --base <rev>           # CI: the same against another revision
```

The commands that were the mise tasks `rust:install`,
`rust:update` and `rust:verify` are these three.
There is no `--reconcile`,
because pnpm has no such word.
`verify` has no pnpm command of its own:
it checks only registry versions that no `Cargo.lock` held at the base revision,
 the counterpart of pnpm's `trustLockfile: true` ([pnpm dependency resolution settings][pnpm-resolution];
it makes `pnpm install` skip re-applying `minimumReleaseAge` and `trustPolicy` to lockfile entries),
because the owner never accepts pull requests that change a lockfile.
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

## Not ported, and limits

- The crates.io database dump path (`https://static.crates.io/db-dump.tar.gz`,
  used above 100 uncached crates) is not ported.
  Every uncached history comes from the API,
  at 1.1 seconds each:
  a fully cold check of all locked crates takes tens of minutes instead of the 914 seconds measured with the dump on 2026-10-29.
  The CI cache of `.cache/cargo-shadow/crates-io` keeps warm runs fast.
- A workspace that has no lockfile is passed through,
  and cargo creates one with its own resolver;
  `cargo generate-lockfile` through the shadow creates it under the policy instead.
- The update is a transaction by snapshot and restore of the live lockfiles.
  A concurrent writer between the snapshot and the restore would lose its change,
  and a process killed in between leaves the unchecked lockfile;
  neither is guarded.
- The step needs the network only for the policy checks of newly locked versions;
  the lock check and the update run offline.

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
