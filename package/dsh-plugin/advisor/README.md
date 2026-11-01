# `@monochromatic-dev/dsh-plugin-advisor`

In-process Cordis plugin that brings the pi Advisor,
an independent reviewer for
the primary coding agent,
to the DeepSeek Harness through the shared Advisor
core in `package/agent-harness-shared/advisor` and the DSH host services.

The package is `private: true` and installs as a Harness bundle patch
(`dsh.bundle.patch` points at `./cordis.patch.yml`).

`engines.dsh` declares the verified host version `0.2.1-rc.5` explicitly.
The exact prerelease limits the compatibility claim to the tested host;
revalidate activation before changing it.
DSH reads this field before peer dependencies,
so local directory links can retain the workspace's `catalog:` dependency specifications.
See the [startup diagnosis](../../../doc/troubleshooting/dsh-rc5-profile-startup.md).

## Host API surface

`src/index.ts` exports the plugin contract the Harness Loader consumes:

- `name`:
  `'advisor'`.
- `inject`:
  `tools`,
  `commands`,
  `llm`,
  `systemPrompt`,
  `workingDirectory`.
- `Config`:
  an empty Schemastery object row.
  Structural fields only;
  every
  behavioral setting lives in the unified `pi-advisor.json` files.
- `apply(ctx)`:
  one async `ctx.effect` that loads the enablement seed,
  builds
  the session state,
  and registers the tool,
  the command,
  and the
  `tool:advisor` system-prompt section (order `2_500`,
  between the tool-goal
  and tool-workflow orders).
  The effect disposer unregisters all three and
  disposes any adopted tool mask.

All review work runs against `ctx.llm` (the `LlmRuntime` surface:
`listProviders`,
`listModels`,
`resolveModelInfo`,
`stream`).
The pi-ai type
vocabulary is imported type-only;
no pi runtime module is imported by this
package's own code.

## Tool and command parity

The `advisor` tool and the `/advisor` command share one execution core
(`executeDshAdvisorToolCall`,
`runDshAdvisorCommand`,
`runDshAdvisorReview`),
so both forms resolve models,
build context,
and render results identically.

- `advisor` tool arguments:
  `model` (exact scoped reviewer slug,
  model id,
  or
  unique model name) and `question` (focused review question).
  The canonical
  JSON tool value carries `text` (review prose),
  `details` (operation ledger,
  attempted models,
  aggregate usage),
  and `progress` (the folded progress
  paragraph).
  The result renderer leads with the operation summary when the
  details parse as an operation view,
  falls back to raw text for unrenderable
  replays,
  and always appends the progress tail.
- Tool calls run sequentially (`isConcurrencySafe` returns `false`),
  matching
  pi's `executionMode: 'sequential'`,
  because default recovery mutates shared
  operation state.
- `/advisor` forms:
  bare (default reviewer review),
  `/advisor <slug>` (exact
  reviewer),
  `/advisor status`,
  `/advisor off`,
  `/advisor on`.
  `off` masks the
  tool through `ctx.tools.restrict` and records the disposer;
  `on` lifts the
  mask.
  Cancellation reports `Advisor review cancelled.`;
  failures return the
  error text,
  and every completed operation persists its ledger as a durable
  `plugin:advisor/operation` plugin record.

The tool description and prompt section carry the pi prompt snippet plus the
four prompt guidelines unchanged.
One text deviation is deliberate:
the
description says the default pick is the `largest-context-window` model,
where
pi says `highest expected-cost`,
because the DSH ranking policy orders by
context window.

## Config locations

Behavioral settings live only in the unified JSON files,
merged global under
project:

- Global:
  `~/.pi/agent/extensions/pi-advisor.json`.
- Project:
  `.pi/extensions/pi-advisor.json` under the session working
  directory.

The merged document includes `enabled` and `defaultReviewerModel`.
The
`defaultReviewerModel` value is user-owned configuration (for example
`opencode-go/kimi-k3`);
it is never a schema default and never written to
`~/.pi/agent/settings.json`.
The plugin enablement seed is read once at
activation from `process.cwd()`;
each operation re-reads the merged config for
the session's own working directory.

## Reviewer selection order (Q12)

Selection prefers,
in order:

1. An explicit `model` argument or `<slug>` command form:
   exact scoped match
   only,
   never a fallback.
2. `defaultReviewerModel` when it resolves inside the current scope:
   it picks
   the initial reviewer of a default call.
