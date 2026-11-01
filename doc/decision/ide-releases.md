# Release the IDE from CI as GitHub pre-releases

## Status

Accepted 2026-10-22.
Owner-authorized:
the owner asked for GitHub releases CI ("Please setup gh releases CI") and chose the IDE and the nested compositor as
the products to release.
This document covers the IDE (`package/ide/app`,
program `monochromatic-ide`);
the compositor has its own workflow and document.
The first real release is a separate act that needs the owner's go-ahead (see the consequences).

Addendum 2026-10-22:
the owner then said the IDE release must also support Linux ARM64.
The section "Linux ARM64" records that decision.
The pre-release `ide-v0.1.0` was published before it,
holds the x86_64 archive only,
and is immutable,
so ARM64 ships from the next version.
The other sections describe the x86_64 workflow as it was published,
and the ARM64 section says what it changes.

## Context

Before this,
the IDE had no release path.
`mise run //package/ide/app:install` puts a locally built `dist/monochromatic-ide` into `~/.local/bin`,
`package/ide/app/Cargo.toml` has `publish = false`,
and nothing built the program anywhere but the owner's machine.

The owner's answers,
as the agent that built this received them:

- The release shape follows the music player's,
  which the owner already accepted (`doc/decision/music-player-releases.md`,
  workflow `.github/workflows/music-player-release.yml`):
  a version change merged to `main` triggers a run,
  a manual dispatch has a `dry-run` input that defaults to true,
  every file is attested,
  and one release job creates the release with every asset in a single `gh release create`.
- Every release is a pre-release,
  because the project is under active development.
- Platform:
  Linux x86_64 only for now
  (changed the same day:
  see "Linux ARM64").
  The program needs a Wayland session and a Vulkan driver for a hardware GPU (`package/ide/app/README.md`,
  sections "Renderer and GPU" and "What the host provides").
- The released file is the single executable that the `bundle` task writes,
  with its notices.

Facts that constrain the design:

- Releases are immutable in this repository.
  A published release can never receive a replacement or an extra file,
  so a bad release is fixed only by a higher version (`doc/decision/music-player-releases.md`,
  "Safety against a bad first release").
- The version `0.1.0` was in `main` before the workflow existed,
  so no push will release it.
- The repository has an allowed-actions list.
  Every action the workflow uses is used by `music-player-release.yml` at the same commit,
  and the pinned `jdx/mise-action` version is the same.
- A GitHub-hosted runner has no display and no GPU.
  The IDE refuses CPU Vulkan adapters (`src/gpu_device.rs`,
  README section "Renderer and GPU"),
  so no check on a runner can open its window.
- The repository's `mise.lock` pins one Rust toolchain for every package,
  a dated nightly.
  The IDE builds in a container whose Rust is the stable toolchain,
  and nothing in the package builds it with that nightly.

## Decision

### Trigger and versions

`.github/workflows/ide-release.yml` runs on a push to `main` that changes `package/ide/app/Cargo.toml`,
and on `workflow_dispatch` with a `dry-run` boolean that defaults to true.
The detect job is the music player's detect job with the IDE's manifest:
it compares the `[package]` `version` before and after the push and builds only when it changed.
The version must be plain `MAJOR.MINOR.PATCH`,
and a push may not lower it (compared with `sort --version-sort`,
so `0.9.0` to `0.10.0` is an increase).
The music player's cap of 99 on minor and patch is not carried over:
it exists for the Android `versionCode`.

The tag is `ide-v<version>`.
If it already exists,
the release is skipped with a notice and the run succeeds;
the release job checks again right before creating the release.
A run is real only on `main`,
from a push or from a dispatch with `dry-run` false;
a real dispatch from any other ref fails.
A dry run builds,
packages,
verifies and uploads workflow artifacts that expire after one day,
prints the `gh release create` command and the notes,
and skips attestation,
the tag and the release.
The concurrency group is `ide-release` with `cancel-in-progress: false`,
so runs queue.
Top-level `permissions` is `{}`;
the detect job reads contents,
the build and checksums jobs also write attestations and the id token,
and only the release job can write contents.

The detect step was run against scratch repositories for these cases,
and each behaved as described here:
a push with a bump,
with the same version,
with a lower version,
with a suffix,
with a manifest absent before the push and with a zero `before`,
and a dispatch on a branch and on `main`,
with and without an existing tag.

### The build runs the repository's mise tasks in podman

The runner runs `mise run //package/ide/app:bundle`,
the flow developers use,
and not a native `cargo build`.
The reasons are in the tasks themselves:

- `build` depends on `runtime` and `notices` (`package/ide/app/mise.toml`,
  the `depends` of `[tasks.build]`).
  `build.rs` stops the build when the prepared runtime directory is missing (`build.rs`,
  the message "Prepare it with `mise run //package/ide/app:runtime`").
- `runtime` fetches the Helix grammars at the pinned revision and compiles them in the container,
  with network;
  `notices` runs `cargo-about` 0.9.2,
  which exists only inside the image (`package/ide/app/Containerfile`,
  the `cargo install ... --version 0.9.2 cargo-about` line,
  and the task's version check).
  Every one of those tasks runs through `runBounded` and podman.
  A native cargo build would have to reimplement `runtime` and `notices` outside mise,
  and the result would be a flow nobody has run.
- The music player's Alpine job already runs podman 4.9.3 on `ubuntu-24.04` with `--memory` and `--cpus` (run
  39785231437,
  job `build-desktop-musl (x64 ...)`:
  `podman version 4.9.3`,
  and a release build of 25 min 10 s with `--memory=12g --cpus=4`).

The steps are:

