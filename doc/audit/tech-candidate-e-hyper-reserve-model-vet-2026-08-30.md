# Candidate E Hyper reserve model vet report

- Status:
  completed without recommendation
- Lifecycle phase:
  rejected at hard gate
- Subject:
  Candidate E Hyper reserve model
- Scope:
  select third distinct vision-capable author and auditor model identity for Candidate E Hyper-only operation
- Started:
  2026-08-30
- Last updated:
  2026-08-30
- Governing skill revision:
  `c8475388d533e4f1c4065682ed2437a29cad3dc1`
- Governing skill SHA-256:
  `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`
- Compatibility fingerprint:
  `14a0cad284fcd75ccc038d31083bd2c3935ddb659c0f21aa1b4b87becd555f22`
- Active audit owner:
  primary agent session
- Prior compatible report:
  none found

## Context

Candidate E Hyper-only run used Qwen3.8-27B and Kimi-K3 as shared Hyper seats.
Kimi author exhausted its 16,000-token provider ceiling without answer content,
and GLM-5.3 had no Hyper mapping.
Only Qwen produced complete candidate,
so revised comparative floor now correctly makes this posture ineligible as architecture evidence.

This evaluation extends existing Charm Hyper roster.
It does not select new provider,
change provider terms,
or change restricted-corpus trust boundary.
Every candidate is managed through same already adopted Hyper API.

## Hard constraints

- Served by current Charm Hyper provider.
- Model identity differs from Qwen3.8-27B and Kimi-K3.
- Reads every page-referenced image sent with source.
- Conforms to strict fixed-key JSON author and auditor schemas.
- Completes pinned 23-slot Carena author contract within provider output envelope.
- Produces exact source and candidate anchors for quote-bound audit.
- Needs no provider other than Charm Hyper.
- Introduces no new vendor.

## Base category and overlays

Base category is managed service or SaaS.
Sensitive-data overlay applies because unpublished restricted corpus text and page images cross existing Hyper boundary.
Incumbent-extension overlay applies because this changes model roster within existing provider.
Open-source precedence is not applicable:
no local component or provider replacement is being selected.
Native,
Wasm,
prebuilt,
multi-platform,
and browser overlays are not applicable.

## Compatibility fingerprint input

```json
{"baseCategories":["managed service or SaaS"],"decisionScope":"Select a third distinct vision-capable author and auditor model identity for Candidate E Hyper-only operation.","deployment":{"corpusPin":"a80634a674f94861ea3b7056fba054ca9eab1a2c","provider":"Charm Hyper","role":["complete immutable-shell author","quote-bound comparative auditor"],"runtime":"Node.js"},"hardConstraints":["Audits candidates through strict fixed-key JSON schema with exact source and candidate anchors.","Completes the pinned 23-slot Carena author contract within provider output envelope.","Has a model identity distinct from Qwen3.8-27B and Kimi-K3.","Needs no provider other than current Charm Hyper service.","Reads every page-referenced image sent with the source.","Uses the already adopted provider rather than introducing another vendor."],"incumbent":{"name":"Hyper-only Qwen3.8-27B and Kimi-K3 two-seat roster","version":null},"overlays":["sensitive data","incumbent roster extension"],"schemaVersion":1,"subject":"Candidate E Hyper reserve model","trustBoundary":"Unpublished restricted corpus text and page-referenced images cross the existing Charm Hyper API boundary."}
```

## Frozen discovery schedule

### Provider and ecosystem index

- Current checked-in Hyper catalog:
  enumerate every `HYPER_MODELS` row,
  without negative filter,
  in catalog order.
- Current provider catalog evidence:
  verify listed model,
  image capability,
  output ceiling,
  and shared identity.

### Repository and parallel systems

- `rg --line-number 'minimax-m3|deepseek-v4|gemma-4|gpt-oss-120b|qwen3.8-27b|kimi-k3' .`
- Read `package/module/translation-repair/src/hyper-catalog.ts` (moved on 2026-10-18 to
  `package/translation-repair/shared-contract/src/hyper-catalog.ts`) and its unit tests.
- Read prior provider probes,
  architecture comparison,
  and Hyper-only run evidence.

### Official and broader web

- `MiniMax M3 official model vision structured output context length`
- `MiniMax M3 API tool use JSON schema official`
- `Charm Hyper MiniMax M3 model catalog vision`
- `MiniMax M3 alternative vision language model API`