3. The variant fallback ranking supplied through the Stage 2b seam
   (`rankDefaultAdvisorCandidates` in the shared core):
   DSH ranks scoped
   candidates by largest `contextWindow`,
   then model `name`,
   then canonical
   slug.
   The selection reason names only the winning window;
   the full
   observed-window roster goes to the debug decision log,
   so model-visible
   output never enumerates every scoped model.

Default calls drop the current main model from the candidate scope before the
ranking:
exclusion matches route slugs and provider-independent identity
(model id plus display name),
so one model served on two routes is not
consulted as its own reviewer.
Ids repeat across unrelated models,
so identity
matching without a display name stays route-exact.

Serial fallback across reviewers and the overlap suppression are unchanged
from pi.

## Recorded behavior picks

- Unknown context window:
  `128_000` tokens.
- Output budget:
  `maxAdvisorOutputTokens` is a request budget,
  never an
  eligibility floor.
  The request clamps to `min(budget, advertised capacity)`;
  a route disclosing no capacity (or a non-positive one) receives the
  configured budget unclamped.
- Reasoning:
  efforts named or identified `max` are never sent;
  the greatest
  remaining effort is sent,
  and `reasoningEffort` is omitted when a route
  advertises none (Q5 pick).
- Credits:
  `QUOTA` and `ACCOUNT_QUOTA` failures append the
  `insufficient_quota` marker so provider exclusion keeps pi parity.
- Usage (Q6 pick):
  provider totals are preserved;
  absent totals fall back to
  the disjoint-counter sum;
  cost is always zero;
  reasoning tokens are passed
  through only when reported.
- `getAvailable` equals `getAll`;
  `getApiKeyAndHeaders` always answers
  `{ok: true}` because `ctx.llm` resolves credentials.

## Reviewer-fallback decision (2026-10-20)

Tier 2 of the reviewer-fallback chain is the initial reviewer of the call:
a
default call starts at the resolved default (or fallback) reviewer and only
then walks the ranked list.
This decision is veto-flagged in the planning
record and mirrored in the DSH variant so both harnesses share one recovery
behavior.

## Capability gaps and fallbacks

- Derived assistant history carries no provider accounting:
  converted
  messages get a fabricated zero `Usage`,
  a `stopReason` of `toolUse` or
  `stop`,
  and the snapshot timestamp.
- Usage-only stream updates publish empty text deltas to the completion seam.
- `AssistantMessageEventStream` is nominal;
  `provider.ts` asserts the structural
  `DshAdvisorStreamSurface` to that type once at the shim boundary.
- `auth.resolve` is a stub returning `undefined`;
  credentials arrive through
  `ctx.llm`.
- Image and file blocks render as placeholder text
  (`[image: name (WxH, mime)]`,
  `[file: name (N bytes)]`) because the review
  prompt consumes text only.
- Progress from `onUpdate` snapshots folds into the result text tail.
- Completed operations persist their ledger through a durable
  `plugin:advisor/operation` plugin record (an ignorable session event every
  reader may skip) instead of pi's session-persistence
  handle;
  the session record stores rendered tool text only,
  and non-message
  events never enter `deriveMessages()`.
- The session toggle and the tool mask are plugin-process scoped
  (`ctx.tools.restrict` is global to the plugin instance).
- `advisorDetailsAsJsonValue` normalizes details through a JSON pass so the
  readonly shared shapes become mutable `JsonValue` for the tool schema.
- The shared core imports `estimateTokens` from `@earendil-works/pi-coding-agent`
  and `getSupportedThinkingLevels`/`normalizeContext` from
  `@earendil-works/pi-ai` at runtime;
  both are therefore peer and dev
  dependencies here even though this package's own code is type-only against
  pi.
- DSH exposes no pi-style scoped-model cycle list:
  the run context maps the
  live scope to the catalog's `getAvailable()` routes,
  so every resolved route
  with runtime-owned auth is in scope.
- `/advisor` commands dispatch only through the host app:
  agent sessions expose
  no command-invocation surface,
  and a leading-slash task line arrives as an
  ordinary user message,
  so command parity is verified at the user boundary.
- There is no `verify:extension` analogue in `mise.toml`;
  live
  `plugin_manager` install and fixture-profile verification are owned
  outside this package.

## Development

```bash
# package/dsh-plugin/advisor
mise run //package/dsh-plugin/advisor:build
mise run //package/dsh-plugin/advisor:lint:types
mise run //package/dsh-plugin/advisor:lint
mise run //package/dsh-plugin/advisor:test:unit
```

Tests import the built artifact from `../dist/final/node/index.mjs` and cover
every exported module;
test helpers follow `testing-practices`.
