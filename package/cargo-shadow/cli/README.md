# monochromatic-cargo-shadow

EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE

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
  `resolving/npm-resolver/src/pickMatchingVersion.ts`,
  `pickRespectingMinReleaseAge`),
  and the owner's decision of 2026-11-03 for a committed lockfile:
  a merge that brings `serde 1.0.230` published three hours ago under a one day limit gives `serde 1.0.229` in `Cargo.lock`,
  and a trust downgrade gives the earlier release with the earlier trust level.
  The refused release never runs;
  the older copy does.
  Under `trustLockfile = true` this fires for the versions new since the base revision;
  a local run's base is the upstream branch of the current branch (`@{upstream}`,
  else `origin/main`,
  else `origin/master`,
  else `HEAD`),
  so a merge that is committed but not pushed is still new,
  and CI keeps `HEAD` (or `CARGO_SHADOW_TRUST_BASE`,
  which wins everywhere).
  The release age and the trust policy both apply to those versions,
  on the local and the frozen path.
- If no older release passes and fits,
  the refusal stands:
  nothing is left changed and your cargo command is not run.
  That is the only case in which the step stops the command because of the policy,
  local or frozen.
- In CI (`CI` set to anything but empty,
  `0` or `false`),
  or when the user's own command carries `--locked` or `--frozen`,
  the install is frozen:
  a stale lockfile fails the step naming it,
  and a release the policy refuses stops the command (pnpm's frozen install fails on a rejected lockfile entry before any fetch),
  nothing is written and nothing is set aside.
  That is `pnpm install --frozen-lockfile`.
  In CI the check also drops `--offline`,
  because a fresh runner has no cached registry index.
- When the step fails for an operational reason (no network,
  a registry lookup that fails,
  a cargo error,
  an unreadable catalog),
  or a frozen step finds a stale lockfile,
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
and the remembered pass is keyed by the commit the base resolves to,
so a base that moves is a different base.
The base is read from the repository's files without starting git
(owner rules of 2026-11-01:
"Invoke real git as rarely as possible."
and "embed instead of spawning"):
`HEAD`,
including a linked worktree's own directory and its `commondir`,
`branch.<name>.remote` and the first `branch.<name>.merge` from the system,
user and repository configuration,
the fetch refspecs of the remote,
the remote-tracking reference from loose files or `packed-refs`,
and the type of the object it names,
from a loose object header or a pack index and pack entry.
The configuration is read with the lexer of Git's `config.c`
(a comment never continues onto the next line,
a quoted subsection runs to its closing quote,
a key without `=` has no value).
Git is started once for everything the files cannot answer exactly,
and that one start (`git cat-file --batch-check`) answers every candidate base:
an explicit `CARGO_SHADOW_TRUST_BASE` (git verifies that it names a commit),
a configuration `include` or `includeIf` in any of the configuration files,
any `extensions.*` variable (reftable,
worktree configuration,
other object formats,
extensions this reader has never seen),
a repository format other than 0 or 1,
a `GIT_DIR`,
`GIT_COMMON_DIR`,
`GIT_WORK_TREE`,
`GIT_NAMESPACE`,
`GIT_CONFIG*`,
`GIT_CEILING_DIRECTORIES`,
`GIT_DISCOVERY_ACROSS_FILESYSTEM`,
`GIT_OBJECT_DIRECTORY`,
`GIT_ALTERNATE_OBJECT_DIRECTORIES`,
`GIT_REPLACE_REF_BASE`,
`GIT_NO_REPLACE_OBJECTS` or `GIT_REF_PARANOIA` variable,
a symbolic-link `HEAD` or reference (`core.preferSymlinkRefs`),
a reference outside `refs/heads`,
`refs/remotes` and `refs/tags` (worktree-private references),
an invalid,
negative or unusual fetch or push refspec of any remote,
a `remote.*` or `branch.*` key that is written without a value,
is not a boolean where it must be,
or has no class here,
a missing upstream of the remote `.` (Git expands such names),
a repository,
`.git` entry or Git directory that another user owns,
or a `.git` link (Git's own safe-directory decision,
inspected with `lstat`),
an empty `commondir`,
a Git that cannot report its system configuration,
a symbolic reference where a commit is expected,
an object that is not seen to be a commit stored whole and accepted by Git's commit parser (a tag,
a malformed `parent` line,
a zlib stream without its end,
a pack that disagrees with its index,
a non-regular file,
a missing object,
a deltified commit),
`objects/info/alternates`,
`info/grafts` or a `refs/replace/` reference,
a legacy `remotes/` or `branches/` remote file,
and a short name that a tag or branch shadows (`origin/main`),
or that also resolves through `refs/remotes/<name>/HEAD`.
The system configuration is the file the Git that will answer reports (`git var GIT_CONFIG_SYSTEM`):
that one question is a Git start,
asked once per Git program and remembered in `.cache/cargo-shadow/git-system-config`
(keyed by the program's canonical path,
size and modification time);
a Git that cannot report it makes the reader decline.
The reader is tested against git itself
(`src/git_differential_tests.rs`:
each case builds a disposable repository with an isolated home,
asks `/usr/bin/git` which base it chooses,
and requires the reader to decline or to agree;
`src/git_fallback_tests.rs` runs the whole choice,
fallback included,
with the same Git and home).
When the batched start stops before every candidate was answered,
each candidate is asked on its own,
so an unanswered query is never read as a missing revision.
Measured on 2026-11-03 (a throwaway repository with an upstream,
`cargo metadata --offline` through the release binaries,
4 arms alternated per round,
40 rounds each,
medians in ms with the standard deviation):
with `git` on PATH being the system git,
cargo alone 40.9 (2.6),
the build that started git 44.7 (2.1),
this build 43.2 (2.2) and the same build again as the control 43.1 (2.3);
with `git` being the repository's wrapper in front of the system git,
cargo alone 40.0 (1.4),
the build that started git 45.8 (1.6),
this build 42.0 (2.1) and its control 41.3 (1.8).
The saving is 1.5 to 4 ms per call;
the control arms agree within a quarter of a millisecond.
The figures in the list below were measured before this change.

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
  while a manifest another process changed meanwhile is left alone with a warning;
  the manifests are journaled,
  so a killed run is recovered for them too;
  and a failure of the `file-enforcer` sync after the catalog write restores the catalog,
  the manifests and the lockfiles),
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
  (`cargo --locked update` is the frozen check,
  `cargo --offline update` forwards `--offline`),
  and `-m <path>` selects the workspace like `--manifest-path`.
  An option that has no mapping is refused by name instead of being dropped from a full update or passed on to a targeted one,
  and `--package=<name>`,
  `--package <name>` and `-p<name>` all name the package.
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
  and without it the base is `HEAD` in CI.
  A local run without it compares with the upstream branch (`@{upstream}`,
  else `origin/main`,
  else `origin/master`,
  else `HEAD`),
  so the lockfile changes that are not pushed yet are the ones checked.
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
or has a field,
row or crate history over the bounds in `csv.rs` and `dump.rs`,
and nothing is cached from a table it refused.
The bounds are chosen ceilings,
not measured on a real dump.
Only crates.io is supported:
a package from any other registry (`registry+` or `sparse+` with another URL) is refused by name,
because every lookup asks crates.io and would approve it by an unrelated release of the same name and version.
The owner list of a crate is asked whenever it can change a trust verdict (an earlier release or the checked one ranks as a first-time publisher without it).
A full update whose strict release-age cutoff cannot resolve (a manifest needs a young release of a crate in `minimumReleaseAgeExclude`)
resolves without the cutoff and then holds every crate that is not exempt to it,
setting its young releases aside or refusing.
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
and every candidate that is a toolchain launcher whose record (`<launcher>.launch`,
its `real` binary)
names the shadow,
directly or through further launchers,
before it is run:
the shadow does not start itself a second time.
A shadow of any other version (or of another crate:
`mpm-shadow`,
`pnpm-shadow`,
`search-shadow`) is skipped the same way,
by what the file is and not by its path:
A shadow never starts a shadow of any version as its real tool,
whatever the path or the version of the candidate
(owner,
2026-11-07:
"We should never invoke ourselves recursively.
That's bad design.").
Every shadow program (`cargo-shadow`,
`mpm-shadow`,
`pnpm-shadow` and `search-shadow`) carries the same identity text in the ELF section `.monochromatic_shadow_identity`
(`src/shadow_identity.rs`,
a byte-identical copy in each crate;
`package/search-shadow/cli/tests/shadow_identity_copies.rs` fails when the copies differ).
The lookup skips a candidate that carries it,
or a launcher whose record leads to one,
without starting it,
and the start checks the open file again.
A shadow that finds `MONOCHROMATIC_SHADOW_STARTED_BY` left for its own process or its direct parent
was started by a shadow:
it does nothing,
prints one line and exits with status 2.
The full description,
the measurement and the cost are in `package/search-shadow/cli/README.md`,
section "Never starts a shadow",
and the incident in `doc/troubleshooting/search-shadow-copies-start-each-other-without-end.md`.

The vetting of the chosen program (`vet` in `src/real_binary.rs`) refuses a program that is a shadow while the file is open,
and the program is started through that open file.
The real cargo that the install step starts inside its contained runner (`src/cargo_run.rs`) sets no arrival marker:
it is chosen by the same lookup that skips shadows by identity,
and it keeps `vet` and the repeated check before its start.
Two arrival stops exist:
the guard value `CARGO_SHADOW_GUARD` (cargo-shadow's own,
with the long message below) and the marker above
(any shadow,
one line);
both exit with status 2.
A cargo-shadow that finds it was started by another cargo-shadow's own start of the real cargo (a script or launcher named `cargo`
with no `.launch` record beside it that leads back) stops at once with exit status 2,
before doing any work,
and prints what happened,
the chain,
the PATH it saw,
every `cargo` on it,
and two fixes with lines that can be copied as printed:
put the real cargo's folder
first on PATH,
or write the `.launch` record beside the launcher.

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
  because the replace protocol uses `renameat2` with `RENAME_EXCHANGE` and the journal uses `flock`.
