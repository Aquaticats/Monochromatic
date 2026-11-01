# mise 2026.10.19 fails lock resolution here: a stale vfox cmake backend and lock platforms that ignore a tool's `os`

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so this record is historical for the Mise-era toolchain it investigated.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`).

## Symptom

The `cargo-publish` workflow failed at the step "Install only the tools this workflow needs" (`mise install rust`)
on every run that reached it after 2026-10-24,
before any publish step.
Release commit `00c4e28af4` (run 40242133604,
two attempts) and a later push (run 40242360125) all failed the same way.
The last passing run (39829260282,
2026-10-22) used mise 2026.10.17.

The log lists `mise lock` lines for every tool and every platform,
then removes and reinstalls the `cmake` plugin and
fails:

```text
mise plugin:cmake  ✓ https://github.com/jdx/vfox-cmake.git#4ecc1f0
mise plugin:cmake    uninstall
mise plugin:cmake    remove ~/.local/share/mise/plugins/cmake
mise WARN  failed to load vfox plugin metadata deps for cmake: runtime error: .../hooks/available.lua:1: module ...
        no field package.preload['util']
        no file '/home/runner/.local/share/mise/plugins/cmake/util.lua'
mise ERROR failed to resolve cmake for linux-arm64: error converting Lua table to PreInstall (no version returned from vfox plugin)
mise ERROR Version: 2026.10.19 linux-x64 (2026-10-24)
```

The job `gpc-publish-crate` fails,
and the jobs that depend on it (`build-binary`,
release creation) are skipped.
The other workflows on the same commits pass.
The crate is not involved:
no step that builds or publishes it ran.

The same repair pass also fails outside CI,
and there it surfaces two error variants on the same repository state.
Against `mise.lock` as committed at `148f47c951`,
which recorded `backend = "vfox:cmake"` for `cmake` (line 546),
running `mise lock` with 2026.10.19 exits 1 with the `cmake` error quoted in the CI log.
After `cmake` is migrated to the aqua backend (the workaround recorded in "Verified workarounds"),
the same command
exits 1 on a different tool:

```text
mise ERROR failed to resolve conda:bubblewrap for macos-arm64: failed to solve bubblewrap for macos-arm64
```

The CI log shows whichever variant the lock state reaches first;
the committed lock hid the conda variant behind the
`cmake` one.

## Root cause

The failure has two independent causes,
both in the lock rebuild that `mise install` runs.
An earlier revision of this document recorded the root cause as not established;
the traces in this section come from
reading mise source at tag `v2026.10.19` and on `main` after the fix,
and from running both binaries against this
repository.

### Defect 1: the lock pins a backend whose plugin has no Linux arm64 entry

`mise.lock` at commit `148f47c951` records `cmake` version 4.4.4 with a backend the registry has replaced
(`mise.lock:546`):

```toml
[[tools.cmake]]
version = "4.4.4"
backend = "vfox:cmake"
specifiers = ["latest"]
```

mise itself warns about this state (`src/args/backend_arg.rs:807`):

```rust
            warn_once!(
                "{short} is locked to {locked}, but the registry now installs it from {registry}. \
                 Run `mise backends switch {short}` to switch.",
                short = self.short,
            );
