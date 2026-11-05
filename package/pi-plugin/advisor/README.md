# Pi Advisor

Pi Advisor adds an `advisor` tool and `/advisor` command to Pi.
It sends a captured conversation snapshot to reviewers selected from the current effective Pi model scope.
Default calls recover serially from failures;
 explicitly enabled overlap collects completed reviews together within a bounded grace period.
Default selection is not a guarantee of review quality.

This differs from Claude Code Advisor.
Claude Code uses an Anthropic server-side beta tool.
Pi Advisor is a local Pi extension,
 works across providers,
 and calls the secondary model through Pi's model registry.

## Pi dependency trust

PJO:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Install

Build the package,
 then load the extension from package settings or the CLI:

```json
{
  "packages": ["./package/pi-plugin/advisor"]
}
```

```bash
pi --extension ./package/pi-plugin/advisor/dist/final/node/index.mjs
```

## Tool usage

Use empty params to start with the output-eligible scoped model with the highest expected Advisor call cost,
 excluding the current main model when another eligible scoped model is available.
Failures can move the call to another eligible scoped model:

```json
{}
```

Use a focused question when the main model wants Advisor to answer a specific uncertainty,
not only provide general review feedback:

```json
{ "question": "Did I miss any verification before declaring this done?" }
```

Use an explicit scoped model slug when a specific reviewer model is needed,
including when you intentionally want the current main model:

```json
{ "model": "anthropic/claude-opus-4-7" }
```

Combine both fields when the question should go to a specific model:

```json
{
  "model": "anthropic/claude-opus-4-7",
  "question": "Which assumption in this plan is weakest?"
}
```

Accepted explicit forms are:

- canonical slug:
   `provider/modelId`;
- bare `model.id`,
   only when unique inside the effective scope;
- `model.name`,
   only when unique inside the effective scope.

Advisor throws when a slug is ambiguous,
 unknown,
 present in the global registry but outside the effective scope,
 or backed by an endpoint advertising fewer output tokens than `maxAdvisorOutputTokens`.
The error lists allowed eligible scoped slugs.

## Slash commands

- `/advisor`:
   run an immediate review with the default non-current scoped model when available.
- `/advisor <slug>`:
   run an immediate review with a specific scoped model.
- `/advisor status`:
   show enablement,
   scope source,
   scoped slugs,
   output-eligible slugs,
   default model,
   config paths,
   serial recovery,
   overlap enablement,
   collection timing,
   and billing uncertainty.
- `/advisor off`:
   disable Advisor for the current session and remove the tool from active tools.
- `/advisor on`:
   re-enable Advisor for the current session.

There is no `/advisor set` command.
Persistent model overrides would bypass scoped tool-parameter selection.

## Configuration

Global config path:

```text
~/.pi/agent/extensions/pi-advisor.json
```

Project config path:

```text
.pi/extensions/pi-advisor.json
```

Example:

```json
{
  "enabled": true,
  "timeoutMs": 600000,
  "hedgingEnabled": false,
  "collectionGraceMs": 30000,
  "maxAdvisorOutputTokens": 32000,
  "includePriorAdvisorResults": true,
  "systemPrompt": "Focus on test coverage gaps and incorrect assumptions."
}
```

`timeoutMs` bounds one complete Advisor operation,
 including local preparation,
 failure fallback,
 bounded no-text recovery,
 and collection.
The original deadline is never reset.
Caller cancellation ends the operation;
 deadline expiry returns already-collected usable reviews or fails if none exists.

`hedgingEnabled` defaults to `false`.
Enabling it requires an explicitly configured `hedgeDelayMs`.
`collectionGraceMs` defaults to `30000`.
These scheduling durations and `timeoutMs` must be positive integers no greater than `2147483647`.
Project configuration can disable overlap without erasing an inherited launch delay.

For example,
 this explicitly opts into overlap after a configured launch delay:

```json
{
  "hedgingEnabled": true,
  "hedgeDelayMs": 60000,
  "collectionGraceMs": 30000
}
```

The example launch delay is a configuration value,
 not a measured latency optimum.

`maxAdvisorOutputTokens` defaults to `32000`.
Advisor requests that response budget and excludes every model endpoint whose advertised `maxTokens` is lower.
A model advertising exactly the configured value remains eligible.
Explicit model requests receive an eligibility diagnostic before provider dispatch.
The provider client repeats the assertion at its final dispatch boundary.

