# Charm Hyper live catalog and model documentation disagree on attachment capability

## Symptom

Model selection can produce opposite answers depending on source:

- Hyper model documentation says attachment-capable examples include DeepSeek V4 Flash and GLM 5.1.
- Hyper `GET /v1/models` reports `capabilities.vision: false` for both.
- Repository allowlist contains 7 models while live endpoint currently returns 29.

Using prose examples or repository allowlist as current provider inventory can omit usable models or send page images to model that live API marks unsupported.
Hyper documentation says unsupported attachment returns error.

## Root cause

Hyper exposes machine-readable catalog meant to decide model id,
output ceiling,
and vision support:

```text
https://hyper.charm.land/docs/api/list-models.html
GET /v1/models
capabilities.vision: Whether the model supports vision (image attachments)
```

Same provider's general model page carries stale examples:

```text
https://hyper.charm.land/docs/models.html
Models that currently support attachments in the Hyper catalog include DeepSeek V4 Flash and GLM 5.1.
```

Live endpoint disagrees with that prose for both named models.
Machine-readable row is operational authority because attachment request handling reads current model capability,
while prose example is not versioned to one catalog snapshot.

Repository adds another deliberately static layer.
The 2026-08-30 source excerpt from
`package/translation-repair/shared-contract/src/hyper-catalog.ts:155-202`
records the then-current allowlist and capability facts.
Its historical `HYPER_ONLY` identifier became `NO_SYNTHETIC_COUNTERPART` on 2026-09-11:
it identifies absence of a Synthetic counterpart,
not Hyper-exclusive serving.

```ts
export const HYPER_MODELS: Readonly<Record<HyperServedId, HyperModelInfo>> = {
  // ...
  'minimax-m3': {
    id: 'minimax-m3',
    sharedWith: HYPER_ONLY,
    readsImages: true,
    maxOutputLength: 512_000,
  },
  // ...
  'deepseek-v4-flash-0731': {
    id: 'deepseek-v4-flash-0731',
    sharedWith: HYPER_ONLY,
    readsImages: false,
    maxOutputLength: 384_000,
  },
};
```

This file is an owner-controlled active allowlist,
not generated live mirror.
`package/translation-repair/provider/src/hyper-client.ts:271-279` refuses labels outside mapping before network
dispatch:

```ts
function servedIdFor(
  { modelId, }: { readonly modelId: RosterModelId; },
): HyperServedId {
  const spelling = hyperIdFor({ modelId, },);

  if (!spelling.served)
    throw new ModelNotServedError({ modelId, },);
  return spelling.id;
}
```

Therefore provider catalog expansion does not automatically expand production roster.
That is intentional safety behavior,
but it makes checked-in inventory stale unless operator explicitly compares it with live endpoint.

The cited searches identified no public Hyper service source.
No upstream source clone can trace how documentation and catalog are generated.
The observable API and published docs are primary deciding sources.

## Verification

Access dates:
2026-08-30 and 2026-09-01.

### Live endpoint

```bash
curl --silent --show-error --fail https://hyper.charm.land/v1/models \
  > "$HOME/temp/agent/hyper-models-E1-reserve-20260830.json"

jq '{count:(.data|length), vision:[.data[]|select(.capabilities.vision==true)|.id]}' \
  "$HOME/temp/agent/hyper-models-E1-reserve-20260830.json"
```

Observed:

- 29 model rows
- 11 rows with `capabilities.vision: true`
- `minimax-m3`:
  vision true,
  512,000 context,
  512,000 maximum output
- `deepseek-v4-flash` and `deepseek-v4-flash-0731`:
  vision false
- `glm-5.1`:
  vision false

The 2026-09-01 focused refresh ran at `2026-09-01T03:05:23.716Z`.
Its private canonical-row artifact is
`/var/home/user/temp/agent/hyper-glm53-transition-live-20260901.json`,
SHA-256 `577e3d7e2d668afba04132f86b7815ac1e427ac027037fb4ca2e24f79e996bc4`.
The file is mode `0600`.
The refresh command was:

```bash
curl --silent --show-error --fail https://hyper.charm.land/v1/models \
  | jq '.data[] | select(.id == "glm-5.2" or .id == "glm-5.3" or .id == "glm-5.3-flash")
    | {id, capabilities, max_output_tokens, reasoning}'
```

### Clean catalog patterns

Models whose checked-in and live capability agree:

- `minimax-m3`:
  vision true
- `qwen3.8-27b`:
  vision true
- `kimi-k3`:
  vision true
- `deepseek-v4-pro-0813`:
  vision false
- `deepseek-v4-flash-0731`:
  vision false

### Failing documentation pattern

Prose attachment examples name DeepSeek V4 Flash and GLM 5.1,
while live API marks both false.

### GLM 5.3 transition snapshot

A fresh `GET /v1/models` read on 2026-09-01 returned all three rows:

- `glm-5.2`:
  vision false,
  32,768 maximum output tokens,
  default reasoning `high`;
- `glm-5.3`:
  vision false,
  262,144 maximum output tokens,
  default reasoning `max`;
