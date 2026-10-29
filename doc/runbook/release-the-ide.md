# Release the IDE

Use this to publish a GitHub pre-release of the IDE (`package/ide/app`,
program `monochromatic-ide`),
to rehearse one with a dry run,
and to recover when a release run fails.
The design and its reasons are in `doc/decision/ide-releases.md`.
The workflow is `.github/workflows/ide-release.yml`.
It is built like the music player's (`doc/runbook/release-the-music-player.md`),
and several sections point there instead of repeating it.

## What a release is

A release holds four files
(`ide-v0.1.0` predates the ARM64 archive and holds the other three):

- `ide-<version>-linux-x64.tar.gz` (Linux x86_64) and `ide-<version>-linux-arm64.tar.gz` (Linux ARM64,
  aarch64):
  `bin/monochromatic-ide` (one executable,
  about 92 MB,
  with its grammars,
  queries and license texts embedded),
  `share/applications/monochromatic.ide.desktop`,
  `LICENSE`,
  `LICENSES/`,
  `THIRD-PARTY-LICENSES.txt`,
  and `INSTALL.txt`.
  Each is built natively on its own runner
  (`ubuntu-24.04` and `ubuntu-24.04-arm`).
- `ide-<version>-third-party-licenses.txt`:
  the license notices,
  byte for byte what `monochromatic-ide --licenses` prints and what each archive holds as `THIRD-PARTY-LICENSES.txt`
  (both executables print the same notices;
  the workflow fails if they do not).
- `SHA256SUMS`.

The tag is `ide-v<version>`.
Every release is flagged as a pre-release.
Every file is attested with GitHub build provenance.
Only Linux x86_64 and Linux ARM64 are built.

The release `ide-v0.1.0` holds the x86_64 archive only:
it was published before the ARM64 build existed,
and it is immutable,
so the ARM64 archive first ships with the version after it.

Releases are immutable in this repository.
After a release is published,
its tag and its files can never be replaced or added to,
so a bad release is fixed only by releasing a higher version.
This is why the first real release of a version is a deliberate act and not an experiment.

## The version

The single source is the `version` line of the `[package]` table in `package/ide/app/Cargo.toml`.
The workflow enforces,
and fails loudly on:

- The version is plain `MAJOR.MINOR.PATCH`.
  A suffix such as `-beta.1` is refused.
- On a push,
  the new version is higher than the old one.
- The built program prints exactly `monochromatic-ide <version>` for `--version`.
- The build leaves `package/ide/app/Cargo.lock` and every other tracked file unchanged.

## Cut a release

1.  Pick the new version and edit the `version` line in `package/ide/app/Cargo.toml` by hand.
2.  Refresh the lock file without touching any dependency,
    because the workflow fails when the build rewrites `Cargo.lock`:
    run `mise run //package/ide/app:fetch` (it runs `cargo fetch` in the build container,
    which writes the new version into the `ide` entry of `Cargo.lock`),
    and check that `git diff` shows exactly one changed line in `package/ide/app/Cargo.lock`.
3.  Commit both files and merge to `main`.
    A push to `main` that changes `package/ide/app/Cargo.toml` starts the workflow.
    The detect job compares the old and new version and starts the build only if the version changed.
4.  Rehearse first when the change is more than a version bump:
    a dry run (see "Dry run") on the branch gives the same build without a tag or a release.
5.  Watch the run:
    `gh run list --workflow ide-release.yml` and `gh run watch <run id>`.
    The build job is the long one,
    because it builds the container images and the program from empty.
6.  Open the release page `https://github.com/Aquaticats/Monochromatic/releases/tag/ide-v<version>` and check that
    the four files are attached (both archives,
    the notices and `SHA256SUMS`) and that it is marked as a
    pre-release.
7.  Verify one file as a user would (see "Verifying attestations").

If the tag already exists when the run starts,
the run skips the release and succeeds with a notice.
It never overwrites a tag.

## The first release of an existing version

The version `0.1.0` was in `main` before the workflow existed,
so no push will ever release it:
a push releases only when the version changes.
The workflow file must be on `main` before `gh workflow run` can find it.
Release `0.1.0` by dispatching a real run from `main`,
after a dry run has passed on `main`,
after the archive of that dry run has been started on a Wayland machine with a GPU (see "Dry run"),
and with the owner's go-ahead:

```bash
# Rehearsal first: builds and verifies everything, creates nothing.
gh workflow run ide-release.yml --ref main --field dry-run=true

# The real release. Immutable: do this only after the dry run is green and you have read its output.
gh workflow run ide-release.yml --ref main --field dry-run=false
```

## Dry run

A dry run builds,
packages and verifies every file,
runs the same checks as a real run,
uploads the files as workflow artifacts that expire after one day,
and prints in the `release` job log the exact `gh release create` command,
the tag,
the asset list and the release notes it would use.
It skips attestation and creates no tag and no release.

