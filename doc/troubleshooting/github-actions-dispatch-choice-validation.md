# GitHub Actions dispatch choice validation during pnpm research

## Observed boundary

The workflow file named in this document is the isolated research branch's copy of
`.github/workflows/music-player-windows.yml`.
On `main` the production workflow of that name moved into the `test-windows` job of
`.github/workflows/music-player-release.yml` on 2026-10-22,
so the file exists only on the retained research branch; to repeat the dispatch, use that branch,
and to dispatch the production tests, run `music-player-release.yml`.

On 2026-10-18,
the isolated pnpm cache experiment's first cache-disabled dispatch failed before allocating jobs.
GitHub CLI returned:

```text
# gh workflow run music-player-windows.yml, isolated research branch
HTTP 422: Provided value 'off' for input 'phase' not in the list of allowed values
```

The pushed workflow at `165e0fb3f71f608e63cde1b9813aa91b5d68833c` declared:

```yaml
# .github/workflows/music-player-windows.yml, isolated research branch
options: [off, cold, seed, warm, branch]
```

The exact dispatch response was retained before any recovery.
A subsequent repository-scoped run listing contained no run at that source revision.
The failed receipt was archived rather than overwritten.

## Verified recovery and uncertainty

The only subsequent workflow edit quoted the `off` string:

```yaml
# .github/workflows/music-player-windows.yml, isolated research branch
options: ['off', cold, seed, warm, branch]
```

Source `2a155eec4935e5df225cb7a90af573357c3a94dc` was pushed before the next dispatch.
GitHub accepted [run 39101112370][accepted],
and every selected GitHub and Depot job passed its actual install and artifact consumers.
No subscription,
cache setting or production workflow was changed.

The hosted validator's deciding source and deployed version are not available.
The published [Actions tooling parser][parser] uses the `yaml` package's `parseDocument`;
its scalar reader distinguishes actual string,
Boolean and number values.
That code is not proof that GitHub's hosted dispatch validator uses the same parser.
The probe did not isolate parser interpretation from workflow-ingestion delay.
Do not claim that GitHub generally parses an unquoted `off` option as a Boolean,
or that quoting alone has been established as the cause of recovery.

## Operational response

Use explicit quoted strings for this experiment's choices.
Preserve validation responses and check for a remote run before repeating a failed dispatch.
Do not reinterpret a pre-allocation input rejection as a runner or cache failure.
The tradeoff is additional reconciliation before retrying a mutation whose response might be ambiguous.

The successful run verifies this concrete recovery,
not a general parser rule or a stable service defect.
No upstream issue or contribution was filed:
there is no isolated hosted-validator defect or supported source change to propose.

[accepted]: https://github.com/Aquaticats/Monochromatic/actions/runs/39101112370
[parser]: https://github.com/actions/languageservices/blob/main/workflow-parser/src/workflows/yaml-object-reader.ts