### One expansion round

Append at most one de-duplicated round for new taxonomy terms found in initial sources,
then freeze permanently.
Likely terms are forced tool choice,
reasoning-token budget,
and Anthropic Messages image input.

## Query ledger

Initial web search ran every frozen query on 2026-08-30.
Provider returned 10 results for each web query page;
all newly discovered serious candidates were then checked against primary sources.

- `MiniMax M3 official model vision structured output context length`:
  official MiniMax model page,
  GitHub model repository,
  and framework docs discovered.
- `MiniMax M3 API tool use JSON schema official`:
  official MiniMax Tool Use guide and API references discovered.
- `Charm Hyper MiniMax M3 model catalog vision`:
  official Hyper model and list-model endpoint docs discovered.
- `MiniMax M3 alternative vision language model API`:
  official MiniMax model page and peer gateway pages discovered.

One expansion round then ran and froze discovery permanently:

- `Kimi K2.6 official vision multimodal model output context`
- `Qwen 3.7 official vision model flash plus multimodal`
- `Hyper Anthropic Messages forced tool choice image attachment structured output`
- `MiniMax M3 forced tool choice strict JSON schema Anthropic`

Official Hyper `GET /v1/models` returned 29 rows.
Direct saved response at `~/temp/agent/hyper-models-E1-reserve-20260830.json` listed 11 vision-capable rows.
Checked-in project catalog is an allowlist of 7 rows rather than live-catalog mirror.
No pagination applies to provider endpoint or repository source.
Discovery is saturated for current provider and active project roster.

## Candidate ledger

### `qwen3.8-27b`

- Discovery:
  current Hyper catalog.
- Screening:
  exits.
- Hard-gate failure:
  same model identity as current Qwen primary,
  so cannot restore candidate or voter diversity.
- Existing evidence:
  complete Hyper author output and strict audit output succeeded.

### `kimi-k3`

- Discovery:
  current Hyper catalog.
- Screening:
  exits as reserve.
- Hard-gate failures:
  same identity as current Kimi seat;
  Hyper author consumed full 16,000-token ceiling without answer content.
- Existing evidence:
  audit and resolver outputs can conform on smaller response envelopes.

### `minimax-m3`

- Discovery:
  checked-in Hyper allowlist and live provider catalog.
- Screening:
  exits after targeted validation.
- Static evidence:
  active Hyper-only roster identity,
  live image capability true,
  provider maximum output 512,000 tokens,
  with project answer ceiling 32,000 tokens.
- Official MiniMax model page and repository describe native multimodality and 1M upstream context.
- Official Tool Use guide documents Anthropic-compatible tool input schemas and interleaved reasoning.
- Runtime audit response passed strict structural schema and exact-anchor admission.
- Runtime complete-author response consumed full 32,000-token project ceiling,
  ended `max_tokens`,
  and was unparseable JSON.
- Hard-gate failure:
  did not return complete fixed-key author response within actual project envelope.

### `gpt-oss-120b`

- Discovery:
  current Hyper catalog.
- Screening:
  exits.
- Hard-gate failure:
  image capability false.
- Additional concern:
  output ceiling 13,107 tokens is below measured failed Kimi author ceiling.

### `gemma-4-26b-a4b-it`

- Discovery:
  current Hyper catalog.
- Screening:
  exits.
- Hard-gate failure:
  image capability false.

### `deepseek-v4-pro-0813`

- Discovery:
  current Hyper catalog.
- Screening:
  exits.
- Hard-gate failure:
  image capability false.

### `deepseek-v4-flash-0731`

- Discovery:
  current Hyper catalog.
- Screening:
  exits.
- Hard-gate failure:
  live `capabilities.vision` is false.
- Width-64 structured-shape success does not satisfy image hard gate.

### Live vision rows outside active roster

Live provider catalog also lists:

- `kimi-k2.6`
- `kimi-k2.7-code`
- `qwen3.6-flash`
- `qwen3.6-plus`
- `qwen3.7-flash`
- `qwen3.7-plus`
- `qwen3.8-flash`
- `qwen3.8-max`

They exit this bounded evaluation because none is current `RosterModelId` or checked-in Hyper allowlist row.
Selecting one would first require separate roster-expansion evaluation and owner allowlisting,
not reserve-seat selection among existing integrated models.
`qwen3.8-max` also remains excluded by prior owner decision and unreachable-run evidence.
This exit is compatibility scope,
not quality judgment.

