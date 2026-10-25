# mise 2026.10.19 fails `mise install rust` in CI: the cmake vfox plugin returns no version for linux-arm64 during lock resolution

## Symptom

The `cargo-publish` workflow failed at the step "Install only the tools this workflow needs" (`mise install rust`)
on every run that reached it after 2026-10-24, before any publish step.
Release commit `00c4e28af4` (run 40242133604, two attempts) and a later push (run 40242360125) all failed the same way.
The last passing run (39829260282, 2026-10-22) used mise 2026.10.17.

The log lists `mise lock` lines for every tool and every platform, then removes and reinstalls the `cmake` plugin and
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

The job `gpc-publish-crate` fails, and the jobs that depend on it (`build-binary`, release creation) are skipped.
The other workflows on the same commits pass.
The crate is not involved: no step that builds or publishes it ran.

## Root cause

What the evidence shows:

- The failing command is `mise install rust` from the repository root, which holds `mise.toml` with `cmake = "latest"`
  (`mise.toml:169`) and a `mise.lock` listing many platforms.
- With mise 2026.10.19, installing any tool also resolves lock entries for every tool and platform.
  The `cmake` plugin is removed and reinstalled during that pass, then its metadata fails to load
  (`no file '.../plugins/cmake/util.lua'`), and resolution for `linux-arm64` ends with "no version returned".
- With mise 2026.10.17 the same files, container and command install cleanly.

What is not established: which code path in mise removes the plugin directory while another resolution still reads it.
The mise source (the vfox plugin loader and the lock resolver) has not been read for this write-up, so the sentence
above is a description of the log, not a traced root cause.
It is also not proven that the plugin removal causes the missing version, only that they appear together in every
failing log.

A hypothesis that was disproved: GitHub anonymous API rate limiting.
The host IP had 59 of 60 core requests left when checked, and the failure is a mise resolution error, not an HTTP
status.

## Verification

Harness: a podman container (Debian 13 image with git and cargo, `--memory=4g --cpus=2`, throwaway home), the committed
`mise.toml`, `mise.lock` and `mise.no-env.toml` only, mise `2026.10.17` and `2026.10.19` from the GitHub release
archives `mise-v<version>-linux-x64.tar.gz`, running `timeout 240 mise install rust` in the directory with the files.

First attempt of each version:

- 2026.10.17: exit 0 after 150 seconds, reached the Rust toolchain install, no `cmake` resolution error.
- 2026.10.19: exit 1 after 171 seconds, `failed to resolve cmake for linux-arm64` present.

The second attempts were still running when this was written; their result is recorded in the handover
`doc/handover/cli-git-rust-implementation.md`.

## Verified workarounds

Pin mise in the workflow: every `jdx/mise-action` step in `.github/workflows/cargo-publish.yml` gets
`version: 2026.10.17`, which the action documents as taking precedence over `minimum_release_age`.

- Tradeoff: the workflow no longer follows new mise releases, so a mise fix is not picked up until the pin is removed.
- Tradeoff: the other jobs in the file stay on the pinned version too, including ones that did not fail.
- Not yet verified end to end: a dispatched run of the pinned workflow (a dry run is the first check).

## What does not work

- Rerunning the failed job: attempt 2 of run 40242133604 failed at the same step.
- Publishing from a developer machine: the crate publishes through crates.io Trusted Publishing from CI.

## Upstream filing decision

Not evaluated. The six-constraint check, the duplicate search in the mise tracker and the check of `.out-of-scope/` have
not been done, and no source clone was read, so there is no fileable draft. Do not file anything from this document
until that work is done.
