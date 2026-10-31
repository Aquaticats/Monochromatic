# Pi 0.87 breaks auto-mode judge context and model-review provider requests

Updating the workspace Pi packages from 0.85.1 to 0.87.1
(`@earendil-works/pi-coding-agent`,
`@earendil-works/pi-ai`,
`@earendil-works/pi-agent-core`)
broke `package/pi-plugin/auto-mode` in two independent ways,
and a third defect kept both invisible to type checking.

## Symptom

Every guarded tool call in a Pi session returned this tool result instead of running:

```text
Unsupported Pi session message role.
```

The session file behind the failure
(`~/.pi/agent/sessions/--var-home-user-Monochromatic--/2026-09-22T22-20-40-582Z_01a0cb34-f505-738b-bad3-7ab0c7b00692.jsonl`)
contains one message entry with `role: "system"`,
`content: ""`,
named `sections`,
and a `toolsAdded` list.

## Root causes

### Hand-copied transcript projection threw on a new role

Pi 0.87 persists the system prompt and tool loadout as `role: "system"` session messages
(`AgentSession._preparePromptAndToolLoadout` in `dist/core/agent-session.js`),
and a compaction entry can carry one as `entry.systemMessage`.
`package/pi-plugin/auto-mode/src/visible-context.ts` reimplemented Pi's
entry-to-message conversion by hand
and ended its per-role `if` chain with `throw new Error('Unsupported Pi session message role.')`.
That throw gave no compile-time signal when the union grew,
and at runtime it failed judge-context construction for every guarded call.

Pi's own interactive renderer handles the same union differently
(`addMessageToChat` in `dist/modes/interactive/interactive-mode.js`):
`case "system"` draws nothing,
and `default` is a compile-time exhaustiveness check that renders nothing at runtime.

`agent-session.js` is byte-identical between 0.87.0 and 0.87.1,
and both versions persist system messages,
so the 0.87.0 to 0.87.1 step did not introduce this behavior;
the workspace lockfile moved from 0.85.1 straight to 0.87.1.

### Direct provider dispatch sent no system prompt and no tools

pi-ai 0.87 provider modules take a `TranscriptContext`
whose prompt and tools live in transcript system messages,
and only `normalizeContext()` (`dist/utils/transcript.js`) builds one.
Public entry points normalize (`dist/models.js`,
`dist/compat.js`),
but the direct `api/*.lazy` modules that `package/pi-shared/model-review/src/provider-streams.ts`
calls do not:
`resolveTranscript` passes the context through unchanged.
model-review passed a raw `{ systemPrompt, messages, tools }` context,
so auto-mode and goal reviewer requests went out with no system prompt and no tools
while still forcing `tool_choice` to the verdict tool.

A local capture server proved the wire body.
For `anthropic-messages`,
the raw context sent `system` absent,
`tools` empty,
and `tool_choice: { type: "tool", name: "render_verdict" }`;
`normalizeContext(rawContext)` sent both.
The existing model-review tests passed throughout because their scripted transport bypasses provider dispatch.

### Type changes that tsc would have reported

- `SimpleStreamOptions.toolChoice` narrowed to provider-neutral `"auto" | "none"`.
  Provider modules still forward provider-specific selectors verbatim
  (`anthropic-messages.js` `buildParams`,
  `openai-responses.js`,
  `openai-completions.js`,
  `bedrock-converse-stream.js`).
- `ProviderHeaders` became `Record<string, string | null>`;
  `null` suppresses a provider default header of that name.

### Type checking was silently disabled repo-wide

`mise run //package/<path>:lint:types` exited 0 without running `tsc`,
so none of these type changes surfaced after the lockfile update.
See [`task-tsc-shared-chunk-silent-pass.md`](task-tsc-shared-chunk-silent-pass.md).

## Fixes

- `5dbade02f`:
  first targeted fix,
  treating `role: "system"` as hidden in `visible-context.ts`.
- `10220eb17`:
  `visible-context.ts` now calls Pi's exported `sessionEntryToContextMessages`,
  the step Pi's renderer runs.
  Pi's extension loader aliases `@earendil-works/pi-coding-agent` to the running host
  (`dist/core/extensions/loader.js`),
  so the projection always matches the host version.
  Unknown roles reach `omitUnknownRoleMessage(message: never)`,
  which fails `lint:types` when installed declarations add a role
  and logs plus omits the message at runtime.
  Pi's projection also normalizes `null` content from older session files,
  which previously threw a `TypeError`.
