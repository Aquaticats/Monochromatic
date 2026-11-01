# Candidate I Kimi verifier model and route vet

- Status:
  rejected;
  no recommendation.
- Lifecycle phase:
  no serious alternative survived hard gates.
- Subject:
  Candidate I Kimi verifier model and route.
- Decision scope:
  choose one Hyper-hosted vision-capable Kimi verifier and default-reasoning structured-output route for
  Candidate I.
- Started:
  2026-08-31.
- Last updated:
  2026-08-31.
- Governing skill commit:
  `a05818ad70a40e5769a36de669697ba109891b31`.
- Governing skill SHA-256:
  `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.
- Compatibility fingerprint:
  `576ae770137b000ed5e89109bb848c2a263b4f2348a44e7281eed2ff20d29012`.
- Active audit owner:
  Pi session `01a041b2-cb17-798b-a0ba-f71c9e0ffd51`.
- Prior compatible report:
  none.
  `doc/audit/tech-candidate-e-hyper-roster-expansion-vet-2026-08-30.md` has related evidence but different
  architecture and route scope.
- Duplicate compatible reports:
  none.

## Context and hard constraints

Candidate H requires one Kimi-family verifier so each Qwen or MiniMax candidate can receive two clean nonself ballots from
distinct model families.
Candidate H's Kimi K3 author and all-candidate verifier both reached 16,000 output tokens with `max_tokens` stop reason.
Candidate I reduces one verifier response to one candidate and replaces status arrays with exact compact status strings.

Owner's standing instruction in `package/module/translation-repair/src/anthropic-request.ts` (moved on 2026-10-18 to
`package/translation-repair/provider/src/anthropic-request.ts`) and
`package/module/translation-repair/src/chat-contract.ts` (moved on 2026-10-18 to
`package/translation-repair/provider/src/chat-contract.ts`) forbids non-default thinking,
reasoning-budget,
temperature,
or effort knobs because prior serving stacks returned errors,
degraded output,
or truncation.
A low-effort transport acceptance probe does not authorize changing that policy.
Every candidate in this vet therefore ran at provider and model defaults.

Repository route evidence at `package/module/translation-repair/src/hyper-catalog.ts:21-25` (moved on 2026-10-18 to
`package/translation-repair/shared-contract/src/hyper-catalog.ts`) already said Hyper OpenAI Chat
Completions accepts and ignores `response_format`.
The initial plan did not honor that deciding source and was wrong to advance OpenAI structured output to spend.
The equal-depth probe reproduced recorded failure and all Kimi alternatives exited.

Hard constraints:

- Hyper-only normal operation;
- vision-capable model from Kimi family;
- complete source,
  archive,
  and every page-referenced image in each request;
- complete compact status coverage;
- structured JSON with caller validation;
- measured complete candidate-scoped response within provider output cap;
- no retry or route fallback;
- default sampling and reasoning parameters;
- explicit output ceiling permitted.

Base category is managed service or SaaS.
Overlays are incumbent model-route replacement and sensitive data.
Deployment is Node.js on Linux x64 against Charm Hyper by Anthropic Messages or OpenAI Chat Completions.
Restricted translation corpus and images cross existing managed-service boundary.
Raw provider output remains quarantined and cannot enter publication without deterministic validation.

No unresolved preference controls discovery or hard gates.
Quality is standing constraint;
cost is not criterion.

### Fingerprint input

```json
{"schemaVersion":1,"subject":"Candidate I Kimi verifier model and route","decisionScope":"Choose one Hyper-hosted vision-capable Kimi verifier and default-reasoning structured-output route for Candidate I.","hardConstraints":["Hyper-only normal operation","complete compact status coverage","default sampling and reasoning parameters; explicit output ceiling permitted","distinct Kimi model family","measured complete candidate-scoped response within provider output cap","no retries","source archive and every page-referenced image in each request","structured JSON with caller validation","vision-capable"],"deployment":{"provider":"Charm Hyper","protocols":["Anthropic Messages","OpenAI Chat Completions"],"runtime":"Node.js on Linux x64"},"trustBoundary":"Restricted translation corpus and page-referenced images cross managed-service boundary; no raw provider output enters publication without caller validation.","incumbentName":"Charm Hyper Anthropic Messages with Kimi K3","incumbentVersion":null,"baseCategories":["managed service or SaaS"],"overlays":["incumbent model-route replacement","sensitive data"]}
```

## Discovery schedule and result

Frozen initial query schedule:

- official Hyper `GET /v1/models` complete live enumeration;
- official Moonshot model list and K2.6,
  K2.7 Code,
  K3,
  thinking,
  vision,
  tool-choice,
  JSON Mode,
  and structured-output pages;
- `site:platform.moonshot.ai/docs Kimi K2.6 vision structured output default thinking`;
- `site:platform.moonshot.ai/docs Kimi K2.7 Code vision structured output default thinking`;
- `site:platform.moonshot.ai/docs Kimi K3 vision structured output reasoning effort default max`;
- `site:hyper.charm.land/docs Kimi model vision reasoning structured output`;
- `Kimi K2.6 K2.7 K3 structured output vision comparison official`;
- `Kimi K3 alternative Kimi K2.6 vision structured output verifier`;
- repository incumbent,
  prior vet reports,
  current transport,
  and Candidate H evidence.

Initial searches used provider relevance order with no include or exclude filter.
Official-model searches returned ten results each.
Hyper query returned ten results.
Broader comparison and alternative queries returned ten results each.
The live Hyper endpoint is exhaustive and has no pagination.
It enumerated Kimi K2.5,
K2.6,
K2.7 Code,
and K3;
no other Kimi-family row was present.

New taxonomy terms were Moonshot Flavored JSON Schema (MFJS),
JSON Schema strict mode,
and reasoning-inclusive truncation.
Frozen one-round expansion schedule:

- `site:platform.moonshot.ai/docs Kimi K2.6 MFJS simple schema strict reliability`;
- `site:platform.moonshot.ai/docs Kimi K2.7 structured output translation evaluation`;
- `site:platform.moonshot.ai/docs Kimi K3 response_format max_tokens reasoning truncation`.

Expansion produced no new model id.
First query returned no result,
second returned one unrelated older-model page,
and third returned four official capability pages.
Discovery is saturated through exhaustive incumbent catalog plus official model-family documentation.

## Candidate ledger

### Kimi K2.5

- Discovery:
  exhaustive live Hyper catalog.
- Live row:
  vision false.
- Screening:
  hard-gate exit because every verifier must receive page-referenced images.

### Kimi K2.6 with Hyper OpenAI structured output

- Discovery:
  exhaustive live Hyper catalog and prior roster vet.
- Live row:
  vision true,
  262,000 context,
  26,214 maximum output,
  medium default reasoning.
- Official behavior:
  thinking enabled by default;
  OpenAI `response_format` supports JSON Schema strict mode;
  official structured-output page warns K2.6 can be unstable on complex MFJS schemas and requires caller validation.
- Model source:
  `https://platform.moonshot.ai/docs/guide/kimi-k2-6-quickstart`.