- Lockfile writes use as few locks as possible,
  as the owner asked ("never discard user data",
  "Use as few locks as possible.").
  The pinned nightly has no `--lockfile-path` (checked 2026-11-03),
  so cargo writes the live lockfile.
  What decides the outcome is not attribution.
  A restore never unlinks an inode that was live at a path:
  the file it displaces,
  and the replacement it exchanged in,
  each get a surviving name first
  (`.cache/cargo-shadow/displaced/` or beside the target),
  so a write through a descriptor opened earlier still lands in a named file,
  and a file that was misattributed is moved aside with a warning,
  not lost.
  The inode change time (`ctime`) of a file against the window of the cargo call
  (a window that ends before it starts counts as the call's)
  only chooses between "put the snapshot back and keep the displaced file" and "leave the file alone";
  it is a hint,
  not proof of authorship,
  and a clock step or a coarse file system can make it wrong in either direction.
  Wrong one way,
  a foreign file is kept under a named copy.
  Wrong the other way,
  cargo's output stays live and the step stops the command (`Unsettled`) instead of building it.
  A filesystem that cannot exchange fails the restore visibly and overwrites nothing;
  a file that vanished after the comparison stays vanished.
  A lockfile or manifest that cannot be read as text is an error and is left alone,
  never an empty file.
  Each lockfile is recorded as this run's output the moment its cargo call returned (even a failing one);
  a rollback that cannot complete stops the command whether cargo failed or its output was refused,
  so the `--locked` fallback never builds it.
  Every run that changes lockfiles takes one exclusive `flock` on `.cache/cargo-shadow/lock`,
  for the whole workspace,
  without waiting,
  and holds it until it finishes:
  a second run fails at once with the holder's process id and the command to run again,
  and a recovery takes the same lock for its whole duration,
  so no transaction starts while files are being put back.
  This is the one lock,
  a pick open to the owner's veto;
  its visible cost is that two lockfile-changing runs in one workspace do not run side by side.
  The lock is close-on-exec,
  so a program that cargo starts (a compiler cache daemon,
  say) cannot keep it held forever.
  What the wrapper promises,
  what it assumes,
  and which test shows each:
  `doc/planning/cargo-shadow-guarantees.md`
  (guarantees,
  assumptions a to e,
  the weaker path,
  the acceptance list and the coverage map).
  A killed wrapper's cargo is found another way,
  by the delegated scope it ran in (round 8;
  round 7 used a group the wrapper entered
  around each spawn,
  and rounds 5 and 6 a process group and a subreaper,
  which failed to show that every process had stopped):
  the program is started as `systemd-run --user --scope --unit=<name> --property=Delegate=yes --collect -- <program>`,
  so the user manager,
  not a thread of this program,
  moves it into a scope of its own before it runs,
  and the wrapper never changes
  its own group.
  The name carries a 128-bit random token and the manager is asked where its slice is (`systemctl --user show app.slice`),
  once per
  process.
  Before the program starts the wrapper writes the scope's path,
  its cgroup namespace and the mount id and inode of the directory that
  holds it to the journal's `groups` file and syncs it (a program whose record cannot be written is not started).
  A recovery or a new transaction stops while any journal of the workspace names a scope with a thread left (`populated 1`),
  or a
  record it cannot read or understand,
  or a journal that lost its record while other files remain,
  or a journal or a scope seen
  with another mount id or inode or in another cgroup namespace than the record was made in (assumption c);
  it names the journal.
  A recorded scope counts as gone only when the run also recorded its end (or its launcher is recorded and gone) and the directory that held it is seen as it was
  recorded,
  because the manager removes a scope only when it is empty (probed).
  The end is a `settled <scope path>` line that the run appends to the journal after it reaped the launcher and showed the scope
  empty.
  Until then neither an absent scope nor an empty one counts:
  the manager creates a scope's control group and attaches the launcher in
  two steps,
  and a look that finds the scope absent or empty says nothing about a launcher that has not finished entering it.
  Right after the wrapper spawns the launcher (which becomes the program) it appends
  `launcher <pid> <start time> <pid namespace> <scope path>` to the journal,
  without a sync (a lost or torn line is only a missing
  proof).
  The question is answered through one procfs descriptor that is opened once and checked (a procfs root that numbers this process
  as its own namespace does),
  a pidfd and the `Pid:` line of its `fdinfo`,
  which show that the procfs numbers the launcher's process
  as the launcher's id,
  and an `openat2` lookup beneath that descriptor that crosses no mount and follows no link;
  a procfs of another
  namespace,
  a lookup that cannot be made (a kernel before 5.6) and any number other than the `-1` of an ended process give
  "unknown",
  never "gone".
  A program that deliberately changes mounts or `/proc` while a run or a recovery is in progress is outside what the wrapper
  promises (assumption a of the contract),
  like a same-user process that writes `Cargo.lock` directly.
  A recovery asks about that process before it reads the scope:
  a launcher that no longer exists (no process has its id,
  in the
  recorded process id namespace) can never join the scope,
  so an absent or empty scope is final and the journal recovers by itself;
  a launcher classified alive (the id with the recorded start time) blocks,
  and no acknowledgement overrides that.
  The id with another start time is unknown,
  not gone:
  the start time in the stat file is relative to the reader's time namespace,
  and nothing recorded can show that the reader is in the writer's (namespace numbers are eligible for recycling after the namespace's last reference is released),
  so a reused id whose start
  time differs from the record waits for the acknowledgement like the cases below.
  What the line cannot prove waits for `cargo-shadow acknowledge <journal directory>` (a person's assertion that every process of
  that run has stopped):
  another process id namespace,
  a journal with no launcher line (an older version,
  a failed append,
  a wrapper
  killed between the spawn and the append),
  a line for a scope the journal does not list,
  a torn line,
  a launcher that cannot be
  inspected (including a procfs of another namespace),
  a recovery in another process id namespace than the line,
  and an id that a
  process has with another start time.
  A scope that has a member blocks even then.
  Nothing waits in-process.
  A record that cannot be written leaves the journal unsettled,
  which is the safe side.
  The journal's view record also holds the machine id (`/etc/machine-id`) and the boot id
  (`/proc/sys/kernel/random/boot_id`) of the system that made it,
  written before any record of a scope or a process group.
  Both ids must have their format (the boot id a lowercase UUID,
  the machine id 32 lowercase hexadecimal digits),
  a record with a
  repeated field or an unknown line has no identity,
  and the boot id file must be the kernel's own:
  opened without following a link,
  served by procfs and not the root
  of a mount of its own (checked and read on one descriptor),
  because a bind mount can put another file over it without a reboot (`systemd-nspawn` does it on purpose).
  An identity that fails any of this is no identity and keeps the refusal.
  A recovery on the same machine id with a different boot id treats the run's process liveness as discharged,
  because a reboot
  ended every process of the run (assumption e:
  one machine at a time,
  no checkpoint and restore);
  it then does not compare the
  mount and cgroup views,
  which belong to the boot that ended,
  and it asks for no acknowledgement for the containment.
  Journal enumeration,
  snapshot validation,
  preservation and the mandatory finish still run,
  and a recorded scope that has a member
  after the reboot still blocks.
  A different machine id,
  an unreadable or malformed identity on either side,
  a journal without one,
  or the same boot id (suspend
  and resume keep it) keep the refusal.
  A journal directory that holds nothing at all is what a cut-off `begin` leaves and is removed.
  The scope is stopped by killing the direct child,
  reaping it,
  and writing `cgroup.kill`,
  and then each listed process gets
  `SIGKILL` through a pidfd,
  because `cgroup.kill` alone left a thread group alive whose leader had already exited (measured on
  Linux 7.2.15);
  the stop ends only when the kernel says `populated 0` or the absence of the scope is verified,
  and every failure on
  this path after the program started is "not quiescent".
  `systemd-run` reads the command it starts as a unit's command line (it turns two dollar signs into one and replaces a braced
  variable name),
  so every argument is encoded for that before it is handed over,
  and a command with a cleared environment runs
  through `env --ignore-environment`.
  The scope keeps the resource limits of the control group that started the run.
  The manager makes the scope under its own slice,
  not under the caller's group,
  so a build inside a capped `systemd-run --user --scope` would otherwise run uncapped (found by an audit of every `systemd-run`
  builder in the repository;
  repository rule:
  every command that is not trivial runs under CPU,
  memory,
  process-count and
  disk-weight limits,
  in every agent).
  Once per process,
  the discovery reads `MemoryMax`,
  `CPUQuota`,
  `TasksMax` and `IOWeight` (from `memory.max`,
  `cpu.max`,
  `pids.max`
  and the `default` line of `io.weight`) on the caller's own group and on each ancestor up to,
  and not including,
  the deepest ancestor
  it shares with the slice,
  whose limits reach the scope through ancestry anyway.
  The launcher passes the tightest finite value of each as `--property=` arguments.
  A limit nobody sets stays absent,
  and no size is ever invented.
  An I/O weight at or above the kernel default is not carried,
  so the scope may be throttled more than its caller and never less.
  A quota is passed as a percent with two decimals,
  rounded down,
  because `systemd-run` refuses `CPUQuotaPerSecUSec=` as a property.
  A limit file that cannot be read refuses the route:
  the run takes the weaker path below,
  which stays in the caller's own group,
  so
  its limits still apply,
  and the one-line notice names the limits.
  The precedent is `package/wg/cli-quicker-exempt/src/inherited_limits.rs`;
  `src/inherited_limits.rs` is a port of its four limits,
  not a dependency (that crate is a binary without a library,
  and each crate stands on its own),
  and it leaves out `memory.high`,
  `memory.swap.max` and the per-device `io.max` limits.
  Assumption a of the contract:
  the run's processes do not leave the scope and do not arrange workspace writes through a service outside
  it;
  a same-user process that does is outside what any wrapper can stop.
  Where no scope can be had (no `systemd-run`,
  no reachable user manager,
  no cgroup v2 mount,
  an unknown slice,
  a kernel before 5.14),
  the run goes on in a process group of its own and prints one line naming the missing capability.
  Its journal holds `pgroup starting` before the child starts and the process group after,
  and every recovery and every new
  transaction refuses such a journal,
  naming it,
  until a person runs `cargo-shadow acknowledge <journal directory>`,
  which asserts
  that every process or worker of that run that can modify the workspace has stopped.
  The acknowledgement is written under the workspace lock to that one journal;
  it overrides only the uncertainty of the containment
  (a weaker-path record,
  a record that is damaged or missing,
  a view that differs),
  never a recorded group that is observed to have a
  member,
  and the recovery's other checks still run.
  It is the file `acknowledged` in the journal,
  published whole by an atomic rename (a header line,
  the process id and the time of
  the acknowledging command,
  each ended by a line feed) with its directory synced;
  a reader counts it only as that whole record and
  only after making the file and its directory entry durable itself,
  so a partial file,
  or one the reader cannot make durable,
  does not count.
  The command tries the workspace lock and never waits for it,
  and an unfinished publication leaves the journal unacknowledged.
  The discovery of the manager's slice is bounded (the answer of `systemctl` is read under a deadline,
  a descendant that holds
  its pipes does not extend it,
  and the killed program is reaped within what is left of that same deadline),
  the route cache and the registry of
  unsynchronized directories are consulted without blocking,
  and the answer when either is busy is the conservative one.
  When a crate root or the central folder of kept files cannot be listed,
  the run prints that the number of kept files is unknown
  and where to look;
  it never refuses and never takes an unreadable place as small.
  A trivial run took 23.6 ms median in the test build,
  which asks the manager every time (18.4 minimum,
  33.5 maximum,
  30 runs),
  and
  6.3 ms in the weaker path;
  the production build asks once per process,
  so its figure is lower by the cost of the question,
  which was
  not measured separately.
  Every run keeps a journal under `.cache/cargo-shadow/journal/<unique name>/`,
  created exclusively (a journal is never reused or truncated),
  with the snapshots synced and a `ready` record written last that lists every file with its length;
  a ready transaction's payload is frozen.
  A recovery decides every root of the journal:
  it keeps the leftover lockfile when its new versions pass the policy,
  and otherwise saves it as `Cargo.lock.refused` (or the first free `Cargo.lock.refused.N`)
  before putting the snapshot back.
  A root that failed and a root with no decision are never restored,
  so a lockfile and its manifests are never split.
  A `--breaking` run's manifests are in the journal (lossless path bytes,
  a versioned record,
  required by the ready record):
  a ready journal with a missing,
  short or unlisted file is kept.
  A journal with no ready record is removed only when every lockfile equals its snapshot (the copies are then redundant);
  otherwise it is kept and named,
  because a lost ready record looks the same.
  A replacement temporary left by a killed run is reported and never deleted.
  Finishing a run has three phases that fail differently (round 7):
  the outputs are forced to disk (the owner's "Force at leaving";
  each lockfile and journaled manifest and its directory are
  synced);
  the completion marker is published (the journal is renamed to `.done` and the parent is synced);
  the journal is
  removed.
  A failure of the first two stops the command and is never hidden behind another error (an operational error of the install
  step,
  which would let the shim run the command again with `--locked`,
  is joined with it);
  only the removal of a journal that
  is already marked done is a warning.
  "Force at leaving,
  else background" has no background carrier in a command that exits,
  so cargo-shadow forces what must be
  durable at its exit barrier and syncs nothing else (the coordinator's pick of round 7,
  open to the owner's veto).
  A removal that stopped partway is finished by the next recovery,
  which syncs the parent before it deletes anything.
  Data is synced before it is relied on:
  the displaced file (a failed data barrier is an error,
  never a settled outcome),
  every directory from the resolved kept directory up to the first directory that existed before the transaction (assumption b of
  the contract),
  every time a file is linked into it
  (the directory that holds the replaced file is not an anchor just because the file is there,
  since its own entry in its parent may
  not be durable;
  the path is resolved first,
  so a symbolic link and a relative path name the directories that really hold the
  entries;
  a pre-existing directory and its path are assumed durable,
  except one this process knows it left unsynchronized after a
  failed sync,
  which the walk goes through;
  nothing above the anchor is opened).
  Power-loss behavior depends on the filesystem honoring `fsync` and is not tested by fault injection.
  The kept directory is never capped and never emptied by the tool.
  Owner answer of round 6,
  "Warn past a count,
  never refuse":
  past 1,000 files every run prints how many files are kept there,
  their total size and the folder to review,
  and cargo keeps working.
  The 1,000 is the warning point,
  kept from round 5 and open to veto;
  the rate at which files are kept has not been measured.
  Before removing a kept file,
  make sure no program still has it open,
  because a program that holds it keeps writing to the inode,
  and a copy made earlier misses those writes.
- Nothing waits forever
  (owner rule of 2026-11-03:
  "nothing should ever wait forever or indefinitely").
  Locks are tried,
  never waited for.
  Every child process runs in a delegated scope of its own under a bound (above,
  "A killed wrapper's cargo";
  the weaker path is a
  process group).
  Every wait this program controls has a deadline:
  the question to the manager,
  the lifetime of a command,
  the draining of its pipes,
  the reaping of its child,
  the stop of a scope,
  and every lock (tried,
  never waited for).
  An ordinary blocking call of the operating system (`read`,
  `write`,
  `fsync`,
  `rename`) has no deadline of its own;
  the contract
  assumes such calls make progress,
  as the owner's "Assume the final disk save would never be cut short."
  does for the disk.
  At the bound the group and every process in it are killed and the run waits for `populated 0`;
  the error names the call and the command the user ran (quoted so it can be copied);
  after a normal exit any process left is killed too.
  A process that cannot be killed (an uninterruptible wait) is "not quiescent":
  install and update then stop with the lockfiles
  neither rolled back nor accepted and the journal kept,
  and the command is not run.
  Output is returned only when it is complete,
  and the pipe workers are cancelled when the call ends.
  Two runs of one process do not wait for each other and share no lock:
  each has its own scope.
  The bounds,
  with what was measured in this repository on 2026-11-04 (load average 6.2 to 6.9 on 32 CPUs,
  3 runs each):
  a cargo call 30 minutes (offline `cargo update`/`cargo metadata` took 0.05 s to 0.91 s;
  the cold-index case is unmeasured),
  a Git call 60 seconds (0.006 s to 0.018 s warm;
  cold disk and promisor fetch unmeasured),
  `file-enforcer write` 5 minutes (about 0.6 s),
  the dump download 1 hour (the dump is 2.0 GiB,
  so an assumed floor of about 595 KB/s),
  a registry request 120 seconds (unmeasured),
  a pause for an HTTP 429 at most 2 minutes (unmeasured:
  no `Retry-After` was observed),
  name searches 10,000 (kept names) and 100 (journals,
  taken names;
  collisions not counted).
  A per-call bound is not an end-to-end bound:
  five registry requests with four pauses can take about 18 minutes.
  The owner's rule justifies that a bound exists,
  not its number;
  each number is a measurement named beside it in the source,
  or a
  delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back" (handover "Picks made
  under the owner's delegation"),
  open to veto.
  The one exemption is the user's own command that the shim starts after the checks (`cargo build`,
  `cargo run` and the like):
  it runs as long as the user lets it,
  and cargo-shadow does nothing while it runs.