- `glm-5.3-flash`:
  vision true,
  131,072 maximum output tokens,
  default reasoning `max`.

The newly available full `glm-5.3` route therefore fails translation repair's mandatory-image hard gate.
Its larger output limit cannot offset absent image carriage.
`glm-5.3-flash` remains the only one of these live rows eligible for image-bound Candidate L verifier screening.
Its live default reasoning is `max`.
Candidate L will not override that default under owner policy,
so completion remains a calibration risk rather than a reason to choose image-ineligible full GLM 5.3.
No provider request was needed to reach that result.

An unverified user-provided notice said Hyper's GLM 5.2 would be deprecated shortly.
The inspected Hyper pages and live row contained no deprecation date,
so that notice is retained as unconfirmed timing information.
GLM 5.2 was already outside the active translation-repair roster and remains ineligible because the live row says vision
false.

### Static inventory drift

The 2026-08-30 checked-in `HYPER_MODELS` snapshot had 7 allowlisted rows.
That day's live provider endpoint had 29 rows.
The difference is not itself a bug because the allowlist is deliberate;
reading the allowlist as provider inventory is a bug.

### V4.1 Flash admission snapshot

The 2026-09-11 catalog reports `deepseek-v4.1-flash` with vision support
and a 26214-token maximum output.
The corresponding OpenRouter route also reports image input,
with a different output ceiling.
Both actual compiled text clients passed a streamed structured-response probe.
This establishes serving compatibility,
not image-reading quality.

Reader admission remains separately held pending its live image check.
A new model is not admitted to judge,
writer or reader roles merely because both providers serve it.
The existing reader-only Gemma 4 31B role is preserved.
See the
[admission record](../planning/translation-repair-deepseek-v41-flash-2026-09-11.md)
for the live responses and role-boundary comparison.

## Verified workarounds

### Screen with live machine-readable catalog

Before model evaluation,
fetch `GET /v1/models` and require:

- exact id present
- `capabilities.vision: true` when page image is mandatory
- output limit sufficient for role

Tradeoff:
live availability does not grant repository roster authority.
Model still needs explicit allowlisting,
consumer validation,
and decision record.

### Keep static refusal for unapproved labels

Continue refusing any live provider model absent current `RosterModelId` and `HYPER_MODELS`.
This prevents silent roster expansion and allows deterministic manifests.

Tradeoff:
new provider models remain unusable until reviewed and shipped.

### Bind validation evidence to fetched catalog snapshot

Save live JSON privately with access date and digest during model evaluation.
Record exact model row in vet report.

Tradeoff:
snapshot ages immediately and must be refreshed before later adoption or spend.

## What does not work

### Treating prose examples as capability list

Examples are stale for at least DeepSeek V4 Flash and GLM 5.1.
They cannot safely drive attachment routing.

### Treating checked-in allowlist as provider inventory

Allowlist omits provider rows by design.
It answers what project permits,
not what provider serves.

### Treating live listing as automatic approval

Presence and vision flag prove only provider routing surface.
They do not prove strict tool schema,
complete output,
translation quality,
or project authorization.

### Choosing full GLM 5.3 from output limit alone

Full `glm-5.3` has a larger listed output limit than Flash,
but its live row says vision false.
Every translation-repair node must receive every page-referenced image,
so the full route fails before output capacity is compared.

## Upstream filing decision

`.out-of-scope/` contains no Charm Hyper or provider-documentation exemption.
Web search found no public Hyper documentation issue tracker entry matching this contradiction.

1. Upstream fault:
   yes for stale prose example;
   no for repository allowlist drift,
   which is consumer policy.
2. Upstream can fix:
   yes,
   by generating attachment examples from live catalog or removing named examples.
3. Supported use case:
   yes,
   provider docs explicitly instruct checking attachment badge before sending images.
4. Contribution welcome:
   unknown.
   Hyper service documentation source and contribution policy were not found publicly.
5. Likely fix:
   unknown;
   no public tracker or maintainer signal was found.
6. Minimal compatible prototype:
   no.
   Documentation source is unavailable,
   so consumer cannot create source-compatible patch.

Constraints 4 through 6 do not pass.
Default is not to file.

### Draft issue, do not file as-is

~~~md
Title: Attachment examples disagree with `/v1/models` vision flags

The Models documentation currently names DeepSeek V4 Flash and GLM 5.1 as models that support attachments.
On 2026-08-30,
`GET https://hyper.charm.land/v1/models` reported `capabilities.vision: false` for both labels.

Could the named examples be generated from the live catalog or replaced with guidance that does not enumerate models?
The List Models documentation already identifies `capabilities.vision` as deciding field.

Reproduction:

```bash
curl --silent --show-error --fail https://hyper.charm.land/v1/models \
  | jq '.data[] | select(.id == "deepseek-v4-flash" or .id == "glm-5.1") | {id, capabilities}'
```

Expected: attachment examples and live catalog agree.
Observed: prose says supported;
live endpoint says vision false.
~~~
