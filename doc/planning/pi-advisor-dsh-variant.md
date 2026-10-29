# Pi advisor DSH variant design

Purpose: the implementation plan for `package/dsh-plugin/advisor`,
the in-process DSH adapter of the pi advisor
on the shared core `package/agent-harness-shared/advisor`.
This is a design-only document; implementation follows in a later series.

Canonical decisions live in
`doc/handover/pi-advisor-dsh-variant-grilling.md`
(Q1 to Q13, forced adoptions, rejections).
This document records the probed host API surface,
the parity transcription,
and the build and verification plan for those decisions.

Probes for this document ran on 2026-10-20 against the live profile
with `cordis_inspect_list` and `cordis_inspect_query`,
plus installed harness package declarations and README files.
Where the live manifests contradict an assumed shape,
the manifests win and the difference is called out in place.

## Scope of the variant

- Day-one surface (Q4): the `advisor` tool plus `/advisor` command parity,
  all five forms: bare, `<slug>`, `status`, `off`, `on`.
- Shared core depth (Q7): middle.
  Pure logic plus the pi-ai-typed pipeline move behind three narrow ports
  (model resolution, transcript, theme).
  Adapters keep registration, commands, pi-tui rendering, session plumbing.
- Config (Q8, Q11): unified file at today's pi paths,
  shared loader; DSH `Config` holds structural fields only.
- Selection (Q12): explicit `model` argument,
  then `defaultReviewerModel`,
  then the DSH fallback ranking.
  No text-price side table.
- Unchanged by this series: the DSH profile default model (Q12),
  pi-visible behavior (Q8),
  full operation machinery: hedging, overlap, deadline, ledger (forced adoption).

## Probe method and one transport gap

Inspection used the harness inspect providers named by `cordis_inspect_list`:

- host `Service` `listService`: the compact Service directory
  and method-signature catalog.
- host `Event` `listEvents`: the compact Event directory with modes.
- host `Config` `listConfigs`: the live plugin Config entry directory.
- host `Tool` `listTools`: the live Agent tool schemas.
- client `Slots` `listSubTree`, client `Theme` `listTokens`,
  client `Builtin` `listBuiltins` (available; not needed for this host-only plugin).

Capability gap found during this probe:
in this design session the tool-call transport delivers the untyped
`input` parameter of `cordis_inspect_query` as a raw string,
so every keyed query fails host validation with
`rejected input: "input" must be an object`.
Catalog-mode calls (no `input`) work, and typed object parameters
decode correctly, which isolates the fault to the untyped `json` parameter.
Fallback used here: catalog signatures above,
plus installed `lib/types/**/*.d.ts` and README declarations of the harness packages.
Fixture verification runs the keyed queries inside a normal
`dsh --profile <fixture>` session,
where the agent loop JSON-parses whole argument objects.

## Host API surface

### Plugin entry and bundle shape

Host plugin export form, per the `cordis-plugin-development` skill
reference `references/host-plugin.md`:

```ts
export function apply(ctx, config) { /* register with ctx.effect */ }
export const inject = [
  'tools', 'commands', 'llm',
  'sessions', 'sessionProjections', 'systemPrompt', 'workingDirectory',
];
export const Config = z.object({ /* Schemastery structural fields */ });
```

- A bundle package declares `dsh.bundle.patch` in `package.json`
  and ships `cordis.patch.yml`, `locale/en.json`, and `icon.svg`.
- `Config` is a Schemastery schema
  (`z` from `@deepseek-ai/schemastery`, as `@deepseek-ai/dsh-tools/lib/index.js`
  shows in `ToolRuntime.Config`),
  validated at activation against the row's `config`.
- Registration cleanup goes through `ctx.effect` or `ctx.on`,
  returning the dispose function.

Precedent correction (manifests win):
`package/dsh-plugin/search-fetch` in this repo is a configuration-only bundle
(`cordis.patch.yml` mounting `@deepseek-ai/dsh-mcp-client`, no `src/`, no `apply`).
The in-process precedent for `apply(ctx, config)` plus `ctx.tools.register`
is the harness package `@deepseek-ai/dsh-tool-cordis`
(`apply(ctx)`, `export const name`, `export const inject = ['tools', 'cordisInspect']`,
`ctx.tools.register(defineTool({ ... }))`).
The advisor plugin follows `dsh-tool-cordis`,
not the configuration-only search-fetch bundle.

### Tool registration

Host `Service` provider `tools`, method `register(definition: ToolDefinition): () => void`.
Tool definitions are built with `defineTool` from `@deepseek-ai/dsh-tools`:

```ts
// @deepseek-ai/dsh-tools lib/types/schema.d.ts
defineTool<const S extends ParameterSchemaSpec, const O extends ValueSchemaSpec>(
  options: DefineToolOptions<S, O>,
): ToolDefinition;
```

- `parameters` is a `ParameterSchemaSpec` map
  (`{ type: 'string', description, required?: true }` per property).
- `output` is a mandatory `ToolOutputDefinition`:
  `schema: JsonSchemaNode` plus `render(args, value): ContentBlock[]`.
