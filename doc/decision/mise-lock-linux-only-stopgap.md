# Linux-only mise lock targets as a stopgap

Status:
accepted as a stopgap on 2026-10-26 under `AGENTS.md` `DRR`;
superseded on 2026-10-28.
This decision is history rather than policy:
the fix shipped in mise v2026.10.20,
and the MPM cutover removed the lockfile and the setting.

## Decision

`mise.no-env.toml` set `lockfile_platforms` to the four Linux platforms
(`linux-x64`,
`linux-x64-musl`,
`linux-arm64`,
`linux-arm64-musl`) under
`[settings]`,
and `mise.lock` recorded URL and checksum entries only for those platforms.
Bare `mise lock` with the then-released mise 2026.10.19 exited 0.
Carried over on 2026-10-26:
 `mise.lock` is retired in `doc/decision/mpm-toolchain-adoption.md`,
 and its replacement `toolchain.lock.json` inherits the same Linux-only
 artifact rows;
 resolving macOS and Windows artifact URLs and checksums remains open work.
Both the setting and `mise.lock` are gone now:
commit `a5cdc3158d` removed the tool and lockfile settings,
and commit `4cb480344f` deleted the lockfile.

## Why

Two defects in mise 2026.10.19 made a full seven-platform lock impossible
with any then-released mise:

- The lock target loop resolved every platform and ignored a tool's
  `os = ["linux"]` option (jdx/mise#14937),
  so `conda:bubblewrap` was resolved
  for `macos-arm64`,
  `macos-x64` and `windows-x64`.
- Conda resolution errors were fatal (jdx/mise#14959),
  so the excluded
  platforms aborted the whole lock instead of warning.
- Conda errors are fatal in every checked tag from 2026.10.9 to 2026.10.19,
  which is why no released mise could pass at that time.

The fix (jdx/mise#14959,
commit `b2e0c56`) merged on 2026-10-25 and shipped in mise v2026.10.20,
published 2026-10-26T21:21:55Z;
release pull request jdx/mise#14948 merged and v2026.10.21 followed on
2026-10-27T10:23:17Z.

## Consequences

These apply to the period before the MPM cutover removed the lockfile.

- Bare `mise lock` exited 0 on 2026.10.19 and reported 240 targets.
- The lockfile kept every tool version,
  backend and Linux URL and checksum;
  the change removed platform metadata for macOS and Windows only (1101
  deleted lines,
  no added lines).
- macOS and Windows developers lost pinned URLs and checksums for those
  platforms,
  and mise resolved those platforms locally when they needed them.
- CI was unaffected:
  the workflows read tool versions from `mise.lock` and run
  on Linux (`doc/decision/mise-free-ci.md`).
- `conda:bubblewrap` stayed declared with `os = ["linux"]` and kept its Linux
  lock entry.

## Outcome

On 2026-10-28,
mise 2026.10.20 (installed locally) was exercised in a
throwaway worktree at commit `7f00576883` with the stopgap removed and
`mise.lock` deleted:

- bare `mise lock` exited 0 and checked 413 targets,
  one per tool and platform (59 tools across seven platforms);
- `aqua:Kitware/CMake` had macOS and Windows platform entries again;
- `conda:bubblewrap` had exactly the four Linux platform entries,
  so the fixed target loop skips the excluded platforms.

Removing only the setting against the existing Linux-only lock kept the four
Linux targets (358 targets),
because an unset `lockfile_platforms` preserves the platforms already in the
lockfile;
a full seven-platform lock needs a missing lockfile or an explicit
`--platform` list.
That distinction is moot now that `mise.lock` is retired.

## Removal condition

Met on 2026-10-28 by supersession rather than by the steps the stopgap
recorded:

1. Commit `a5cdc3158d` removed `[tools]`,
   `lockfile` and
   `lockfile_platforms` from `mise.no-env.toml` for
   `doc/decision/mpm-toolchain-adoption.md`.
2. Commit `4cb480344f` deleted `mise.lock`;
   `toolchain.lock.json` is the pinned record.
3. `doc/handover/mise-lock-2026-10-20-followup.md` was deleted.
   This document stays as history because
   `doc/planning/mpm-toolchain-migration.md` points at it and because
   `toolchain.lock.json` still inherits the Linux-only artifact rows.

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
  `mise lock` stayed broken meanwhile and the release was not guaranteed.
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
  alternative recorded in the alternative list.