- Structured-output source:
  `https://platform.moonshot.ai/docs/guide/response_format`.
- Candidate I fit:
  status strings and bounded flat findings avoid `$ref`,
  `oneOf`,
  and deep nesting.
- Parameters:
  default reasoning and sampling;
  explicit `max_tokens` at live output ceiling.
- Runtime:
  HTTP 200 and `stop` after 140 completion tokens;
  366 content characters did not parse as JSON;
  exact guard failed.
- Screening:
  hard-gate exit because direct route did not return structured JSON.

### Kimi K2.7 Code with Hyper OpenAI structured output

- Discovery:
  exhaustive live Hyper catalog and prior roster vet.
- Live row:
  vision true,
  262,000 context,
  16,000 maximum output.
- Official behavior:
  thinking always enabled;
  official structured-output page documents K2.7 Code support for nested schemas and describes it as more stable than
  K2.6 for that interface.
- Model source:
  `https://platform.moonshot.ai/docs/guide/kimi-k2-7-code-quickstart`.
- Structured-output source:
  `https://platform.moonshot.ai/docs/guide/response_format`.
- Risk:
  code-specialized model has unproven translation-review fit,
  and output ceiling equals K3 ceiling exhausted by Candidate H.
- Parameters:
  default reasoning and sampling;
  explicit `max_tokens` at live output ceiling.
- Runtime:
  HTTP 200 and `stop` after 139 completion tokens;
  364 content characters did not parse as JSON;
  exact guard failed.