```

The vfox plugin at `~/.local/share/mise/plugins/cmake` (from `github.com/jdx/vfox-cmake`,
commit `4ecc1f0` per the CI
log) publishes its downloads in `assets/version.json`.
Its Linux list has 206 entries under `arch: "386"` and 305 under
`arch: "amd64"`,
and no `arm64` entry at all;
the aarch64 tarballs are filed under the `386` key:

```json
{"arch": "386", "version": "3.31.3", "download_url": "https://cmake.org/files/v3.31/cmake-3.31.3-linux-aarch64.tar.gz"}
{"arch": "amd64", "version": "3.31.3", "download_url": "https://cmake.org/files/v3.31/cmake-3.31.3-linux-x86_64.tar.gz"}
```

Resolution for `linux-arm64` therefore finds no entry,
and the plugin returns a table without `version`.
mise converts that table to `PreInstall` and raises the error from the CI log
(`crates/vfox/src/hooks/pre_install.rs:344`):

```rust
impl FromLua for PreInstall {
    fn from_lua(value: Value, _: &Lua) -> std::result::Result<Self, LuaError> {
        match value {
            Value::Table(table) => {
                if !table.contains_key("version")? {
                    return Err(LuaError::FromLuaConversionError {
                        from: "table",
                        to: "PreInstall".into(),
                        message: Some("no version returned from vfox plugin".to_string()),
                    });
                }
```

The lock rebuild turns any resolution error into a hard failure,
whether or not the backend considers the tool
optional (`src/lockfile/generate.rs:570`):

```rust
        let mut info = info.map_err(|error| eyre!(error))?;
```

### Defect 2: lock target platforms ignore a tool's `os` option

`mise.toml:207` declares the other tool that aborts the rebuild:

```toml
"conda:bubblewrap" = { version = "latest", os = ["linux"] }
```

The lock target selection at tag `v2026.10.19` iterates every requested platform and never asks whether the tool's
`os` option covers that platform (`src/lockfile/generate.rs:405`):

```rust
    for (ba, tv) in tools {
        let backend = tv.backend()?;
        for platform in platforms {
            for platform in backend.platform_variants_for(tv, platform) {
                let options = backend
                    .lockfile_options(&tv.request, &PlatformTarget::new(platform.clone()))?;
                keys.insert((
                    crate::toolset::install_state::state_short(&ba.short).into_owned(),
                    platform.to_key(),
                ));
```

So `conda:bubblewrap` is resolved for `macos-arm64`,
`windows-x64` and the other non-Linux targets even though the
tool is declared for Linux only.
The conda backend cannot produce a package for those targets and reports the error
(`src/backend/conda.rs:1187`):

```rust
        let records = self
            .solve_packages(vec![match_spec], platform, opts)
            .await
            .wrap_err_with(|| format!("failed to solve {tool_name} for {}", target.to_key()))?;
```

Unlike the vfox backend,
conda resolution errors are fatal on purpose (`src/lockfile.rs:4579`):

```rust
    let mut error_is_fatal = if backend
        .as_ref()
        .is_some_and(|backend| backend.get_type() == BackendType::Conda)
    {
        LockResolutionStatus::Required
    } else {
        LockResolutionStatus::Optional
    };
```

`generate.rs:570` then aborts the rebuild,
which is why this repository cannot run `mise lock` on 2026.10.19 even
though the committed lock records `conda:bubblewrap` for the Linux platforms only.

### The fix on main

Upstream issue [jdx/mise#14937](https://github.com/jdx/mise/issues/14937) reports the same `os` option gap for a
non-conda tool,
where it only produced a wrong warning.
It was fixed by
[jdx/mise#14959](https://github.com/jdx/mise/pull/14959) (merge commit `b2e0c56`,
merged 2026-10-25T22:18:32Z).
`main` at `b9e1506` skips the excluded platforms before resolution,
in both the target construction and the reuse
checks (`src/lockfile/generate.rs:301`):

```rust
        for platform in platforms {
            if !tv.request.is_os_supported_on(platform) {
                continue;
            }
            for platform in backend.platform_variants_for(tv, platform) {
```

and (`src/lockfile/generate.rs:424`):

```rust
        for platform in platforms {
            if !tv.request.is_os_supported_on(platform) {
                // Nothing is locked here, but an explicit selection still owns
                // these keys, so rows an earlier `os` option allowed are dropped.
                for platform in backend.platform_variants_for(tv, platform) {
                    keys.insert((
                        crate::toolset::install_state::state_short(&ba.short).into_owned(),
                        platform.to_key(),
                    ));
                }
                continue;
            }
```

That commit is contained in the release pull request
[jdx/mise#14948](https://github.com/jdx/mise/pull/14948) for 2026.10.20,
which was open at the time of writing;
no
release published before it contains the fix.

Defect 1 is not upstream's bug:
it is the repository's lockfile holding a superseded backend,
and
`mise backends switch` is the documented migration.

### Why 2026.10.17 passed and 2026.10.19 failed

Both versions run the same lockfile code for `mise lock`;
the failure appeared in CI because the install step changed
what it does with the lockfile.
With 2026.10.19 the failing run prints `mise lock` lines for every tool and platform
before aborting (quoted in "Symptom");
with 2026.10.17 the passing runs and the local simulation print no such lines
and reuse the committed entries.
The version bisection therefore follows the version,
but the two defects in the
lockfile rebuild,
not a difference in the `mise lock` command itself.

### An earlier reading that was wrong

The first revision of this document recorded the root cause as not established and kept two candidates:
the plugin
removal/reinstall,
and the vfox metadata warning (`no file '.../plugins/cmake/util.lua'`).
Both are disproved.
The plugin removal and reinstall also appears in the passing 2026.10.17 runs,
and the failure
reproduces on a host where the plugin loads normally and only the version lookup fails,
as Defect 1 traces.
The GitHub API rate-limiting hypothesis from that revision was already disproved:
the host IP had 59 of 60 core
requests left when checked,
and the failure is a mise resolution error,
not an HTTP status.

## Verification

Version under test:
release tarballs `mise-v2026.10.17-linux-x64.tar.gz` and `mise-v2026.10.19-linux-x64.tar.gz`,
and
a source build of `jdx/mise` `main` at commit `b9e1506` (2026-10-25),
built with
`cargo build --release --ignore-rust-version`,
whose binary reports `2026.10.19 linux-x64 (2026-10-26)` and contains
`b2e0c56`.

### Harness 1: release bisection in a container

A podman container (Debian 13 image with git and cargo,
`--memory=4g --cpus=2`,
throwaway home),
the committed
`mise.toml`,
`mise.lock` and `mise.no-env.toml` only,
mise `2026.10.17` and `2026.10.19` from the GitHub release
archives,
running `timeout 240 mise install rust` in the directory with the files.

Two attempts of each version,
in fresh container homes:

- 2026.10.17:
  exit 0 after 150 seconds and after 124 seconds,
  reached the Rust toolchain install,
  no `cmake` resolution
  error.
- 2026.10.19:
  exit 1 after 171 seconds and after 194 seconds,
  `failed to resolve cmake for linux-arm64` present both
  times.

That is a two-version bisection with two runs each,
so it shows the failure follows the mise version in this setup.
It does not exclude another release between the two being the first to fail:
only 2026.10.17 and 2026.10.19 were run.

### Harness 2: source build against a disposable worktree

The worktree is a detached checkout of the repository HEAD under `${HOME}/temp/agent/mono-lock-test`;
the commands in
this section run there with the built binary in `${HOME}/temp/agent/bin/mise-fix`:

```sh
mise backends switch cmake
mise lock
mise lock
mise lock --dry-run        # with the 2026.10.17 binary
mise install rust          # with the 2026.10.17 binary, empty MISE_DATA_DIR
```

Observed on 2026-10-25:

```text
$ mise backends switch cmake
mise lock                              ✓ 10 targets checked
  cmake@4.4.4  installed 1 tool in 14.4s   (64.9 MB, from the aqua backend)

$ mise lock
mise lock                              ✓ 645 targets checked
✓ Lockfile ready at .../mono-lock-test/mise.lock

$ mise lock
mise lock                              ✓ 645 targets checked
$ cmp <first-run copy> <second-run copy>   # byte identical

$ mise lock --dry-run                   # 2026.10.17
→ Targeting 11 platform(s) ... → Processing 59 tool(s) ... ✓ 655 rows, exit 0

$ MISE_DATA_DIR=<empty dir> mise install rust   # 2026.10.17
mise rust@nightly-2026-10-21 ... installed 1 tool in 4.7s, exit 0
```

Inspecting the written lock:
`conda:bubblewrap` keeps six Linux platform keys (`linux-arm64`,
`linux-arm64-musl`,
`linux-x64` and their baseline and musl variants) and no macOS or Windows keys;
`cmake` gains `linux-arm64` and
`linux-arm64-musl` sections,
records `backend = "aqua:Kitware/CMake"`,
and takes its URLs and checksums from the aqua
backend and the `mise-versions` host;
the android-sdk entries gain archive checksums that the previous lock did not
record.

The same lock with the released 2026.10.19 binary fails as quoted in "Symptom",
because it lacks `b2e0c56`.
The 2026.10.14 release also fails the fixture in the container harness:
the conda solve fails for every platform
(`failed to solve bubblewrap for ...`),
not only for macOS and Windows.

Host quirk while building the fixed binary:
it was first placed under the encrypted temp mount,
where SELinux labels
it `unlabeled_t`,
and every HTTPS connection from it aborted with
`tcp connect error: Software caused connection abort (os error 103)` while the system curl and the released binaries
connected normally.
Copying it to the home filesystem and relabeling it (`chcon -t gconf_home_t`) made the same
requests succeed;
the network failures were the file label,
not mise or the network.

### What works cleanly

- The fix commit `b2e0c56` present in a build:
  `mise backends switch cmake` and `mise lock` both exit 0,
  and a repeat
  run is byte identical.
- 2026.10.17 reads the regenerated lock:
  `mise lock --dry-run` exits 0 with the same 655 planned rows it lists for the
  previous lock,
  and `mise install rust` exits 0 in an empty data directory.
- The committed lock keeps the `os` filter visible:
  only Linux keys exist for `conda:bubblewrap`.

### What fails

- 2026.10.19 with the lock from `148f47c951`:
  `failed to resolve cmake for linux-arm64: error converting Lua table to
  PreInstall (no version returned from vfox plugin)`.
- 2026.10.19 with the cmake backend migrated to aqua:
  `failed to resolve conda:bubblewrap for macos-arm64: failed to
  solve bubblewrap for macos-arm64`.
- 2026.10.14 with the same declarations:
  `failed to solve bubblewrap` for every target platform.
- 2026.10.19 after the lock migration:
  the same conda failure,
  so the migration alone does not make the installed
  release work.

## Verified workarounds

### Migrate the cmake lock entry, then regenerate the lock with a mise that has the fix

This is what landed in the repository.
In a disposable worktree at HEAD:

```sh
mise backends switch cmake   # rewrites backend = "vfox:cmake" to "aqua:Kitware/CMake" at 4.4.4
mise lock                    # rebuilds the full lock: 645 targets, exit 0
```

The result is commit `5fee1731f8`,
which updates `mise.lock` by 70 insertions and 10 deletions:
the cmake backend line,
the two new arm64 platform sections with checksums,
and the android-sdk archive checksums.
Nothing else in the lock
moved,
and `conda:bubblewrap` kept its Linux-only keys.

- Tradeoff:
  the full seven-platform lock needs a mise build newer than 2026.10.19;
  the Linux-only stopgap below makes the released 2026.10.19 work.
- Tradeoff:
  the lock now installs `cmake` from `aqua:Kitware/CMake` instead of `vfox:cmake`,
  so downloads come from
  the Kitware GitHub releases rather than cmake.org and the installed cmake identity changes to the aqua backend.
- Tradeoff:
  the newer lock writer records `url_api` and `checksum_source` fields;
  2026.10.17 ignores them without error
  (checked with `mise lock --dry-run`),
  so the pinned CI version can read the lock.

### Target only the Linux lock platforms (stopgap)

Commit `355707fb7e` records the decision and commit `7f00576883` makes the
change:
because no released mise contains jdx/mise#14959,
the lock now targets
the four Linux platforms.

```toml
# mise.no-env.toml, [settings]
lockfile_platforms = ["linux-x64", "linux-x64-musl", "linux-arm64", "linux-arm64-musl"]
```

Measured on 2026.10.19 with the migrated lock:

- bare `mise lock` exits 0 and reports `✓ 240 targets checked`;
- a repeat run makes no further change;
- `mise.lock` loses 1101 lines of macOS and Windows platform metadata and adds
  none,
  while every tool version,
  backend and Linux URL and checksum stays the
  same;
- `conda:bubblewrap` stays declared with `os = ["linux"]` and keeps its Linux
  lock entry.

- Tradeoff:
  macOS and Windows developers lose pinned URLs and checksums for those
  platforms and resolve them locally instead.
- Tradeoff:
  the setting is temporary;
  the removal steps live in
  `doc/decision/mise-lock-linux-only-stopgap.md`.
- This is why bare `mise lock` works with the installed 2026.10.19 again.

### Pin mise in the workflow

From commit `3e16deb852` until commit `38cb13b4d1` (2026-10-25) every `jdx/mise-action` step in
`.github/workflows/cargo-publish.yml` carried `version: 2026.10.17`,
which the action
documents as taking precedence over `minimum_release_age`.
Commit `38cb13b4d1` removed mise from every workflow,
per `doc/decision/mise-free-ci.md`,
so that pin no longer exists;
the tradeoffs below are the record from the time it did.

- Tradeoff:
  the workflow no longer follows new mise releases,
  so a mise fix is not picked up until the pin is removed.
- Tradeoff:
  the other jobs in the file stay on the pinned version too,
  including ones that did not fail.
- Verified locally that the pin still works with the regenerated lock:
  an isolated data directory and the 2026.10.17
  binary run `mise install rust` to exit 0 (Harness 2).
  A dispatched CI run of the pinned workflow was not repeated
  after the lock update,
  so the local simulation is the current evidence.

## What does not work

- Switching `cmake` to the aqua backend with the 2026.10.19 release alone (`mise backends switch cmake`).
  The switch
  rewrites the lock entry and adds the arm64 URLs,
  but the following lock pass still fails,
  now on
  `conda:bubblewrap`.
  Defect 2 explains why:
  the switch does not change the target loop.
  It works only together with
  the `b2e0c56` fix (first release 2026.10.20).
- Running the full lock on releases before the fix:
  2026.10.14 fails the conda solve for every platform,
  2026.10.17 fails on the excluded macOS target,
  and 2026.10.19 fails on whichever defect it reaches first.
- Rerunning the failed job:
  attempt 2 of run 40242133604 failed at the same step.
- Publishing from a developer machine:
  the crate publishes through crates.io Trusted Publishing from CI.
- Dropping `conda:bubblewrap` from `mise.toml` to get a clean lock:
  removes a Linux-only runtime dependency that the
  headless GPU inspection needs,
  for a failure that has an upstream fix.
- Hand-editing `mise.lock` to add the missing entries:
  the repository rule is to regenerate lockfiles through the
  owning tool,
  and the checksums would be unverified.

## Upstream filing decision

Checked `.out-of-scope/` for a mise exemption:
none exists.
The directory holds entries for other tools
(`bun-install.md`,
`cargo-workspace.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
and others),
and none
mentions mise as out of scope,
so the exemption path does not apply.

Duplicate search:
the tracker already contains the report,
[jdx/mise#14937](https://github.com/jdx/mise/issues/14937) "lock:
warns about platforms that a tool's `os` option
excludes" (closed 2026-10-25T22:18:34Z),
fixed by [jdx/mise#14959](https://github.com/jdx/mise/pull/14959).
No second
issue was opened.

The six constraints,
walked explicitly:

1. Is it really upstream's fault?
   For the `os` handling,
   yes:
   the target loop ignores a documented tool option in the
   lock rebuild,
   which is behavior,
   not wording.
   The `cmake` half is repository state (a stale backend in the lock),
   not an upstream defect.
2. Can upstream fix it?
   Yes,
   and it did:
   `b2e0c56` on `main` skips the excluded platforms before resolution.
3. Are they supporting this use case?
   Yes:
   the `os` tool option exists for OS-specific tools,
   the reporter of #14937
   used it with `lockfile_platforms`,
   and this repository uses it for `conda:bubblewrap`.
4. Would the repo welcome our contribution?
   No filing is possible from this work.
   `CONTRIBUTING.md` says:
   "AI replies
   to Discussions and Issues are restricted.
   Only use AI to reply to a thread if you created it,
   opened a PR that
   fixes it,
   or have already had a contribution,
   attributed to your GitHub account,
   merged into the default branch of
   mise."
   This session meets none of those conditions,
   and the policy calls the alternative an instant ban.
   The audit
   therefore ends with no comment and no issue.
5. Will they likely fix it?
   Already fixed:
   merged 2026-10-25 and listed in the 2026.10.20 release pull request
   [jdx/mise#14948](https://github.com/jdx/mise/pull/14948).
6. Have we prototyped a minimal fix compatible with their architecture?
   The upstream fix itself was built from `main`
   at `b9e1506` and exercised against this repository (Harness 2):
   pre-fix 2026.10.19 reproduces the failure,
   the
   fixed build resolves all 645 targets and writes a lock whose `conda:bubblewrap` keys stay Linux-only.
   The
   candidate-fix gate holds:
   the fix is absent from `v2026.10.19`,
   present on `main`,
   and the reproduction inputs
   (an `os`-limited tool plus multi-platform lock targets) match issue #14937,
   with the additional fatal conda variant
   traced in "Root cause".

Outcome:
**nothing to add**.
The merged pull request already covers the behavior,
the vfox plugin data gap is moot for
this repository because the lock moved to aqua,
and the contribution policy forbids an AI-authored comment on the
thread.
Do not file from this document.

## Status and follow-up

- `mise.lock` was migrated and regenerated in commit `5fee1731f8`;
  the verification in Harness 2 was run before and
  after that commit against the same inputs.
- CI no longer runs mise:
  commit `38cb13b4d1` replaced the `jdx/mise-action` steps,
  including the `version: 2026.10.17` pins,
  with direct tool setup,
  as decided in `doc/decision/mise-free-ci.md`.
  There is no pin left to remove.
- The incident closed on 2026-10-28.
  mise v2026.10.20 (published 2026-10-26T21:21:55Z,
  with v2026.10.21 following on 2026-10-27) ships the
  jdx/mise#14959 fix.
  The released 2026.10.20 binary ran a seven-platform lock to exit 0 in a
  throwaway worktree at commit `7f00576883` with the stopgap removed and
  `mise.lock` deleted:
  413 targets checked,
  macOS and Windows entries back for `aqua:Kitware/CMake`,
  and `conda:bubblewrap` still Linux-only.
  Removing only the setting against the Linux-only lock kept four platforms
  (358 targets),
  because an unset `lockfile_platforms` preserves the platforms already in
  the lockfile.
- The repository no longer runs `mise lock` at all:
  commit `a5cdc3158d` removed `[tools]` and the lockfile settings for
  `doc/decision/mpm-toolchain-adoption.md`,
  and commit `4cb480344f` deleted `mise.lock`;
  mise now only runs tasks.
  The stopgap decision is superseded and
  `doc/handover/mise-lock-2026-10-20-followup.md` is deleted,
  so no lock remains to regenerate.
