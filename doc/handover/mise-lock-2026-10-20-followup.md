# mise lock follow-up after 2026.10.20 publishes

Purpose:
 finish the mise lock incident recorded in
 `doc/troubleshooting/mise-2026-10-19-cmake-lock-resolution.md`.
The committed `mise.lock` is correct,
but the installed mise 2026.10.19 still
fails bare `mise lock` in this repository because it lacks upstream fix
`b2e0c56` (jdx/mise#14959).
Release 2026.10.20 contains the fix;
its pull request (jdx/mise#14948) was
open on 2026-10-25,
held by the `release-window` check until
2026-10-26T08:00Z.

Changes:
- `mise.lock`:
  the cmake backend moved from `vfox:cmake` to
  `aqua:Kitware/CMake` at 4.4.4,
  with `linux-arm64` and `linux-arm64-musl`
  entries and checksums;
  android-sdk archive checksums recorded
  (commit `5fee1731f8`).
- Troubleshooting doc:
  both root causes traced to mise source,
  the upstream fix
  (jdx/mise#14937,
  #14959,
  commit `b2e0c56`,
  first release 2026.10.20) and the
  no-filing decision recorded (commits `de3444dbe6`,
  `2079e4fb36`,
  plus the
  release-window note).
- CI no longer runs mise (commit `38cb13b4d1`,
  `doc/decision/mise-free-ci.md`),
  so no workflow pin remains to remove.

Remaining steps,
after v2026.10.20 publishes:
1. `mise self-update 2026.10.20`;
   the explicit version skips the minimum
   release age.
2. In the repository root,
   run bare `mise lock`;
   expect exit 0 and
   `✓ 645 targets checked`.
3. Confirm `git status --short` is clean and `sha256sum mise.lock` still
   reports `a0a16fd86a330d7c7d7c4aea0b3001fd5eaf568502b28098c72d21066bf5ec19`.
4. Run `mise lock` once more;
   expect the same output and no diff.
5. Update the Status and follow-up section of the troubleshooting doc with the
   release version and verification,
   then commit with explicit pathspecs.
6. Delete `${HOME}/temp/agent/mise-2026-10-25` (build clone) and
   `${HOME}/temp/agent/bin/mise-fix` (built binary) if they still exist.

What to inspect:
- `git log --oneline -3 -- mise.lock`
- `git show 5fee1731f8 --stat`
- `mise --version` should report `2026.10.20 linux-x64`
- The troubleshooting doc's Verification and Status and follow-up sections.

How to respond:
- If the release is not out yet,
  wait until after 2026-10-26T08:00Z plus build
  time,
  then retry `gh release list --repo jdx/mise --limit 1` and check for
  `v2026.10.20` with `isDraft` false.
- If `mise lock` still fails after the upgrade,
  capture the error and check
  whether the release contains `b2e0c56`;
  report it rather than hand-editing
  `mise.lock`.
- Do not post upstream:
  the jdx/mise `CONTRIBUTING.md` restricts AI-authored
  issue replies,
  and the duplicate issue is already closed and fixed.
