# `hdiutil create` failed with "Resource busy" once on a `macos-15` GitHub-hosted arm64 runner
while building the music player disk image

## Symptom

The macOS arm64 job of `.github/workflows/music-player-release.yml` builds `Music Player.app`,
signs it ad hoc and packs it with:

```bash
hdiutil create -volname 'Music Player' -srcfolder "$stage" -fs HFS+ -format UDZO -ov "dist/${asset}"
```

In dry run 38658350643 (job `build-desktop (macos, arm64, aarch64-apple-darwin, macos-15)`, step
`Package (macOS disk image)`) it printed:

```text
hdiutil: create failed - Resource busy
##[error]Process completed with exit code 1.
```

The Intel job of the same run (`macos-15-intel`, since removed from the matrix) passed the same step.
The same step on the arm64 job passed in run 38657771731 before it and in the six runs after it
(38662748558, 38665954226, 38667490084, 38668625715, 38669432843, 38669453427).
So the failure was 1 in 8 of the executions of that step that were counted, and it did not repeat.
Two cancelled runs (38660009854 and 38661273133) were not checked and are not in the count.

## Root cause

Not established here.
`hdiutil` is closed-source, so no source trace was possible.

The upstream tracker has a long thread for the same message:
[actions/runner-images#7522][runner-images-7522] (opened 2023-05-02 for `macos-13`, closed 2026-01-23).
What its participants and maintainers say, none of it reproduced here:

- A maintainer (2023-05-22) attributed `Resource busy` on macOS 13 to `XProtectBehaviorService` locking the newly
  created image, and suggested a retry or stopping that service.
- A maintainer (2024-08-20) reproduced it with `create-dmg`, whose default filesystem is HFS+, and reported
  consistent results after switching the image filesystem to APFS.
  A later commenter still saw intermittent failures with APFS.
- The closing comment (2026-01-23) calls it a known, intermittent `hdiutil` condition caused by sensitivity to disk I/O
  state and background activity (security scanning, indexing, delayed flushes), and not specific to the runner images.

This job uses HFS+ with UDZO on `macos-15`, so the 2024 observation applies to its filesystem choice.
Whether any of these explanations holds for the failure seen here is unknown:
the failing run's volume state, running processes and Spotlight state were not captured.

## Verification

Version under test: the `macos-15` hosted image of 2026-10-15 (the exact image version was not recorded) on arm64.
No reproduction was attempted: the failure was seen once and did not recur in six later executions.
The workaround below was added after the failure.
**Its retry path has not run in CI**: the logs of the six later runs hold no `failed (attempt` line,
so what the retry does on a real `Resource busy` is untested.

## Verified workarounds

Not verified against a real recurrence.
The job wraps the two disk-image commands that can hit a busy volume in a retry of up to five attempts with a
10 second wait:

```bash
retry hdiutil create -volname 'Music Player' -srcfolder "$stage" -fs HFS+ -format UDZO -ov "dist/${asset}"
# ... verify, attach, run --version ...
retry hdiutil detach "$mount_point"
```

Tradeoffs: a failure that is not transient costs up to four extra waits of 10 seconds before the job fails,
and a partial image file left by a failed attempt is overwritten by `-ov` on the next attempt.
The retry hides an intermittent failure instead of explaining it.

## What does not work

- Nothing was tried and rejected, because no recurrence has been observed.
- Not tried: `--filesystem APFS` (the 2024 maintainer suggestion), `sync` before `hdiutil create`, and stopping
  `XProtectBehaviorService`.
  Switching the filesystem would change the format of the shipped image.

## Upstream filing decision

`.out-of-scope/` was checked: no exemption names `hdiutil` or the macOS runner images.
A duplicate exists: [actions/runner-images#7522][runner-images-7522], whose maintainers already describe the cause as
known and give retry as the workaround.

1.  Is it upstream's fault: undetermined; the maintainers say it is macOS behavior.
2.  Can upstream fix it: unknown.
3.  Supported use case: building DMGs is the thread's subject.
4.  Contribution welcome: not assessed.
5.  Likely to be fixed: the closing comment treats it as macOS behavior and not an image defect;
    the rest of that comment was not read.
6.  Prototype: none.

Nothing additive: a single `macos-15` arm64 occurrence with HFS+ and UDZO, without a recorded image version or
volume state, adds nothing to a thread with dozens of reports.
No comment is drafted and no new issue is filed.

[runner-images-7522]: https://github.com/actions/runner-images/issues/7522