- `cargo add` and `cargo remove` run cargo first and then only warn about dependencies that skip the catalog.
  They run no code of the dependency,
  and the policy is enforced by the install step of the next command that resolves or builds,
  which checks the lockfile they left.
  The policy is not applied at the moment of the edit;
  that is a decision,
  not an oversight.
- Until the checks pass,
  cargo's output is in the live lockfile,
  so a plain cargo that is not the shadow can build on it.
  A second shadow call never takes that lockfile for approved:
  only a remembered pass of the exact text approves it,
  so the second call checks it itself.
  A private lockfile that the checks approve before it is published is not possible with the pinned nightly,
  and the window is accepted and documented here (owner rule "Use as few locks as possible";
  delegated pick of 2026-11-03).
- The step needs the network only for the policy checks of newly locked versions and for a frozen install's verification of a lockfile it has not seen;
  the lock check and the update run offline (the check drops `--offline` in CI,
  where no registry index is cached).

## Keeping the build folders lean

`cargo-shadow lean` removes what nothing needs from the build folders of this machine.
Plan,
measurements and the owner's decision of 2026-11-07 ("A to F,
B allowed (Recommended)"):
[`doc/planning/lean-build-dirs.md`](../../../doc/planning/lean-build-dirs.md).
The numbers (grace periods,
idle periods,
retention,
caps) are in `src/lean_policy.rs`,
each beside the measurement it rests on,
and are coordinator picks with the owner's veto open.

