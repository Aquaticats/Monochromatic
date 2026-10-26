# mise lock follow-up: remove the Linux-only stopgap when the fix releases

Purpose:
 finish the mise lock incident recorded in
 `doc/troubleshooting/mise-2026-10-19-cmake-lock-resolution.md`.
Bare `mise lock` now exits 0 with the installed 2026.10.19
because the lock targets the four Linux platforms,
a stopgap recorded in
`doc/decision/mise-lock-linux-only-stopgap.md`.
The stopgap goes away when a released mise carries
jdx/mise#14959 (commit `b2e0c56`),
first expected in 2026.10.20,
whose release pull request is jdx/mise#14948.

Changes:
- `mise.lock`:
  the cmake backend moved from `vfox:cmake` to
  `aqua:Kitware/CMake` at 4.4.4,
  with `linux-arm64` and `linux-arm64-musl`
  entries and checksums;
  android-sdk archive checksums recorded
  (commit `5fee1731f8`).
- Stopgap:
  commit `355707fb7e` records the decision,
  commit `7f00576883` sets `lockfile_platforms` in
  `mise.no-env.toml` and regenerates `mise.lock`,
  which now removes 1101 lines of macOS and Windows
  platform metadata and adds none.
  The committed lock hashes to
  `a703a7cf8dd86d2702600e5649d3dadd2b1ffe977af25e6022ac5e79c5a6c51f`
  at 240 targets.
- Troubleshooting doc and decision document:
  both root causes traced to mise source,
  the stopgap and its tradeoffs recorded
  (commits `355707fb7e`,
  `7f00576883`).
- CI no longer runs mise (commit `38cb13b4d1`,
  `doc/decision/mise-free-ci.md`),
  so no workflow pin remains to remove.

Update 2026-10-26:
`doc/decision/mpm-toolchain-adoption.md` retires `mise.lock`,
so the remaining steps below are moot for that file.
The macOS and Windows artifact-resolution gap this stopgap tracked
carries to `toolchain.lock.json` and stays open work
in `doc/decision/mpm-toolchain-adoption.md`.

Remaining steps,
once a release containing `b2e0c56` exists:
1. `mise self-update <version>`;
   the explicit version skips the minimum release age.
2. Delete the `lockfile_platforms` line and its `#region` comment from
   `mise.no-env.toml`,
   then run `mise run //:sync:files`.
3. In the repository root,
   run bare `mise lock`;
   expect exit 0,
   a seven-platform target count (645 when the stopgap
   landed),
   and the macOS and Windows platform entries back.
4. Confirm `git status --short` is clean,
   run `mise lock` once more,
   expect the same output and no diff,
   and record the new `sha256sum mise.lock`.
5. Update the Status and follow-up section of the troubleshooting doc,
   then follow the removal condition in the decision document,
   which deletes that document,
   and delete this handover.
   Commit with explicit pathspecs.
6. Delete `${HOME}/temp/agent/mise-2026-10-25` (build clone) and
   `${HOME}/temp/agent/bin/mise-fix` (built binary) if they still exist.

What to inspect:
- `git log --oneline -3 -- mise.lock`
- `rg -n lockfile_platforms mise.no-env.toml mise.toml`
- `mise --version`
- The troubleshooting doc's Verification and Status and follow-up sections.

How to respond:
- If no release contains `b2e0c56` yet,
  check `gh release list --repo jdx/mise --limit 1`
  and `gh pr view 14948 --repo jdx/mise` occasionally;
  do not expect a date,
  since the release may never publish,
  and the stopgap keeps the repository working meanwhile.
- If `mise lock` still fails after the upgrade,
  capture the error and check whether the release contains `b2e0c56`;
  report it rather than hand-editing `mise.lock`.
- Do not post upstream:
  the jdx/mise `CONTRIBUTING.md` restricts AI-authored
  issue replies,
  and the duplicate issue is already closed and fixed.
