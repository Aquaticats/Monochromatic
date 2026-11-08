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