Six mechanisms
(the order a run takes them in is its own section below):

- A,
  toolchain-keyed build folders.
  For a host cargo run the shadow sets `CARGO_BUILD_BUILD_DIR` for the child cargo only,
  to `<base>/toolchain/<toolchain>/<xx>/<14 hex>`,
  where `<base>` is what precedes `{workspace-path-hash}` in `build.build-dir` of `$CARGO_HOME/config.toml`,
  the toolchain is a leading `+name` argument or `RUSTUP_TOOLCHAIN`,
  and the key is SHA-256 of the canonical workspace root.
  It sets nothing when the user already set `CARGO_BUILD_BUILD_DIR`,
  when no `build.build-dir` template ends in `{workspace-path-hash}`,
  or when no toolchain is known.
  A ledger file `.cargo-shadow-workspace` in the folder names the workspace.
  Runs that bypass the shadow (an IDE starting cargo itself) and container builds keep their own location
  and are reached by B to F.
  The container runners (`package/ide/app`,
  `package/nested-wayland-session/cli`) are not changed.
- B,
  the sweep of units an older compiler built.
  In each profile directory the newest compiler group (the `rustc` field of the fingerprints) and every group invoked within the grace period stay;
  the rest,
  with the incremental directories written before the newest group first ran,
  goes.
  A profile directory with one compiler group is left alone.
  The reader of fingerprint files is the one the owner allowed
  (`doc/decision/cargo-build-dir-host-cache.md`,
  amendment of 2026-11-07T22:19:43-05:00);
  a layout it does not know is left untouched and reported.