`maxContextChars` is optional.
When omitted,
 Advisor derives the serialized-context budget from the selected model's context budget.
When present,
 it caps the model-derived budget.
Advisor starts from Pi's compaction-aware context entries,
 so messages summarized by the latest compaction are not resent.
Pi-loaded project context files are captured separately from
`before_agent_start.systemPromptOptions` and retained across automatic
compact-and-continue runs that do not emit another prompt event.
The snapshot is replaced authoritatively on each later agent run and cleared at
the session boundary.
Manual `/advisor` calls read current prompt options directly after pending work
settles.

Project context is JSON-encoded and appended to the Advisor system prompt,
so `AGENTS.md`,
 global context,
and ancestor context instructions remain available even when conversation
messages containing earlier file reads were compacted.
The selected model's context budget includes this expanded system prompt.

Project config overrides global scalar values.
Preference ranking is not configurable here.
The initial model comes from empty params or the explicit `model` tool parameter;
 default failure recovery remains constrained by the operation's eligible scoped candidates.

## Default recovery and bounded collection

Without overlap,
 default calls try candidates serially until a usable review arrives,
 candidates are exhausted,
 or the original deadline expires.
With overlap enabled,
 the launch delay starts with the first logical reviewer call,
 including authentication.
If that review is unfinished when the delay expires,
 another candidate can start.
At most two logical reviewer calls are active,
 including their authentication and preparation.
Failures before any usable review can be replaced by untried candidates within the same deadline.
Prefer a different provider for the alternate;
 another model on the same provider is allowed when no different-provider candidate remains.

After the first usable review arrives:

- Stop starting replacements or retries.
- Give every already-started reviewer,
   including one still authenticating,
   the configured collection grace.
- Return all usable completed reviews together when those calls settle.
- At the earlier of grace expiry and the original deadline,
   request cancellation of remaining calls and return the usable reviews collected so far.
- Do not extend the wait to obtain cancellation acknowledgement.

A successful `stop` or `length` response with non-whitespace visible text is operationally usable.
Length-limited output is labelled.
This does not assess semantic review quality.
A successful empty response can retry once on the same model,
 but only before any usable review exists.
Provider errors,
 independent provider aborts,
 deferred responses,
 and unavailable tool-use outcomes are not usable reviews.
Provider-internal retries are explicitly disabled through the supported `maxRetries` option;
 recovery dispatches remain visible to the operation ledger.

Explicit model requests remain exact:
 no cross-model fallback and no overlap,
 even when overlap is enabled globally.

## Exhausted-credit providers

Advisor detects supported credit-exhaustion diagnostics from failed requests just in time.
It does not query balances before dispatch.
After observing exhausted credits,
 it excludes every model on that registered provider for the current operation.
The dispatch gate checks again after authentication,
 so a request still preparing cannot bypass a newly discovered exclusion.
Other providers remain eligible.

Already-running requests are not discarded merely because another request reported exhausted credits.
Their usable results can still be collected.
The provider exclusion is not persisted into the next Advisor call.
A generic HTTP 402 or unrelated payment error is not sufficient evidence of exhausted credits.

## Accounting and progress

Tool progress contains model identities,
 attempt states,
 context sizes,
 reasoning level names,
 and remaining timing bounds.
Partial updates exclude provider diagnostics,
 serialized evidence,
 and review text.
Final results retain the operation ledger and display every attempted model.

Each attempt contributes its latest available usage once.
Reasoning tokens are a subset of output tokens,
 not another contribution to total tokens.
Successful and failed tool results expose aggregate nested usage through Pi's top-level tool usage field.
Failed operations also persist a `pi-advisor.operation` entry.
Slash-command successes retain the same ledger and aggregate in custom-message details;
 failures retain them in a custom operation entry.

Usage from a cancelled or failed call can be incomplete.
Cancellation requests do not guarantee remote termination or prevent billing.
The completed reviewer set is nondeterministic when overlap is enabled.

## Effective scoped model set

Advisor resolves scope in this order:

1. Live scoped models from Pi extension context,
    if Pi exposes them.
2. Startup `--models` patterns from `process.argv`.
3. Merged Pi `enabledModels` settings from global and project settings.
4. `ctx.modelRegistry.getAvailable()` when no restricted scope exists.