1. Build the base image with `mise run //package/terminal/app:image:release`.
   The release workflow builds in Ubuntu 24.04 images, not the CachyOS development images
   (`doc/decision/container-base-distro.md`):
   the base is `localhost/monochromatic/terminal-release`, built from `package/terminal/app/Containerfile.release`.
   The IDE release image is `FROM localhost/monochromatic/terminal-release` (`package/ide/app/Containerfile.release`),
   and the tasks build the IDE image on demand but not its base.
   The job sets `IDE_RELEASE=1`, which makes every IDE task use the release image.
   (Until 2026-10-29 the release built in the Fedora 41 images of `Containerfile`;
   the decisions and measurements here that name Fedora 41 describe those.)
2. Build the IDE image with `mise run //package/ide/app:image:release`;
   it is tagged `localhost/monochromatic/ide-release`.
3. Run a bounded container once to print `rustc --version --verbose`,
   `cargo --version` and `cargo-about --version`,
   with the same kinds of limits the tasks pass (memory,
   CPUs,
   processes,
   open files).
   The Alpine job proves `--memory` and `--cpus` on this runner image;
   `--pids-limit` and `--ulimit` were not proven,
   and this step makes them fail early and not an hour into the build.
4. Run `mise run //package/ide/app:bundle`.
   `MISE_JOBS=1` runs one task at a time
   (`mise run --help` of mise 2026.10.0 lists `--jobs` with `[env: MISE_JOBS]`),
   because the container bound applies per container and not per task graph.
   `IDE_CONTAINER_CPUS=4` and `IDE_CONTAINER_MEMORY=12g` replace the default 90 percent share
   (`package/ide/app/bin/container-bounds.ts`),
   as the Alpine job's `BUILD_CPUS` and `BUILD_MEMORY` do,
   so the runner keeps room for podman and the runner processes.

The `build` task passes no `--locked` to cargo.
The workflow fails the job if `git diff` shows a change in `package/ide/app/Cargo.lock` or in any tracked
file after the build,
so a dependency change the build made on its own never ships.

The Rust toolchain of the release is whatever `stable` was when the image was built
(`package/terminal/app/Containerfile`,
the `--default-toolchain stable` line),
not the `mise.lock` nightly.
`mise.lock` supplies node,
which runs the task scripts (`mise install node`).
The job prints the toolchain and passes the `rustc --version` line to the release notes,
so each release names its compiler.
This conflicts with the instruction to read the toolchain from `mise.lock` as the music player job does,
and the conflict is resolved for the tested flow:
changing the container's toolchain changes what every developer builds with and belongs to its own decision.

### Gates before upload

All of these run on the runner before anything is uploaded or attested:

- The executable prints exactly `monochromatic-ide <version>` for `--version` with an empty environment.
  The output format was read from the installed binary (`monochromatic-ide 0.1.0`) and from `bin/inspect-bundle.ts`,
  which expects `monochromatic-ide <major>.<minor>.<patch>`.
  `--version` exits before the program touches a display or a home folder (README,
  "Project startup").
- The dynamic section lists exactly `ld-linux-x86-64.so.2 libc.so.6 libfontconfig.so.1 libgcc_s.so.1 libm.so.6` and
  the newest glibc symbol version is at most 2.39.
  Both were measured on the installed binary on 2026-10-22.
  `INSTALL.txt` and the README state those requirements,
  so a build that needs more fails until they are updated.
- `mise run //package/ide/app:inspect:no-software-renderer`:
  Slint's software renderer is not linked.
- `inspect:bundle` with the checks `inventory`,
  `license-texts` and `crate-licenses`:
  the executable embeds every runtime file and license text byte for byte,
  `--licenses` prints them in full,
  and every linked crate has a license text.
- After packaging,
  the archive is extracted and its program must print the version,
  equal the built executable,
  print notices equal to the notices file,
  and the launcher entry must pass `desktop-file-validate`.

```sh
# The inspect:bundle gate of the workflow: file, runtime directory, and the checks that need no window.
mise run //package/ide/app:inspect:bundle \
  dist/monochromatic-ide target/release/runtime inventory,license-texts,crate-licenses
```

Local results are in the section "Checks run before landing".

### Checks left out, and why

- The seven other `inspect:bundle` checks (`lone-copy-highlights`,
  `later-start-reuses-cache`,
  `damaged-cache-rebuilt`,
  `shadowing-query-ignored`,
  `concurrent-first-starts`,
  `damaged-embedded-part-reported`,
  `old-cache-folders-removed`) start the program in the nested compositor.
  They demand a Wayland session (`WAYLAND_DISPLAY`) and the `monochromatic-nested-wayland-session` tool,
  and the program refuses a CPU Vulkan adapter,
  so none can run on a runner.
  `old-cache-folders-removed` fails the display on purpose and needs no compositor,
  but nothing established that the program reaches its cache tidy without a GPU,
  so it is not in the list.
  A dry run on a runner can show it;
  adding it then is a one-word change.
- The test suites (`test`,
  `test:native`,
  `test:annotations-native`,
  and the rest) and `lint:clippy` and `lint:release`:
  they compile the debug profile again (the local debug target directory is 23 GB,
  the release one 3.5 GB),
  and the native ones render on the hardware GPU with `/dev/dri`.
  They are the development gate,
  run before a change lands,
  and not part of a release.
- `inspect:bundle-guards`:
  it damages copies and observes the checks fail,
  which proves the checks and not the executable.
- No window is ever opened in CI.
  The runbook therefore has the maintainer start the archive of a dry run on a Wayland machine with a GPU before a
  real release.

### Files

One release holds three files:

