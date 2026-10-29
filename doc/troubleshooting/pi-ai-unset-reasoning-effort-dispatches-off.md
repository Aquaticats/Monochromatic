# pi-ai 1.3.0 with dsh-llm-pi-ai 0.2.1-rc.2: an unset reasoning effort dispatches the off wire and silently disables thinking

## Symptom

A session on a NeuralWatt reasoning model (`mimo-v2.6-pro`) shows **no thinking
blocks at all**,
across every step,
while the thinking level picker sits at its
untouched `Default` item.
Nothing errors.
The user report:

> I find it difficult to believe there's no thinking whatsoever in this task.
> Are we not retrieving or displaying thinking blocks correctly?

The trigger patterns and what each produces,
measured through a capture proxy
against `https://api.neuralwatt.com`:

- thinking level `Default` (no level chosen):
  the request carries
  `"reasoning_effort": "none"`,
  the response streams no `delta.reasoning`,
  `completion_tokens_details.reasoning_tokens` is `0`,
  and the session log
  records zero `reasoning` blocks.
- thinking level `Off`:
  byte-identical dispatch,
  `"reasoning_effort": "none"`.
- thinking level `High`:
  the request carries `"reasoning_effort": "high"`,
  the
  response streams `delta.reasoning`,
  and the log records `reasoning` blocks.

So `Default` behaves exactly like `Off`,
and only an explicitly chosen thinking
level produces thinking.
The listing for `mimo-v2.6-pro` says the opposite
default:
`"default_enabled": true`,
`"default_effort": "high"`.

Retrieval and display are healthy;
only the request dispatch is wrong.
The same
GUI renders thinking as `Think` rows (93 visible in one long session),
and one
session log held 464 `reasoning` blocks across `neuralwatt/mimo-v2.6-pro` and
`opencode-go/mimo-v2.6-pro` before this mapping existed.

## Root cause

The chain,
walked from the picker to the wire.

### Step 1: the UI default resolves to no effort, and the route fills nothing

`dsh-llm` substitutes the model's advertised default effort only when the route
declares one.
`dsh-llm/lib/index.js:2188` (installed `@deepseek-ai/dsh-llm`):

```js
const reasoning = info.reasoning;
const requested = defaulted.reasoningEffort;
...
const effective = requested ?? reasoning.defaultEffort;
```

`reasoning.defaultEffort` comes from `describableReasoningLevel(model,
profile.reasoning)` in `dsh-llm-pi-ai/lib/index.js:1767`,
and the provider
profile declares no `reasoning` field,
so `defaultEffort` is `undefined`.
The
request-level effort therefore stays unset.
(Proof:
fixture session headers show
`"reasoningEffort":"high"` only when a level is set;
the failing session's header
carries none.)

### Step 2: the adapter collapses `off` and unset into one state

`dsh-llm-pi-ai/lib/index.js:1754` (`profileOptions`):

```js
function profileOptions(profile, reasoning, apiKey) {
	const enabledReasoning = reasoning === "off" ? void 0 : reasoning;
	return {
		...apiKey === void 0 ? {} : { apiKey },
		...enabledReasoning === void 0 ? {} : { reasoning: enabledReasoning },
```

An explicit `off` and an unset level both reach pi-ai as "no reasoning option".

### Step 3: pi-ai reads "no reasoning option" as "ask for the off wire"

`@earendil-works/pi-ai` `dist/api/openai-completions.js:527` first re-deletes
`off`:

```js
const clampedReasoning = options?.reasoning ? clampThinkingLevel(model, options.reasoning) : undefined;
const reasoningEffort = clampedReasoning === "off" ? undefined : clampedReasoning;
```

Then `dist/api/openai-completions.js:720` dispatches the model's off spelling
whenever no effort survived:

```js
} else if (options?.reasoningEffort && model.reasoning && compat.supportsReasoningEffort) {
    // OpenAI-style reasoning_effort
    params.reasoning_effort = model.thinkingLevelMap?.[options.reasoningEffort] ?? options.reasoningEffort;
} else if (!options?.reasoningEffort && model.reasoning && compat.supportsReasoningEffort) {
    const offValue = model.thinkingLevelMap?.off;
    if (typeof offValue === "string") {
        params.reasoning_effort = offValue;
    }
}
```