- Screening:
  hard-gate exit because direct route did not return structured JSON.

### Kimi K3 with Hyper OpenAI structured output

- Discovery:
  exhaustive live Hyper catalog and incumbent Candidate H.
- Live row:
  vision true,
  1,048,576 context,
  16,000 maximum output,
  max default reasoning.
- Official behavior:
  official structured-output page documents K3 support for nested objects,
  arrays,
  and `anyOf`.
- Model source:
  `https://platform.moonshot.ai/docs/guide/kimi-k3-quickstart`.
- Structured-output source:
  `https://platform.moonshot.ai/docs/guide/response_format`.
- Runtime counterevidence:
  Candidate H K3 author and all-candidate verifier each ended at `max_tokens` with no usable tool
  response.
- Scope difference:
  Candidate I asks for one candidate and compact strings,
  so H result does not prove candidate-scoped response fails.
- Parameters:
  default reasoning and sampling;
  explicit `max_tokens` at live output ceiling.
- Runtime:
  HTTP 200 and `stop` after 5,853 completion tokens and 17,139 reasoning characters;
  content parsed as JSON but failed exact caller guard.
- Screening:
  hard-gate exit because strict schema request did not produce exact required status semantics.

### Kimi K3 low-effort Anthropic tool route

- Discovery:
  bounded transport probe.
- Runtime:
  HTTP 200,
  forced tool accepted,
  96 output tokens,
  214 reasoning characters.
- Hard-gate exit:
  violates owner default-reasoning policy and does not prove effort was honored.

## Managed-service gates

Charm Hyper is incumbent service and common to every evaluated candidate.
`doc/audit/tech-candidate-e-hyper-roster-expansion-vet-2026-08-30.md` and existing provider decisions already inspect:

- layoffs and headcount;
- customer reviews;
- outages;
- funding,
  ownership,
  and business model;
- signup friction;
- security and abuse history;
- terms and pricing;
- suspension,
  appeal,
  and termination;
- privacy,
  data use,
  retention,
  deletion,
  and geography;
- availability,
  support,
  export,
  portability,
  lock-in,
  deprecation,
  and business continuity.

No candidate changes provider,
account,
credential,
data category,
retention path,
or geography.
Those findings are inherited common-mode evidence and do not order model ids.
Fresh live catalog and exact route behavior remain required because model capability can drift independently.

Open-source,
local executable,
native,
Wasm,
prebuilt,
human-auditability,
and multi-platform browser gates are not applicable:
selection changes one managed model row and provider protocol only.
No new local dependency or artifact is proposed.

## Frozen scoring criteria

Hard gates stay outside arithmetic.
Weights were frozen before candidate-specific runtime validation:

- structured-output reliability:
  weight 5;
- verifier fidelity and English-quality fit:
  weight 5;
- complete-response headroom:
  weight 4;
- operational simplicity within incumbent provider:
  weight 3;
- end-to-end latency:
  weight 1.

Ratings use zero through four:
zero serious concern,
one weak,
two acceptable,
three good,
and four strong.
Maximum score is
`(5 + 5 + 4 + 3 + 1) * 4 = 72`.
No rating is assigned because every candidate failed hard gate before quality scoring.
Sensitivity cannot change hard-gate outcome.

## Executed manifest

Candidate versions were exact live ids `kimi-k2.6`,
`kimi-k2.7-code`,
and `kimi-k3`.
Fresh catalog digest matched pre-spend snapshot:
`d68a71501ffa7c67825467ef935c177ad0abb01560f81f17c3fe65fe88742ab7`.

Inspected Node scratch runner used built-in `fetch` only.
Script SHA-256 was
`223f6cb6d17d39af6ff5e40b8e715d7a4a037e82a5656f138bde833e39d5795c`.
It sent one payload per model concurrently to Hyper OpenAI Chat Completions with:

- same synthetic 1 by 1 image;
- same compact Candidate I-shaped JSON Schema;
- same non-corpus calibration prompt;
- model-default reasoning;
- each model's live output ceiling;
- streaming enabled;
- no retry.

Network endpoints were public `GET /v1/models` and authenticated `POST /v1/chat/completions` only.
Writes were private metadata-only files under
`~/temp/agent/probe-hyper-kimi-candidate-i-json-schema-20260831/`.
No raw response,
credential,
corpus text,
or repository image was retained.
Summary SHA-256 is
`f2fd14972da243503e7078197965ecd87d7ee98715d5549adad783bff0cc6ee1`.
Exactly three attempt rows settled and no redispatch occurred.