- C,
  verified-idle trees of the host build base.
  A tree goes only when its workspace is gone (ledger,
  or a lead from dependency information),
  or its toolchain is retired,
  or nothing identifies it and it has been idle longer;
  no other tree's workspace lies inside it;
  every profile directory yields its build locks;
  and no process uses anything under it.
  "Other tree" means every tree of the base,
  including the ones a pass that continues after a recorded position
  does not judge again.
  A tree is also kept while the workspace its own ledger names lies inside another tree of the base and still exists,
  even when its toolchain is retired:
  that ledger is the only thing that keeps the other tree from being removed,
  and the ledger goes with the tree,
  so removing it first would let a later pass delete the live workspace inside that other tree.
- D,
  retention of verification run records under `target/verification`:
  records (`RECORD_PREFIXES` in `src/lean_run_records.rs` names every program of this repository that writes one,
  with its prefix:
  `campaign-`,
  `clippy-`,
  `container-`,
  `coverage-`,
  `embedding-fuzz-`,
  `fuzz-clippy-`,
  `git-image-`,
  `guard-observer-`,
  `guard-protected-`,
  `guard-without-`,
  `mutation-`,
  `native-`,
  `pathname-mutation-`,
  `planted-`,
  `processors-mutation-`,
  `release-test-`,
  `startup-mutation-`,
  `test-`,
  `windows-`) older than the retention age go,
  and the oldest go while the directory is over its cap.
  Anything else in the directory is never touched:
  a rendered handover,
  a manifest,
  a cache directory,
  and a log a person redirected there by hand,
  which has no producer here and so no prefix.
  The sweep runs from `cargo-shadow lean run` over every checkout's directory,
  and from `cargo-shadow lean records <directory>...` over the directories one producing task names:
  the task that writes records calls that first,
  so the directory is bounded without the task reimplementing the retention.
  A sweep of one package's directory costs a read of that directory and one `/proc` scan;
  the machine-wide pass measured 398 s on 2026-11-08 (359 checkouts,
  119 verification directories,
  nothing due for removal),
  which is why a producing task names its own directory instead of running the whole pass.
