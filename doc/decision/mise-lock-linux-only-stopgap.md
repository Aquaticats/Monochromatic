# Linux-only mise lock targets as a stopgap

Status:
accepted as a stopgap on 2026-10-26 under `AGENTS.md` `DRR`.
This decision is temporary and has an explicit removal condition.

## Decision

`mise.no-env.toml` sets `lockfile_platforms` to the four Linux platforms
(`linux-x64`,
`linux-x64-musl`,
`linux-arm64`,
`linux-arm64-musl`) under
`[settings]`,
and `mise.lock` records URL and checksum entries only for those
platforms.
Bare `mise lock` with the released mise 2026.10.19 then exits 0.

## Why

Two defects in mise 2026.10.19 make a full seven-platform lock impossible
with any released mise:

- The lock target loop resolves every platform and ignores a tool's
  `os = ["linux"]` option (jdx/mise#14937),
  so `conda:bubblewrap` is resolved
  for `macos-arm64`,
  `macos-x64` and `windows-x64`.
- Conda resolution errors are fatal (jdx/mise#14959),
  so the excluded
  platforms abort the whole lock instead of warning.

The fix (jdx/mise#14959,
commit `b2e0c56`) merged but is unreleased.
Release pull request jdx/mise#14948 for 2026.10.20 was still blocked when
this stopgap landed,
and no released mise can pass:
conda errors are fatal in
every checked tag from 2026.10.9 to 2026.10.19.

## Consequences

- Bare `mise lock` exits 0 on 2026.10.19 and reports 240 targets.
- The lockfile keeps every tool version,
  backend and Linux URL and checksum;
  the change removes platform metadata for macOS and Windows only (1101
  deleted lines,
  no added lines).
- macOS and Windows developers lose pinned URLs and checksums for those
  platforms,
  and mise resolves those platforms locally when they need them.
- CI is unaffected:
  the workflows read tool versions from `mise.lock` and run
  on Linux (`doc/decision/mise-free-ci.md`).
- `conda:bubblewrap` stays declared with `os = ["linux"]` and keeps its Linux
  lock entry.

## Removal condition

When a released mise contains jdx/mise#14959:

1. Delete the `lockfile_platforms` setting and its `#region` comment from
   `mise.no-env.toml`,
   then regenerate `mise.toml` with
   `mise run //:sync:files`.
2. Run bare `mise lock` and confirm the macOS and Windows platform entries
   return with every tool version unchanged otherwise.
3. Delete this decision document and update the background in
   `doc/troubleshooting/mise-2026-10-19-cmake-lock-resolution.md` and
   `doc/handover/mise-lock-2026-10-20-followup.md`.

## Alternatives rejected

- Installing a locally patched mise:
  rejected by the user.
- Excluding only `conda:bubblewrap` from the lockfile:
  mise has no per-tool
  lockfile opt-out.
- Removing `conda:bubblewrap` and using the distro package instead:
  works
  (measured exit 0,
  165-line diff) but changes Linux provisioning and drops
  the mise-only-machine guarantee.
- Waiting for the release without a local change:
  rejected because bare
  `mise lock` stays broken meanwhile and the release may never publish.
- Running `mise lock --platform linux-...` per invocation:
  keeps the lock
  intact but is not the bare command.

## Evidence

Measured in a throwaway worktree with mise 2026.10.19:

- Control:
  bare `mise lock` exit 1 on
  `failed to resolve conda:bubblewrap for macos-arm64`.
- `lockfile_mode = "merge"`:
  exit 1,
  so the older write behavior does not
  help.
- Linux-only `lockfile_platforms`:
  exit 0,
  240 targets checked,
  lock diff of
  1102 lines in the worktree (`git diff` in this repository shows 1101
  deleted lines and no added lines).
- Removing `conda:bubblewrap`:
  exit 0 with a 165-line diff,
  the rejected
  alternative recorded in the previous section.