- `execute(args, exec: ToolRunContext): Promise<unknown>` returns only the
  canonical JSON value; the registry validates it against `output.schema`
  and projects model content through `output.render`.
  This is the "tool results as JSON" convention.
- `exec` (`ToolRunContext`) carries `agent?: Agent`, `signal`,
  `callId`, `token`, `rootCallId`, `deferContext`, `concludeTurn`.
- `ToolRestriction { allow?: readonly string[]; deny?: readonly string[] }`
  (host `Service` `tools`, method `restrict(filter: ToolRestriction): () => void`)
  is the DSH form of pi's `setActiveTools` toggling,
  used by `/advisor off` and `/advisor on`.
- `ToolExecutionSuccess` and `ToolExecutionFailure`
  (`@deepseek-ai/dsh-tools/lib/types/index.d.ts`) have no `usage` member;
  see the usage-reporting gap in the rendering section.

### Command registration

Host `Service` provider `commands`,
method `register(definition: CommandDefinition): () => void`.
Exact declarations from `@deepseek-ai/dsh-commands/lib/types/index.d.ts` and `types.ts`:

- `CommandDefinition`: `name` (lowercase, no leading slash),
  `description`, optional `input: CommandInputDescriptor`,
  optional `recordInput` (defaults to `true`),
  `handler(invocation: CommandInvocation): CommandResult | Promise<CommandResult>`.
- `CommandInvocation`: `commandId`, `agent`,
  `rawInput` (exact text after the name, including separator whitespace),
  `attachments`, `signal`.
- `CommandResult` is exactly `{ kind: 'success'; text?; sourceEventSeq? }`
  or `{ kind: 'error'; text }`.
  A thrown or aborted handler settles as `kind: 'error'`.
- Lifecycle events `command/run` and `command/done` log the invocation
  and the verbatim `kind`/`text` outcome.

The handler returns `{ kind: 'success' | 'error', text }`,
so every `/advisor` form produces plain text,
and subcommand parsing reads `invocation.rawInput`.

### LLM seam

Host `Service` provider `llm`, the abstract adapter registry plus streaming
model-call API. Methods the plugin uses:

- `listProviders(): LlmProviderInfo[]`, where `LlmProviderInfo` is `{ id, name }`.
- `listModels(provider: string): Promise<LlmModelInfo[]>`.
- `resolveModelInfo(provider, model, signal?): Promise<LlmResolvedModelInfo>`.
- `stream(options: GenerateOptions): AsyncIterable<StreamChunk>`.
- `providerRetryPolicy(provider: string): ResolvedRetryPolicy` (read-only).

`GenerateOptions` (`@deepseek-ai/dsh-llm/lib/types/types.d.ts`):
`provider`, `model`, optional `reasoningEffort`, `messages: RequestMessage[]`,
optional `system`, `tools`, `temperature`, `maxTokens`, `stop`, `signal`,
`sessionId`, `purpose`.
`purpose` accepts only `'compaction' | 'session-title'`;
advisor calls leave it unset.
Stream chunks are `block-start`, `text-delta`, `reasoning-delta`,
`tool-call-delta`, `block-end`, `usage`, `finish`;
`BlockAssembler` (`@deepseek-ai/dsh-llm`) is the canonical
chunk-to-message assembly.

What `LlmModelInfo` actually carries:

- `provider`, `id`, `name`, optional `description`,
  optional `inputModalities`.
- No context window field.
  Capacity arrives on `LlmResolvedModelInfo`:
  `context?: LlmModelContext { contextWindow: number }`,
  `defaultMaxTokens?: number`,
  `reasoning?: LlmModelReasoningInfo { efforts, defaultEffort? }`,
  `systemPromptUpdate?`, `toolUpdate?`.
- `LlmDiscoveredModel` (`id`, `name?`, `contextWindow?`, `maxTokens?`)
  is the draft-interrogation shape from `discoverModels`,
  not the live catalog shape.
- No text pricing exists anywhere in the seam.
  The only pricing entry point is
  `imageRequestPricing(provider, model)` for image occurrences
  (`LlmImageRequestPricing`),
  whose doc comment states that the caller prices `text` with its own estimator.
  This confirms the Q12 premise;
  a pi-ai-style text price side table stays rejected.

Failure vocabulary (`@deepseek-ai/dsh-llm/lib/types/error.d.ts` and the package
README): `HarnessError` carries a stable `code`;
canonical codes include `NO_ADAPTER`, `MISSING_CREDENTIAL`,
`INVALID_CREDENTIAL`, `AUTH`, `RATE_LIMIT`, `CONTEXT_WINDOW_EXCEEDED`,
`EMPTY_RESPONSE`, `QUOTA`, `ACCOUNT_QUOTA`.
Route on `code`, never on message text.

Retry boundary (README of `@deepseek-ai/dsh-llm`):
each `stream` call is exactly one provider attempt and the service never
re-runs a request; `@deepseek-ai/dsh-llm-retry` re-runs failed requests at
durable agent-step boundaries.
There is no per-call retry switch on `GenerateOptions`,
so the advisor cannot disable route retries the way pi does with `maxRetries`.

### Config schema mechanism