### Provider documentation contradiction

Hyper model prose says attachment-capable examples include DeepSeek V4 Flash and GLM 5.1.
Same provider's live `GET /v1/models` reports `capabilities.vision: false` for both.
Live machine-readable endpoint controls screening because API docs instruct clients to use that field before attachments.
The contradiction is recorded separately in troubleshooting documentation.

## Managed-service gates

Charm Hyper is incumbent provider and unchanged trust boundary.
Provider terms,
account enforcement,
security,
privacy,
retention,
availability,
export,
lock-in,
ownership,
and business continuity are not candidate differentiators in this roster-only decision.
They remain inherited risks from existing provider decision and receive no model score effect.
Underlying model capability,
schema behavior,
output envelope,
and translation quality are directly applicable and require fresh validation.

## Frozen scoring criteria

Cost receives no criterion because standing quality rule makes cost non-constraint.
Hard gates stay outside score.

- Publication fidelity and English quality:
  weight 5.
- Strict schema reliability:
  weight 5.
- Complete-document output capacity:
  weight 4.
- Concrete audit-defect recall:
  weight 3.
- End-to-end latency:
  weight 1.

Ratings use 0 through 4 only after every finalist receives same author,
auditor,
and consumer-boundary validation.
Sensitivity will vary each weight from 1 through 5 and every medium or low-confidence rating by 1 point.

## Execution manifest

Validation runner is committed as `3f160d740`.
It uses existing inspected project client and task runner,
not third-party local code.
`hyper-client.ts` builds Anthropic Messages request,
forces schema tool,
streams response,
and applies provider-neutral JSON guard.
It sent 2 concurrent zero-retry Hyper payloads for only surviving finalist:

- complete 23-slot immutable-shell author with full source,
  archive,
  and page-referenced `photo1.webp`;
- strict quote-bound auditor over retained complete candidates with same evidence.

Network endpoint was existing Charm Hyper Messages API.
Local writes were fresh private prototype root,
prompt-keyed reply artifacts,
node records,
admission record,
and result.
No candidate was compiled because author failed strict admission.
No production corpus,
main worktree output,
or credential file was modified.
Stop condition was schema failure after both independent payloads settled.
No retry or alternate model was dispatched.

Manifest digest `0f85c5852b4603ada93e4e27b3425a0eb71d7929f9254add7b864ddf79d6175d`
binds corpus,
source,
archive,
shell,
`photo1.webp`,
retained candidate digests,
model identities,
Hyper-only routing,
32,000 requested output tokens,
2-payload ceiling,
1 dependency wave,
and zero retries.

## Evidence records

### Official MiniMax capability

- Candidate:
  MiniMax M3 through existing Hyper service.
- Claim:
  native multimodality and long context fit complete source plus image.
- Status:
  documentation support confirmed;
  provider-bound author validation failed.
- Primary sources:
  `https://www.minimax.io/models/text/m3` and
  `https://github.com/MiniMax-AI/MiniMax-M3`,
  accessed 2026-08-30.
- Evidence:
  both describe native multimodal model;
  official page states 1M upstream context with guaranteed minimum 512K.
- Outcome:
  pass documentation gate;
  complete-author runtime later failed.

### Official tool schema capability

- Candidate:
  MiniMax M3.
- Claim:
  accepts Anthropic-compatible tool schemas.
- Status:
  Hyper gateway validation completed.
- Primary source:
  `https://platform.minimax.io/docs/guides/text-m3-function-call`,
  accessed 2026-08-30.
- Evidence:
  guide shows Anthropic `tools[].input_schema`,
  tool-use blocks,
  and complete reasoning preservation.
- Outcome:
  pass documentation gate;
  forced-tool auditor passed while complete author failed.

### Hyper live model capability

- Candidate:
  every current Hyper model.
- Claim:
  model id,
  output ceiling,
  and image capability.
- Status:
  primary operational evidence.
- Primary source:
  `https://hyper.charm.land/v1/models`,
  accessed and saved 2026-08-30.
- Command:
  `curl --silent --show-error --fail https://hyper.charm.land/v1/models`.
- Result:
  29 rows,
  11 with `capabilities.vision: true`;
  `minimax-m3` reports 512,000 context and output tokens.
- Outcome:
  MiniMax passes provider static gate;
  current active-roster peers fail identity or image gates.

### MiniMax targeted validation

- Candidate:
  `minimax-m3`.
