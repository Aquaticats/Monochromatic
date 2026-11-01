# Jev 1.14 release and route availability

## Scope and conclusion

This October 14,
2026 refresh answers the contract and availability part of the user's Jev-versus-Luna question.
It uses public documentation,
an unauthenticated gateway model-list read,
and current gateway source.
No inference request or production change occurred.

Jev 1.14 is documented as available natively.
The inspected gateway catalogue and source still resolve Jev to 1.13.
The new native long-context option could change the previous capacity assessment,
but gateway support,
full-input acceptance,
latency,
and semantic accuracy remain unqualified.
This is a candidate for qualification,
not evidence for switching the production model.

## Native release contract

The [model reference][models] dates `jev-1.14.0` availability to October 14,
2026.
`jev-preview` points to it;
`jev-latest` still points to `jev-1.13.0`.
Versioned IDs are accepted even when the native model-list endpoint lists only aliases.
The documented input price remains US$0.042 per million tokens,
with free output.

Default limits remain 64k tokens for state plus all questions and 32k for state plus any one question.
There is no separate question-count limit.
The same page now documents `TypeSafe-Danger-Zone-Long-Context: true`,
which raises both limits to 262k tokens on Jev 1.14 and Jev 1.13.
It costs the same per token but is explicitly slower and less accurate.
Rate limits are now organization-specific,
available in the native console.
These are vendor contract statements,
not measured account limits.

The [release announcement][release] describes incremental improvements,
with the largest expected gain in multilingual support.
It plans to move `jev-latest` after a week and states intended deprecation warnings of 60 days for 1.13
and 30 days for 1.14,
pending a published policy.
These intentions are not fixed retirement dates.
The announcement explicitly declines to vouch for experimental capabilities in production.
It also cautions that its internal benchmark is evolving,
potentially biased toward its models,
and contains only open-weight comparators.
It provides no direct evidence that Jev 1.14 beats Luna on our task.

## Gateway catalogue and deciding source

The public [model-list endpoint][catalogue] returned 301 model entries during this refresh.
Filtering IDs case-insensitively for `jev` or `typesafe` returned only `jev-1.13.0`,
with aliases `jev`,
`jev-latest`,
and `jev-preview`.
The [provider page][provider] and [Decisions guide][decisions] also still list 1.13.
Catalogue absence alone does not prove a hosted request would fail.

The [gateway source repository][gateway] was inspected at
`ebbeaf029ec4fbddf1a23ae03e46bf7c1c7d9259`:

- `packages/models/src/models/typesafe.ts` declares only 1.13 and maps its aliases to that version.
- `apps/gateway/src/decisions/handle-decision-request.ts:93`
  resolves IDs and aliases against the catalogue.
  Its no-match branch returns `model_not_found` before forwarding.
- `apps/gateway/src/decisions/decisions.ts:299`
  and `apps/gateway/src/systemone/systemone.ts:236` use that shared handler.
- `handle-decision-request.ts:605` constructs upstream headers from `Content-Type`
  and `getProviderHeaders(providerId, token, { requestId })`.
- `packages/actions/src/get-provider-headers.ts` gives TypeSafe the default bearer-authentication branch
  and optional request ID.
  It has no long-context argument or TypeSafe experimental header.
- `apps/gateway/src/lib/fetch-provider.ts` passes those constructed request options onward.
  It does not add incoming caller headers.

Thus the inspected source does not select 1.14 or forward the new long-context flag through either route.
In particular,
the native and gateway meanings of `jev-preview` differ in this evidence.
This conclusion is a source trace,
not a hosted inference result;
the hosted deployment's matching revision is unknown.

Captured source files and the complete public catalogue are retained under
`${HOME}/temp/agent/jev-114-current-contract-20261014`.
The source files were fetched by explicit revision with `gh api`,
not inferred from a search snippet.

## What this changes for our previous failures

The [retained Jev capacity experiments](pi-auto-mode-jev-current-contract.md)
rejected complete lossless representations under the previous route contract.
Those results do not test the new native long-context header.
Because that option also applies to 1.13,
capacity expansion and model-version improvement are separate variables.
Combining both in one comparison would not identify which change caused a difference.

The next admissible comparison needs an actually supported route,
the complete required input,
and the preparation-inclusive five-second budget.
It must test acceptance separately from semantic correctness,
including previous binding errors and unchanged-input variation.
Luna's retained results remain the baseline for that comparison;
release claims and increased context alone cannot establish a replacement.

[models]: https://docs.typesafe.ai/models
[release]: https://typesafe.ai/blog/jev-1-14
[catalogue]: https://api.llmgateway.io/v1/models
[provider]: https://llmgateway.io/providers/typesafe
[decisions]: https://docs.llmgateway.io/features/decisions
[gateway]: https://github.com/theopenco/llmgateway
