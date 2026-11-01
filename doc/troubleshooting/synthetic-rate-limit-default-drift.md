# Synthetic service on 2026-08-29 changed its request baseline and made static planning weights stale

## Symptom

The translation-repair package estimated Synthetic five-hour request weight against
`0.000001` dollars per prompt token.
Synthetic's rendered [rate-limit documentation][rate-limits] now says its default model is
`moonshotai/Kimi-K3` and one call to it counts as exactly one request.
The active catalog records Kimi-K3 at `0.000003` dollars per prompt token.

No runtime error is emitted.
Routing keeps working because live `/quotas` data remains authoritative,
but relative planning weights are three times too large while old denominator remains.

Same-day model-catalog output also showed provider alias movement:

```text
syn:large:text -> zai-org/GLM-5.3-Flash
syn:small:text  (zai-org/GLM-4.7-Flash)
```

Those alias rows are provider catalog facts.
They do not mean translation-repair called GLM-4.7-Flash.

## Root cause

Synthetic is hosted service and no provider source repository was found in its
[API documentation][api-overview] or public repository search.
This diagnosis therefore stops at documented service boundary rather than inventing source-level call chain.

Consumer-side call chain is visible:

1. `package/translation-repair/provider/src/synthetic-catalog.ts:242` stores static denominator.

   ```ts
   export const SYNTHETIC_BASELINE_PROMPT_DOLLARS_PER_TOKEN = 0.000003;
   ```

2. `package/translation-repair/provider/src/synthetic-catalog.ts:296` divides each model's current prompt price by that
   denominator.

   ```ts
   return modelPrice / SYNTHETIC_BASELINE_PROMPT_DOLLARS_PER_TOKEN;
   ```

3. Provider changed documented default while package still carried earlier same-day GLM-5.2 denominator.
   No live field binds five-hour request denominator to current default,
   so static consumer value did not move automatically.

Earlier hypothesis that current model alias identifies request-weight baseline was wrong.
Live catalog now maps `syn:large:text` to GLM-5.3-Flash,
while rate-limit page names Kimi-K3 as baseline.
These are separate provider concepts.

## Verification

Observed service date:
2026-08-29.
Synthetic publishes no release version for this page,
so date and live response are version boundary available.

Run current package catalog comparison:

```sh
mise run //package/translation-repair/bench:model-catalog
```

Working catalog facts:

- configured roster rows missing from provider:
  `0`
- `syn:large:text` resolves to GLM-5.3-Flash
- `syn:small:text` and exact GLM-4.7-Flash remain unlisted provider rows
- Nemotron remains provider-served but unlisted by package

Inspect rendered [rate-limit documentation][rate-limits].
Working baseline statement names Kimi-K3 and one request.
Stale statement names GLM-5.2 or uses `0.000001` denominator.

Run package guard:

```sh
mise run //package/translation-repair/<package>:build
node package/translation-repair/provider/src/synthetic-catalog.unit.test.ts
```

Working catalog passes `0.000003` baseline assertion and derives every model weight from it.
Changing constant back to `0.000001` fails assertion.

## Verified workarounds

Keep denominator explicit and update it with rendered rate-limit documentation.
This preserves current planning behavior and leaves live `/quotas` meter authoritative.
Tradeoff:
static value can drift again when provider changes default without versioned API field.

Do not admit aliases to active roster.
Exact model ids keep model identity and voting seats stable when aliases move.
Tradeoff:
provider retirement becomes visible failure requiring repository update rather than transparent migration.

## What does not work

- Inferring request baseline from `syn:large:text` does not work.
  Live alias maps GLM-5.3-Flash while rate-limit baseline is Kimi-K3.
- Reading `/quotas` does not recover denominator.
  Endpoint reports aggregate remaining quota,
  not per-model baseline price used by planner.
- Treating every GLM-4.7 mention as call evidence does not work.
  Provider catalog output and stream-parser tests mention model without dispatching it.
- Deleting historical model names does not prevent calls.
  Callable roster and request validation are relevant boundaries;
  historical evidence must retain original identity.

## Upstream filing decision

No `.out-of-scope/` entry matches Synthetic.
Public search found provider documentation and third-party integrations,
but no Synthetic service source repository or matching provider issue tracker.

1. **Is it upstream's fault?**
   No.
   Provider intentionally changed documented default and aliases.
   Stale constant was consumer responsibility.
2. **Can upstream fix it?**
   Not applicable as defect.
   Provider could expose baseline in API,
   but absence is not documented contract violation.
3. **Are they supporting this use case?**
   Partly.
   Rate-limit page documents weight rule,
   but not machine-readable discovery.
4. **Would repository welcome contribution?**
   Unknown.
   No provider source or contribution policy was found.
5. **Will they likely fix it?**
   Unknown and no defect was established.
6. **Have we prototyped compatible fix?**
   Yes at consumer boundary:
   update denominator and pin it with package test.

Nothing should be filed upstream.
There is no provider defect report or additive comment to draft.

[api-overview]: https://dev.synthetic.new/docs/api/overview
[rate-limits]: https://synthetic.new/rate-limits
