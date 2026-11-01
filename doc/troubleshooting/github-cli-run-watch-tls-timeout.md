# GitHub CLI 2.102.0 exits run watching on a TLS handshake timeout

## Symptom

During the native runner recovery,
three local `gh run watch` processes exit with code `1` and this diagnostic:

```text
# GitHub CLI output; the run ID varies between the affected monitors.
failed to get run: Get "https://api.github.com/repos/Aquaticats/Monochromatic/actions/runs/39095637308?exclude_pull_requests=true": net/http: TLS handshake timeout
```

The other affected run IDs are `39095642563` and `39095647113`.
Subsequent API responses show the workflows remain active.
The [retained diagnostics and terminal consumer results][evidence]
distinguish the local monitor failure from provider job outcomes.

## Root cause and boundary

The network cause of the TLS handshake timeout is unestablished.
The GitHub CLI source explains why that transport error terminates watching.
The inspected release is `v2.102.0`,
commit `fc4b137cdef0a6bd28fd461b7cf9c84a5812a8cd`,
from a read-only shallow clone of `cli/cli`.

In `pkg/cmd/run/watch/watch.go:220`,
`renderRun` returns the failed refresh to its caller:

```go
// cli/cli v2.102.0, pkg/cmd/run/watch/watch.go:220
run, err = shared.GetRun(client, repo, fmt.Sprintf("%d", run.ID), 0)
if err != nil {
    return nil, fmt.Errorf("failed to get run: %w", err)
}
```

In `pkg/cmd/run/watch/watch.go:164`,
the watch loop breaks on that error:

```go
// cli/cli v2.102.0, pkg/cmd/run/watch/watch.go:164
run, err = renderRun(out, *opts, client, repo, run, prNumber, annotationCache)
if err != nil {
    break
}
```

In `pkg/cmd/run/watch/watch.go:190`,
the caller returns the error before its final workflow-conclusion handling:

```go
// cli/cli v2.102.0, pkg/cmd/run/watch/watch.go:190
if err != nil {
    return err
}
```

These source paths explain the local process exit.
They do not identify a Depot runner failure or a network root cause.

## Verification

The real invocation used the existing run ID:

```sh
# Executed against the existing trial workflow, without dispatching another run.
gh run watch 39095637308 --repo Aquaticats/Monochromatic --exit-status --compact --interval 30
```

The failing catalog consists of the three recorded refresh diagnostics.
The working catalog consists of subsequent run metadata reads,
resumed watches and terminal artifact retrieval for those same IDs.
All workflows reached `completed` and their consumers were collected.
Their workload failures remain workload failures.
A transport failure is intermittent;
the command does not deterministically reproduce the handshake timeout on demand.

## Verified workaround

After a monitor exits,
read metadata for the same workflow before deciding whether execution ended:

```sh
# Executed during recovery; this reads state and does not create or cancel a run.
gh api repos/Aquaticats/Monochromatic/actions/runs/39095637308
```

If it remains active,
resume watching its existing ID.
The recovery did this for all affected workflows and collected their final artifacts.
The tradeoff is delayed local visibility during a transport interruption;
resuming cannot guarantee network availability.
No duplicate compute allocation is needed.

## What does not work

Equating the monitor's exit code with workflow completion is contradicted by the subsequent active metadata.
Equating it with a provider infrastructure failure also lacks evidence.
No cancellation,
redispatch or provider change was attempted for this incident.

## Upstream filing decision

- Upstream fault is unestablished:
  the observed trigger is a transport timeout and the CLI returns it.
- Retry handling could be changed,
  but no causal upstream fix for the observed network interruption is established.
- Watching existing runs is a supported CLI path,
  as verified by the inspected source and real invocation.
- Contribution acceptance was not assessed because no filing is proposed.
- Maintainer intent to change this behavior was not established.
- Consumer-side resumption was verified;
  no upstream source patch is justified by the available cause evidence.

The `.out-of-scope/` catalog has no GitHub CLI exemption.
The tracker search `repo:cli/cli "TLS handshake timeout" "watch"` returned no results.
The broader transport search was also checked;
an empty narrow query does not establish a new defect.
There is nothing actionable to file from this incident,
and no upstream message was sent.

[evidence]: ../audit/depot-native-consumers-and-variation-2026-10-18.evidence.json
