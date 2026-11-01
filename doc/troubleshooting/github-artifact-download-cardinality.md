# actions/download-artifact v7 flattens one pattern match and breaks a nested receipt consumer

## Symptom

With `actions/download-artifact` at `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c`,
a workflow that downloads a pattern into `receipts/` and then expects an artifact-named subdirectory
fails when only one artifact matches.
Node 22.23.3 reports:

```text
Error: ENOTDIR: not a directory, open 'receipts/routing-receipt.json/routing-receipt.json'
```

The two-provider routing fixture passes,
but the GitHub-only and Depot-only fixtures fail in the final consumer after their worker jobs succeed.
The original failed runs are
[39099183237](https://github.com/Aquaticats/Monochromatic/actions/runs/39099183237)
and [39099184929](https://github.com/Aquaticats/Monochromatic/actions/runs/39099184929).
This is an incorrect consumer assumption,
not a failed runner allocation or application test.

## Root cause

In the pinned upstream `src/download-artifact.ts:189`,
the extraction target depends on the number of selected artifacts as well as explicit inputs:

```ts
// actions/download-artifact/src/download-artifact.ts:189
path:
  isSingleArtifactDownload ||
  inputs.mergeMultiple ||
  artifacts.length === 1
    ? resolvedPath
    : path.join(resolvedPath, artifact.name),
```

One pattern match extracts the receipt directly into `receipts/`.
Multiple matches with the default merge setting extract into artifact-named subdirectories.
The consumer treated the direct receipt file as a directory.
Its assumption that the default merge setting guarantees subdirectories was wrong.
The pinned `README.md:66` explicitly documents single-match pattern flattening.
The clone was inspected at `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` from `actions/download-artifact`.
No upstream source was modified or executed.

## Verification

The initial harness is the isolated research commit
`02d1d845baf27a3613fc44f5382866d930ffe1f3`,
at `.github/workflows/music-player-windows.yml`
(a file that exists only on the research branch since the production workflow of that name moved into the
`test-windows` job of `.github/workflows/music-player-release.yml` on 2026-10-22).
Its comparison run
[39099180101](https://github.com/Aquaticats/Monochromatic/actions/runs/39099180101)
passes with two artifacts.
Its intentional worker-failure control
[39099181740](https://github.com/Aquaticats/Monochromatic/actions/runs/39099181740)
fails the final gate as intended.
The single-provider runs reproduce the consumer failure quoted in `Symptom`.

The corrected harness is research commit `3ea5cab1cba6ca42b4eb9ad8b63f91e21ec04ea9`.
Its complete expected-outcome catalog passed:

- [39099643878](https://github.com/Aquaticats/Monochromatic/actions/runs/39099643878):
  both pools and the final consumer pass.
- [39099648238](https://github.com/Aquaticats/Monochromatic/actions/runs/39099648238):
  the GitHub-only worker and final consumer pass.
- [39099650257](https://github.com/Aquaticats/Monochromatic/actions/runs/39099650257):
  the Depot-only worker and final consumer pass.
- [39099645918](https://github.com/Aquaticats/Monochromatic/actions/runs/39099645918):
  the deliberate Depot worker failure still fails the final consumer gate.

The harness is retained on `research/depot-pool-routing-20261018`.
To reproduce its corrected positive comparison at the retained branch,
dispatch the existing workflow with explicit inputs
(this works only on the retained research branch; on `main` there is no `music-player-windows.yml` to dispatch,
and the production tests run through `music-player-release.yml`):

```sh
# .github/workflows/music-player-windows.yml on the research branch only
gh workflow run music-player-windows.yml --repo Aquaticats/Monochromatic \
  --ref research/depot-pool-routing-20261018 \
  --raw-field pool=compare --raw-field fail_pool=none --raw-field hold_seconds=0
```

For a failing control,
change `fail_pool` to `depot`.
For cardinality controls,
set `pool` to `github` or `depot` and `fail_pool` to `none`.
The retained branch includes the subsequently verified bounded cancellation control;
the `hold_seconds=0` input preserves normal receipt behavior.

## Verified workarounds

Use distinct filenames at the producer boundary and explicit merged extraction at the consumer:

```yaml
# Producer and consumer steps in the research workflow
- uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
  with:
    name: 'routing-receipt-${{ github.run_id }}-${{ matrix.pool }}'
    path: 'routing-receipt-${{ matrix.pool }}.json'
    if-no-files-found: error
- uses: actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c
  with:
    pattern: 'routing-receipt-${{ github.run_id }}-*'
    path: receipts
    merge-multiple: true
```

The consumer checks that the entire extracted filename set exactly equals its selected pool set,
then verifies source SHA,
run ID,
attempt,
and worker results.
Merged extraction can overwrite colliding filenames,
so pool-specific producer filenames are part of the workaround.
The exact filename check also rejects unexpected or missing receipts.
All downloaded receipt archives were hashed against GitHub's artifact digests;
see the retained [routing proof](../planning/depot-developer-pool-routing-proof.json).
No production workflow has changed.

## What does not work

Leaving `merge-multiple` at its default and always adding an artifact-named directory fails
when the pattern selects one artifact.
The successful two-artifact fixture cannot validate the single-artifact consumer path.

## Upstream filing decision

Nothing to add upstream.
The authored receipt consumer must handle the action's documented extraction contract.

- Upstream fault:
  no.
  `README.md:66` and `src/download-artifact.ts:189` agree with the observation.
- Fixability:
  the upstream conditional is editable,
  but changing documented behavior is unnecessary to fix this consumer.
- Supported use:
  yes.
  Patterns and merged downloads are documented and are exercised by the corrected fixture.
- Contribution policy:
  the pinned `README.md:89` says the repository is not taking contributions
  and directs high-priority bugs to GitHub Community or support.
  `CONTRIBUTING.md` describes an older contribution process;
  the current README supplies the explicit restriction.
- Likely movement:
  no upstream behavior change is needed.
  A lack of maintenance guarantees would not prohibit requested investigation or consumer fixes.
- Minimal fix:
  the consumer workaround is implemented and verified by both cardinalities and the negative worker control.
  No upstream prototype is required because the upstream-fault condition fails.

The repository's `.out-of-scope/` inventory contains no matching GitHub artifact-action exemption.
The upstream clone has no `.out-of-scope/` directory.
On 2026-10-18,
issue and PR searches in `actions/download-artifact` for `single pattern` returned no results;
issue searches for the deciding expression and for `artifact single directory` also returned none.
These searches do not establish absence of every related discussion.
The deciding README already supplies the behavior needed here,
so no issue,
comment,
or external message was posted.