- `ide-<version>-linux-x64.tar.gz`,
  native build on `ubuntu-24.04`.
  It holds `bin/monochromatic-ide` (the `bundle` task's executable,
  about 92 MB;
  grammars,
  queries and license texts are embedded,
  so nothing sits beside it),
  `share/applications/monochromatic.ide.desktop` (the file the `install` task installs,
  unchanged,
  with its stock icon name),
  `LICENSE`,
  `LICENSES/` (the application's two license texts),
  `THIRD-PARTY-LICENSES.txt` and `INSTALL.txt`.
  The tar stream has fixed order,
  modes,
  owner and timestamps,
  as the music player's.
- `ide-<version>-third-party-licenses.txt`,
  the license notices as a file of their own.
- `SHA256SUMS`,
  computed by a job that runs only after the build succeeded.

All three are attested in a real run.
Workflow artifacts are named `asset-linux-x64`,
`asset-third-party-licenses` and `checksums`,
so the music player's pattern `asset-*` collects the assets.
The checksums job fails unless the asset set is exactly the two expected names.

### License notices

The IDE has no separate notices job.
The executable embeds its notices and prints them with `--licenses` (README,
"What the executable carries" and "Rust crate license texts"):
the application's license texts,
both font notices,
Helix's license,
a notice for every bundled grammar,
and the license text of every Rust crate that `cargo-about` collected during the build.
The workflow writes the notices file from the executable that passed `inspect:bundle`
(`env --ignore-environment dist/monochromatic-ide --licenses`),
so the shipped notices are the embedded ones,
not a second copy that could drift.
The archive carries the same bytes as `THIRD-PARTY-LICENSES.txt`,
and the archive check compares them.

Slint is used under `GPL-3.0-only` (`about.toml`,
and README "Choices made by default and open to veto").
That is a recorded default open to the owner's veto,
not an owner decision,
and the release now publishes the result.
The release notes say it in plain words and point at the source tree of the tag.
The Slint royalty-free attribution that the music player's notes carry (the `#MadeWithSlint` badge) is not used,
because the IDE does not use that license.

### No caches

The workflow caches nothing.
The container image,
the `ide-cargo` volume and the target directory are fresh in every run,
as in the music player's Alpine jobs (`doc/decision/music-player-releases.md`,
"Build mechanics").
The repository's cache budget was at its cap when this was written (`doc/decision/ci-caching.md`),
a release runs once per version,
and a cold build cannot carry a stale artifact into attested bytes.
This is a deviation from the owner's wish for caching in general,
and the owner may veto it;
the cold time that a cache would be measured against is in "First dry run".

### Runner disk

The build needs the base image (2.05 GB locally),
the IDE image (2.25 GB),
the release target directory (3.5 GB locally),
a debug build of the `ide-runtime` helper,
the cargo volume and the grammar sources.
That debug build is also a time cost:
the `runtime` task compiles every dependency in the dev profile
(optimized by `[profile.dev.package."*"]`)
before the release build compiles them again,
and the measured total of the whole `bundle` task is in "First dry run".
A step removes the preinstalled Android SDK,
.NET,
GHC and CodeQL directories,
which the job does not use,
and prints `df` before and after that step,
after the image build and after the bundle.
The first dry run showed that the runner had 85 GB free before the step,
so the step may be dropped.

## First dry run

The workflow was dispatched on `main` as a dry run on 2026-10-22 (run 39832705367).
The `build` job took 15 min 37 s from set-up to the failing step.
It passed every step up to the renderer gate,
and failed there (see "The renderer gate on a fresh cargo volume").
Measured on that run:

- Podman 4.9.3,
  mise 2026.10.9,
  node 26.12.0,
  and the image's `rustc 1.99.1 (58cf8f9c3 2026-10-13)`,
  `cargo 1.99.1` and `cargo-about 0.9.2`.
- Step times:
  base image 1 min 04 s,
  IDE image 1 min 30 s,
  `bundle` 10 min 26 s (the task printed `Finished in 625.54s`),
  against 25 min 10 s for the music player's Alpine release build.
  The container bound was 4 CPUs,
  12g memory,
  1024 processes and 4096 open files.
- The bounded container with `--pids-limit=1024` and `--ulimit nofile=4096:4096` ran,
  so rootless podman on the runner accepts those limits.
- Disk on `/` (145 GB):
  85 GB available at the start,
  104 GB after the step that removes the preinstalled SDKs (it freed 18 GB and took 1 min 50 s),
  101 GB after the images,
  96 GB after `bundle`.
  The build itself used about 8 GB,
  so the removal step was not needed on this image.
  It stays until a second run confirms the figures,
  and the owner may drop it.
- The `--version` check printed `monochromatic-ide 0.1.0`.
  The dynamic section was exactly `ld-linux-x86-64.so.2 libc.so.6 libfontconfig.so.1 libgcc_s.so.1 libm.so.6`
  and the newest glibc symbol version was 2.39,
  as measured on the installed binary.
- The tracked-files check passed,
  so the build did not rewrite `Cargo.lock`.

### The renderer gate on a fresh cargo volume

`mise run //package/ide/app:inspect:no-software-renderer` stopped with
`no matching package named async-net found`,
required by `i-slint-backend-testing v1.18.1`,
and cargo's note that `--frozen` is offline mode.
The cause is the second positive control of the task,
`--features slint/mcp`.
It enables a feature that no build of the package uses,
so its crates (`async-net` among them) are not in `Cargo.lock` and `cargo fetch` never resolves them.
`--frozen` cannot download registry index entries,
and the volume on the runner is new in every run,
while the author's long-lived `ide-cargo` volume already held the entries from earlier work.
The `--all-features` request and the `slint/renderer-software` control resolved offline without trouble.

