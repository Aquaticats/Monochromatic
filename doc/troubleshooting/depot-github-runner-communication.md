# Depot GitHub runner loses communication before recording steps during the October 15 trial

## Symptom

GitHub Actions failed job `116693196264` in run `38813764705`,
attempt 1,
on runner label `depot-ubuntu-24.04-4`.
Its annotation began:

```text
The self-hosted runner lost communication with the server.
```

GitHub's remaining diagnostic suggested checking machine availability,
network connectivity,
process termination,
and resource starvation.
It did not identify which condition occurred.
The job recorded no step results and its job-log endpoint returned HTTP 404.
The failed worker's runner version and private diagnostic logs were unavailable.
The later successful worker reported Actions runner `2.337.0`.
[Original run](https://github.com/Aquaticats/Monochromatic/actions/runs/38813764705/attempts/1)

## Root cause

The cause is unresolved.
An empty step list does not identify whether the failure was in runner provisioning,
network communication,
job acquisition,
or another part of dispatch.
No memory-exhaustion or workflow-process-kill explanation was established.

The inspected public `actions/runner` source revision was
`796cc18ffaaafd5cea525d6231f5a8c93c11c603`.
In `src/Runner.Listener/JobDispatcher.cs:758`,
the running-job renewal path calls the service and logs renewal success:

```csharp
// src/Runner.Listener/JobDispatcher.cs:760
var renewResponse = await runServer.RenewJobAsync(planId, jobId, token);
Trace.Info($"Successfully renew job {jobId}, job is valid till {renewResponse.LockedUntil}");
```

That is a possible diagnostic boundary to inspect,
not a traced cause of this incident.
A repository-wide search did not find the complete service annotation in this public source checkout.
The failed worker's diagnostics are needed to locate its actual call path.
GitHub documents runner and worker diagnostic logs as the investigation surface.
[Runner source](https://github.com/actions/runner),
[GitHub troubleshooting documentation](https://docs.github.com/en/actions/how-tos/manage-runners/self-hosted-runners/monitor-and-troubleshoot?platform=linux)

## Verification

Read-only inspection of the original execution:

```sh
# shell, from the repository root
gh run view 38813764705 --repo Aquaticats/Monochromatic --attempt 1
```

Failing catalog:

- Original Depot build job `116693196264`:
  failed before any step results,
  with the lost-communication annotation.
- Original job-log retrieval:
  HTTP 404,
  so no runner-side cause can be inferred from those logs.

Passing catalog:

- Minimal Depot runner connection run `38815232329` completed its command.
- The original build retried as job `116700352199` in attempt 2,
  completed cold and warm builds,
  and printed `music-player 0.1.4` from the runtime stage.
  Its cold build took 757.394 seconds;
  warm repetitions took 1.173 and 1.093 seconds.

[Connection probe](https://github.com/Aquaticats/Monochromatic/actions/runs/38815232329),
[Successful retry](https://github.com/Aquaticats/Monochromatic/actions/runs/38813764705/attempts/2)

## Verified recovery

After the original workflow ended,
the following retry completed successfully:

```sh
# shell, historical recovery command; executing again creates additional cloud work
gh run rerun 38813764705 --repo Aquaticats/Monochromatic --job 116693196264
```

Retrying restored execution for this trial.
It consumes additional runner time and does not prevent recurrence,
prove a root cause,
or justify ignoring the original failure in reliability measurements.

## What does not establish a fix

- A later minimal command passing establishes later availability,
  not the cause of the failed build dispatch.
- The successful full retry does not prove that workflow configuration changed or that the service was repaired.
- A repository runner-list query returned no persistent runners,
  but that is not an organization-wide runner inventory or proof that ephemeral workers were unavailable.
- [Actions runner issue 4703](https://github.com/actions/runner/issues/4703)
  reports a similar empty-step failure with runner `2.336.0` and a diagnostic trace after job acknowledgement.
  This trial lacks the failed runner's version and trace,
  so the report is comparable evidence,
  not a matching diagnosis or candidate fix.

## Upstream filing decision

The `.out-of-scope/` entries were checked;
none specifically exempts Depot runner incidents.
The Actions runner tracker was searched for the lost-communication annotation.
Issue 4703 was read in full,
including its absence of comments.

- Upstream fault:
  unproven because the failed worker's diagnostic evidence is unavailable.
- Fixability:
  unknown until the failed boundary is identified.
- Supported use case:
  Depot documents the GitHub runner product,
  and the successful retry demonstrates this workflow can execute there.
- Contribution acceptance:
  not assessed for a filing because there is no source-backed candidate defect to report.
- Likelihood of a fix:
  unknown;
  the comparable issue is not evidence about this incident's cause.
- Prototype:
  none,
  because no matching defect or candidate remedy was established.

There is no additive source-backed finding to post to issue 4703.
No upstream issue or comment was sent,
and no speculative patch was retained.
The next diagnostic requirement is the failed worker's runner/worker logs or provider dispatch trace.
[Depot runner documentation](https://depot.dev/docs/github-actions/overview)