Start one with `gh workflow run ide-release.yml --ref <branch> --field dry-run=true` (the workflow file must exist on
`main` before `gh workflow run` can find it).
A dispatch from any ref other than `main` is always a dry run,
and the run refuses `dry-run=false` there.

The `build` job runs once per architecture (`build (x64)` and `build (arm64)`),
and both legs must pass before the `checksums` and `release` jobs start.
A failed leg blocks the whole release,
and an immutable release cannot receive the missing archive later,
so read the log of both legs.

Read these steps of each `build` leg:

- "Check the runner architecture" prints the machine name the runner reports (`x86_64` or `aarch64`).
- "Free disk space on the runner" and the later `df` lines print how much of the disk the build used.
  The first dry run's numbers and the cold build time are in the decision document,
  section "First dry run".
- "Record the toolchain of the build image" prints `rustc`,
  `cargo` and `cargo-about` versions.
  The `rustc` line goes into the release notes.
- "Smoke test the executable (--version)" prints `monochromatic-ide <version>`.
- "Check the shared libraries and the glibc floor" prints the `needed` libraries and the newest glibc symbol version.
  On x64 the list includes the dynamic loader `ld-linux-x86-64.so.2`;
  on arm64 it includes `ld-linux-aarch64.so.1`
  (`ld-linux-aarch64.so.1 libc.so.6 libfontconfig.so.1 libgcc_s.so.1 libm.so.6`, dry run 40887621364).
  An arm64 executable linked in the Fedora 41 image had no loader entry, and one linked in the Ubuntu 24.04 release image has it.
- "Check that no software renderer is linked" ends with a line that starts
  `Slint software renderer is not linked into any build of this package`,
  names both targets (`x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`),
  and says that the positive controls reach the renderer for each.
  Both legs check both targets.
- "Check the single executable" prints one JSON line per check and ends with
  `Bundle checks passed ... (3 of 10 checks)`.
  The other seven checks need a Wayland session and a GPU,
  so CI does not run them.
- "Check the archive" ends with `The archive holds the verified program, its notices and a valid launcher entry.`

Download a dry run's files with `gh run download <run id> --dir <new empty directory>`.
Each `asset-*` artifact becomes a subdirectory
(`asset-linux-x64`,
`asset-linux-arm64`,
`asset-third-party-licenses`),
and the run also keeps `checksums`,
`notices-arm64` and the two `toolchain-*` artifacts,
which are not released.
The continuous integration run opens no window,
so before a real release start the program from the download on a Wayland machine with a hardware GPU:

```bash
# Extract the archive of the dry run into a new empty directory, then start the program on a project folder.
tar --extract --gzip --file <download dir>/asset-linux-x64/ide-<version>-linux-x64.tar.gz --directory <extract dir>
<extract dir>/ide-<version>-linux-x64/bin/monochromatic-ide <a project folder>
```

On an ARM64 machine with a Wayland session and a hardware Vulkan GPU,
do the same with `asset-linux-arm64` and `ide-<version>-linux-arm64`.
No ARM64 machine has run the program yet (see "What is not tested"),
so this start is the first evidence about the ARM64 build.
If no such machine is at hand,
whether to release without that start is the owner's call;
the release notes state that nobody has opened the ARM64 window.

The window must open and show the folder,
and the file tree must list it.
Close the window and check that the exit status is 0.
The session checks of `inspect:bundle` cannot be repeated on the download,
because they compare the executable with the runtime directory it was built from,
and the grammar libraries compiled on your machine are not byte for byte the ones compiled on the runner.

## When a job fails

Before the release job started,
nothing is published.
Read the failure with `gh run view <run id> --log-failed`,
fix the cause on a branch,
merge,
and start a new run (a dispatch of the same version,
or the next push).
`gh run rerun <run id> --failed` also works for a transient failure of the build job,
unless the job had already uploaded its files:
the upload then conflicts with the file already uploaded and a fresh dispatch is needed.

Failures that belong to this workflow:

- "Check the runner architecture" fails:
  the label in the matrix (`ubuntu-24.04` or `ubuntu-24.04-arm`) no longer gives the machine the entry names.
  Fix the matrix;
  do not weaken the check.
- The arm64 leg fails and the x64 leg passes (or the other way round):
  read the failing step of that leg.
  The steps that differ by architecture are the expected library list (`matrix.needed`) in
  "Check the shared libraries and the glibc floor",
  the image builds (the base image downloads an architecture-specific Zig and `cargo-nextest`),
  and the compile itself.
  A new library,
  a missing or additional loader or a newer glibc symbol on one architecture only means the promise in
  `INSTALL.txt` is wrong for it;
  the entry "Check the shared libraries and the glibc floor" in this list says what to do.