Reproduced locally on 2026-10-22 with the bound at 2 GiB and 2 CPUs for the probes:
after `cargo fetch` on a new volume,
the `slint/mcp` request failed with the same message under cargo 1.99.1,
the runner's version;
the whole task chain `bundle`,
then the gate,
on another new volume failed at the same command.
On that volume,
`cargo fetch` again and `cargo metadata --locked --all-features` with the network did not help,
and `cargo tree --locked` with the network and the same arguments did,
after which the offline command passed.

The fix is in the task,
not in the workflow:
`inspect:no-software-renderer` now makes one pass per request with the network on and `--locked`
(the same lock file,
nothing written) before its offline checks.
A workflow step would have to repeat the task's feature list,
and a developer with a new volume would hit the same failure.
The checks themselves are unchanged and still run offline with `--frozen`,
so the gate is neither dropped nor weakened.
If the network is down the pass reports that it was skipped,
and the offline checks use the entries the volume holds.
The task ran on a new volume in the failing state,
passed,
and printed its success line (the software renderer is not linked,
and both controls reach it).

## Checks run before landing

All of these ran on the author's machine on 2026-10-22,
against the installed `monochromatic-ide 0.1.0` (91,945,952 bytes,
a copy of it as `dist/monochromatic-ide` of the worktree) because a release build was not repeated:

- `actionlint` 1.7.12 and `zizmor` 1.31.1 (`--offline`,
  also `--pedantic`) on `.github/workflows/ide-release.yml`:
  no findings except the informational `anonymous-definition` on unnamed jobs,
  which the music player's workflow has as well.
- The YAML parses with the repository's `yaml` package,
  and the shell steps were extracted and run with `bash -e`:
  version smoke test,
  library and glibc check (newest symbol version 2.39),
  notices file (11,241 lines,
  581,554 bytes,
  first line `Monochromatic IDE 0.1.0 carries these 238 license and notice texts`),
  packaging,
  archive check (the extracted program prints `monochromatic-ide 0.1.0`),
  asset set and `SHA256SUMS`,
  release notes,
  and the dry-run `gh release create` command.
  The archive was 23,441,579 bytes.
  The gates were also made to fail on purpose:
  a wrong version,
  a glibc floor of 2.38,
  and a different executable.
- `mise run //package/ide/app:inspect:no-software-renderer` passed.
- After the fix of "The renderer gate on a fresh cargo volume",
  the gate chain was run against a real build from a new cargo volume
  (the `bundle` task through a wrapper that named another volume and the runner's cargo 1.99.1,
  8 CPUs and 24g memory;
  5 min 6 s):
  the tracked-files check,
  the `--version` check,
  the library and glibc check (newest symbol 2.39),
  `inspect:bundle` with the three window-free checks (1,255 embedded files,
  449 crates covered),
  the notices file (581,554 bytes),
  the packaging (23,457,136 bytes),
  the archive check,
  and the checksums step all passed.
- The version-bump procedure of the runbook was run on a scratch copy of the package with the version set to `0.1.1`:
  the `cargo fetch` of the `fetch` task,
  run in the build image,
  changed exactly one line of `Cargo.lock`
  (the `ide` entry from `0.1.0` to `0.1.1`).
- `mise run //package/ide/app:lint` and the markdown lint (`monochromatic-lint`) of the three documents
  written here passed after the findings in the README section were fixed.
- The `inspect:bundle` command of the block in "Gates before upload" passed with 3 of 10 checks in 15.39 s:
  1,255 embedded files,
  27 grammars,
  392 linked crates,
  and 449 crates covered by license texts.

## Not verified locally

Only a real runner can prove these.
The first dry run (section "First dry run") proved the build and the first three gates,
and stopped at the fourth:

- The steps after "Check that no software renderer is linked" have not run on a runner:
  "Check the single executable" (`inspect:bundle` with `usage`-style task arguments under mise 2026.10.9,
  which worked with mise 2026.10.0 and no `usage` on `PATH`),
  the notices file,
  the packaging,
  and the archive check.
  Their shell was run locally against a build from a fresh cargo volume (section "Checks run before landing").
- Attestation,
  the artifact upload and download steps,
  the checksums job,
  and the `gh release create` call itself.
  The release step is the music player's,
  which has run successfully many times,
  but the IDE's workflow has not got that far.
- Whether the `old-cache-folders-removed` check can run on a runner.

## Linux ARM64

Accepted 2026-10-22.
Owner-authorized:
the owner said the IDE release must also support Linux ARM64 (aarch64).
The decision is for the workflow and the documents;
it changes no version,
so no release starts from it.

### Release history this builds on

- `ide-v0.1.0` is published as a pre-release with three files:
  `ide-0.1.0-linux-x64.tar.gz`,
  `ide-0.1.0-third-party-licenses.txt` and `SHA256SUMS`.
  The dry run of the full workflow (run 39878025410) passed before it.
- Releases are immutable in this repository,
  so `ide-v0.1.0` can never receive an ARM64 archive.
  ARM64 first ships with the next version,
  and that release holds four files (see "Files and names").
- The `version` in `package/ide/app/Cargo.toml` is not changed by the ARM64 change,
  because a version change merged to `main` starts a real release.
  The owner raises it after reviewing the change.

### One native build per architecture

The `build` job is a matrix of two entries on named runners:
`x64` on `ubuntu-24.04` and `arm64` on `ubuntu-24.04-arm`.
Both are Ubuntu 24.04 and so share glibc 2.39.
The labels are named and not `ubuntu-latest`,
which GitHub moves to Ubuntu 26.04 in November 2026
(`doc/decision/nested-wayland-session-releases.md` records the same reason for the compositor).
`fail-fast` is off:
a failed leg already blocks the release,
because the `checksums` and `release` jobs need both,
and a dry run is most useful when it shows both results.