Run was on host because credential injection is scoped to project worktree and runner used only inspected standard-library
code plus direct HTTPS.
It started no subprocess and wrote no repository file.
Each HTTP request had 360,000-millisecond abort signal.

All three responses returned HTTP 200 with terminal `stop`.
None passed caller guard.
This synthetic image run proves route failure only;
it does not satisfy Candidate I corpus-image or translation-review constraint and does not justify Carena spend.
No Kimi model advances to consumer validation.

## Evidence records

### Live Hyper inventory

- Candidate:
  every ledger row.
- Claim:
  exact hosted presence,
  vision,
  context,
  output,
  and default reasoning metadata.
- Gate:
  hard constraint and operational envelope.
- Status:
  K2.5 fails vision;
  K2.6,
  K2.7 Code,
  and K3 pass static screening.
- Primary source:
  `https://hyper.charm.land/v1/models`,
  accessed 2026-08-31.
- Private snapshot:
  `~/temp/agent/hyper-models-candidate-i-kimi-vet-20260831.json`,
  17,212 bytes,
  SHA-256 `d68a71501ffa7c67825467ef935c177ad0abb01560f81f17c3fe65fe88742ab7`.
- Corroboration:
  official Moonshot model docs.
- Outcome:
  finite Kimi roster enumerated.

### Structured-output compatibility

- Candidate:
  K2.6,
  K2.7 Code,
  and K3.
- Claim:
  direct OpenAI route can request strict JSON Schema at model defaults.
- Gate:
  hard structured-output boundary.
- Status:
  runtime hard-gate failure for all three models.
- Provider documentation:
  `https://hyper.charm.land/docs/api/openai-chat-completions.html`,
  accessed 2026-08-31,
  says standard OpenAI parameters are accepted.
- Model documentation:
  `https://platform.moonshot.ai/docs/guide/response_format`,
  accessed 2026-08-31,
  documents model-side capability.
- Deciding repository evidence:
  `package/module/translation-repair/src/hyper-catalog.ts:21-25` (moved on 2026-10-18 to
  `package/translation-repair/shared-contract/src/hyper-catalog.ts`) records that Hyper accepts and ignores
  `response_format`.
- Runtime evidence:
  `~/temp/agent/probe-hyper-kimi-candidate-i-json-schema-20260831/summary.json`.
- Outcome:
  no Kimi OpenAI route advanced.

### Default reasoning and sampling policy

- Candidate:
  every survivor.
- Claim:
  no non-default thinking,
  reasoning budget,
  effort,
  temperature,
  or other sampling parameter may be sent;
  output token ceiling remains explicit transport bound.
- Gate:
  hard repository policy.
- Status:
  pass by planned omission.
- Primary source:
  `package/module/translation-repair/src/anthropic-request.ts` (moved on 2026-10-18 to
  `package/translation-repair/provider/src/anthropic-request.ts`) and
  `package/module/translation-repair/src/chat-contract.ts` (moved on 2026-10-18 to
  `package/translation-repair/provider/src/chat-contract.ts`).
- Counterevidence:
  low-effort K3 probe established transport acceptance only.
- Outcome:
  low-effort route excluded.

### Candidate H K3 truncation

- Candidate:
  K3.
- Claim:
  current all-candidate shape did not fit K3 default 16,000-token completion.
- Gate:
  scored headroom concern,
  not candidate-scoped hard failure.
- Status:
  observed twice.
- Evidence:
  `~/temp/agent/prototype-Carena-H-bounded-verdict-20260831/calibration-summary.json`.
- Outcome:
  K3 candidate-scoped OpenAI route also failed exact guard before Carena validation.

## Current disposition

No recommendation.
K2.5 failed vision gate.
K2.6 and K2.7 Code returned non-JSON content despite strict schema request.
K3 returned parseable JSON but failed exact caller guard.
Low-effort K3 tool route remains prohibited by owner policy.
No Kimi model and route met Candidate I hard constraints,
so this focused vet closes rejected before scoring or pinned-Carena spend.
A replacement architecture needs a different third model family or different evidence floor;
it must not silently adopt OpenAI `response_format` on Hyper.