The mapped profile pins `thinkingLevelMap.off` to the listing's `"none"`
spelling,
so the empty selection dispatches `"reasoning_effort": "none"`.

### Step 4: the listing says the model thinks by default

`GET https://api.neuralwatt.com/v1/models` entry `mimo-v2.6-pro`,
`metadata`:

```json
"reasoning": {
  "mandatory": false,
  "default_enabled": true,
  "supported_efforts": ["high", "none"],
  "default_effort": "high",
  "accepted_efforts": ["max", "xhigh", "high", "medium", "low", "minimal", "none"],
  "effort_aliases": { "max": "high", "xhigh": "high", "medium": "high", "low": "high", "minimal": "none" }
}
```

Naming `"none"` explicitly turns off the thinking the model does by default,
so
the empty selection must not name it.

### Why the old hand-written route thought

The pre-plugin hand-written route declared no `reasoningEfforts`,
so
`resolveModelReasoning` (`dsh-llm-pi-ai/lib/index.js:583`) returned
`{ reasoning: base?.reasoning ?? false }`,
`model.reasoning` stayed false,
and
neither dispatch branch fired.
The request carried no reasoning parameter at all
and the server ran its default (`high`).
That route's thinking was accidental:
it had no thinking-level control.

### Earlier readings that were wrong

- The behavior note recorded with the plugin's first cut ("`off: "none"`
  dispatches `reasoning_effort: "none"` when thinking is off") was incomplete:
  the same branch fires when **no** level is chosen,
  which is the UI's
  `Default`.
  The distinction is lost in `profileOptions` before pi-ai sees it.
- `dsh-llm-pi-ai` `resolveModelReasoning` documents a declared-empty `off` as
  "supported,
  send nothing" (`dsh-llm-pi-ai/lib/index.js:575`),
  but its mapping
  leaves the key **absent**,
  and pi-ai's OpenRouter branch reads an absent key
  as `?? "none"` (`dist/api/openai-completions.js:691`).
  The doc comment and the
  emitted map shape disagree.
- The `session/title-llm-request` event's `"reasoningEffort":"off"` looked like
  the smoking gun in one session log.
  It is the title path's own override,
  not
  the agent turn's;
  the agent requests carry no `reasoningEffort` at all.

### Where the intent contradicts the dispatch

`dsh-agent/lib/index.js:150` documents the intended semantics:

> An absent selected effort clears any inherited effort,
> restoring the selected
> model's provider/default behavior.

pi-ai's empty-selection branch does the opposite:
it actively names the off
wire.

## Verification

Versions under test:
`@earendil-works/pi-ai` 1.3.0 (installed) and
`earendil-works/pi` commit `d5b3de2` (prototype clone,
`@earendil-works/pi-ai`
1.4.0 source);
`@deepseek-ai/dsh-llm-pi-ai` 0.2.1-rc.2;
`@deepseek-ai/dsh-llm` 0.2.1-rc.2.

### Harness A: end to end through the real CLI

Fixture profile from the headless template with the mapped route's exact model
entry and a capture proxy as `baseURL`.
`think-default.patch.yml`:

~~~yaml
- id: credentials
  name: "@deepseek-ai/dsh-credentials-local"
  disabled: false
- id: llm-pi-ai
  name: "@deepseek-ai/dsh-llm-pi-ai"
  config:
    providers:
      neuralwatt-probe:
        apiKeyEnv: NEURALWATT_API_KEY
        api: openai-completions
        baseURL: http://127.0.0.1:8978/v1/
        compat:
          supportsReasoningEffort: true
          supportsDeveloperRole: false
        models:
          - id: mimo-v2.6-pro
            name: MiMo-V2.6-Pro
            contextWindow: 1048560
            maxTokens: 393216
            input:
              - text
              - image
            reasoningEfforts:
              high: high
              'off': none
- id: agent-default-model
  name: "@deepseek-ai/dsh-agent-default-model"
  config:
    provider: neuralwatt-probe
    model: mimo-v2.6-pro
