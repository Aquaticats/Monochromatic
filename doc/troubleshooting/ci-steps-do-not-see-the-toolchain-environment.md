# A CI step that calls `task-run` sees neither the toolchain's `PATH` nor its environment

## Symptom

A workflow step that runs `task-run <task>` cannot find a tool the repository installed,
or quietly uses one from the runner image instead:

```text
cargo-about is not on PATH (spawnSync cargo-about ENOENT). Install the repository tools with
`mise install cargo:cargo-about` and run this through its mise task.
```

Dry run 42448767356 of `music-player-release.yml` failed the Android job's notices step with that message,
after the toolchain install in the same job reported that it had installed `cargo-about` and `cargo-ndk`.

## Cause

`toolchain:install`,
which the Android job runs as `task-run prepare:android`,
links every tool `toolchain.lock.json` records into one directory,
`.cache/toolchain-bin`,
and writes the lock's `env` blocks into `.config/mise/conf.d/30-toolchain-env.local.toml`,
with each relative value joined to the toolchain root.

Two mise files read those outputs locally:
`.config/mise/conf.d/20-env.toml` puts the exposure directory first on `PATH`,
and mise reads every file of its `conf.d` directory as environment.
`task-run` is not mise.
It adds only the `node_modules/.bin` directories of the task's ancestors to `PATH`,
and passes the invocation's environment through unchanged,
so neither output reaches a child process.

The workflow therefore has to write both into the runner's own files itself:
one step appends `mpm-shadow bin` to `$GITHUB_PATH`,
and one step appends the two Android names of the tool environment file to `$GITHUB_ENV`.
The Android job of `music-player-release.yml` has both,
named "Put the provisioned toolchain on PATH" and "Export the Android SDK the toolchain installed".

## A hosted runner already sets some of these names

