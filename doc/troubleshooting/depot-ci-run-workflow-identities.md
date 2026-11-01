# Depot CLI 2.102.23 rejects a workflow ID passed to run status

## Symptom

Passing `5rrgj5qvst`,
the workflow ID in an observed GitHub check detail URL,
to `depot ci status` produced:

```text
# depot ci status 5rrgj5qvst --org l4zncjjbj5 --wait --output json
failed to get run status: not_found: Run not found
```

This diagnostic did not establish that the real parent run was missing.
The command received a different resource's ID.

## Source trace and cause

The installed CLI is `2.102.23`.
The inspected `depot/cli` source is commit `1f96d6d488f938de97ea640b115e9567ebfd6fbf`.
In `pkg/cmd/ci/status.go:25`,
the command declares `status <run-id>`.
At line 63,
it passes that argument to the run-status API:

```go
// pkg/cmd/ci/status.go:63
resp, err := ciGetRunStatus(ctx, tokenVal, orgID, runID)
if err != nil {
    return fmt.Errorf("failed to get run status: %w", err)
}
```

The workflow command declares `show <workflow-id>` in `pkg/cmd/ci/workflow.go:129`.
Its call at line 157 uses a separate API:

```go
// pkg/cmd/ci/workflow.go:157
workflow, err := ciGetWorkflow(ctx, tokenVal, orgID, workflowID)
if err != nil {
    return fmt.Errorf("failed to get workflow: %w", err)
}
```

This is a consumer ID-selection error.
The server implementation that decides `not_found` is not public in this CLI source;
no claim about its internal lookup algorithm follows from these calls.

## Verification

The failing catalog contains the workflow ID passed to `ci status`.
The working catalog contains the workflow-specific lookup of that same ID:

```sh
# Actual workflow lookup, 2026-10-18
depot ci workflow show 5rrgj5qvst --org l4zncjjbj5 --output json
```

That invocation returned workflow `5rrgj5qvst`,
parent run `nhwk20hk2v`,
job `42bvn31hvc`,
and attempt `sw53q8jn74`.
It also returned the actual PR head and merge checkout,
preserving their separate identities.
The subsequent `ci status nhwk20hk2v` lookup succeeded during the retained PR campaign.
Its intentional fixture failure is a run result,
separate from the initial lookup diagnostic.

The evidence collector is the `collect-pr-campaign` task in the private research scratch manifest.
The executable PR fixture is retained on branch `research/depot-runtime-orchestration-20261018`.
The campaign is documented in
[the runtime research note](../planning/depot-runtime-orchestration-research.md).

## Verified workaround

Use `depot ci workflow show` for a known workflow ID,
then take `run.run_id` from its JSON when a run-scoped command is required.
This adds a lookup and preserves the distinction between workflow execution and parent run.
The check-detail URL observed in this incident contained a workflow ID;
the URL path alone should not be used to infer the CLI argument type.

## What does not work

- Repeating `ci status` with the same workflow ID does not change the resource kind.
- Treating `Run not found` as evidence of lost execution invents a runtime failure.
- Treating the workflow ID as an attempt ID would also discard the parent and job identities.

## Upstream filing decision

No upstream issue or comment is proposed.
The tracker search in `depot/cli` for the exact phrase `Run not found` returned no issue matches.
That search is not evidence that every comparable report is absent.

1.   Upstream fault is not established:
     the command's declared argument is a run ID.
2.   Automatic ID resolution could be added in principle,
     but no change to that contract is needed for this research.
3.   Workflow lookup is a supported use case through its dedicated command.
4.   Contribution policy was not evaluated because no upstream defect is proposed.
5.   Maintainer disposition was not evaluated for the same reason.
6.   No upstream prototype is needed for the verified command correction.

The `.out-of-scope/` filing check is unnecessary because no filing or draft is being created.
The durable artifact is this diagnosis and the working lookup,
with nothing to send upstream.

## Run creation has a different output contract

On 2026-10-18,
the same CLI rejected `depot ci run --workflow ... --output json` with
`unknown flag: --output`.
The retained negative control exited 1 with no stdout.
`depot ci run --help` lists no `--output` option.
The command's supported text output is explicit in `pkg/cmd/ci/run.go:241`:

```go
// pkg/cmd/ci/run.go:241
fmt.Printf("Org: %s\n", resp.OrgId)
fmt.Printf("Run: %s\n", resp.RunId)
```

Removing the unsupported flag created run `rvvg2kv0jk` at harness
`9779cb896b92969b84f45e2737117231638aa700`.
The collector extracts the anchored `Run: <id>` line,
then uses `ci status <id> --output json` for structured state.
This adds text parsing at creation;
it does not imply that every CI subcommand supports the same output flags.
The original creation error was a harness invocation error,
separate from any subsequent container result.

The upstream filing decision remains unchanged:
the advertised command interface matches the observed behavior,
and no upstream patch or report is proposed.