~~~

The capture proxy forwards to `https://api.neuralwatt.com` and appends each
request body and response body to a log.
The three arms run one identical turn:

~~~sh
dsh --profile thinkfix --from-default-profile headless \
  --patch think-default.patch.yml --json "What is 17 * 23? Reply with only the number."
~~~

Arm variants change only `agent-default-model`'s `reasoningEffort` (`high`,
or
the key absent,
or `off`) or the model's `reasoningEfforts` dict.

#### Catalog: failing patterns

- `Default` (no `reasoningEffort`):
  request body keys
  `model, messages, stream, stream_options, store, max_completion_tokens, tools,
  reasoning_effort` with `"reasoning_effort":"none"`;
  response has no
  `delta.reasoning` and `"reasoning_tokens": 0`;
  `--json` output has
  `{"type":"text","text":"391"}` and no `thinking` event.
- `reasoningEffort: off`:
  same wire,
  same output.

#### Catalog: working patterns

- `reasoningEffort: high`:
  `"reasoning_effort":"high"`;
  response streams
  `"delta": {"reasoning": "The user asks"...}` with `"reasoning_tokens": 23`;
  `--json` output carries `{"type":"thinking","text":"The user asks: 17 * 23 =
  391. Reply with only the number."}`.
- The same `Default` arm with the mapping below (no `off` entry):
  the request
  body has **no** `reasoning_effort` key,
  the response streams `delta.reasoning`
  with `"reasoning_tokens": 22`,
  and the `thinking` event appears.

Session headers confirm the arm input:
the `high` arm's
`request/header` config is `{"provider":"neuralwatt-probe","model":"mimo-v2.6-pro",
"reasoningEffort":"high","maxTokens":393216}` and the `Default` arm's omits
`reasoningEffort`.

### Harness B: upstream dispatch probe

`probe-dispatch.ts` drives `streamSimple` from the clone's own build against a
local capture server and prints the reasoning directive each request body
carries.
Model descriptors mirror the mapped profile (`thinkingLevelMap:
{ off: "none", high: "high", ... }`);
one variant pins `off: null`
(cannot-disable),
one sets `compat.thinkingFormat: "deepseek"`.
Seven cases per
run.

~~~ts
// node probe-dispatch.ts <dist-root> <out.json>; capture server on :8979
const server = createServer((req, res) => {
  const chunks = [];
  req.on('data', (chunk) => chunks.push(chunk));
  req.on('end', () => {
    captured.push(JSON.parse(Buffer.concat(chunks).toString('utf8')));
    res.writeHead(200, { 'content-type': 'text/event-stream' });
    res.end('data: {"id":"p","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"ok"},"finish_reason":"stop"}]}\n\ndata: [DONE]\n\n');
  });
});
const base = {
  name: 'probe', api: 'openai-completions', provider: 'openai',
  baseUrl: 'http://127.0.0.1:8979/v1', input: ['text'],
  cost: { input: 0, output: 0 }, reasoning: true,
  contextWindow: 1048560, maxTokens: 393216,
};
const models = {
  openai: { ...base, id: 'openai-shape',
    thinkingLevelMap: { off: 'none', high: 'high', minimal: null, low: null, medium: null, xhigh: null, max: null },
    compat: { supportsReasoningEffort: true, supportsDeveloperRole: false } },
  'cannot-disable': { ...base, id: 'cannot-disable',
    thinkingLevelMap: { off: null, high: 'high', minimal: null, low: null, medium: null, xhigh: null, max: null },
    compat: { supportsReasoningEffort: true, supportsDeveloperRole: false } },
  deepseek: { ...base, id: 'deepseek-shape',
    thinkingLevelMap: { off: 'none', high: 'high', minimal: null, low: null, medium: null, xhigh: null, max: null },
    compat: { supportsReasoningEffort: true, supportsDeveloperRole: false, thinkingFormat: 'deepseek' } },
};
async function runCase(modelLabel, reasoning) {
  const before = captured.length;
  const stream = streamSimple(models[modelLabel],
    { messages: [{ role: 'user', content: [{ type: 'text', text: 'hi' }] }] },
    { apiKey: 'probe-key', ...(reasoning === undefined ? {} : { reasoning }) });
  for await (const _chunk of stream) { /* drain */ }
  const body = captured[before];
  console.log(JSON.stringify({ case: modelLabel + '/' + (reasoning ?? 'unset'),
    reasoning_effort: body.reasoning_effort ?? '<absent>', thinking: body.thinking ?? '<absent>' }));
}
~~~