- Claim:
  same model can fill complete immutable-shell author and quote-bound auditor seats.
- Evidence type:
  direct provider and consumer-boundary validation.
- Date:
  2026-08-30.
- Preflight:
  fresh live catalog still reported vision true and output at least 32,000;
  provider budget sample reported Hyper wet with balance 7,055.
- Command:
  `mise run //package/module/translation-repair:prototype-hyper-reserve-evaluation` (task removed before
  2026-10-18 and tasks now live in `package/translation-repair/<package>/mise.toml`).
- Build evidence:
  package `buildAndTest` passed 871 suite verdicts with no failures before spend.
- Independent review:
  advisor found no pre-commit or pre-spend blocker and required fresh catalog plus budget preflight,
  both performed.
- Actual dispatch:
  2 Hyper `minimax-m3` payloads and zero retries.
- Provider evidence:
  manifest-bound `providerSelection: hyper-only` used routing branch whose isolation was previously GFP-proven;
  no Synthetic route was eligible.
- Duration:
  154,875 milliseconds.
- Private run root:
  `~/temp/agent/prototype-Carena-minimax-reserve-evaluation-20260830/`.
- Preserved private evidence:
  `~/Downloads/Carena0442-minimax-reserve-validation-rejected-20260830/`.
- Preservation check:
  all 8 files byte-identical by relative-path SHA-256 lists.

Author record:

- prompt tokens:
  14,407
- completion tokens:
  32,000
- finish reason:
  `max_tokens`
- response text characters:
  19,650
- reasoning stream characters:
  119,487
- all 23 slot keys appeared,
  but provider output lacked final outer JSON brace
- persisted state:
  `spent-unusable`
- failure type:
  `schema-mismatch`
- failure detail:
  `unparseable-json`

Appending missing byte would parse output,
but caller repair is forbidden:
producer must return complete candidate itself or have no effect.
Schema-rejected output is operational evidence only and was not read or scored as page quality.

Auditor record:

- prompt tokens:
  38,925
- completion tokens:
  20,917
- finish reason:
  `tool_use`
- strict structural response completed
- exact-anchor admission retained 4 findings across 3 candidates
- duplicate-key pruning rejected 1 additional finding
- admitted examples included omitted water support in slot `s4` and unsupported language-version detail in slot `s21`
- response located only subset of defects already established by complete D1 reading

Audit proves schema and quote binding,
but cannot compensate for failed complete-author hard gate.

Inherited guard coverage:
structural slot membership,
source and candidate anchor binding,
duplicate pruning,
source echo,
presentation artifacts,
prompt uniqueness,
and provider isolation were already GFP-proven in Candidate E controls.
Runner introduced wiring only,
not new admission logic.

## Scoring and sensitivity

No weighted score is calculated.
MiniMax failed complete-document hard gate before scoring,
and every other candidate exited earlier hard gate.
Sensitivity cannot change hard-gate outcome.

## Ranking

Recommendation:
none.

No current active Hyper roster model distinct from Qwen3.8-27B and Kimi-K3 satisfies every required author and auditor gate under tested request contract.
MiniMax ranks first among rejected reserve candidates because it alone passed identity,
vision,
and strict-auditor gates;
it still cannot be adopted because complete author failed.
All other active candidates fail identity or image gate before targeted validation.

Ranking:
rejected MiniMax M3 > rejected non-vision active rows,
because MiniMax reached both runtime roles and passed auditor contract while non-vision models cannot receive mandatory page evidence.
Incumbent Qwen and Kimi are not reserve alternatives because model identity is not distinct.

## Resolved questions

- Hyper accepted image-bearing MiniMax request and returned strict auditor schema;
  text-only findings do not prove image attention.
- MiniMax did not finish complete author JSON within 32,000 requested output tokens.
- Provider advertises higher ceiling,
  but one-model plus one-canonical-prompt uniqueness forbids redispatching same author prompt with altered ceiling.
  Any future run requires distinct substantive contract and new manifest,
  not retry of this validation.
- Author quality is not evaluated because schema-invalid response is ineligible quality evidence.
- Auditor returned exact anchors with one deterministic duplicate pruned,
  but recalled only subset of known D1 defects.
- MiniMax must not receive author,
  auditor,
  or resolver seat in Candidate E production roster under tested contract.
- Evaluating live models outside active roster requires separate roster-expansion decision and new compatibility fingerprint.