The Ubuntu runner images set `ANDROID_HOME` and `ANDROID_SDK_ROOT` to their preinstalled SDK
(`/usr/local/lib/android/sdk`,
listed in the image's own README),
whose NDK is not the one `toolchain.lock.json` pins and whose components move with the image.
A step that never exports the toolchain's values therefore builds and packages against the image's SDK,
and no line of the job log says so.
That is why the export step reads the file the toolchain wrote instead of only putting a directory on `PATH`.

## What not to export

The tool environment file also carries `JAVA_HOME`,
and on a machine whose JDK came from the toolchain that value names the tool's download folder
(`.../java/temurin-21.0.13+9.0.LTS`),
not the extracted JDK inside it (`.../temurin-21.0.13+9.0.LTS/jdk-21.0.13+9`).
`gradlew` refuses such a directory with
`ERROR: JAVA_HOME is set to an invalid directory`,
which `mise exec -- ./gradlew --version` in `package/music-player/android-app` reproduces on 2026-11-08.
The release workflow sets `JAVA_HOME` and `RUSTUP_TOOLCHAIN` from its own JDK download and `rustup` instead,
so the export step carries only the Android names.

## The Rust toolchain a CI step installs first never gets the lock's Android targets

### Symptom

The Android job of `music-player-release.yml` fails its first native crate with a missing standard library,
while the same task passes on a developer machine:

```text
error[E0463]: can't find crate for `core`
  = note: the `aarch64-linux-android` target may not be installed
  = help: consider downloading the target with `rustup target add aarch64-linux-android`
error: could not compile `cfg-if` (lib) due to 1 previous error
```

Dry run 42521054342 failed the `build-android` job this way in "Build the signed release APK".

### Cause

`toolchain:install` (the Android job runs it as `task-run prepare:android`) applies the `rustTargets` row of
`toolchain.lock.json` from `install_rust_components`,
which `install_tool` calls.
`sync_tools` calls `install_tool` only for a tool it does not observe as current:
for the `rust` tool it runs `rustup toolchain list`,
takes a listed toolchain whose name starts with the locked version as current,
and returns before `install_tool`.

The Android job installed that same toolchain one step earlier,
with `rustup toolchain install "$toolchain" --profile minimal`,
so the sync always observes it as current and the Android standard library is never added.
A developer machine reaches the other branch:
a lock that moves to a new nightly date no longer matches the toolchain name
`rustup toolchain list` reports,
so the tool is not current,
and `install_tool` installs that toolchain and applies the same row.
The dated nightlies this machine held on 2026-11-08,
from 2026-10-29 to 2026-11-08,
each carry both Android standard libraries,
which is what that branch leaves behind,
while the toolchain the workflow installed with `--profile minimal` carried neither.

### Evidence

Two runs of the installed `mpm-shadow` 0.2.7-alpha.6 (the version the lock names) against a throwaway scope
whose `toolchain.lock.json` held only the `rust` tool,
with a fake `rustup` first on `PATH` that logged its
arguments:

- with `rustup toolchain list` answering the locked nightly,
  as on the runner after that step,
  `mpm-shadow install` ran that one command and printed `1 unchanged, 0 installed`,
  with no `component add` and no `target add`;
- with the same fake answering a toolchain that is not there,
  the same command ran `toolchain install`,
  the four `component add` calls and
  `target add aarch64-linux-android`,
  then `target add x86_64-linux-android`.

The Android job's own log line agrees:
`toolchain install: 64 installed, 2 unchanged, 0 failed`.

The consumer path was measured too,
because installing a standard library proves nothing about the compiler that reads it:
with the exposure directory first on `PATH` and `RUSTUP_TOOLCHAIN` set from the lock,
the `rustc` the build finds is the `rustup` proxy,
`rustc --print sysroot` names `~/.rustup/toolchains/nightly-2026-11-08-x86_64-unknown-linux-gnu`,
and a `#![no_std]` fixture compiles for both `aarch64-linux-android` and `x86_64-linux-android`.
The step installs into that same toolchain,
which is the one the export step's `RUSTUP_TOOLCHAIN` selects.

### What the workflow does about it

The step "Install the Rust toolchain pinned in toolchain.lock.json" reads the triples from the lock's
`rustTargets` row and runs `rustup target add` for them,
then fails the step when `rustup target list --installed` does not list every one,
so a missing standard library names itself there instead of in the first crate the build compiles.
It proves the triples the lock declares are installed,
not that the lock declares both ABIs `build:native` asks for;
that stays a property of the lock.
It installs no `rustComponents`:
nothing in that job compiles with them.

The skip itself stayed in `sync_tools` (`package/mpm-shadow/cli/src/toolchain/sync.rs`),
so any toolchain this repository did not install itself kept missing the components and targets its lock row
declares.
The fix went there:
the declared components and targets are reconciled even when the toolchain's version is current,
rather than only on the install path.

### The fix

For a tool `sync_tools` observes as current it now runs
`rustup target list --installed --toolchain <version>` and
`rustup component list --installed --toolchain <version>`,
adds only the declared parts `rustup` does not list,
and counts that tool as installed with one line per added part:

```text
toolchain install: 1 installed, 0 unchanged, 0 failed
toolchain install: rust: nightly-2026-11-08 was already installed; added the declared target wasm32-unknown-unknown, which `rustup target list --installed` did not show
```

A listing or an add that fails fails that tool,
so a missing standard library can no longer hide behind "unchanged",
and the parts added before a later failure stay in the report,
because the machine did change.

The target listing comes first because it is what decides the component check:
`rustup component list --installed` spells a component with the target triple it was built for,
so `clippy` counts as installed only when the listing holds `clippy-<triple>` for one of the
toolchain's own targets.
A target must be listed exactly,
because `wasm32-wasip1-threads` is a different standard library from `wasm32-wasip1`.
After a component add the target listing is read once more,
since `rustup component add rust-std-<triple>` also supplies a target.

One declared shape stays wrong:
a bare `rust-std`,
which rustup has no such component for (`rustup component add rust-std` fails),
counts as installed when any of the toolchain's targets has its standard library listed.
Telling the host standard library from a cross-target one costs another `rustup` process on every
sync,
for a declaration rustup refuses anyway,
so name the triple (`rust-std-x86_64-unknown-linux-gnu`) if one is ever needed.
A part added earlier in the same run is not added again,
so two declarations that name one component,
or one triple twice,
add it once.

On a machine that already holds every declared part the whole cost is the two listings.
Measured on 2026-11-08 on this machine,
a no-change `mpm-shadow install` of a one-tool lock
(the `rust` tool,
`nightly-2026-11-08`,
all four declared components and both Android targets present)
took 21.0 ms median from a build of the commit before the fix,
which runs `rustup toolchain list` alone,
and 60.6 ms from the fix build.
9 rounds interleaved the three arms,
and the control arm was timed twice,
once as its own binary and once as a second series of the same executable,
which is the harness's noise floor:
medians 21.0 and 21.6 ms,
so the two listings account for the whole 39.6 ms difference,
and each listing on its own takes about 21 ms.
That cost is two more `rustup` starts plus a scan of the printed names,
so it grows with the number of declared parts and listed names,
not with the size of the toolchain.

### What the workflow keeps until a release with the fix is locked

The Rust step of `music-player-release.yml` installs the triples itself,
with the two lines
`mapfile -t targets <<< "$lock_targets"` and
`rustup target add --toolchain "$toolchain" "${targets[@]}"`.
They can go once `toolchain.lock.json` pins an `mpm-shadow` release that carries this fix,
because `task-run prepare:android` then adds the row's triples itself.
The `jq` read above them and the `rustup target list --installed` guard below them exist only to feed and
prove those two lines,
so removing them is one decision about the whole block and its comment,
not three.
That release also installs the lock's four `rustComponents` in this job for the first time,
which nothing there compiles with;
the block's comment says the opposite today
and has to change with the two lines.
