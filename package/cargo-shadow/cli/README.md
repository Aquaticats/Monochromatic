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
  A killed wrapper's cargo is found another way:
  the process group of every child the run starts is recorded in the run's journal,
  and a recovery leaves a journal whose recorded group still has a member (it names the group) for a later run.
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
  Finishing a run forces its outputs to disk before the completion marker (the owner's "Force at leaving"):
  each lockfile and journaled manifest and its directory are synced,
  then the journal is renamed to `.done`,
  the parent is synced,
  and the journal is removed.
  A removal that stopped partway is finished by the next recovery,
  which syncs the parent before it deletes anything.
  Data is synced before it is relied on:
  the displaced file (a failed data barrier is an error,
  never a settled outcome),
  the kept directory and its two parents every time a file is linked into it.
  Power-loss behavior depends on the filesystem honoring `fsync` and is not tested by fault injection.
  The kept directory is never capped and never emptied by the tool.
  Owner answer of round 6, "Warn past a count, never refuse":
  past 1,000 files every run prints how many files are kept there, their total size and the folder to review,
  and cargo keeps working.
  The 1,000 is the warning point, kept from round 5 and open to veto;
  the rate at which files are kept has not been measured.
  Before removing a kept file, make sure no program still has it open,
  because a program that holds it keeps writing to the inode, and a copy made earlier misses those writes.
- Nothing waits forever
  (owner rule of 2026-11-03:
  "nothing should ever wait forever or indefinitely").
  Locks are tried,
  never waited for.
  Every child process runs in its own process group under a bound.
  At the bound the whole group is killed and shown to be gone,
  and the error names the call and the command the user ran;
  after a normal exit any member left in the group is killed too,
  and output is returned only when it is complete (a process outside the group that holds the pipe makes it an error).
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
  not its number.
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