Pre-patch (commit `d5b3de2`):

~~~json
{"case":"openai/unset","reasoning_effort":"none","thinking":"<absent>"}
{"case":"openai/off","reasoning_effort":"none","thinking":"<absent>"}
{"case":"openai/high","reasoning_effort":"high","thinking":"<absent>"}
{"case":"cannot-disable/off","reasoning_effort":"high","thinking":"<absent>"}
{"case":"deepseek/unset","reasoning_effort":"<absent>","thinking":{"type":"disabled"}}
{"case":"deepseek/off","reasoning_effort":"<absent>","thinking":{"type":"disabled"}}
{"case":"deepseek/high","reasoning_effort":"high","thinking":{"type":"enabled"}}
~~~

Post-patch (patch below applied,
`npx tsc -p tsconfig.build.json` in
`packages/ai`):

~~~json
{"case":"openai/unset","reasoning_effort":"<absent>","thinking":"<absent>"}
{"case":"openai/off","reasoning_effort":"none","thinking":"<absent>"}
{"case":"openai/high","reasoning_effort":"high","thinking":"<absent>"}
{"case":"cannot-disable/off","reasoning_effort":"high","thinking":"<absent>"}
{"case":"deepseek/unset","reasoning_effort":"<absent>","thinking":"<absent>"}
{"case":"deepseek/off","reasoning_effort":"<absent>","thinking":{"type":"disabled"}}
{"case":"deepseek/high","reasoning_effort":"high","thinking":{"type":"enabled"}}
~~~