Advisor then keeps only entries whose endpoint advertises
`model.maxTokens >= maxAdvisorOutputTokens`.
Provider-specific entries are evaluated independently,
so one endpoint for a model can remain eligible while another endpoint for the same model is excluded.

Pi 0.74 does not expose live session-only `/scoped-models` changes in the typed extension context.
Advisor probes for a future runtime API,
 but first delivery reconstructs startup and settings scope when that API is absent.
Exact live `/scoped-models` support needs a Pi API such as `ctx.getScopedModels()`.

## Default model ranking

For `advisor({})`,
 Advisor first excludes endpoints below the configured output requirement.
It then removes the current main model from default candidates when another eligible scoped model remains,
 estimates input tokens for the serialized request,
 and computes:

```text
expectedCost = inputTokens * model.cost.input + maxAdvisorOutputTokens * model.cost.output
```

Cache prices are ignored.
Ties break by higher output cost,
 higher input cost,
 larger context window,
 then canonical slug lexical order.

## Reasoning effort

Every Advisor call uses the highest reasoning level below `max` advertised by the selected model.
Models without extended levels use `high`.
Models that advertise `xhigh` use it,
while models that advertise `max` without `xhigh` remain at `high`.
Advisor never requests `max` or a later level because `max` calls frequently time out.
Non-reasoning models receive no reasoning option.
Advisor passes the selected level through pi-ai's unified simple API,
which maps it to each provider's request format.

## Privacy and cost

Advisor sends the serialized conversation and Pi-loaded project context to every attempted reviewer,
 including failure fallbacks and overlapping calls.
That can include prompts,
 tool calls,
 tool results,
 edits,
 command output,
 compaction summaries,
global context files,
ancestor context files,
absolute context-file paths,
and repository `AGENTS.md` contents.
Each Advisor call can incur provider cost for the selected model.

## Troubleshooting

### Empty scope

`advisor: no scoped models with configured auth` means no scoped model has usable auth.
Check `--models`,
 `enabledModels`,
 `/scoped-models`,
 and provider login.

### Ambiguous slug

A bare id or display name matched more than one scoped model.
Use the canonical `provider/modelId` slug from `/advisor status`.

### Out-of-scope slug

The requested model exists globally but is not in the effective scope.
Change the tool argument to one of the listed scoped slugs,
 or change Pi model scope before starting the session.

### Missing auth

Auth errors come from `ctx.modelRegistry.getApiKeyAndHeaders()`.
Log in to the provider or configure the provider's API key in Pi.

### Insufficient output capacity

`advisor: no scoped models advertise at least ... output tokens` means every scoped endpoint advertises a
`maxTokens` value below `maxAdvisorOutputTokens`.
Lower the configured budget or add an endpoint advertising enough output capacity.

An explicit ineligible model reports both configured requirement and endpoint advertisement.
Use an eligible slug from `/advisor status`.

### Context truncation

Advisor derives an effective serialized-context budget from the selected model's context budget.
It reserves tokens for the Advisor system prompt,
 provider framing overhead,
 and `maxAdvisorOutputTokens`.
When serialized context exceeds that effective budget,
 Advisor keeps the head and tail and inserts an omission marker.
Set `maxContextChars` only when a project needs a lower hard cap than the selected model allows.

FLK:
 One provider "Context limit exceeded" isn't a stable limit:
 measure repeated same-input outcomes before lowering global context budgets;
 prefer model-aware budgeting.

## Verification

Package checks:

```bash
# package/pi-plugin/advisor/README.md
mise run //package/pi-plugin/advisor:build
mise run //package/pi-plugin/advisor:test:unit
mise run //package/pi-plugin/advisor:lint:types
mise run //package/pi-plugin/advisor:lint:oxlint
mise run //package/pi-plugin/advisor:verify:extension
mise run //package/pi-plugin/advisor:verify:host
```

`verify:host` loads the built extension in disposable Pi homes with only faux providers.
It verifies actual tool results,
 top-level usage,
 metadata-only progress,
 straggler cancellation,
 and successful and failed slash-command records.
It supplies no ambient credentials or live conversation,
 bounds captured output,
 and rejects child stderr.
The dedicated environment guard is set by the task itself.
