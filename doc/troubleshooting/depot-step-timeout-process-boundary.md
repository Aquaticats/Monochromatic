# CI step timeout does not establish descendant completion or VM release

## Symptom

The frozen `timeout-renew` task writes its identity,
waits 70000 ms and writes a completion file.
Its CI step has `timeout-minutes: 1`.
It installs no signal handler.
An independent completed-writer marker and a subsequent twelve-second observer validate the observation horizon.

The same harness,
`44b3303eca52a58806e28a6e8b681be07d43c278`,
ran on standard GitHub,
Depot-managed GitHub Actions and native Depot CI.
Every writer completed after 70009 ms,
despite the sixty-second step limit.
Every observer retained outcome `failure` and conclusion `success`
because this deliberately bounded diagnostic step uses continuation.
The [public receipt](../audit/depot-process-timeouts-2026-10-18.evidence.json)
retains all three original ZIPs,
provider metadata and exact step-boundary events.

Native run `cpjv1rfqzw` emits:

```text
exec-step child exited with signal SIGTERM and produced no result
```

Its next step starts at `2026-10-18T07:24:46.969Z`.
The prior writer's completion is `2026-10-18T07:24:57.299Z`.
This establishes completion after the native executor advanced to the next step.

The paired GitHub run is `39107000367`.
Both GitHub-orchestrated observers start after their writers complete.
Those observations establish deadline overrun,
but do not establish the same post-step descendant lifetime.

## Deciding source and unknown cause

The task in `.depot/research/native-semantics/mise.toml` contains ordinary timer and file operations:

```javascript
// .depot/research/native-semantics/mise.toml, timeout-renew task
await setTimeout(observation.intendedHoldMilliseconds);
fs.writeFileSync('evidence/timeout-renew-completed-' + attempt + '.json',
  JSON.stringify({ attempt, completedAt: new Date().toISOString() }) + '\n');
```

The actual GitHub logs report runner version `2.337.0`.
Its release source is `397b032cbf865e9c3ddfab89d533ec19325e1273`.
`src/Runner.Sdk/ProcessInvoker.cs:32` declares these grace intervals:

```text
# actions/runner, src/Runner.Sdk/ProcessInvoker.cs
private readonly TimeSpan _sigintTimeout = TimeSpan.FromMilliseconds(7500);
private readonly TimeSpan _sigtermTimeout = TimeSpan.FromMilliseconds(2500);
```

`CancelAndKillProcessTree` at line `443` can send SIGINT,
wait,
then send SIGTERM and wait before killing the tree.
That branch depends on `killProcessOnCancel`.
The script handler passes `killProcessOnCancel: false`
in `src/Runner.Worker/Handlers/ScriptHandler.cs:346`.
The public implementation permits a cancellation interval beyond the nominal deadline;
the observation does not independently identify every signal delivered to the child.

The native executor is a separate implementation.
Its diagnostic identifies a SIGTERM exit of its step child,
but inspected public source does not establish how its process-group or descendant cleanup works.
Do not assign the native result to the GitHub runner implementation or to Mise alone.

## Verification and scoped protection

There is one unchanged-input observation per provider,
one writer execution per observation and no retry in this profile.
The original evidence includes the earlier completed-writer positive control,
the actual seventy-second completion and an observer that rejects a horizon shorter than 71000 ms.
ZIP CRC,
size and SHA checks pass;
GitHub ZIP hashes match their advertised API digests.
Native ZIP hashes are local measurements.

Keep step verdict,
child lifetime,
job status,
VM release and billable interval as separate facts.
The trial's finite writer and observer completed;
this is containment of the diagnostic rather than a repair to the executor.

An independently exercised owned-process driver is described in
[`depot-toml-mutation-completion.md`](depot-toml-mutation-completion.md).
Its container timeout control verifies termination of its own signal-resistant child and grandchild.
That is a separately qualified mechanism with container and lifecycle overhead,
not evidence that the CI step field itself guarantees the same cleanup.
This investigation did not change production timeout handling.

## What does not follow

The GitHub deadline overrun does not erase the native post-step observation.
The native observation does not establish that either GitHub-orchestrated observer raced a live writer.
None of these observations establishes an indefinite orphan,
every descendant's termination,
physical host release,
billable VM lifetime or a vendor invoice.

The related
[`nano-spawn-timeout-is-not-a-hard-deadline.md`](nano-spawn-timeout-is-not-a-hard-deadline.md)
uses a child that handles SIGTERM.
This CI fixture has no such handler and remains a separate incident.

## Upstream filing decision

- Responsibility:
  the measured native boundary is at the step executor;
  the deciding descendant implementation is not public in the inspected source.
- Fixability:
  no source-backed provider patch is available to verify;
  the scoped owned-process driver has its own independent controls.
- Supported use:
  the actual uploaded workflows accept the step timeout and continuation fields.
- Contribution policy:
  no external issue or message is authorized or sent by this investigation.
- Maintainer intent:
  the observation does not establish an intended native descendant-cleanup contract.
- Minimal patch:
  no speculative private executor patch is claimed;
  retained source-bound receipts support the repository's routing and containment decision.
