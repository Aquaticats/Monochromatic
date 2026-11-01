# SubQ current contract check

## Scope and authorization

On October 14,
2026,
the user authorized sending anything needed to `subq.ai`
and supplied the credential name `AUTO_MODE_SUBQ_API_KEY`.
This extends the existing provider investigation;
it does not adopt a production replacement or change the admission policy.
The key is available in the repository's ignored `.env.local.json`,
although it was absent from the current shell environment.
No credential value is retained in this document.

The documented authenticated `GET https://api.subq.ai/v1/models` returned HTTP 200
and only `subq-snap-preview`,
named `SubQ Snap Preview`,
owned by `subquadratic`.
Correction on October 14:
that call sent the encrypted SOPS value rather than the decrypted credential.
Its HTTP 200 therefore does not verify the supplied key.
The later 16-question and one-question probes returned HTTP 403 `permission_error`
using the same incorrectly loaded value.
Neither result establishes account access or model capability.
The user suggested mise might explain the failure;
inspection confirmed an owned credential-loading mistake.
`mise env --json` now resolves the real key in memory without printing it.
The original frozen control was repeated in a separate directory with the decrypted key
and a guard rejecting `ENC[` values before dispatch.

## Candidate and incumbent boundaries

SubQ Snap Preview is a managed service with sensitive-input and incumbent-replacement overlays.
No third-party local client or executable is introduced.
Source and native-artifact gates for a local dependency are therefore inapplicable to this compatibility probe.
Production adoption remains unqualified;
SaaS operational and contract gates are pending if the compatibility screen passes.

The [Jev incumbent](pi-auto-mode-jev-current-contract.md) still rejects the complete captured native input
with `max_tokens_exceeded`.
The [Drex alternative](pi-auto-mode-drex-current-contract.md) accepts a complete input beyond the deadline
but has not met the adopted semantic gate.
Keeping the incumbent preserves the current fail-closed behavior;
it does not satisfy successful governed execution.
This narrow named-provider check makes no technology ranking or adoption recommendation.

## Current primary documentation

The [models page][models] lists `subq-snap-preview` for Decisions
and a separate early-access `subq-preview` for generated chat.
Only the decision model appeared in the listing;
the ciphertext-bearing models request did not establish account authentication.
The [Decisions contract][decisions] uses `POST /v1/systemone`,
structured state,
and named typed questions with structured instructions and criteria.
It documents a 393,216-token context window and 1 to 16 questions per request.
Choice responses contain a selected label,
option probabilities,
and confidence.
These claims were inspected on October 14,
2026.

The [retention page][retention] documents organization-level retention with indefinite storage as the default,
plus time-based deletion and requested zero retention.
Its per-request `store: false` option is documented for Chat Completions and Responses,
not Decisions.
The account's actual retention setting has not been inspected or changed.
The user's explicit input-sharing authorization is recorded in this document.

The initial discovery query was `site:subq.ai API documentation`.
Targeted follow-ups were `site:docs.subq.ai/endpoints/decisions-api "Choice"`
and `site:docs.subq.ai/endpoints/decisions-api "confidence"`.
Direct retrieval of the Decisions page twice returned web-tool timeout diagnostics;
indexed primary-page content provided the complete request and response contract.
No broader selection discovery or saturation is claimed.

## Frozen compatibility probe

The current complete captured assessment has 84 questions and 73 source occurrences.
The production ceiling remains three calls and five seconds.
If a 16-question cap is enforced,
even three full batches cannot carry all 84 questions.
No question omission or batching-policy change is authorized by this probe.

The private schedule at `${HOME}/temp/agent/pi-subq-boundary-20261014`
uses the public synthetic state `The card is green.`
and identical typed yes/no questions asking whether the card is green.
It first sends 16 questions,
then 17 only if the control returns complete valid answers.
Both calls pin `subq-snap-preview`,
have five-second deadlines,
run sequentially,
and never retry.
Any rejection,
timeout,
or invalid response ends the schedule.
Acceptance of 17 would justify a separately frozen complete-input test;
it would not prove all 84 questions fit or establish semantic quality.

## Resolved-key boundary result

`${HOME}/temp/agent/pi-subq-boundary-resolved-20261014` retains both requests and results.
The synthetic encrypted credential was rejected before dispatch or a started marker.
The resolved-key control returned HTTP 200 with model `subq-snap-preview`
and all 16 expected Noul answers in 845.179 ms.
Their probabilities ranged from 0.9669985771179199 to 0.9682185649871826;
usage was 749 input tokens and zero output tokens.
The 17-question request returned HTTP 422 `validation_error` in 182.896 ms:
`questions: must NOT have more than 16 properties`.
The frozen schedule stopped without retry.

This confirms the documented question limit for the tested route and account.
Three calls carry at most 48 questions,
which cannot cover this assessment's 84 questions.
The compatibility gate therefore fails without changing the adopted assessment contract.
The result does not establish poor general model quality.
The initial authentication-based dismissal was wrong.

[models]: https://docs.subq.ai/using-the-api/models/
[decisions]: https://docs.subq.ai/endpoints/decisions-api/
[retention]: https://docs.subq.ai/using-the-api/data-retention/