- `edd680e89`:
  restores `lint:types`.
- `d1f53ed92`:
  model-review normalizes the reviewer context at the provider boundary,
  types provider-specific `toolChoice` apart from `SimpleStreamOptions`,
  and adds `provider-request.unit.test.ts`,
  which captures real provider-module requests on a local server.
  `BudgetModelAuth.headers` in `package/pi-shared/model-selection/src/types.ts` accepts `null` values.
- `590014391`:
  `package/pi-plugin/advisor/src/advisor-client.ts` had the same defect through
  `ctx.modelRegistry.getProvider(...).streamSimple`,
  which returns the raw provider and skips the normalization `ModelRegistry.streamSimple` applies.
  A local capture showed an empty Anthropic `system` field without the fix
  and the advisor prompt with it.

## Verification

- `context.unit.test.ts` failed with the original error before the first fix
  and covers system messages,
  compaction system messages,
  an unknown role,
  and `null` content.
- Removing the `system` branch in `visible-context.ts` makes `lint:types` report
  `SystemMessage ... is not assignable to parameter of type 'never'`.
- Removing `normalizeContext` makes both `provider-request.unit.test.ts` cases fail.
- The rebuilt auto-mode dist builds judge context from the failing session file without error.

## Rejected approaches

- Keeping the hand-copied projection and adding one `system` branch:
  fixes today's role only and repeats the failure on the next additive union change.
- Serializing unknown roles into judge context by denylisting known-hidden fields:
  would forward full system prompts and any future hidden payload to the judge.
- Throwing on unknown roles as a fail-closed guard:
  omission only removes judge context and cannot fabricate authorization,
  while throwing blocks every tool call and pushes users to disable the guard.

## Type errors surfaced by restoring `lint:types`

A sequential sweep of every package `lint:types` task on 2026-09-22,
after `edd680e89`,
found further failures outside auto-mode and its shared dependencies.
These are fixed,
and every `package/pi-plugin` and `package/pi-shared` package now passes `lint:types`:

- `e8f76b7d2`:
  `src/mise.verify-extension.ts` fakes in advisor,
  guardrail,
  search-fetch,
  and thinking-default
  return an unsubscribe callback from `on`,
  accept the complete-provider `registerProvider` overload,
  and implement `registerMarkdownTransformer`.
- `c5d262a7b`:
  `package/pi-plugin/current-time-context/src/pi-test-harness.ts` supplies collection-complete
  `NormalizedBuildSystemPromptOptions`,
  since Pi does not export `normalizeBuildSystemPromptOptions`.
- `856330f26`:
  goal scripted tool calls use Pi AI `JsonObject` arguments,
  and ask-user-question annotates an exported constant for `--isolatedDeclarations`.

The remaining logger type-check failure was resolved under issue #254:
`package/done/app/src/lib/db-migrations.ts` no longer imports or awaits
`initPromise`,
which the lazy default logger does not export.

## Pi 1.3.0 `ExtensionAPI` member additions broke the verify fakes

### Symptom

`mise run //package/pi-plugin/advisor:lint:types`,
`mise run //package/pi-plugin/guardrail:lint:types`,
and `mise run //package/pi-plugin/thinking-default:lint:types` each failed
at the fake `ExtensionAPI` object in that package's `src/mise.verify-extension.ts`.
`tsc` reported `TS2740`
(measured at pre-fix `b710410e81`):

```text
src/mise.verify-extension.ts(203,9): error TS2740: Type '{ on(event: string): () => void; [...] }' is missing the following properties from type 'ExtensionAPI': registerToolRenderer, getSettings, registerMcpServer, unregisterMcpServer, and 3 more.
```

