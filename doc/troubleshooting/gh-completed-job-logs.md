# GitHub CLI 2.102.0 rejects completed-job logs while the workflow is running

## Symptom

During a matrix run,
`gh run view --job 116712966842 --log` rejected a completed job because other jobs were still active:

```text
# gh run view diagnostic
run 38819692881 is still in progress; logs will be available when it is complete
```

The job-specific REST endpoint returned the completed job's log successfully.
The initial `gh api` invocation also exposed a separate output guard:

```text
# gh api diagnostic
the response contains terminal escape sequences; pass --allow-escape-sequences to output it anyway
```

## Root cause

GitHub CLI 2.102.0 corresponds to source tag `v2.102.0`,
commit `fc4b137cdef0a6bd28fd461b7cf9c84a5812a8cd`.
The source clone's `pkg/cmd/run/view/view.go:313` checks the selected job,
then line 318 independently checks the whole run:

```go
// pkg/cmd/run/view/view.go:318
if run.Status != shared.Completed {
    return fmt.Errorf("run %d is still in progress; logs will be available when it is complete", run.ID)
}
```

The next operation obtains the run-log archive.
Selecting a completed job does not bypass the run-level gate on this path.
This is narrower than a claim that GitHub cannot expose the completed job's logs.

In `pkg/cmd/api/api.go:538`,
raw non-JSON output passes through `CopyGuardedContent` unless explicitly allowed:

```go
// pkg/cmd/api/api.go:538
if !isJSON && !opts.AllowEscapeSequences && bodyWriter != io.Discard {
    err = iostreams.CopyGuardedContent(bodyWriter, responseBody, opts.IO.IsStdoutTTY())
```

The `ErrEscapeSequence` branch at line 543 emits the diagnostic quoted in the symptom section.
The native flag is declared at line 307.

## Verification

The following completed-job endpoint was read successfully while run `38819692881` was active:

```sh
# Run from the repository; save raw bytes privately instead of rendering terminal controls.
gh api repos/Aquaticats/Monochromatic/actions/jobs/116712966842/logs \
  --allow-escape-sequences > "${HOME}/temp/agent/depot-trial-2026-10-15/sizing-first-complete.log"
```

The saved log includes the artifact assertion failure and all completed measurements from that job.
The monitor subsequently retrieved other completed jobs through the same endpoint.
Its displayed excerpts are JSON-encoded strings,
so terminal escape bytes are escaped rather than interpreted.

Working cases are job-specific REST retrieval after that job finishes,
and `gh run view --log` after the entire workflow finishes.
Failing cases are the run-level log command during an active workflow,
and raw `gh api` output containing escape sequences without the native opt-out flag.
This verification makes no claim about logs from a job that is itself still running.

The [official workflow-jobs API](https://docs.github.com/en/rest/actions/workflow-jobs)
documents the job-log endpoint and its redirect response.

## Verified workaround

Use the job-specific REST endpoint for each completed matrix job.
Use `--allow-escape-sequences` only with raw output captured into a private file or process buffer,
then parse or escape excerpts before displaying them.
This yields whole-job logs;
it does not supply live streaming or per-step retrieval.
The archive remains subject to GitHub's log retention.

## What does not work

- Adding `--job` to `gh run view --log` did not bypass the run-completion check.
- Redirecting the initial API request alone did not remove the escape-sequence diagnostic in this invocation.
  The native flag succeeded.
- An empty search for live logs would not establish inability to retrieve completed-job logs.
  The two cases have different completion boundaries.

## Upstream filing decision

The `.out-of-scope/` inventory contains no GitHub CLI exemption.
Tracker searches included `completed logs`,
`logs progress`,
and `completed job logs running`.
Issue [11157](https://github.com/cli/cli/issues/11157) concerns per-step and still-running-job retrieval,
which is a different boundary.
Its comments do not invalidate the completed-job REST workaround.
No upstream message was sent.

1.   Fault attribution:
     the source establishes a command-path restriction;
     this trial does not establish a defect in the job-log endpoint or output guard.
2.   Fixability:
     a different command implementation could retrieve completed jobs independently,
     as the existing REST command already does.
3.   Supported scope:
     the REST endpoint supports the completed-job operation used by the monitor.
4.   Contribution policy:
     no upstream change is proposed,
     so submission eligibility was not established.
5.   Maintainer direction:
     issue 11157 discusses a different feature request;
     no claim about willingness to change this gate follows from it.
6.   Prototype:
     the native REST command is the exercised consumer workaround;
     no upstream patch was needed.

There is no new upstream filing artifact:
the task is served by an existing supported command,
and this investigation makes no new defect claim.
