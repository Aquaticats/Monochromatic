# Liquid AI current contract check

## Scope and authorization

On October 14,
2026,
the user authorized sending anything needed to `liquid.ai`
and added `AUTO_MODE_LIQUID_API_KEY`.
The credential is present in the repository's ignored `.env.local.json`.
This extends the provider investigation without adopting a replacement.
The production cutoff,
five-second deadline,
and three-call ceiling remain unchanged.

Liquid's hosted `d1` is the candidate managed service.
The sensitive-input and incumbent-replacement overlays apply.
No third-party executable or SDK installation is proposed for this screen.
Local-code and native-artifact audit gates are therefore inapplicable to this direct HTTP probe.
Remaining SaaS contract and operational gates stay pending if technical screening succeeds.

## Current primary documentation

The [hosted d1 reference][d1] documents bearer authentication and
`POST https://api.liquid.ai/decisions/v1/systemone`.
The model name is `d1`.
State and named questions produce typed Noul,
Choice,
or Score answers with probabilities and zero generated output tokens.
The documented request body ceiling is 4.5 MB.
The inspected reference does not state a text-context or question-count maximum.
No claim of unlimited capacity follows.

Initial queries were `site:docs.liquid.ai API inference authentication models`
and `site:liquid.ai API probabilities logprobs`.
The official documentation index led to the decision-model overview;
its Markdown card target led to the dedicated hosted `d1` reference.
Public unauthenticated probes of `/openapi.json` and `/v1/models` returned 404.
Those were discovery probes,
not documented model-list endpoints or evidence of invalid credentials.
The dedicated reference supplies the correct Decisions route.

The [Jev incumbent](pi-auto-mode-jev-current-contract.md),
[Drex](pi-auto-mode-drex-current-contract.md),
and [SubQ](pi-auto-mode-subq-current-contract.md)
remain separate candidates with their existing evidence.
No cross-provider ranking or adoption recommendation is made by this compatibility check.

## Frozen access control

Before other queued work,
send one request to the documented route with model `d1`,
state `The card is green.`,
and one Noul question `Is the card green?`.
The request has a five-second deadline,
no retry,
no tools,
and no represented action.
Retain the request hash and raw response privately at
`${HOME}/temp/agent/pi-liquid-access-20261014`.
Require HTTP 200,
the requested model,
the complete expected answer map,
and a finite probability in the unit interval.
This tests access and serving only;
it is not semantic qualification or complete-input capacity evidence.

## Credential-loading correction

The initial request and one explicit access recheck returned HTTP 401 `invalid_api_key`
in 704.051 ms and 699.277 ms.
Both sent the encrypted SOPS value read directly from `.env.local.json`.
They did not exercise the user's decrypted key.
The claim that the actual key lacked the documented prefix was also incorrect:
that check examined ciphertext.
After the user questioned the result,
`mise.toml`'s SOPS configuration and `mise env --help` established the correct loading boundary.
An in-memory `mise env --json` capture resolves a nonempty decrypted Liquid key
with the documented prefix and a nonempty decrypted SubQ key.
The real-key control ran in a separate directory with a guard rejecting encrypted values before dispatch.

## Resolved-key result

`${HOME}/temp/agent/pi-liquid-access-resolved-20261014` retains the frozen request and result.
The synthetic encrypted credential was rejected before any dispatch or started marker.
With the Mise-resolved key,
the identical 118-byte request returned HTTP 200 with model `d1`,
the expected answer,
and probability 0.9964063980321786 in 591.486 ms.
Usage was 32 input tokens and zero output tokens.
This verifies serving access only.
The next frozen probe will change only the model in the complete captured native assessment to `d1`,
preserving every state field and all 84 typed questions.
It has one attempt,
a five-second deadline,
and no represented execution.
Complete answers and original Choice criteria must validate before interpreting probabilities.

## Complete-input boundary

The model-only adaptation was 504,929 bytes with SHA-256
`ed2639100ae7f5d8a14487b0bc23c5aa08abac1bc4c27397b838d10f898dad6c`.
Its state and all 84 questions exactly matched the captured native request.
Liquid returned HTTP 422 in 1,098.969 ms:
`The prompt for question `
`head_0`
` is over the model's input limit of 65536 tokens, images included, or 250000 bytes of text.`
The raw response preserves the provider's backticks around the question name.
No model answers were returned and no retry occurred.
This establishes a rejection of this input,
not which of the two named size measures independently triggered it.
Artifacts are in `${HOME}/temp/agent/pi-liquid-complete-native-20261014`.

The next hypothesis is shared-content encoding:
the already decoded and exactly reconstructed Jev question-placement candidate preserves every source occurrence,
every original instruction,
and every question envelope while sharing repeated values.
Send it once to Liquid with only the model name changed to `d1`,
with the same five-second limit and no retry.
This does not adopt the representation or establish equivalent model interpretation.

The shared-content request was 140,601 bytes with SHA-256
`e5aeb7b3c3fb7987009f76aeedc80ffdd1713dcfe49cab0bf2e394ba1f091597`.
The single attempt ended with client `TimeoutError` after 5,005.097 ms,
without a response or answer map.
This fails the unchanged deadline for that attempt;
it does not establish acceptance,
a capacity limit,
or semantic quality.
No retry occurred.
Artifacts are retained in `${HOME}/temp/agent/pi-liquid-shared-content-20261014`.

[d1]: https://docs.liquid.ai/lfm/models/d1
