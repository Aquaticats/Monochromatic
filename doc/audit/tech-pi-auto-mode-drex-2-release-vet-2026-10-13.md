# Drex 2.0 compatibility with native auto-mode input

## Metadata

- Status: complete; immediate replacement excluded by native request rejection.
- Started and last updated: 2026-10-13.
- Owner: current Pi auto-mode migration session.
- Governing skill commit: `37117e36397e30233062828581a5e38217da9458`.
- Governing skill SHA-256: `552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`.
- Compatibility fingerprint: `e87ef0d6a50564d8638a4e544f0c22cf8b3cdeb689578170a86f9401e76e0bb0`.
- Prior compatible report: none; the release has a distinct serving-context constraint.

Fingerprint input uses sorted object keys,
ASCII strings,
and de-duplicated sorted set arrays:

```json
{
  "baseCategories": [
    "managed-service"
  ],
  "deployment": {
    "host": "Linux x86_64",
    "interface": "HTTPS System One",
    "runtime": "existing owned Node fetch adapter"
  },
  "hardConstraints": [
    "Complete current policy retained",
    "Exact serving-model attribution",
    "Five-second assessment budget",
    "Models estimate; code decides",
    "No account mutation or rented compute",
    "Original host input domain remains unchanged"
  ],
  "incumbent": "typesafe/jev-1.13.0 via LLM Gateway",
  "overlays": [
    "incumbent-replacement",
    "sensitive-data"
  ],
  "schemaVersion": 1,
  "scope": "Drex 2.0 release compatibility for the existing full-policy native assessor",
  "subject": "Pi auto-mode Drex 2 release",
  "trustBoundary": "Remote semantic estimates; no model-authenticated authority or final decision"
}
```

## Task and decision boundary

The user reported Drex 2.0 and delegated the choice of next action on 2026-10-13.
The ongoing target remains a working Pi auto-mode migration.
This release check must not restart an open-ended provider survey or defer native integration indefinitely.
Model assessment remains separate from original-source accounting,
authority,
authorization-channel evidence,
and human confirmation.
The original Qwen choice still applies to the Pi coding agent;
Drex is an assessor candidate.

The selected initial gate is exact serving-model compatibility with the actual complete native input.
No model-quality score or production threshold is inferred from the release announcement.
If this gate fails,
there is no full finalist score or production adoption from this phase.

## Sources and discovery ledger

- Official registry: authenticated `GET https://console.nace.ai/v1/models` returned 200.
  It lists `drex-v2.0` with release date 2026-10-06 and `alias_for: null`.
  The alias `drex-latest` points to that version.