The build is native on both and needs no cross compiler and no emulation.
The tasks start the IDE image in podman,
and the image is built for the architecture of the host.
The base release `Containerfile.release` (`package/terminal/app/Containerfile.release`) has branches for `x86_64` and `aarch64`
where the architecture matters (the Zig tarball with its SHA-256,
and the `cargo-nextest` download),
and everything else is an Ubuntu 24.04 package or a rustup download that exists for both
(until 2026-10-29 the packages were Fedora 41 packages of `Containerfile`).
The compositor release and the music player release use the same two runners
(`doc/decision/nested-wayland-session-releases.md`,
`doc/decision/music-player-releases.md`).
The music player's Alpine job already runs podman with `--memory` and `--cpus` on `ubuntu-24.04-arm`
(dry run 38684256021 in `doc/decision/music-player-releases.md`:
the aarch64 job took 14 min 07 s).
The step "Record the toolchain of the build image" is where `--pids-limit` and `--ulimit` meet the ARM64 runner for
the first time,
as they met the x86_64 runner in the first dry run.

The matrix passes these facts to the steps through the environment:
`ARCH` (`x64` or `arm64`,
the spelling of the asset names),
`MACHINE` (`x86_64` or `aarch64`,
what `uname -m` and the compiler's host triple say),
and `EXPECTED_NEEDED` (the exact sorted list of shared libraries in the dynamic section).
`GLIBC_FLOOR` stays one value.

### Files and names

The ARM64 archive is `ide-<version>-linux-arm64.tar.gz`.
Its contents are those of the x64 archive,
under a single top-level directory `ide-<version>-linux-arm64/`.
The spelling follows the music player's (`music-player-<version>-linux-x64.tar.gz` and
`music-player-<version>-linux-arm64.tar.gz` in `.github/workflows/music-player-release.yml`),
and the IDE's existing `linux-x64` already matches it.
The compositor release uses Rust target triples
(`monochromatic-nested-wayland-session-<version>-aarch64-unknown-linux-gnu.tar.gz`)
for a reason that does not apply here:
its `[package.metadata.binstall]` names the archive by the target,
and the IDE is `publish = false` with no binstall metadata.
The x86_64 name `ide-<version>-linux-x64.tar.gz` is unchanged.

A release from the next version holds four files:
the two archives,
`ide-<version>-third-party-licenses.txt` and `SHA256SUMS`.
`SHA256SUMS` covers all three assets,
because the `checksums` job lists the downloaded `asset-*` artifacts,
and it fails unless that set is exactly the three expected names.
The workflow artifacts are `asset-linux-x64`,
`asset-linux-arm64`,
`asset-third-party-licenses` and `checksums`.

Attestation subjects:
each leg attests its own archive;
the x64 leg also attests the notices file;
the `checksums` job attests `SHA256SUMS`.
Every released file is attested,
as before.

### The notices stay one file

The notices file is what the executable prints with `--licenses`.
It is built from `target/crate-licenses.json`,
which the `notices` task writes with `cargo-about` for the targets `about.toml` lists.
That list was `x86_64-unknown-linux-gnu` only,
so on ARM64 the notices would have described the x86_64 graph.
Measured on 2026-10-22 with `cargo-about generate --frozen --fail` and its `--target` flag:
the x86_64 graph has 449 crates in 204 license texts,
the aarch64 graph has 448 crates in 203 texts,
and the only crate that aarch64 lacks is `core_detect`.
With both targets in `about.toml` the output is byte for byte the x86_64-only output.
The x86_64 notices of the next release therefore equal what they would have been,
and the ARM64 executable carries a list that covers all it links (448 of the 449) plus one crate it does not link.
The safe direction was chosen on purpose:
a notice that is too long is harmless,
and one that is too short is not.
The `crate-licenses` gate stays:
it fails when a linked crate has no text.

Both executables therefore print the same notices,
and the release carries one file.
The x64 leg uploads it as the asset.
The arm64 leg uploads its own copy as `notices-arm64`
(a name that does not start with `asset-`,
so it is never released),
and the `checksums` job fails unless the two files are identical.
That gate also fails when a later change makes the graphs differ in a crate that `about.toml` does not cover.

### Gates per leg, and what changed in them

Each leg runs every gate of the x86_64 workflow,
and none is weakened:

- A new step fails the leg when `uname -m` is not the `MACHINE` of the matrix entry,
  so a runner label that changes meaning cannot produce a build of the wrong architecture unnoticed.
- The step "Record the toolchain of the build image" now also fails unless the compiler's host triple is
  `<MACHINE>-unknown-linux-gnu`.
  The `rustc` line goes to the release notes through an artifact (`toolchain-x64` and `toolchain-arm64`),
  because a matrix job cannot hand a job output to a later job:
  the legs would overwrite each other.
  The notes print one line when the two match and both lines when they differ.