Only the two `unset` cases move (off wire or disabled thinking becomes the
endpoint's own default);
`off` and level cases are byte-identical.
The
`cannot-disable/off` case dispatching `"high"` is a pre-existing quirk of
`clampThinkingLevel` climbing from an unsupported `off` (`packages/ai/src/models.ts:692`);
both runs share it and it is out of scope here.

The upstream build needed only `npx tsc -p tsconfig.build.json` in
`packages/telemetry` and `packages/ai`;
upstream-authored scripts
(`hydrate:model-data`,
`check:model-data`) were not executed,
so the catalog
data files stay absent and the probe uses literal model descriptors.

## Verified workarounds

### Consumer mapping: keep the off level out for default-enabled models

Shipped in `package/dsh-plugin/neuralwatt/src/listing.ts` (commit
`6b1716c8a4`).
The level dict now carries `off` only when the listing says
`default_enabled: false`;
a thinking-by-default model maps its thinking levels
only,
so naming no effort reaches the service as the parameter's absence and
the model runs its own `default_effort`.

Tradeoffs:
the `Off` item disappears from the picker for thinking-by-default
models (`mimo-v2.6-pro` offers `Default` and `High`),
so thinking cannot be
switched off there until the upstream fix lands.
Session titles request the
lowest offered level and,
with `off` gone,
clamp up to a thinking level (a few
title tokens spent).
Models that think off by default (`deepseek-v4-flash`,
`gemma-4-31b`) keep an honest `Off` and an honest `Default`.

### Pick a thinking level explicitly

Selecting `High` (or any offered level) dispatches that level's spelling and
thinking appears.
Tradeoffs:
manual,
per session or per model selection,
and
unset sessions stay silent.

### Upstream fix: make unset distinct from off

The patch in [pi-ai-unset-reasoning-effort-dispatches-off.patch](pi-ai-unset-reasoning-effort-dispatches-off.patch)
pins the dispatched model's `thinkingLevelMap.off` to `null` when the caller
names no reasoning level,
which suppresses every format's explicit-off dispatch
on the empty selection.
Verified by Harness B.

Tradeoffs:
it needs the companion change in `dsh-llm-pi-ai`
(`profileOptions` must pass `off` through instead of deleting it) or explicit
`Off` selections stop disabling thinking;
until both land,
consumer behavior is
unchanged.
With both landed,
the consumer mapping can restore `off: "none"`
for every model and regain the `Off` item.

## What does not work

- Route-level `reasoning: high` on the provider profile.
  It fills
  `defaultEffort` only for models that offer `high` (`describableReasoningLevel`,
  `dsh-llm-pi-ai/lib/index.js:1767`),
  leaves `qwen-3.8-27b`,
  `nw-large`,
  `glm-5.3`,
  `kimi-k3` and `nw-small` dispatching the off wire,
  and shifts
  `max`-default models to `high`.
- Declaring `off` empty (`off:` with no value).
  `resolveModelReasoning` then
  leaves `thinkingLevelMap.off` absent (`dsh-llm-pi-ai/lib/index.js:599`),
  the
  picker still offers `Off` (absent reads as supported in
  `getSupportedThinkingLevels`,
  `dsh-llm-pi-ai/lib/index.js:773`),
  and that
  `Off` becomes inert:
  it dispatches nothing and the model keeps thinking.
- Filling one level name with every model's `default_effort` spelling.
  The
  picker label then lies wherever the model's default is not that level:
  `nw-large` would answer `Medium` with its `max` default.
- `compat.supportsReasoningEffort: false`.
  Both OpenAI-style branches require
  the flag,
  so explicit levels stop dispatching too and the picker becomes
  decorative.
- `compat.thinkingFormat: "ant-ling"`.
  Its branch fires only when an effort is
  named and sends `reasoning: { effort }`,
  a shape this service does not take.
- A per-model default effort in the configuration.
  `dsh-llm-pi-ai`'s schema has
  only the route-level `reasoning` field (`dsh-llm-pi-ai/lib/index.js:1061`);
  `modelFields` carries none.

## Upstream filing decision

### Out-of-scope check

`.out-of-scope/` holds `pi-gpt55-long-context.md` (a Pi GPT-5.5 context-limit
non-goal) and `codex-harness.md` (Codex plugin maintenance),
neither of which
covers reasoning dispatch;
no entry names `earendil-works/pi`,
`@earendil-works/pi-ai`,
or `deepseek-ai/deepseek-harness`.
No exemption
applies.

### Duplicate search

`gh search issues --repo earendil-works/pi` over the symptom and root-cause
terms found the same root-cause family:

- [#8454](https://github.com/earendil-works/pi/issues/8454),
  closed
  2026-08-25,
  "OpenRouter reasoning-mandatory models ... adapter sends
  `reasoning:{effort:"none"}` when caller omits reasoning":
  the OpenRouter
  branch of the same empty-selection dispatch,
  surfacing as an HTTP 400 instead
  of silent thinking suppression.
  The thread records the same `thinkingLevelMap`
  convention and a contributor fix branch;
  the issue was auto-closed without a
  maintainer resolution.
- [#4474](https://github.com/earendil-works/pi/issues/4474),
  closed
  "closed-because-refactor",
  "sends `reasoning_effort: none` by default,
  breaking tools":
  the same class on the OpenAI-style branch.

A matching report exists,
so the kept artifact is an additive comment draft on
[#8454](https://github.com/earendil-works/pi/issues/8454),
not a new issue.

### Upstream filing decision

1. **Is it really upstream's fault?**
   Yes,
   with nuance.
   pi-ai's
   empty-selection dispatch is a deliberate convention (omit means off),
   but it
   gives callers no way to say "let the endpoint decide",
   and
   `dsh-llm-pi-ai` both deletes the one value that could say it (`off`) and
   documents a map shape its own mapping never emits.
2. **Can upstream fix it?**
   Yes.
   The one-hunk prototype above proves it.
3. **Are they supporting this use case?**
   Yes.
   `dsh-agent/lib/index.js:150`
   names "the selected model's provider/default behavior" as the intent for an
   absent effort,
   and the two issues above treat the off-wire dispatch as a bug.
4. **Would the repo welcome our contribution?**
   Not as an open filing today.
   pi's `CONTRIBUTING.md` auto-closes all new-contributor issues and PRs,
   requires issue text in the author's own voice with any LLM text clearly
   labeled,
   gates PRs behind a maintainer `lgtm`,
   and threatens account blocks
   for agent-generated issue spam.
   `deepseek-ai/deepseek-harness`
   `CONTRIBUTING.md` states "we cannot accept external pull requests at the
   moment" and routes reports to GitHub Discussions.
5. **Will they likely fix it?**
   Unknown.
   No maintainer signal either way;
   [#8454](https://github.com/earendil-works/pi/issues/8454) sits unactioned
   and its contributor fix branch is unmerged.
6. **Prototyped?**
   Yes,
   in a disposable clone (`mktemp` under `~/temp/agent`,
   origin verified as `https://github.com/earendil-works/pi.git`,
   commit
   `d5b3de2`),
   verified by Harness B with the recorded differential.

Constraint 4 does not hold,
so the default policy applies:
**do not file now.**
Both drafts below are kept as an auditable record and may be posted once a
maintainer invitation exists (`lgtmi`/`lgtm` on pi,
or a Discussions thread
worth reviving).
The user decision on 2026-10-25 was prototype and draft only.

### Additive comment draft for earendil-works/pi#8454

~~~md
The same empty-selection dispatch exists on the OpenAI-style branch
(`packages/ai/src/api/openai-completions.ts`, the `offValue` block), where it
surfaces differently: the endpoint does not reject the request, it silently
disables thinking on models whose server default is thinking on. Observed on a
custom OpenAI-compatible endpoint whose model listing declares
`default_enabled: true, default_effort: "high"` and accepts
`reasoning_effort: "none"`: a caller that names no reasoning level received
`"reasoning_effort": "none"` and the model produced zero thinking tokens across
a whole session.

The `thinkingLevelMap: { "off": null }` catalog convention from this issue does
not cover it, because the model genuinely supports disabling thinking (`"none"`)
and the caller wants both: "off" to disable, empty selection to keep the
endpoint default. There is no state for "let the endpoint decide" today:
`streamSimple` rewrites `"off"` to `undefined`
(`clampedReasoning === "off" ? undefined : clampedReasoning`), so both callers
land in the same `!options?.reasoningEffort` dispatch.

Minimal fix that preserves the current explicit-off behavior: when the caller
names no reasoning level, pin the dispatched model's `thinkingLevelMap.off` to
`null` before the request build. Verified against `d5b3de2` with a
`streamSimple` probe over three model shapes and seven cases: the two `unset`
cases move to "no directive", `off` and level cases stay byte-identical.

diff --git a/packages/ai/src/api/openai-completions.ts b/packages/ai/src/api/openai-completions.ts
(the full patch is recorded in the linked troubleshooting doc)
~~~

### Discussion draft for deepseek-ai/deepseek-harness

~~~md
Title: llm-pi-ai: an unset reasoning effort reaches pi-ai as "off", so UI
Default disables thinking on thinking-by-default models

In `dsh-llm-pi-ai`, `profileOptions` maps the `off` level to "omit the
reasoning option", the same state an unset effort produces, and pi-ai's
`openai-completions` path then dispatches `thinkingLevelMap.off` for that empty
selection. On a custom provider whose model declares `reasoningEfforts:
{ high: "high", off: "none" }` and whose server defaults to thinking
(`default_enabled: true`), leaving the thinking picker at its Default item
sends `reasoning_effort: "none"` and the model never thinks. Picking `High`
works.

This contradicts the documented intent in `dsh-agent` ("An absent selected
effort clears any inherited effort, restoring the selected model's
provider/default behavior"). A second wrinkle: `resolveModelReasoning`
documents a declared-empty `off` as "supported, send nothing", but the emitted
map leaves the key absent and pi-ai reads absence as `?? "none"` on some
formats.

Consumer-side mitigation in our plugin: omit the `off` level for
default-enabled models (tradeoff: the Off item disappears there). A pi-ai-side
prototype (pin `thinkingLevelMap.off` to `null` on the empty selection) is
verified with a seven-case differential and would restore both controls once
`profileOptions` passes `off` through.
~~~
