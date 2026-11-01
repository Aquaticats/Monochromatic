# Decision review

## Purpose and interface

`native/contract/integration/action-policy/decision-core.mjs` reviews one original prepared group.
It captures selected instruction and authorization inputs,
assesses applicable clause meanings,
compiles admitted meanings,
and requests complete-operation review when the clause path remains unresolved.
Its caller is the prepared group created by the native batch handler.
The compiler returns review data and currentness checks;
the prepared group owns the final confirmation route.

`src/semantic-operating-policy.ts` fixes the model,
supported labels,
and code-owned acceptance callback.
`src/instruction-resolution.ts`,
`src/authorization-requirements.ts`,
and `src/guard-outcome.ts` resolve established instruction effects and pending authorization.
Provider mechanics live behind the
[decision transport](decision-transport.md)
interface.

## Why the responsibilities remain separate

A model estimate is not an authenticated source,
established effect fact,
or user approval.
The review therefore retains original question bindings,
source identity,
model attribution,
and currentness before consuming estimates.
The compiler does not promote missing premises into facts.

Complete-operation review interprets the actual proposed group under the selected permission scope.
Formatting,
tool-use preferences,
and workflow conventions alone do not become consent requirements.
Explicit access prohibitions,
restricted destinations,
destructive effects outside authorization,
and actual consent requirements remain permission boundaries.

The operating cutoff remains `0.60` for a supported uniquely leading choice.
It is not an accuracy guarantee.
An unresolved answer reaches scoped confirmation rather than an automatic allow.
An established answer for changed dependencies cannot be replaced by evidence-free approval.

## Alternatives

Collapsing every rule mention into a prohibition causes ordinary authorized inspection to request approval.
Treating the model's answer as source authority loses instruction provenance.
Reducing the cutoff to clear a reported case would alter the adopted decision policy.
The current implementation instead preserves separate capture,
estimate admission,
resolution,
and fallback responsibilities.
Question and evidence ownership remains inside the source compiler's operation boundary.
These checks authenticate source and model responses;
they do not create a second native execution owner.

## Verification

`test:semantic-operating-policy`,
`test:native:authorization-inputs`,
`test:native:decision-fallback`,
and `test:native:decision-context` cover cutoff admission,
original authorization,
automatic outcomes,
manual fallback,
and source scope.
Live consumer controls include prohibited operations as well as authorized inspection.

Task names use `mise run //package/pi-plugin/auto-mode:<task>`.