- The shared libraries gate compares the dynamic section with a list in the matrix entry.
  On x64 it is `ld-linux-x86-64.so.2 libc.so.6 libfontconfig.so.1 libgcc_s.so.1 libm.so.6`,
  as before.
  On arm64 it is `libc.so.6 libfontconfig.so.1 libgcc_s.so.1 libm.so.6`:
  the aarch64 executable names no dynamic loader (see "The aarch64 build,
  run to the end under emulation").
  A list per architecture keeps the gate exact on both,
  where dropping the loader from the comparison would have loosened the x64 gate.
- The glibc floor is the same ceiling of 2.39 on both.
  It is an upper bound (the gate fails when a symbol needs more),
  so an ARM64 executable that needs less passes,
  and the step prints the version it found.
- `inspect:no-software-renderer` runs every query for both release targets on whichever host runs it,
  with the positive controls for both:
  a renderer that only one target pulls in is seen,
  and an empty answer on a target counts only if the controls reach the renderer on that target.
  The index pass that fills the cargo volume is unchanged.
- The `linked-crates` task uses the target the container compiles natively
  (`aarch64-unknown-linux-gnu` when node reports `arm64`,
  `x86_64-unknown-linux-gnu` otherwise),
  so the `crate-licenses` gate compares the crates this executable links.
- `INSTALL.txt` names the machine (`x86_64` or `aarch64`) in its first line and in "Needs".
  Everything else in it holds for both.
- The release notes list both archives,
  state that no Windows,
  macOS,
  musl or other-architecture build exists,
  and say that the maintainers have run the program on x86_64 only
  and that nobody has opened its window on ARM64 hardware.
- The dry run is unchanged in kind:
  both legs build,
  package and verify,
  both upload one-day artifacts,
  and the release job prints the `gh release create` command with the three assets plus `SHA256SUMS`.

### Audit of x86_64 assumptions

Found and changed:
the matrix and the runner;
the expected library list of the libraries gate (one per architecture);
the `--target x86_64-unknown-linux-gnu` of `inspect:no-software-renderer` and of `linked-crates`;
the `targets` of `about.toml`;
the archive name,
the stage directory,
the artifact names,
the attestation subjects,
the expected asset set,
the release notes,
and the first line and the "Needs" line of `INSTALL.txt`;
the README sections "What the host provides" and "Releases";
the runbook.

Found and left alone,
with the reason:

- `bin/gpu-arguments.ts` names `radeon_icd.x86_64.json`.
  It is the RADV driver file of the author's machine for the native GPU test tasks,
  which no release step runs.
- `bin/inspect-language*.ts` and `bin/inspect-server-watches.ts` copy `@typescript/typescript-linux-x64` from a
  pnpm store.
  They are developer inspection scripts that start language servers,
  and no release step runs them.
- The test fixtures in `src/language/mise_tests.rs` and `src/language/mise/parse_tests.rs` that contain
  `linux-x64` are sample output of `mise` for a parser test.
- `bin/container-bounds.ts` has no architecture in it.
- `mise.lock` pins node 26.12.0 with a `linux-arm64` platform entry (URL and SHA-256),
  so `mise install node` in the workflow resolves on the ARM64 runner.
  The pinned mise 2026.10.9 publishes a `linux-arm64` binary (`gh release view v2026.10.9 --repo jdx/mise`);
  that `jdx/mise-action` installs it on the runner is not proven until the dry run.
- The IDE `Containerfile` installs Fedora packages (`gcc-c++`,
  `ripgrep`,
  `vulkan-loader`,
  `mesa-vulkan-drivers`)
  and builds `cargo-about` from source,
  none of which names an architecture.
  (Written about the Fedora 41 image.
  Since 2026-10-29 the release image is Ubuntu 24.04: `package/ide/app/Containerfile.release` installs `g++`,
  `ripgrep`, `libvulkan1` and `mesa-vulkan-drivers` with `apt` and builds the same `cargo-about` 0.9.2,
  and dry run 40887621364 built it on both architectures.)
- The base `Containerfile` already had the `aarch64` branches for Zig and `cargo-nextest`.
  They were checked against their sources on 2026-10-22:
  the Zig 0.15.2 index at `ziglang.org` lists the SHA-256 `958ed7d1...` for `aarch64-linux` and `02aa270f...` for
  `x86_64-linux`,
  both equal to the digests in the file,
  and `https://get.nexte.st/latest/linux-arm` redirects to the `aarch64-unknown-linux-gnu` archive of
  `cargo-nextest`.
  The ARM64 images also built from them (see "What was proven here,
  and how").

### What was proven here, and how

All of this ran on 2026-10-22 on the author's x86_64 machine.

- `cargo check --frozen --target aarch64-unknown-linux-gnu` of the whole package with its default features passed
  in the repository's IDE image
  (rustc 1.98.0 on x86_64 with the `rust-std` component for aarch64 added;
  392 crates checked or compiled in 1 min 18 s from an empty aarch64 target directory;
  the library and the binary both type-checked).
  This covers Slint with winit,
  femtovg and vello,
  wgpu,
  the Wayland crates,
  fontconfig,
  the Helix crates and
  `tree-house`.
  The C code of `tree-house-bindings` compiled with Zig as the aarch64 C compiler,
  and `yeslogic-fontconfig-sys` found the host's fontconfig through `PKG_CONFIG_ALLOW_CROSS`.
  `cargo check` links nothing,
  so this proves that the Rust and the C sources compile for aarch64 and says nothing about linking.
  It needed the embedded runtime in `target/aarch64-unknown-linux-gnu/debug/runtime`,
  where `build.rs` looks for it in a cross build,
  which is why it is not a mise task:
  the repository's flow is always a native build.
- The ARM64 build images built from the repository's Containerfiles under `qemu-user` (podman with
  `--platform linux/arm64`,
  binfmt registered on the machine):
  the base image in 8 min,
  the IDE image in 26 min with `cargo-about` 0.9.2 compiled from source.
  That proves the Fedora packages,
  the Zig download with its digest check,
  the rustup installer (`stable-aarch64-unknown-linux-gnu`,
  rustc 1.99.1),
  the `cargo-nextest` download and the `cargo-about` build for aarch64.
  (Measured on 2026-10-22 with the Fedora 41 images, which the release has not used since 2026-10-29.)
- `inspect:no-software-renderer` passed on x86_64 with the two-target loop,
  and the positive controls reached the renderer on both targets.
- `linked-crates` and `notices` ran on x86_64 with the changed tasks;
  the notices were 204 texts for 449 crates,
  as before the change.
- The new and changed workflow steps ran with `bash -e` on the author's machine,
  as the section "Checks run before landing" records for the x86_64 workflow:
  the architecture check,
  the toolchain step against the x86_64 image,
  the libraries gate (the x86_64 build passes,
  and the same executable fails with the arm64 list),
  the notices,
  the packaging and the archive check for both `ARCH` values,
  the asset set and the notices comparison (and its failure when the notices differ),
  and the release notes and the dry-run command.
  The ARM64 packaging ran with the x86_64 executable as a stand-in,
  so it proves the shell and not an ARM64 binary.
- `actionlint` 1.7.12 and `zizmor` 1.31.1 (`--offline`) report nothing on the workflow.

### The aarch64 build, run to the end under emulation

The repository's own `bundle` task ran in the ARM64 build image under `qemu-user`
(private podman storage,
16 CPUs and 32g for the container,
with the host loaded by other work),
from a worktree at the commit of the workflow change.
It is the nearest thing to the runner that this machine can run:
the same Containerfiles,
the same tasks,
the compiler `rustc 1.99.1 (58cf8f9c3 2026-10-13)` of the image
(host `aarch64-unknown-linux-gnu`),
and a native aarch64 link by Fedora 41's toolchain
(the image of 2026-10-22; the release image is Ubuntu 24.04 since 2026-10-29).
It is not the runner:
the CPU is emulated,
and the host is not Ubuntu.

- `bundle` finished in 8,774 s (the `runtime` task's debug build took 83 min of it and the release build 55 min),
  and wrote `dist/monochromatic-ide`,
  90,611,544 bytes
  (the x86_64 executable of `ide-v0.1.0` is 91,945,952).
- Run inside the ARM64 image (the host cannot run an aarch64 dynamic executable without an aarch64 sysroot):
  `env --ignore-environment dist/monochromatic-ide --version` printed `monochromatic-ide 0.1.0`,
  and `objdump -p` showed the file as `elf64-littleaarch64` with the interpreter `/lib/ld-linux-aarch64.so.1`.
- The dynamic section lists `libc.so.6 libfontconfig.so.1 libgcc_s.so.1 libm.so.6` and nothing else.
  The loader `ld-linux-aarch64.so.1` is not in it,
  although the x86_64 executable lists `ld-linux-x86-64.so.2`.
  (This is the executable linked in the Fedora 41 image.
  The arm64 executable linked in the Ubuntu 24.04 release image does list `ld-linux-aarch64.so.1`:
  dry run 40883876777 failed this gate on it, and the matrix entry gained the loader in `cb03f20350`.)
  The first plan for the gate (replace the loader name with the aarch64 one) would have failed on the first run,
  so the gate now takes the whole expected list from the matrix entry:
  the x64 list is unchanged,
  and the arm64 list was `libc.so.6 libfontconfig.so.1 libgcc_s.so.1 libm.so.6`
  (it is `ld-linux-aarch64.so.1 libc.so.6 libfontconfig.so.1 libgcc_s.so.1 libm.so.6` since the Ubuntu 24.04 release image).
  `objdump -T` gave the newest glibc symbol versions 2.34,
  2.35 and 2.39,
  so the ceiling of 2.39 holds with no change.
- `--licenses` printed 11,241 lines and 581,554 bytes that `cmp` finds byte for byte equal to the x86_64
  executable's notices (both at version 0.1.0).
  The `notices` task of this run wrote 204 texts for 449 crates under `about.toml` with both targets,
  as on x86_64.
- The `inventory` check of `inspect:bundle` passed on the aarch64 executable
  (1,255 embedded files,
  27 grammars,
  341 highlight rules,
  the pinned Helix revision),
  the same counts as the x86_64 build.
  The `license-texts` and `crate-licenses` checks could not run in this setup:
  `inspect:bundle` starts the executable from the host's node,
  and `qemu-aarch64-static` could not open `/lib/ld-linux-aarch64.so.1` on an x86_64 host.
  That is a limit of the setup and not of the executable.
  Their content was checked another way:
  the notices equal the x86_64 ones byte for byte,
  and all 391 crates that `cargo tree --target aarch64-unknown-linux-gnu` lists as linked
  and all 320 registry crates whose source paths the aarch64 executable names are among the crates of the list.
  The `linked-crates` task of that run used the host's node,
  which reports `x64`,
  so it listed the x86_64 graph
  (393 crates);
  on the runner both are arm64.
- `inspect:no-software-renderer` passed in the ARM64 image on a new cargo volume,
  after its index pass had fetched the entries for the positive controls,
  with the two-target queries and both positive controls.

### What only the ARM64 runner can prove

- That podman on `ubuntu-24.04-arm` accepts `--pids-limit=1024` and `--ulimit nofile=4096:4096`
  (the music player proves `--memory` and `--cpus` there).
- The disk,
  the time and the memory of a native cold build on that runner.
  The step that removes preinstalled software is kept for both legs;
  the x86_64 dry run showed that it was not needed,
  and nothing is known about the ARM64 image.
- That the executable the runner links has the dynamic section the gate expects and a glibc ceiling of 2.39 or less.
  The emulated build measured both with the same Containerfiles and a native aarch64 link,
  so the runner is expected to agree,
  but the image is rebuilt on the day with whatever the Fedora 41 packages and `stable` Rust then were.
  (Resolved by the dry runs on the Ubuntu 24.04 release image: 40877021793 and 40887621364 measured the newest glibc symbol at 2.39 on both runners.
  The Rust toolchain is now the nightly pinned in `toolchain.lock.json`.)
- That the `--version` smoke test,
  the archive check and `desktop-file-validate` run on that runner.
- Attestation,
  the artifact upload and download,
  the `checksums` job with two archives,
  and the `gh release create` call with four files:
  none can be run before a dry run and a real release,
  as with the x86_64 workflow.
- That the program starts at all on ARM64 hardware.
  No ARM64 machine was available,
  CI never opens a window,
  and the runbook asks for a start of the dry run's archive on an ARM64 Wayland machine with a hardware Vulkan GPU.
  Nothing here shows that the Vulkan stack,
  the GPU device selection or the fonts behave on ARM64 desktops.

### Consequences of the ARM64 build

- A real release needs both legs green.
  If the ARM64 leg fails after the x64 leg has passed,
  nothing is published and no partial release exists.
  A dry run of the next version on `main` is the guard,
  as before.
- Two runners double the runner time of a release,
  and the two legs run in parallel.
- The ARM64 archive ships without a start on real hardware,
  and the release notes say so.
  If the owner wants a hardware start first,
  the runbook has the command.
- The glibc floor of 2.39 is one value for both architectures.
  If one of them needs a newer glibc the gate fails and the decision is revisited.
- The notices of the ARM64 executable list one crate (`core_detect`) it does not link,
  and would list any crate that only the x86_64 graph links.

### Alternatives rejected for ARM64

- Cross compiling the ARM64 executable on the x86_64 runner:
  it needs an aarch64 sysroot with the fontconfig and Wayland development files,
  the repository's mise tasks and the container are native-only,
  and the gates (`--version`,
  the archive check,
  `--licenses`) could not run the result.
- Building under `qemu-user` on the x86_64 runner:
  the emulated `bundle` took 2 h 26 min on the author's machine (the images another 34 min),
  and a native runner exists.
- A separate workflow or release for ARM64:
  releases are immutable,
  so one release has to carry every asset in one `gh release create` call.
- The compositor's target-triple names (`...-aarch64-unknown-linux-gnu.tar.gz`):
  they exist because of `cargo-binstall`,
  which the IDE does not use,
  and they would make the IDE's names inconsistent with its own `linux-x64`.
- A separate notices file for ARM64:
  the lists are the same by measurement,
  and a second file would double the licensing text for no reader.

## Consequences

- The first real release of a version is permanent.
  A broken release can be fixed only by a higher version,
  and its tag stays reserved.
  The dry run on `main` and a read of its log are the guard,
  and `doc/runbook/release-the-ide.md` says how.
- Version `0.1.0` is already in `main`,
  so the first release needs a dispatch from `main` with `dry-run` false,
  after a green dry run on `main`,
  and the owner's go-ahead.
  A dispatch can only find the workflow after it is merged to `main`,
  so the first dry run happens after the merge.
- A version bump needs a refreshed `Cargo.lock`.
  The build has no `--locked`,
  and the workflow's lock check fails when the build rewrites the lock file.
- The release is not reproducible bit for bit,
  and a day's change in the Ubuntu 24.04 packages (the `ubuntu:24.04` tag follows point releases)
  or the `cargo-nextest` `latest` download can fail or alter a build with no repository change.
  The release image installs them unpinned (`package/terminal/app/Containerfile.release`).
  The Rust toolchain is pinned to the nightly of `toolchain.lock.json` (`ARG RUST_TOOLCHAIN`, kept in step by hand),
  and Zig by version and SHA-256, so those two no longer move on their own.
  Pinning the rest changes what developers build with and is a separate decision.
  (Until 2026-10-29 the list also named Fedora 41 and floating `stable` Rust.)
- The Rust toolchain of a release is not the `mise.lock` nightly.
  Each release names its compiler in the notes.
- The release job's checks open no window.
  A program that builds and passes every gate but cannot start on a GPU would still ship.
  The runbook asks for a start of the dry run's archive on the Wayland machine first.
- Only Linux x86_64 on glibc 2.39 or newer is built.
  There is no Windows,
  macOS,
  arm64 or musl build,
  and the notes say so.
  Since 2026-10-22 the workflow also builds Linux arm64 (see "Linux ARM64"),
  but `ide-v0.1.0` is x86_64 only and stays so.
- The licensing of the shipped bytes (Slint under `GPL-3.0-only` inside a package whose manifest says
  `LGPL-3.0-or-later`) is now published.
  The owner should confirm the choice before the first real release.
- The `jdx/mise-action` pin is shared with the music player;
  a person updates it (`doc/runbook/release-the-music-player.md`,
  "The mise version pin").
- Cold builds cost run time on every release:
  15 min 37 s for the `build` job up to the renderer gate in the first dry run.

## Alternatives rejected

- A native `cargo build` on the runner with the `mise.lock` toolchain:
  it needs the image's packages installed on the runner and a re-implementation of `runtime` and `notices` outside mise,
  and it has never been run for this package.
  (Written when the image was Fedora 41, where it needed Ubuntu equivalents of the Fedora packages.)
- Building the executable once in a separate job and checking it in another:
  there is one platform and one build,
  and a second job would repeat the container image build.
- Shipping the notices only inside the archive:
  the music player ships them as a file of their own so that a person can read them without extracting 23 MB.
- Adding the window checks with a software Vulkan adapter on the runner:
  the program refuses CPU adapters on purpose (owner decision of 2026-10-19),
  and a check that needs a switch that production lacks proves something other than the release.
- An AppImage or a deb package:
  the program is one executable,
  as the music player's Linux archive is,
  and a package manager install has no tested distribution matrix behind it.