- Official [models reference](https://console.nace.ai/docs/drex/reference/models):
  Drex 2.0 serves states up to 8,192 tokens and rows up to 16,384 tokens;
  larger accepted requests are answered by Drex 1.5.
- Official [pricing reference](https://console.nace.ai/docs/drex/reference/pricing):
  2.0 bills 60 nano-USD per input token;
  fallback billing uses the reported serving version.
- Official [quickstart](https://console.nace.ai/docs/drex/quickstart):
  the current base endpoint is `https://console.nace.ai/v1/systemone`.
- Web searches executed: `site:nace.ai Drex 2.0`,
  `site:docs.nace.ai drex-v2`,
  and `site:github.com nace-ai drex 2.0`.
  Search-provider relevance order,
  first returned page,
  no negative filter;
  these discover documentation and do not claim ecosystem saturation.
- Repository incumbents: the native wire pins `typesafe/jev-1.13.0`.
  Historical matched Drex 1.5 and Jev evidence lives in
  [the prior release recheck](../planning/pi-auto-mode-drex-1-5-recheck.md).

### Candidate ledger

#### Drex 2.0 direct API

Managed service candidate,
with incumbent-replacement and sensitive-data overlays.
Screening failed the native request compatibility gate.
The public model page and exact account registry identify the candidate.

#### Drex 1.5 direct API

Managed service comparison and documented fallback destination.
It is not silently counted as a successful Drex 2.0 run.
Its prior semantic measurements retain their historical input scope.

#### Jev 1.13.0 through LLM Gateway

Current native wire model identity.
Keeping it avoids an unqualified model change while native integration is completed.
Current native tests use authored transport fixtures,
so their success is not current Jev accuracy evidence.

The initial gate can exclude a release before a full vendor or finalist audit.
No source-code install,
self-hosted weights,
or native executable is proposed in this phase.
Local-code platform and native-artifact gates therefore do not apply to these HTTP probes.
Broader SaaS and replacement scoring remains unperformed unless a serving-compatible candidate proceeds.

## Frozen execution schedule

The no-network native capture completed before live inference.
It contains 244 questions,
663,405 serialized request bytes,
and 301,368 serialized state bytes.
The complete current policy is 32,553 bytes with SHA-256
`62faf8d90fcc04553d4949ceabbb29c7310d4ea0563e5c4efd4341438d9bb280`.
The capture executed no protected child call and sent no provider inference.

Run these fixed probes once each,
without retries or score-dependent rewording:

1.  Public constant-state API control,
    requesting `drex-v2.0`,
    to verify that the account can receive that exact version.
    This is an API control,
    not a policy assessment or qualification example.
2.  The complete current policy once,
    with the existing fixed rule-prerequisite question and its exact selected clause.
    This checks the lower boundary before native source/request duplication.
3.  The original complete native wire,
    changing only the requested model to `drex-v2.0`.
    Retain the original provider request text,
    all sources,
    every question,
    and all original source locations.

Each probe has a five-second deadline,
a one-MiB request bound,
a two-MiB response bound,
explicit response-model recording,
and no transport retry.
All results remain retained,
including fallback,
errors,
and timeouts.
The schedule does not repeat a failed authorization request or release a protected action.
The API control has no expected semantic score;
the policy probes establish serving compatibility only.

If the native request is answered by 1.5,
this is a failed direct-2.0 compatibility gate for the existing input.
Do not delete policy,
filter governing sources,
or relabel fallback output to obtain a nominal upgrade.
If all serving gates pass,
freeze a separate unchanged-input repeat and semantic comparison before using any result for adoption.

## Execution boundary

The owned runner uses Node's existing HTTPS fetch,
not a downloaded SDK or CLI.
Only the named existing Nace credential reaches the documented Nace endpoint.
It is neither printed nor persisted.
The current automatic worker cgroup supplies 4 GiB memory,
zero swap,
200% CPU,
and 256 tasks;
Node is additionally limited to a 256 MiB heap.
These are HTTP and serialization probes,
not model training or local inference.
No billing settings,
credit purchase,
account settings,
or provider package installation occurs.

Private evidence is `contract/research/drex-2.0-recheck` in the existing local qualification repository.
It has no configured Git remote or push destination,
verified with `git remote --verbose` and `git config --local --list`.

## Explicit execution confirmation

Automatic approval review initially rejected the provider phase,
first for unconfirmed payload consent,
then because it did not accept repository records of prior consent.
Both rejections occurred before inference dispatch.
The user then explicitly approved the complete current policy and captured disposable native fixture
being sent to `console.nace.ai` for the frozen three-call test.
No real session history is included.
The unchanged phase started after that confirmation.

## Results and disposition

The frozen three-call phase completed with no retry:

- The API control returned 200 and `model: drex-v2.0`.
  It reported 22 input tokens and a 1,031.995 ms local assessment interval.
- The complete-policy-once probe returned 200 and `model: drex-v2.0`.
  It reported 8,731 billed input tokens and a 915.553 ms local assessment interval.
  Billed input tokens include the question;
  they are not a measured state-only token count.
- The native batch returned 422 before model evaluation:
  `questions.head_0.instructions: must be a string`.
  Its error list reports that same field-type failure for all 244 questions.
  No serving model was reported;
  this is not evidence that the request reached either model.

These are individual intervals,
not a latency comparison or run-to-run spread measurement.
The two successful probes imply US$0.00052518 at the published returned-token rate.
Nace documents 422 responses as unbilled;
account invoices were not inspected.
The runner conservatively retained the missing usage rather than inventing a zero-token response.

### Schema-corrected follow-up

The official [question schema](https://console.nace.ai/docs/drex/guides/questions)
requires text instructions and explicitly rejects JSON objects.
The existing Jev wire uses structured instruction objects.
This is an owned adapter incompatibility,
not a demonstrated model or upstream-service failure.

Private `101929c7` freezes one separate corrected request.
Every original instruction object is encoded with `JSON.stringify`.
Local round-trip assertions recover each original object,
and verify unchanged state,
criteria,
question membership,
and original provider-request text.
The corrected request is 692,952 bytes with the same 244 questions.
It retains the same five-second deadline and no-retry policy.
The completed three-call phase is neither overwritten nor relabeled.

The corrected request also returned 422,
with `too many questions (244); limit is 16`.
No serving model or usage was reported.
The public limits page advertises 512 questions.
This discrepancy excludes an immediate replacement on the captured native input;
it does not establish a universal 2.0 question limit or a model-quality result.
Splitting this request into groups of 16 would require 16 calls,
which exceeds the accepted three-call ceiling.

The official public runtime was cloned read-only at
`nace-ai/drex-decision-models` commit `be8ba7c8157241b9393862e9fcec104d0dc5caff`.
Its `kev/api.py:51` declares `questions: dict[str, Question] = Field(min_length=1)`.
The exact returned question-limit diagnostic is absent from that checkout.
The local runtime is therefore not evidence of the hosted guard's implementation.
Source and hosted-contract discrepancy investigation remains bounded to this failure.
Scores and sensitivity remain inapplicable before the serving-model hard gate.
Keep the incumbent route for the ongoing native integration.
Drex 2.0 is not adopted on this evidence.
A changed hosted contract or separately accepted assessment design can reopen this gate;
no further provider calls are queued.

### Ranking for the current delivery

1.  Keep the incumbent: preserves the current native wire and accepted call ceiling;
    it supplies no new quality claim and retains existing qualification requirements.
2.  Direct Drex 2.0 replacement: the exact-version control works,
    but the complete native request is rejected.
3.  Split Drex requests into batches of 16: preserves question membership in principle,
    but needs 16 calls and violates the accepted assessment ceiling.

Ranking: keep the incumbent > direct replacement > split batching,
because keeping the route avoids a demonstrated request failure,
and direct replacement preserves the accepted call budget whereas split batching does not.
The failed alternatives are not implementation choices currently available within the constraints.