`export const Config` on the plugin entry, Schemastery syntax,
validated at activation by the Loader.
Live schemas are discoverable through host `Config` `listConfigs`
(directory, then one entry's projected JSON Schema).
Deviations from pi's single-file approach are deliberate (Q11) and documented
in the plugin `README.md`:
DSH `Config` holds structural fields only,
while tunables stay in the unified advisor JSON files read by the shared loader.

### Session surface and transcript path

Live `Agent` face (`@deepseek-ai/dsh-agent/lib/types/runtime-types.d.ts`):
`options: AgentOptions { provider?; model?; reasoningEffort?; maxTokens? }`,
`session: Session`, `ctx`, `whenIdle()`, `cancel()`, `send`, `steer`, `followup`.
A tool reaches it as `exec.agent`;
a command reaches it as `invocation.agent`.
When `exec.agent` is absent the call must fail with a clear diagnostic,
the way `@deepseek-ai/dsh-tool-cordis` does with `requireAgent`.

Exact transcript access path found:

- `Session.deriveMessages(): Message[]`
  (`@deepseek-ai/dsh-session/lib/types/index.d.ts`),
  documented as "the derived LLM message history" rebuilt from the ordered
  surface, with compaction `replace` nodes deleting shadowed history.
  This is the compaction-aware equivalent of pi's context entries.
- `session.surface` (`get surface(): SessionSurface`) is the ordered event
  surface behind that derivation.
- host `Service` `sessionProjections`,
  methods `stateOf(session, key)` and `snapshot(session, keys?)`,
  reads fold state (`SessionProjectionMap`).
  Probed keys include `tokenUsage`, `contextPressure`, `contextBreakdown`,
  `inbox`, `permissions`, `agentPreset`, `userQuestions`, `agentTeam`.
  There is no message-history key,
  so `deriveMessages()` is the transcript source and projection state is
  auxiliary (usage and pressure facts only).
- host `Service` `sessions` (`get`, `list`) resolves a `Session` by id
  when the caller holds only an id.
- host `Service` `workingDirectory`,
  methods `get(session: Session)` and `ensure(agent, signal)`,
  supplies the project cwd for config resolution.
- host `Service` `systemPrompt`, method `assemble(context?)`,
  returns `PromptAssembly { sections, contexts }`,
  the DSH source of Pi-loaded project context
  (pi reads `before_agent_start.systemPromptOptions.contextFiles`).

### Events worth knowing

From host `Event` `listEvents`:

- `llm/stream` (waterfall) can replace the frozen call configuration;
  advisor one-shot calls do not register it.
- `tools/change` (emit) fires on register, unregister, or restriction change.
- `commands/change` (emit) fires on command registration changes.
- `agent/status`, `agent/created`, `agent/disposed` (emit/serial)
  are available if lifecycle hooks are needed later.

## Advisor tool definition

Behavior transcribed read-only from `package/pi-plugin/advisor/src/tool.ts`,
`tool-params.ts` (shared core), and `package/pi-plugin/advisor/README.md`.

### Parameters

- `model?: string`, `question?: string`, both optional strings,
  `additionalProperties: false` at the schema level
  (`AdvisorToolParametersSchema` in the shared core).
- `prepareAdvisorArguments` normalization:
  empty, null, or blank-string args mean `{}`;
  a bare string arg is taken as `model`;
  unknown fields throw `advisor: unsupported argument fields: <names>`;
  non-string `model` or `question` throw their own diagnostics.
- Accepted explicit model forms (README):
  canonical `provider/modelId` slug;
  bare `model.id` when unique inside the effective scope;
  `model.name` when unique inside the effective scope.
  Ambiguous, unknown, out-of-scope, or output-incapable slugs throw
  and the error lists the allowed eligible scoped slugs.

### Tool identity and prompts

- Name `advisor` (`ADVISOR_TOOL_NAME` in the shared core `constants.ts`).
- Description: "Consult independent scoped reviewer models using the current
  conversation context. Empty params start with the highest expected-cost
  eligible non-current model and recover through other scoped models on failure.
  Optional configured overlap collects completed reviews together after bounded
  straggler grace. Explicit model requests never switch models."
- `promptSnippet` and the four `promptGuidelines` carry over
  from `package/pi-plugin/advisor/src/tool.ts` unchanged.
- `executionMode: 'sequential'` carries over as the tool's call policy.

### Execution semantics (parity)

- Session-disabled calls throw
  `advisor: disabled for this session. Run /advisor on to re-enable.`
- `execute` runs `runAdvisor` with the tool call id,
  config snapshot, serialized project context,
  optional `requestedSlug` and `question`, the abort signal,
  and an `onUpdate` progress callback.
- Default calls recover serially through untried eligible scoped candidates
  within the original deadline;
  explicitly requested models stay exact with no fallback and no overlap.
- Overlap (hedging) semantics, collection grace,
  credit-exhaustion provider exclusion,
  bounded no-text retry,
  and the operation ledger are shared-core behavior (forced adoption)
  and are not re-specified here.
- Failure accounting persists a `pi-advisor.operation` record;
  the DSH adapter logs the same operation payload
  through host `Service` `sessionPersistence` append
  or a session event of its own custom type,
  since DSH has no `pi.appendEntry` equivalent.

### Result shape

- The canonical JSON value returned by `execute` carries the review text
  fields plus `AdvisorDetails` metadata (operation ledger, attempted models,
  aggregate usage when available).
- `output.render` projects that value as text `ContentBlock`s
  (`renderAdvisorSummary` and `renderAdvisorOperationSummary`
  from the shared core are the plain-text summarizers).

## `/advisor` command parity surface

Behavior transcribed read-only from
`package/pi-plugin/advisor/src/commands.ts`, `command-message.ts`,
`status.ts`, and `index.ts`.
The DSH definition registers one command named `advisor`
with description "Run Advisor, inspect scoped models, or toggle Advisor
for this session" and default `recordInput: true`.

Parsing uses `invocation.rawInput` trimmed (pi passes `args?.trim()`):

1.  Bare `/advisor`:
    run an immediate review with the default reviewer.
    pi waits for pending work to settle (`ctx.waitForIdle()`)
    before reading prompt options;
    the DSH form awaits `invocation.agent.whenIdle()`
    before reading `session.deriveMessages()`.
    Success returns `{ kind: 'success', text: result.text }`.
2.  `/advisor <slug>`:
    run an immediate review with that scoped model,
    passing `requestedSlug: <slug>` and otherwise identical behavior.
3.  `/advisor status`:
    return `{ kind: 'success', text: <status text> }` with these lines,
    transcribed from `buildAdvisorStatus`:

    ```text
    Advisor: on|off
    Scope source: <scope source>
    Default failures: serial fallback through untried scoped models; explicit requests remain exact
    Overlapping default reviews: enabled after <hedgeDelayMs>ms|disabled
    Collection grace: <collectionGraceMs>ms after first usable review, capped by original deadline
    Overlap can bill both requests; the set of completed reviewers is nondeterministic
    Credit exhaustion excludes that provider for the current call only;
    cancellation does not guarantee complete billing data
    Scoped models: <count|none>
    Advisor request budget: <maxAdvisorOutputTokens> output tokens, clamped to each route's advertised maximum
    Default model: <canonical slug|none>
    Default ranking: <ranking reason|none>
    Config: global=<path|absent> project=<path|absent>
    Context budget: <N> chars effective for default model, cap=<N chars|none>, <maxAdvisorOutputTokens> output tokens
    Prior Advisor results: included|omitted
    ```

    The `Scoped models` line reports a count instead of a slug list,
    so `/advisor status` never enumerates every scoped model.
    The former `Eligible Advisor models (>=<maxAdvisorOutputTokens> output tokens)` line is gone:
    commit `91edd0b719` retired the output-eligibility gate,
    so every scoped model stays selectable
    and the request budget line states the clamping rule.

    DSH adaptations inside those lines:
    `Scope source` reports the `llm.listProviders` catalog,
    and one added line names `defaultReviewerModel` and its resolution state.
    The credit-exhaustion line wraps in this document for line length only;
    runtime output emits it as one line.
4.  `/advisor off`:
    flip the session toggle off,
    apply `ctx.tools.restrict({ deny: ['advisor'] })` (or dispose the registration),
    return `{ kind: 'success', text: 'Advisor disabled for this session.' }`.
5.  `/advisor on`:
    flip the session toggle on, clear the restriction,
    return `{ kind: 'success', text: 'Advisor enabled for this session.' }`.

Parity strings for the disabled and failure paths:

- Disabled bare or slug form:
  `Advisor is disabled for this session. Run /advisor on to re-enable.`
  as `{ kind: 'error', text }`.
- Cancelled review: `Advisor review cancelled.`
- Failed review: `Advisor review failed: <caught value text>`.
- On `AdvisorOperationError`, the operation payload is persisted
  the same way tool failures are.

There is no `/advisor set` command (README):
persistent model overrides would bypass scoped tool-parameter selection.

Manual review output parity:
pi sends a rendered custom message (`pi-advisor.review` custom type,
`display: true`, `details`);
DSH returns the review as the command result text instead.
The custom-message renderer (`pi.registerMessageRenderer`) has no DSH
counterpart; this is part of the rendering gap.

## Config resolution

- Unified file paths (Q8), resolved by the shared loader
  (`loadMergedConfig`, `getConfigPaths`, `CONFIG_FILE_NAME = 'pi-advisor.json'`):
  global `~/.pi/agent/extensions/pi-advisor.json`,
  project `.pi/extensions/pi-advisor.json` under the working directory.
- Both variants read the same files;
  project configuration overrides global scalar values;
  pi-visible behavior stays untouched.
- Shared fields (valibot `AdvisorConfigFileSchema`):
  `enabled`, `timeoutMs`, `hedgingEnabled`, `hedgeDelayMs`,
  `collectionGraceMs`, `maxContextChars`, `maxAdvisorOutputTokens`,
  `includePriorAdvisorResults`, `systemPrompt`.
  Scheduling durations are positive integers at most `2147483647`;
  `hedgingEnabled: true` requires an explicitly configured `hedgeDelayMs`.
- DSH `Config` (Schemastery) holds structural fields only:
  the anchor `id`, package name, and any wiring shape the Loader validates.
  No timeout, token budget, or model name goes in the DSH row.
  This is the documented deviation from the pi single-file approach (Q11),
  and the plugin `README.md` states it in its Configuration section.
- `defaultReviewerModel` is a user-config-file key only.
  It is read from the unified JSON files,
  is never a value of the DSH `Config` row,
  and is never a schema default.
  Shared-core requirement:
  add `defaultReviewerModel: v.exactOptional(v.string())`
  to `AdvisorConfigFileSchema` with no default,
  and carry it through `AdvisorConfig`;
  today's schema omits the key and valibot strips unknown keys.
- Current live user value (read from `~/.pi/agent/extensions/pi-advisor.json`
  on 2026-10-20): `"defaultReviewerModel": "opencode-go/kimi-k3"`.
  That value stays user-owned; no package ships it.

## Model selection order

Q12 order, implemented in shared `selectAdvisorModel` with a variant
ranking seam:

1.  Explicit `model` tool argument or `/advisor <slug>`:
    exact request, resolved as the accepted explicit forms above.
    No cross-model fallback and no overlap,
    even when overlap is enabled globally.
2.  `defaultReviewerModel`, when it resolves in the current scope.
    Resolution means its provider id appears in `ctx.llm.listProviders()`
    (host `Service` `llm`, method `listProviders`)
    and its model id or name appears in `ctx.llm.listModels(provider)`.
    An explicitly cleared key, or a name absent from the scope,
    falls through to the variant fallback without error.
    Adopted interpretation (flagged for veto):
    tier 2 chooses the initial reviewer of a default call,
    so serial recovery and optional overlap still apply to it;
    only tier 1 is exact.
    Rationale: pi's default calls are the fallback-capable mode,
    and a permanently pinned exact request would disable recovery.
3.  Variant fallback:
    the DSH ranking picks the largest `contextWindow`,
    ties broken by `name`,
    with the current main model excluded when another eligible candidate remains
    (`scopeAvoidingCurrentMainModel` parity).
    Capacity comes from `ctx.llm.resolveModelInfo`
    (`LlmResolvedModelInfo.context.contextWindow`),
    because `LlmModelInfo` carries no context window field.

Ranking seam requirement in the shared core:
`selectAdvisorModel` currently calls `selectDefaultModel`
from `@monochromatic-dev/agent-harness-shared-model-selection`
(highest expected cost using pi `model.cost` fields).
Add an explicit ranking input (for example a
`rankDefaultCandidates` callback or a `defaultRanking` discriminator),
with pi passing the expected-cost ranking and DSH passing
the `contextWindow` then `name` ranking,
and insert the `defaultReviewerModel` lookup before ranking.
Until that seam lands, the DSH adapter must not fake tier 2
through `requestedSlug`,
because that would silently make tier 2 exact.

No text pricing is used or consulted anywhere in this order (Q12).
A pi-ai-style cost side table is explicitly rejected;
do not propose one.

Related capability adaptation:
pi filters the scope to endpoints advertising
`model.maxTokens >= maxAdvisorOutputTokens`.
DSH advertises `defaultMaxTokens` as an adapter-configured request default,
not an endpoint advertisement.
Fallback rule:
exclude a candidate only when `defaultMaxTokens` is present and lower than
`maxAdvisorOutputTokens`;
treat unknown capacity as eligible,
and send `GenerateOptions.maxTokens = maxAdvisorOutputTokens`
so an incapable route fails at dispatch and enters normal recovery.
`/advisor status` reports candidates whose capacity is unknown.

Reasoning-effort parity:
pi requests the highest level below `max`,
never `max`.
The DSH form reads `LlmResolvedModelInfo.reasoning.efforts`
(least to greatest) and selects the greatest effort not named `max`,
omitting `reasoningEffort` when the route advertises none.

Credit-exhaustion parity:
the core classifier `isAdvisorCreditExhaustion` matches diagnostic markers.
The DSH adapter maps terminal `finish` failures with code `QUOTA`
or `ACCOUNT_QUOTA` onto the marker `insufficient_quota`
before classification,
so provider exclusion semantics stay identical.

## Transcript acquisition

Primary path (probed):

1.  Tool call: `exec.agent.session.deriveMessages()`.
    Command call: `invocation.agent.session.deriveMessages()`.
2.  Map the returned `Message[]` into `AdvisorSessionEntry` values
    (`{ type: 'message', message, timestamp }` shape) for
    `AdvisorTranscript.buildContextEntries()`,
    and let the shared filter and budgeting run unchanged.
3.  Auxiliary facts from host `Service` `sessionProjections`
    (`stateOf`, `snapshot`): usage and pressure only,
    never message content.

Parity notes:

- `deriveMessages()` is compaction-aware:
  compaction `replace` nodes delete shadowed nodes,
  matching pi's "messages summarized by the latest compaction are not resent".
- The snapshot is built once per operation;
  the core never re-reads the session mid-operation
  (`AdvisorTranscript` contract).
- Project context is captured separately and appended to the advisor system
  prompt:
  DSH source is host `Service` `systemPrompt`, method `assemble`,
  rendered to text and passed as `projectContext`
  to `serializeAdvisorProjectContext` semantics,
  so `AGENTS.md` and other context instructions survive compaction.
- Content-block mapping is adapter-owned.
  DSH `ContentBlockMap` kinds `tool-addition` and `tool-removal`
  have no pi-ai counterpart and are dropped in conversion;
  `text`, `reasoning`, `tool-call` map structurally.
- Missing-agent calls fail loud with a named diagnostic
  (no transcript without an agent-backed session).

Fallback (Q4 clause):
when the session binding proves insufficient
(for example a resumed session whose derived history is unavailable),
the tool's `question` parameter carries the content;
the adapter passes it as the review subject and records
the degraded acquisition in `/advisor status` output.

## Text-ContentBlocks-only rendering gap

Documented gap (Q4), restated with probed shapes:

- `ToolDefinition.output.render(args, value): ContentBlock[]`
  produces text blocks only for advisor results;
  markdown formatting rides inside the text content.
  There is no pi-tui component tree and no theme token channel.
- No progress channel exists:
  a tool call settles once,
  so the pi `onUpdate` progress stream
  (`formatAdvisorProgress` metadata-only updates)
  has nowhere to render live.
  The operation ledger still reaches the final result;
  progress text is folded into the final result text or omitted,
  and the plugin `README.md` states this.
- `AdvisorTheme` (the `fg` and `bold` port) is implemented as plain text
  in the DSH adapter, since no styled channel exists.
- No custom message type equivalent to `pi-advisor.review` and no
  `registerMessageRenderer`; command reviews return as `CommandResult` text.
- No tool usage field exists on `ToolExecutionSuccess` or
  `ToolExecutionFailure`, unlike pi's top-level tool usage field.
  Aggregate nested usage therefore folds into the canonical JSON value
  and the operation ledger, and `TokenUsage` projections remain readable
  through host `Service` `sessionProjections` key `tokenUsage`.
- Authoritative implementation records of these choices land in the plugin
  `README.md` and the end-of-series behavior-pick report (Q5).

## Fixture-profile verification plan

Disposable fixtures first (THR),
never the live profile before the Q10 step.

Profiles to create with `dsh --profile <fixture>`:

1.  `advisor-fixture-minimal`:
    empty profile plus the built bundle,
    no advisor config files.
    Expect: plugin row activates, `Config` row validates `config: {}`,
    `/advisor status` shows both config paths absent and enabled default.
2.  `advisor-fixture-config`:
    the same plus a global `~/.pi/agent/extensions/pi-advisor.json`
    (disposable home per fixture) carrying `defaultReviewerModel`
    and one project `.pi/extensions/pi-advisor.json` override.
    Expect: project override wins on scalars,
    `defaultReviewerModel` resolves or falls through as specified,
    `/advisor status` reports the source values.
3.  `advisor-fixture-toggle`:
    minimal profile for `/advisor off` and `/advisor on` behavior.

Each fixture boots under `dsh --profile <fixture>`,
which sets `DSH_PROFILE` and `DSH_PROFILE_DIR` in its harness shells
and keeps state under `$DSH_HOME/profiles/<fixture>`.
`dsh --profile <fixture> --dump-config` confirms the composed tree first.

`cordis_inspect_query` checks that must pass inside the fixture session:

- host `Tool` `listTools`:
  `advisor` is present with `model` and `question` parameters
  and the JSON output schema.
- host `Config` `listConfigs` (entry for `@monochromatic-dev/dsh-plugin-advisor`):
  `status: "schema"` and the projected Config schema contains
  only structural fields.
- host `Service` `listService`:
  `tools`, `commands`, `llm`, `sessions`, `sessionProjections`,
  `systemPrompt`, `workingDirectory` contracts match the signatures cited in
  this document (drift check against the probe).
- After `/advisor off`:
  `Tool.listTools` no longer lists `advisor`;
  after `/advisor on` it lists again.

Known gap in this plan:
no inspect provider lists live command registrations,
so `/advisor` command parity is verified by invocation
(`/advisor status` returns the transcribed lines,
bare and `<slug>` forms return `kind: 'success'` or the exact error texts),
which is the VB1 rule to trigger the capability through the host app.

Behavioral checks without live providers:
unknown slug, ambiguous slug, empty scope, and disabled-session errors
all produce the exact parity strings,
which needs no credentials and no model call.

### Measured fixture findings (2026-10-20)

Wiring each fixture needs three additions recorded in
[`dsh-fixture-profile-plugin-verification.md`](../troubleshooting/dsh-fixture-profile-plugin-verification.md):
inserted rows `cordis-host-runner`, `cordis-inspect-providers`, and `tool-cordis`;
a disposable LLM overlay;
and provider keys exported from the launching environment.

Fixture 1 exposed a plugin defect:
every `advisor` tool call failed with
`prompt variable "{{model}}" has no value for this assembly`
(section `deployment:persona-prefix`),
because `buildDshAdvisorProjectContext` called `systemPrompt.assemble()`
without the agent assembly context the prompt variables read.
Fixed in `85434d0c40` via `DshAdvisorAgentSurface.assembleContext`,
which forwards `assembleContextFor` from `@deepseek-ai/dsh-agent`.

Measured: agent sessions expose no command-invocation surface
(no command tool or commands service),
so `/advisor` parity checks in this plan run at the user boundary
(verification in the web GUI chat input after Q10),
and fixture runs cover the tool surface only.
Fixture config observability is limited likewise:
the tool result reports the chosen reviewer model,
while `/advisor status` config echo needs command dispatch.

### Q10 live install and rollback

1.  After fixture checks pass:
    `plugin_manager` `install_bundle`
    with `target` set to the absolute `package/dsh-plugin/advisor` directory.
2.  Read `application` and `warnings` from the result;
    `application: applied` is the live signal.
3.  Confirm the new rows.
    The filtered `cordis_inspect_query` check is blocked by the
    harness-wide input coercion recorded in
    [`dsh-cordis-inspect-query-input-coerced-to-string.md`](../troubleshooting/dsh-cordis-inspect-query-input-coerced-to-string.md),
    so this step deviates from the inspect preference toward
    `plugin_manager` `list_plugins`/`list_bundles`
    plus an unfiltered `Tool.listTools` call.
4.  Exercise `/advisor status` at the user boundary
    in the web GUI chat input (no provider call needed),
    the VB1 trigger for command capability.
5.  Rollback path:
    `plugin_manager` `remove_bundle` with the same target identity,
    then re-run the `Config.listConfigs` and `Tool.listTools` checks
    to confirm the rows are gone.
6.  Report the Q5 behavior-pick list with before/after evidence
    in the end-of-series report.

## Build order

Gate (Q1 and the adopted order):
implementation starts only after the shared-core extraction lands
(`package/agent-harness-shared/advisor` plus the pi rewiring,
the in-flight series forked at `40eef920`),
and after the Q9 model-selection move (`b710410e81`).
Two shared-core additions are prerequisites of the adapter:

- `defaultReviewerModel` optional key in `AdvisorConfigFileSchema`
  and `AdvisorConfig`, with no schema default.
- The `selectAdvisorModel` ranking seam described in the model selection section.

Then `package/dsh-plugin/advisor`
(`@monochromatic-dev/dsh-plugin-advisor`, `private: true`,
directory per `AP1`, name per `SGD`, `/ts` subpath exports per `ST3`,
`README.md` and sibling-mirroring `mise.toml` tasks including
a `verify:extension` analogue).

### Imports from the shared core

From `@monochromatic-dev/agent-harness-shared-advisor/ts`
(the Q7 middle split; every name below is exported by its `index.ts`):

- Run pipeline: `runAdvisor`, `AdvisorRunOptions`, `AdvisorRunResult`,
  `AdvisorRunContext`, `run-preparation` types,
  `AdvisorOperationSnapshot`, `AdvisorOperationError`,
  `AdvisorDetails`, `AdvisorProgressDetails`, `AdvisorToolParams`.
- Tool contract: `AdvisorToolParametersSchema`, `prepareAdvisorArguments`.
- Config: `loadMergedConfig`, `getConfigPaths`, `DEFAULT_CONFIG`,
  `AdvisorConfigFileSchema`, `AdvisorConfigFile`, `AdvisorSettingsFile`,
  `AdvisorConfig`.
- Selection: `selectAdvisorModel`, `resolveAdvisorRequestedModel`,
  `scopeAvoidingCurrentMainModel`, `filterAdvisorScopeByOutputCapacity`,
  `requireAdvisorScopeWithOutputCapacity`,
  `assertAdvisorEndpointOutputCapacity`, `assertAdvisorLiveScope`,
  `dispatch-scope` types.
- Context budgeting: `buildAdvisorContext`, `maxContextCharsForAdvisorModel`,
  `truncateContext`.
- Project context: `serializeAdvisorProjectContext`,
  `createAdvisorProjectContextState`, `AdvisorProjectContextFile`,
  `AdvisorProjectContextState`.
- Prompt: `buildAdvisorSystemPrompt`, `buildAdvisorSystemPromptForProject`.
- Completion seams: `completeAdvisor`, `requestAdvisor`, `extractAdvisorText`
  (used through `runAdvisor`, named here for tests).
- Operations: `operation-types`, `operation-usage`, `operation-clock`,
  `operation-candidates`, `operation-ledger`, `operation-attempt`,
  `formatAdvisorProgress`, `advisor-deadline`, `advisor-request`.
- Rendering (plain text only): `renderAdvisorSummary`,
  `renderAdvisorOperationSummary`, `firstAdvisoryLine`, `isAdvisorDetails`,
  `fallbackDetails`, `NO_ADVISOR_OPERATION_SUMMARY`.
- Providers and failures: `isAdvisorCreditExhaustion`,
  `assertAdvisorProviderAvailable`, `AdvisorProviderExcludedError`,
  `advisor-completion-error` types.
- Ports and types: `AdvisorModelResolution`, `AdvisorRequestAuth`,
  `AdvisorTranscript`, `AdvisorTranscriptSerialization`,
  `AdvisorSessionEntry`, `AdvisorAgentMessage`,
  `AdvisorTheme`, plus the `types.ts` re-exports.
- Constants: `ADVISOR_TOOL_NAME`, `ADVISOR_MESSAGE_TYPE`,
  `ADVISOR_OPERATION_TYPE`, `CONFIG_FILE_NAME`, `MAX_ADVISOR_TIMER_MS`.
- Active-tool helper: `containsToolName`.

From `@monochromatic-dev/agent-harness-shared-model-selection/ts`
(Q9 move):
`resolveEffectiveScope`, `selectDefaultModel`,
`EffectiveModelScope` and scope-entry types.

### What stays adapter-side

- `apply(ctx, config)`, `export const inject`, `export const Config`,
  bundle manifest, `cordis.patch.yml`, locale files, icon.
- `ctx.tools.register(defineTool(...))`, `output.render`,
  `ctx.tools.restrict` toggling, session-enabled state.
- `ctx.commands.register(...)` and the five-form `rawInput` parser.
- Port implementations:
  model resolution over `ctx.llm`
  (including a pi-ai `Provider.streamSimple` shim that calls
  `ctx.llm.stream` and assembles with `BlockAssembler`),
  transcript over `Session.deriveMessages()`,
  theme as plain text,
  project context over `ctx.systemPrompt.assemble()`.
- Status text assembly (DSH scope source, `defaultReviewerModel` line),
  fixture verification harness and `mise` tasks, plugin `README.md`.

Per-call `maxTokens` is set to `maxAdvisorOutputTokens`,
`GenerateOptions.purpose` stays unset,
and route retry policy from `providerRetryPolicy` is accepted as part of one
logical attempt (documented gap: pi disables provider-internal retries with
`maxRetries`; the DSH seam has no such switch).

## Capability gaps and fallbacks

1.  Inspect `input` transport in this design session:
    keyed `cordis_inspect_query` calls fail.
    Fallback: catalog signatures plus installed `lib/types` declarations;
    keyed queries run in fixture sessions.
2.  No text pricing on the LLM seam:
    expected by Q12; the DSH fallback ranks by `contextWindow` then `name`.
3.  No tool progress channel and one-shot settlement:
    progress text folds into the final result; documented gap.
4.  No custom message type or message renderer:
    command reviews return as `CommandResult` text.
5.  No usage field on tool results:
    aggregate usage folds into the canonical JSON value and ledger.
6.  No per-call retry disablement (`GenerateOptions` lacks `maxRetries`):
    `dsh-llm-retry` policy is accepted inside one logical attempt.
7.  No inspect view of live command registrations:
    `/advisor` parity verified by invocation in fixture sessions.
8.  `LlmModelInfo` has no output-capacity advertisement
    (`defaultMaxTokens` is a configured default):
    exclude only advertised-too-low routes; unknown capacity stays eligible
    and fails into normal recovery at dispatch.
9.  `GenerateOptions.purpose` has no advisor value:
    leave it unset.
10. `SessionProjectionMap` has no message fold:
    `Session.deriveMessages()` is the transcript path;
    projection state carries usage and pressure only.
11. `LlmModelInfo` has no context window:
    read `LlmResolvedModelInfo.context.contextWindow` via
    `resolveModelInfo` for the fallback ranking.

## Open coordination points

- The shared-core extraction agent owns `config-schemas.ts`,
  `advisor-selection.ts`, and `config.ts`;
  the `defaultReviewerModel` key and the ranking seam
  must land there before the DSH adapter implements tier 2 and tier 3.
- Q3 attribution discipline continues:
  every behavior pick lands with an updated test in the same commit.
- Slopo issue 621 (shared fake pi extension host) stays out of this series;
  the DSH fixture harness is DSH-only.
- Issue tracking follows the `pi-advisor-413` precedent:
  this planning doc plus one GitHub issue opened `needs-triage`
  (now `#1792`).

## Completion status (2026-10-20)

Implemented, fixture-verified, and installed in the live `web` profile.
Landing commits: `16a22cd7de`, `39d452035a`, `effcff526e`, `3583eec279`,
`85434d0c40`, `07ad8133c5`; every behavior pick landed with its test.
Q10 live install: `plugin_manager install_bundle` reported
`application: applied`, `warnings: []`,
and `dsh --profile web --dump-config` shows `id: advisor`, `config: {}`.
Deviations from the plan, measured and recorded:
`cordis_inspect_query` coerces input to string (unfiltered calls only),
`plugin_manager list_plugins` ignores `target` as a filter,
and the web app has no CLI task mode,
so the live tool catalog is confirmed in a fresh GUI chat.
The Q5 end-of-series picks report closed the series in the session record;
the picks live in `package/dsh-plugin/advisor/README.md`.
Remaining at the user boundary (issue `#1792`):
`/advisor status`, `/advisor off`, `/advisor on`,
and tool-catalog presence in a new GUI chat.
Rollback: `plugin_manager remove_bundle` with the package target.
Residue: fixture profile `advisor-fixture-minimal`,
`~/temp/agent/advisor-fixture-*` inputs.
