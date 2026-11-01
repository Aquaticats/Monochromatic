# GitHub Actions rejects runner context in job environment declarations

## Symptom

On 2026-10-16,
dispatching `.github/workflows/npm-publish.yml` at commit `af6cc7ad5b` returned HTTP 422:
`Unrecognized named-value: 'runner'. Located at position 1 within expression: runner.temp`.
The offending declarations put `${{ runner.temp }}/fs-path-pack` in `jobs.pack.env`
and `jobs.publish.env`.
No runner job started for the invalid workflow.

## Root cause

This was an error in our workflow configuration.
The hosted service's workflow parser source and version are not available for this incident.
GitHub's [context availability reference][contexts] excludes `runner` from `jobs.<job_id>.env`,
but includes it for `jobs.<job_id>.steps.env` and `jobs.<job_id>.steps.with`.
Putting the same expression in a supported step field resolved the validation error.

## Verification

Failing declaration:

```yaml
# .github/workflows/npm-publish.yml
jobs:
  pack:
    env:
      FS_PATH_PACK_DIR: ${{ runner.temp }}/fs-path-pack
```

Working declaration:

The task was renamed from `npm:fs-path:pack` to `npm:module:pack` on 2026-10-16
when the workflow gained a logger choice;
the historical declaration retains the task name used during the fix.

```yaml
# .github/workflows/npm-publish.yml
jobs:
  pack:
    steps:
      - run: mise run npm:fs-path:pack
        env:
          FS_PATH_PACK_DIR: ${{ runner.temp }}/fs-path-pack
```

Dispatch at the invalid revision failed before execution.
After moving the environment declaration into the invoking step,
GitHub accepted [run 38837217115][accepted].
That run was cancelled to align the source checkout with the dispatched revision;
[run 38837297263][release] uses the corrected workflow from the release branch.

## Verified workaround

Set runner-dependent environment values on the step that needs them.
Use `${{ runner.temp }}` directly in artifact action inputs.
The tradeoff is repeating the expression at the relevant steps instead of declaring it once per job.

## What does not work

Repeating the job-level expression in another job produces the same validation error.
Installing tools or changing npm authentication cannot resolve an error that occurs before jobs start.

## Logger host trial recurrence

On 2026-10-17,
the isolated logger host pilot at `c6aa8b698c66343a28b4dc84b3d7185d7d11c6e1`
hit the same HTTP 422 diagnostic before any jobs started.
Its job environment declared `PROVENANCE_OUTPUT` and `LOGGER_HOST_RESULT_OUTPUT`
using `runner.temp`.
The final trial workflow also declared `DEPOT_COHORT_IDENTITY_OUTPUT_DIRECTORY` at job scope.

Root trial `81d751db414445c281aebbdc2eaf2735434799c6` moves those values into the consuming steps.
The development-only pilot applies the same correction at `a4a8e2501c02d590056a9f53991025f8aa93e5af`.
GitHub accepted [the corrected host pilot][logger-host-pilot].
The source remains immutable,
the original platform matrix and commands remain unchanged,
and no final common manifest was published by this development workflow.
Dispatch acceptance proves this parser repair;
source prerequisites and authored checks require their own terminal receipts.

## Service-container context recurrence

The isolated service research profile at `6b3c75517e798161e79ddb7e496b78cf1ffc4240`
also put `job.services` in a job environment declaration.
GitHub run `39108872769` concluded `failure` with zero jobs.
The retained run/check APIs did not recover its raw hosted parser diagnostic;
do not substitute the older `runner.temp` error text for this incident.
The [context availability reference][contexts] excludes `job` from `jobs.<job_id>.env`
and includes it for step environments.
Moving the service expression to a consuming step was accepted in the subsequent profile.
The final service run `39113547458` has six successful GitHub jobs.
The [context and positive-control receipt](../audit/depot-container-prerequisites-2026-10-18.evidence.json)
retains both job-count API responses and the exact historical workflow path,
`.github/workflows/readonly-semantic-bridge.yml`.
The later Git initialization and Node-loader failures are separate incidents in the
[container prerequisite note](github-actions-container-node-runtime-prerequisites.md).

## Upstream filing decision

- Fault:
  ours;
  the context restriction is documented.
- Upstream fixability:
  no service defect was established.
- Supported usage:
  step environment and action input fields support the required expression.
- Contribution acceptance:
  no contribution is proposed because the configuration correction resolves the incident.
- Upstream direction:
  the published context reference explicitly describes this restriction.
- Prototype:
  the consumer workflow correction was exercised through GitHub dispatch.

No upstream issue or comment is warranted.
There is no additive upstream finding or draft to file.

[contexts]: https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability
[accepted]: https://github.com/Aquaticats/Monochromatic/actions/runs/38837217115
[release]: https://github.com/Aquaticats/Monochromatic/actions/runs/38837297263
[logger-host-pilot]: https://github.com/Aquaticats/Monochromatic/actions/runs/39023040772