- E,
  size caps per profile directory with oldest-first eviction.
  Entries touched within the grace period are never evicted.
- F,
  a systemd user timer.
  `cargo-shadow lean timer-install` writes `cargo-shadow-lean.service` and `cargo-shadow-lean.timer` into
  `~/.config/systemd/user` and enables the timer.
  The timer fires once a day at 04:30 local time and at no other moment:
  `Persistent=false` (a missed run is not made up at boot or login),
  the service has no `[Install]` section,
  and nothing in cargo-shadow starts itself.
  The service runs `<repository>/.cache/toolchain-bin/cargo-shadow lean run` with disk weight 5,
  two threads of CPU,
  4 GiB of memory and a start timeout above the run's own deadline,
  and shows nothing on the desktop.

Beyond the build folders,
`lean run` also retires dated nightly toolchains that no checkout pins
(`toolchain.lock.json`,
`.cache/toolchain.env.json`,
`rust-toolchain` files,
rustup's default and overrides),
and removes Podman images and volumes that no container uses and no current source names,
as the user and never as root.
A container in any state uses its image,
so an image a container holds is kept;
that includes a Buildah working container,
which only `podman ps --external` lists.
An image is asked for by every tag it carries,
because Podman refuses an image with more than one tag when it is named by its id,
and a refusal that says the image is not known counts it as gone,
because removing one image also removes an ancestor that only it referenced.

### The order a run takes its mechanisms in

One deadline covers the whole run,
so the run takes the cheapest mechanism first and the units pass last:
run records,
toolchains,
Podman,
idle trees,
then the older-compiler sweep and the size caps.
Measured in the first run (2026-11-07T23:52:28-05:00 to 2026-11-08T01:24:39-05:00):
2.5 minutes for records and toolchains together,
20.5 minutes for Podman,
29 minutes for the idle trees,
and hours for the units pass,
which reached about 700 of several thousand profile directories in 40 minutes.
In the plan's order that pass came second and starved the Podman pass of the deadline.
What a deadline cuts off waits for the next run,
and each of the two long passes continues where it stopped:

- The idle-trees pass and the units pass record the last tree and the last profile directory they judged,
  in one line each,
  in `lean-progress` under `$XDG_STATE_HOME/cargo-shadow`,
  or under `$HOME/.local/state/cargo-shadow` when the variable is unset.
  The next run starts after that key,
  and a pass that finished its list leaves no key,
  so the run after it starts at the front.
- A record that is missing,
  unreadable,
  or of a shape this version does not know means the front of the list,
  never a failed run,
  and the run says which of the two happened.
- The key is a path,
  not a count,
  because the list changes between runs:
  a key whose tree was removed in the same pass still orders the list.
- The file is replaced by an atomic rename and a dry run never writes it,
  so a run killed in the middle leaves the position it last recorded,
  and `lean run --dry-run` never moves it.
- Neither name of the record is ever opened through a link or waited on as a pipe.
  A symbolic link,
  a hard link (a second name for a file this run does not own),
  or a pipe at `lean-progress.new` is removed and this run writes its own file there,
  so the text never lands in the file a link names and no open waits for the other end of a pipe;
  a pipe at `lean-progress` itself is read for what it already holds,
  which is not this version's header,
  so the run starts at the front and says so.
  Both files are created readable and writable by this user alone (`0600`).
  A name that cannot be used after that is reported once and costs the run its position,
  never the run itself.
- A resumed idle-trees pass still holds the whole base's workspaces as guards.
  The trees before the recorded position are not judged again,
  but their ledgers are read,
  because a tree this pass judges may hold the workspace one of them records:
  without them a resumed pass removed a tree that a pass from the front keeps,
  and the live sources inside it with it.
  Only the ledger is read for those trees,
  never the dependency information,
  so the guard costs one small file read each:
  measured 2026-11-11 in a fixture base on this host,
  4,000 trees with a ledger on every one took 62 ms
  against 33 ms for the same base with no ledger (best of seven alternating runs),
  which is 29 ms for 4,000 reads and about a second for the 33,804 trees of the real base.

The bound is set from what each mechanism costs,
measured 2026-11-10 on this machine:

- The idle-trees pass judged 14,850 of 33,804 trees in 10,475 s of the run that ended
  2026-11-10T07:37:24-05:00,
  0.71 s per tree including the removal of 511.49 GiB,
  so a pass over every
  tree costs about 6.6 h at today's size.
  Before it judges anything it reads every tree's recorded workspace,
  and under the load of that day
  (a load average of 70 to 148,
  a build base with 18 GiB free) that fixed read alone ran past an hour;
  the pass now says how long it took.
- The units pass judged 1,782 of 68,755 profile directories in the 921 s a planning pass had left after
  listing them (0.52 s each,
  `lean run --dry-run --only units --deadline-minutes 30`),
  so a pass over
  every profile directory costs about 9.9 h,
  and listing them costs 880 s before the first one is judged.
- Run records,
  toolchains and Podman took 24 s,
  under a second and 334 s in the journaled run above,
  and 193 s,
  6 s and 1,944 s in the one before it.

A pass over both lists is therefore about seventeen hours today,
so no bound a person would accept makes one run cover it.

`RUN_DEADLINE_SECONDS` is 5 h (`src/lean_policy.rs`),
the service's `TimeoutStartSec` is that plus thirty minutes,
both written by the next `cargo-shadow lean timer-install`,
while the unit installed today still holds the old bound's 12,600 s,
and a run that starts at the timer's 04:30 ends at 09:30 at the latest,
or 30 minutes past that at worst,
when one large tree's deletion outlasts the run's own bound.
The trees pass at 6.6 h then needs a second night,
whose last 3.4 h the units pass takes,
and the two continue that way while a whole pass costs more than the bound.
What the bound costs a person is small:
systemd recorded `2min 22s CPU time over 3h 30min wall clock` and a 517 MiB peak for the run that ended
2026-11-09T08:00:00-05:00,
the unit holds `IOWeight=5` against the default 100 on a disk whose scheduler is BFQ (which honours that weight)
and `IOSchedulingClass=idle` with `Nice=15`,
and under a load average near 100,
with the trees pass reading the same btrfs volume,
a commit of fourteen
files in this repository took 2.69 s.
`lean run --deadline-minutes <n>` overrides the bound for one run;
the tree deletion itself has no deadline of its own,
so a run can pass its bound while it finishes one large tree,
which the service's extra thirty minutes covers.

The proof of "not in use",
recorded in the log line of every removal:

- The profile directory's build locks (`.cargo-build-lock`,
  `.cargo-artifact-lock`,
  `.cargo-lock`) are taken exclusively without waiting
  (through a container's bind mount too,
  probed 2026-11-04);
  a busy lock,
  a waiter queued in `/proc/locks`,
  a file system that cannot lock and a directory with no lock file all skip it.
