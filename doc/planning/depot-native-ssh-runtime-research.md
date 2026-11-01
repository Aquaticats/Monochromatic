# Native SSH runtime research

## Scope

This is a successor to the immutable
[source and admission proposal](../audit/depot-native-ssh-acceptance-2026-10-18.evidence.json).
The [runtime receipt](../audit/depot-native-ssh-runtime-2026-10-18.evidence.json)
records selected results and original private-body hashes.
The proposal and its admitted controller inputs remain unchanged.

## Terminal execution

The admitted native run is `sqrz4sml30`,
job `q74vcmxkk1`,
attempt `fhxx4s51rf`,
sandbox `g64shw2rl3jzrxsmq92x`.
Its harness is `b6a60493c16087043a7043f50cd31d8aebf4bdee`,
preserving application `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
The requested shape is `2` CPU and `8` GiB,
with a `3` minute server job limit and cancellation requested after `150` seconds.
The maximum nominal reservation across a reviewed correction is `USD 0.036`.

Direct OpenSSH and the native CLI RPC both returned matching source,
boot identity,
marker and challenge responses from the VM.
Direct SSH ran as UID/GID `0/0` in `/root` and exited `255`.
Native CLI RPC ran as UID/GID `1001/117` in `/home/runner` and exited `0`.
Neither inherited the synthetic workflow environment value.
The successful remote proof establishes execution despite the direct client's nonzero close.
Its close cause remains unqualified.
The fixture asserted CPU count and a memory interval;
it did not log exact observed memory bytes.

The controller's finite watchdog cancelled the run.
Fresh status confirmed `cancelled`.
Controller session `68688` was reaped with exit `1`,
reflecting the unqualified paused/final consumer.
Every recorded owned local process identity was freshly absent,
and the askpass socket was absent.
Cancellation does not establish immediate VM deletion or credential revocation.

## Paused-session limit

The final `276` retained log events contain no paused tmate connection target,
paused proof or final workflow file consumer.
The last observed installer output was:

```text
Get:1 file:/etc/apt/apt-mirrors.txt Mirrorlist [237 B]
```

Pinned `mxschmitt/action-tmate` source
`35b54afac29c97fb54faba5b513f8fbd1882f113`,
`src/index.js:82`,
initializes the executable as `tmate`.
Its dependency branch invokes APT update/install before downloading static tmate `2.4.0`.
Setting `install-dependencies: false` skips that entire installation branch,
so a valid `tmate` must already be on `PATH`.
`src/helpers.js` streams command output while awaiting child exit.
The retained APT output does not independently identify the blocking instruction or root cause.
No tmate transport failure is established.

A correction must review and bind the same-version binary,
its download/integrity bounds,
the changed fixture and replacement controller identities.
The original source and admission remain immutable.
No second dispatch is admitted by this receipt.
Final workflow ownership,
file-mode readback and resume remain unexecuted.

## GitHub check consumer

Read-only GitHub API queries for the exact SSH harness returned check-run `117756320157`,
`native-ssh-acceptance / ssh-proof`,
with status `completed` and conclusion `cancelled`.
Its details URL contains the exact owned job ID `q74vcmxkk1`,
which the native status hierarchy binds to run `sqrz4sml30`.
The app metadata is `219785`,
`depot-code-access`,
`Depot Code Access`.
This is the existing check publisher's identity;
it does not mean the Depot Code product was enabled.

The known GitHub Actions positive,
run `39140975786` at `234ccc92dabb81e998e0a0bb5e838713924dd83f`,
returned `6` checks:
`4` GitHub Actions checks linked to that run and `2` Depot native graph checks.
Both exact SHAs returned `0` legacy commit statuses.
The positive calibrates check-run visibility,
not a nonempty legacy-status publisher.

The runtime receipt records selected source,
run,
status and app fields with original body hashes.
Original actor and output bodies remain private.
No app or account settings were inspected or changed.
These reads allocated no compute.

## Next work

- Prepare the source-bound paused-session correction for parent review and coordinated scheduling.
- Preserve the original cancelled execution and both successful transport proofs.
- Complete the independent report sufficiency review after the parent factual queues close.