- The `checksums` job fails with "The arm64 executable prints other license notices than the x64 executable":
  the two build graphs now differ in a crate that `about.toml` does not cover.
  Both release targets must be listed in `targets` there;
  rerun the `notices` task on each architecture and compare.
- The step "Build the base image (terminal release)" or "Build the IDE release build image" fails while downloading:
  the release images (Ubuntu 24.04, `Containerfile.release` of `package/terminal/app` and `package/ide/app`,
  selected by `IDE_RELEASE=1` in the job `env`) fetch Zig from `ziglang.org`,
  the Rust installer from `sh.rustup.rs`,
  `cargo-nextest` from `get.nexte.st`,
  Ubuntu packages with `apt`,
  and build `cargo-about` from source.
  Any of them can be unavailable for a while.
  Rerun the failed job.
- "Record the toolchain of the build image" fails with a cgroup or `OCI runtime` error:
  the runner refused a limit (`--memory`,
  `--cpus`,
  `--pids-limit` or `--ulimit`).
  The music player's Alpine jobs prove `--memory` and `--cpus` only.
  Change the limit in that step and in the bound (`IDE_CONTAINER_*` in the job `env`) together.
- "Build and bundle the single executable" fails with no space left on the device:
  the `df` lines of the job show where the disk ran out.
  Free more of the runner (the step "Free disk space on the runner") or move the container storage.
- "Check that the build left the tracked files unchanged" fails:
  the version was raised without refreshing `Cargo.lock` (see "Cut a release",
  step 2),
  or the build changed a dependency on its own.
  The diff is in the log.
- "Check the shared libraries and the glibc floor" fails:
  the executable now needs a library or a glibc version that `INSTALL.txt`,
  the README section "What the host provides" and the release notes do not name.
  Decide whether that is intended,
  then update those texts and the expected list (`matrix.needed`,
  one per architecture)
  or `GLIBC_FLOOR` in the step.
  The floor is one value for both architectures;
  if only one needs a newer glibc,
  the floor is raised for both or the check gets a per-architecture value,
  which is a decision for the decision document.
- "Check that no software renderer is linked" fails with `no matching package named ... found` and the note about
  `--frozen`:
  the cargo volume lacks a registry index entry for a crate that the positive control `slint/mcp` resolves.
  The task fills those entries with a network pass before it checks offline;
  if that pass could not reach the network,
  its message says so in the log,
  and a rerun is the cure.
  A different message means the software renderer is linked,
  and the tree it prints names the path.
- "Check the single executable" fails with a named check:
  the message names the embedded file or license text that is missing or differs.
  `mise run //package/ide/app:runtime` and `notices` rebuild the inputs locally.

The release job can fail after it started.
Look at `gh release list --repo Aquaticats/Monochromatic`:

- No entry,
  or a draft entry:
  nothing was published.
  Delete a draft with `gh release delete ide-v<version> --yes`,
  check that `git ls-remote --tags origin ide-v<version>` prints nothing,
  and dispatch again.
- A published entry with missing or wrong files:
  it cannot be repaired,
  because the release is immutable and its tag is reserved even if you delete the release.
  Release the next patch version instead,
  and state in that release's notes that the previous version is broken.

## The mise version pin

The `build` job installs mise with `jdx/mise-action` in the step "Set up mise (skip auto-install of all tools)" and
pins `version: 2026.10.9`.
The reason,
the owner,
and the steps to update are in `doc/runbook/release-the-music-player.md`,
section "The mise version pin".
That section names the music player workflow's steps;
update this step to the same version in the same change,
and check `mise --version` in this workflow's dry run.

## Caches

The workflow caches nothing.
Every run builds the container images,
fetches the crates and the grammars,
and compiles the program from empty.
A cache would need the key and size rules of `doc/decision/ci-caching.md`;
the decision document records why there is none.

## Verifying attestations

Each file carries build provenance that binds its bytes to this workflow run and commit:

```bash
# Verify one downloaded file; fails if the bytes were changed after the workflow built them.
gh attestation verify ide-<version>-linux-x64.tar.gz --repo Aquaticats/Monochromatic
gh attestation verify ide-<version>-linux-arm64.tar.gz --repo Aquaticats/Monochromatic

# Compare the whole set with the checksum file (verify SHA256SUMS itself the same way).
sha256sum --check --ignore-missing SHA256SUMS
```

Dry runs are not attested.

## What is not tested

The project has run the program on Linux x86_64 under Wayland only.
The ARM64 archive is built and checked natively on an ARM64 runner,
but nobody has started its program on ARM64 hardware.
CI starts each executable with `--version` and checks its embedded files and notices,
and it never opens a window,
because a runner has no display and no GPU.
A bug report from another distribution,
session type or GPU is the first evidence about it.
