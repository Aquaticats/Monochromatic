# Pi Bash result-loss and crash handover

## Status

The Pi priority investigation is complete through implementation and user-boundary verification.
Todos `#31`,
 `#32`,
 and `#33` are complete.
Todo `#34` is recording final handover and issue state.
Cli-git issue `#353` may resume after `#34` closes.

The durable diagnosis is
`doc/troubleshooting/pi-bash-output-spool-write-failure.md`.
The applied dependency patch and companion source patch were removed at the user's request.

## Established cause

The recurrent batch failure was an unhandled Pi Bash spill-file stream error.
Linux errno `122` is `EDQUOT`.
Node `26.5.0` and libuv `1.52.1` rendered it as
`Unknown system error -122` because their system-error map lacks that value.

At incident time,
`/tmp` was a quota-controlled `tmpfs`.
The user limit measured `6390M`,
and usage reached about `6.3G` while global free space remained.
The user removed that quota on July 10,
 2026.
A post-change probe reported `460M` used,
`16G` available,
and `0K` in both quota and limit columns.

Pi `0.80.6` writes complete truncated Bash output to
`os.tmpdir()/pi-bash-*.log`.
`OutputAccumulator` opened and wrote its `WriteStream` without a durable `error` listener.
The listener in `closeTempFile()` arrived too late for errors emitted while output was still streaming.
An early error terminated Pi;
a finalization error rejected the Bash promise before result recording.
Three concurrent overflowing commands therefore showed `No result provided` together.

`package/coding-agent/src/core/bash-executor.ts` independently duplicated the same unguarded pattern for direct
`!` Bash and RPC execution.

## Removed local fix

The workspace runs unpatched Pi `0.80.6`.
The user removed the per-user `/tmp` quota,
then requested removal of both the pnpm dependency patch and the retained source patch.
The durable local mitigation is a writable disk-backed `TMPDIR` if temporary-file limits return.

Historical prototype checkpoints are:

- `d0c388a16`,
   ordinary Bash stream hardening;
- `92f8d209c`,
   direct Bash spooler hardening;
- `91ac69a42`,
   direct metadata and TUI warning rendering;
- `b1be09ac8`,
   complete diagnosis and source prototype.

`NXR` in `AGENTS.md` and generated `CLAUDE.md` remains required:
after a missing synchronous result or transport loss,
inspect processes and artifacts before changing to a narrower or managed execution path.

## Verification evidence

The disk-backed upstream clone remains at:

```text
/var/home/user/temp/agent/pi-bash-result-loss-20260710-1329434
```

It is tag `v0.80.6`,
commit `2b3fda9921b5590f285165287bd442a25817f17b`.
Its focused source verification passed:

```text
Test Files  2 passed (2)
Tests       78 passed (78)
Type check  passed
Build       passed
```

The deterministic external harness is:

```text
/var/home/user/temp/agent/pi-output-write-failure-repro.mjs
```

It points `TMPDIR` at a regular file and starts three overflowing Bash tools plus three direct executors.
Unpatched `0.80.6` exited `1` before stdout.
The removed prototype returned all results,
retained each output tail,
and reported `ENOTDIR` as `fullOutputError` or a direct warning.

A fresh `pi -ne` TUI with the same invalid temp base stayed alive after a `60,000` byte direct Bash command
and rendered:

```text
Output truncated. Full output unavailable: ENOTDIR: not a directory, open '.../pi-bash-*.log'
```

After the user removed the quota,
the original three concurrent commands completed through the installed package in `1,241 ms`.
One remained below the truncation threshold.
The other two returned valid `/tmp/pi-bash-*.log` paths.
No command reported `fullOutputError`.

A separate direct SDK harness ran the cli-git unit command twenty times before the fix.
Every run passed in about `4.2` to `4.6` seconds with about `18.4 KB` of result text.
This proves that ordinary single-command execution was healthy after quota usage fell;
it does not prove the exact child wait state for the earlier missing unit results.

## Distinct observations

The controlled `SIGABRT` core validates crash capture only.
The signal was intentional.
Its V8 string-flattening stack is not evidence that V8 caused an organic failure.
No spontaneous incident produced a core.

The monitored and passive Luna forks both completed the target command.
No observer-dependent failure was established.
Large inherited sessions triggered compaction and were less diagnostic than the direct harness.

The two earlier missing unit-test results belong to the quota incident cluster by timing and environment,
but their sessions preserve no child-process stack.
Do not overstate the evidence as a proven exact child wait site.

## Upstream decision

Closed issue `earendil-works/pi#5667` reports the same unguarded spill stream with macOS `EACCES`.
No new issue was filed.
The troubleshooting record contains an additive-comment draft,
but contribution constraints do not justify an unsolicited agent-authored comment.
A human contributor may rewrite the draft in their own voice,
retain the AI-assistance label,
and add the Linux `EDQUOT` evidence to that existing issue.

## Preserved and cleaned artifacts

Crash and monitor evidence remains below:

```text
/var/home/user/temp/agent/pi-crash-monitor-*
```

The controlled PID `1309170` core remains available through `systemd-coredump` subject to host retention.

Residual `pi-crash-repro` tmux sessions,
monitor and signal-trace services,
Inspector watchdogs,
and Inspector listeners on ports `9229` and `9230` were stopped after evidence preservation.
Disposable unpatched package trees,
pnpm patch-extraction trees,
invalid-temp fixtures,
and post-quota spill files were removed.

## Resume point

Cli-git issues `#353` through `#355` are closed.
Issue `#356` is closed after hosted CI,
cross-platform trust,
paired performance,
packaging,
documentation,
and independent standards and specification review passed.
Issue `#357` is closed after hk/Pkl removal,
exact disposable-tested Git-config cleanup,
built-shim and packed-install verification,
independent CI,
documentation,
and final review passed.
Keep npm publication issue `#358` deferred until a maintainer explicitly resumes it.

Issue `#360`,
which replaces the faulty third-party goal extension,
remains separate and does not block cli-git.