The message names four members and counts three more:
`getMcpServers`,
`registerVirtualModel`,
and `unregisterVirtualModel`.
Object literals inside these messages are elided as `[...]`.
The same fakes had drifted once before,
in the Pi 0.87 step recorded in
[Type errors surfaced by restoring `lint:types`](#type-errors-surfaced-by-restoring-linttypes)
(`e8f76b7d2`),
and the search-fetch fake already implemented the 1.3.0 member set
when its typed doubles landed (`78bbc27d5f`).

### Root cause

Installed `@earendil-works/pi-coding-agent` `1.3.0` grew `ExtensionAPI` additively,
and the installed declarations decide the member set
(`pi-coding-agent/dist/core/extensions/types.d.ts`,
reachable as
`package/pi-plugin/advisor/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/types.d.ts`):

```ts
// pi-coding-agent/dist/core/extensions/types.d.ts, version 1.3.0, selected declarations
    registerToolRenderer(resolver: ToolRendererResolver): void;
    getSettings(): Settings;
    registerMcpServer(name: string, config: McpServerConfig): void;
    unregisterMcpServer(name: string): void;
    getMcpServers(): RegisteredMcpServer[];
    registerVirtualModel<TState = unknown>(model: ExtensionVirtualModel<TState>): void;
    unregisterVirtualModel(provider: string, id: string): void;
```

These sit at `:1256`,
`:1286`,
`:1388` to `:1392`,
and `:1402` to `:1404` of that interface
(declared at `:1186`).
Each `src/mise.verify-extension.ts` builds a structural fake typed as `ExtensionAPI`
for the package's `verify:extension` probe,
so a fake implementing the old member set no longer satisfies the interface.

### Fix

Commit `c8d9af87d5` adds the seven members to the advisor,
guardrail,
and thinking-default
fakes as registration-recording stubs and empty returns,
mirroring `package/search-fetch/pi-plugin/src/mise.verify-extension.ts`.
Tradeoff:
the fakes are hand-maintained mirrors of `ExtensionAPI`,
so every additive release re-drifts them;
structural typing forces all members to exist even when the handler under test never calls them.

### Recurrence check

After any `@earendil-works/pi-coding-agent` bump run
`mise run //package/pi-plugin/advisor:lint:types`,
`mise run //package/pi-plugin/guardrail:lint:types`,
`mise run //package/pi-plugin/thinking-default:lint:types`,
and `mise run //package/search-fetch/pi-plugin:lint:types`.
`TS2740` naming `ExtensionAPI` means the fakes need the new members;
mirror the search-fetch fake again.

Upstream filing:
nothing to file;
the class decision in
[`deps-update-api-drift.md`](deps-update-api-drift.md)
covers this incident.

## Pi 1.3.0 prompt options and the `before_agent_start` event split

### Symptom

Two packages failed for one dependency change
(measured at pre-fix `b710410e81`).

`mise run //package/pi-plugin/current-time-context:lint:types` reported `TS2322`
and `TS1360` in `src/pi-test-harness.ts`:

```text
src/pi-test-harness.ts(213,5): error TS2322: Type '{ cwd: string; selectedTools: never[]; [...] }' is not assignable to type 'NormalizedBuildSystemPromptOptions'.
  Property 'hiddenTools' is missing in type '{ cwd: string; selectedTools: never[]; [...] }' but required in type '{ selectedTools: string[]; hiddenTools: string[]; [...] }'.
src/pi-test-harness.ts(225,5): error TS1360: Type '{ type: "before_agent_start"; prompt: string; [...] }' does not satisfy the expected type 'BeforeAgentStartEvent'.
```

`tsc` reports the `TS2322` at `:213` alone while `hiddenTools` is missing;
supplying that member surfaces `TS2322` on the returned event
and the `TS1360` at the `satisfies BeforeAgentStartEvent` clause (`:225`).
Both codes come from this one drift.

`mise run //package/pi-plugin/terminal-title:lint:types` reported
`TS2769`,
`TS2344`,
`TS2322`,
and `TS2352`
in `src/index.ts` and `src/index.unit.test.ts`:

```text
src/index.ts(304,5): error TS2769: No overload matches this call.
  The last overload gave the following error.
    Argument of type '"before_agent_start"' is not assignable to parameter of type '"input"'.
src/index.ts(306,51): error TS2344: Type '"prompt"' does not satisfy the constraint '"messageType" | "systemPrompt" | "systemPromptOptions" | "type"'.
src/index.ts(314,13): error TS2322: Type 'unknown' is not assignable to type 'string'.
src/index.unit.test.ts(239,11): error TS2352: Conversion of type '{ type: "before_agent_start"; prompt: string; systemPrompt: string; systemPromptOptions: never; }' to type 'BeforeAgentStartEvent' may be a mistake because neither type sufficiently overlaps with the other. If this was intentional, convert the expression to 'unknown' first.
  Property 'messageType' is missing in type '{ type: "before_agent_start"; prompt: string; systemPrompt: string; systemPromptOptions: never; }' but required in type 'BeforeAgentStartUserEvent'.
```

At runtime the terminal-title handler titled custom-triggered runs
`π Received prompt: undefined`,
because it read `event.prompt` from every run.

### Root cause

Installed `@earendil-works/pi-coding-agent` `1.3.0` changed two shapes,
and its installed declarations decide both.
`NormalizedBuildSystemPromptOptions` now requires `hiddenTools: string[]`
(`pi-coding-agent/dist/core/system-prompt.d.ts:39`,
optional on `BuildSystemPromptOptions` at `:16`):

```ts
// pi-coding-agent/dist/core/system-prompt.d.ts, version 1.3.0, selected declarations
    hiddenTools?: string[];
export type NormalizedBuildSystemPromptOptions = BuildSystemPromptOptions & {
    hiddenTools: string[];
```

`BeforeAgentStartEvent` split into user and custom variants behind a `messageType`
discriminant
(`pi-coding-agent/dist/core/extensions/types.d.ts:728` to `:742`,
reachable as
`package/pi-plugin/terminal-title/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/types.d.ts`):

```ts
// pi-coding-agent/dist/core/extensions/types.d.ts, version 1.3.0, selected declarations
export interface BeforeAgentStartUserEvent extends BeforeAgentStartEventBase {
    messageType: "user";
    prompt: string;
    images?: ImageContent[];
}
export interface BeforeAgentStartCustomEvent extends BeforeAgentStartEventBase {
    messageType: "custom";
    message: CustomMessage;
}
export type BeforeAgentStartEvent = BeforeAgentStartUserEvent | BeforeAgentStartCustomEvent;
```

`prompt` now exists on the user variant only,
so the terminal-title handler parameter typed
`Pick<BeforeAgentStartEvent, 'prompt'>` stopped type-checking
and `pi-test-harness.ts` could neither supply `hiddenTools`
nor construct an event without `messageType`.

### Fix

Commit `fce8efd40e` supplies `hiddenTools: []` and `messageType: 'user'` in
`package/pi-plugin/current-time-context/src/pi-test-harness.ts`,
the only construction site for both shapes in that package.
Commit `ad7a5a67b4` retypes the terminal-title handler parameter as
`Readonly<BeforeAgentStartEvent>`,
narrows with `event.messageType !== 'user'`,
and returns without a title for custom runs,
with a new unit case covering the custom variant.
Tradeoff:
custom-triggered runs no longer get a prompt-based title
(they previously got the `π Received prompt: undefined` title),
while user runs keep their exact titles;
defaulting a placeholder title would fabricate a prompt that does not exist.

### Recurrence check

After any `@earendil-works/pi-coding-agent` bump run
`mise run //package/pi-plugin/current-time-context:lint:types`,
`mise run //package/pi-plugin/terminal-title:lint:types`,
and `mise run //package/pi-plugin/terminal-title:test:unit`.
`TS2344`,
`TS2352`,
`TS1360`,
or `TS2322` naming `BeforeAgentStartEvent` or
`NormalizedBuildSystemPromptOptions` means the shapes moved again;
read the installed declarations for which members each variant carries
before touching handlers.

Upstream filing:
nothing to file;
the class decision in
[`deps-update-api-drift.md`](deps-update-api-drift.md)
covers this incident.

## Open questions

- Provider-specific `toolChoice` relies on provider modules forwarding a value their
  `SimpleStreamOptions` type no longer declares.
  Switching model-review to per-API `stream()` options would make it typed,
  at the cost of importing non-lazy provider modules.
- `package/deps-update/cli` runs no type check after a dependency bump,
  so a restored `lint:types` still depends on someone running it.
  The 2026-10-20 drift batch this caused is recorded in
  [`deps-update-api-drift.md`](deps-update-api-drift.md).