- A scan of `/proc` finds no process whose working directory,
  program,
  open file or mapped file lies under the entry;
  processes in another mount namespace are judged by the device and inode of what they hold.
  A process whose entries cannot be read is classified from its `/proc/<pid>/status`,
  which every process shows:
  a kernel thread (`Kthread: 1`) and an ended process (`State: Z`) hold no file and are counted as such,
  and what is left is counted as uninspectable,
  split into the processes of another user that `other_uids_can_reach` shows the path lets in
  (one directory between `/` and the entry that grants its group and every other user nothing stops every other user)
  and those of this user or of root,
  whose descriptors the kernel hides from a plain user.
  Neither kind stops a removal:
  the kernel shows a plain user no descriptors of either,
  and refusing every removal while another user's system daemon exists would stop the sweep wherever the build base is
  world-searchable (the host base here is on a `0777` mount with a `uid 70` daemon running,
  measured 2026-11-08).
  The limit is therefore narrowed and named,
  not closed:
  the proof line says how many processes it could not judge and which kind,
  and closing the rest needs `CAP_SYS_PTRACE` or root,
  which this tool never takes.
- The entry's device and inode are read when it is judged,
  before it is moved into a trash directory on the same file system by an atomic rename that never replaces,
  and again just before it is deleted.
  A mismatch leaves the entry in the trash and removes nothing.
- A directory inside the claimed entry that this user owns and may not write
  (`0500`,
  `0555`,
  `0000`) gets this user's read,
  write and search before the walk removes what it holds,
  because unlinking what a directory holds needs the first and last of those and listing it needs the second.
  The removal's line names every directory whose mode it changed,
  as it names the locks it took,
  and a tree that could not be emptied whole is reported as partly removed with a reason for each part that stayed,
  never as skipped (issue 6282).
  A directory that belongs to another user is left exactly as it is and named:
  this removal changes no mode it does not own.
  Only a directory the walk enters is changed and a symbolic link is removed as a link,
  so nothing outside the claimed entry is touched.

Nothing waits without end:
every walk obeys the run's deadline and every `podman`,
`rustup` and `systemctl` call has its own.
`lean` never starts `cargo`,
never starts itself,
and takes no lock of its own (the claim is the rename).

```sh
# package/cargo-shadow/cli/README.md
cargo-shadow lean run --dry-run
cargo-shadow lean run --only units,records
cargo-shadow lean records target/verification --dry-run
cargo-shadow lean timer-install
```

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
