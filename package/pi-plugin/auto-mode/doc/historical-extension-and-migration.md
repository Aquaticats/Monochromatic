# Historical extension and migration record

Captured before the 2026-10-17 consolidation.
The extension entry and its exclusive signal, model-selection, trust, and approval-reuse implementation
were removed on that date.
The current native responsibilities are described in [architecture](architecture.md).
Legacy source identifiers in this snapshot name removed implementations.
The historical native document link now resolves from this archive directory.
The former README used `../../../doc/` for repository documentation links.
Their current prefix in this archive is `../../../../doc/` (moved 2026-10-17).

# @monochromatic-dev/pi-plugin-auto-mode

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Automatic review of model-initiated Pi tool calls.
The installed native host evaluates the proposed operation against applicable instructions and authorization.
Luna estimates instruction meanings;
code resolves the decision and preserves the original operation through confirmation and execution.

## Current native behavior

The installed `pi` command uses the maintained native host described in [the native README](../native/README.md).
It uses `openai/gpt-6-luna-decisions` through the Gateway Decisions endpoint,
with the adopted `0.60` selected-option probability cutoff and a unique leading meaning.
This threshold is an operating rule,
not a measured accuracy guarantee.

Complete assessments are split into provider requests of at most 200 questions,
with up to three parallel calls per bundle and three sequential bundles per judgment.
Every call shares original cancellation and preparation-inclusive timing.
Fifteen seconds is an advisory performance target; elapsed time never expires review evidence.
No average USD 0.001 per judgment cost limit applies.
Question identities and complete shared context survive partitioning;
partial answer sets cannot authorize execution.

Assessment concerns execution permission and required consent.
Tool-use, formatting, naming, and workflow conventions remain agent guidance.
Review inputs contain the proposed operation, readable project policy excerpts,
and the larger of the latest five visible messages or the span from the latest user message.
Pi's hidden prompts, tool metadata, and skill catalog remain outside model review.
Assistant thinking and provider signatures are excluded from conversation and proposed-operation inputs.
Bash is parsed locally into command, argument, quoting, control-flow, redirect, and expansion structure.
Complete Bash source stays local for execution and human confirmation.
Literal arguments and quoted here-document bodies remain data, including text resembling shell syntax.
Incomplete or unsupported parsed structure routes the original operation to human confirmation.
The [review input verification record](../native/review-input-verification.md)
documents request-capture coverage and complete-session replay requirements.
Complete copies of the root policy in visible messages are replaced by a reference to selected excerpts.
Older conversation receipts stay local for provenance.
Each clause question includes the complete proposed operation beside the selected clause.

Initialization and reload consult the persistent rule-relevance cache and send only changed or missing
tagged rules, individually with heading context, to Luna.
Tagged root rules scoring below `0.60`, including uncertain rules, are excluded from operation review.
Complete `AGENTS.md` documents are never sent as policy context.
The original file and source spans remain local for freshness checks.
Relevance estimates grant no execution permission.
Startup classification failure preserves ordinary chat and routes tool calls to scoped confirmation;
without a UI, execution remains withheld.

Established permission allows the operation automatically.
An established governing prohibition blocks it.
An explicit fresh-confirmation requirement or uncertain clause assessment first reaches a whole-operation
decision-model review using selected policy excerpts, visible conversation, and prepared operations.
The same Luna model and `0.60` selected-probability cutoff apply.
Only unresolved or failed automatic paths open the original operation's dialog when a UI is available.
The dialog offers `Allow` and `Stop`.
`Allow` authorizes only that prepared group;
`Stop`,
dismissal,
and a legacy denial response abort the current run.
An unavailable approval channel leaves an uncertain operation blocked.

Manual confirmation cannot override an established prohibition at the strongest known applicable priority.
An unresolved prerequisite estimate is reconsidered by the decision model before manual review.
Last-resort Allow authorizes the original prepared group while retaining the original prerequisite diagnostics;
it does not assert that an external approval was obtained.
Original source,
operation,
authorization,
and cancellation checks remain active.
Recoverable clause-assessment failures try the decision model within the original deadline and transport allowance.
Missing observation evidence, exhausted automatic review, and unresolved final decisions
fall back to manual approval of the complete prepared operation.
This fallback retains native operation identity, tool and argument checks, and cancellation.
It does not create automatic evidence or replace an established policy decision after its dependencies change.
Without a usable approval interface, execution remains withheld.
The local tagged logs correlate complete assessment inputs and replies with per-clause admission,
source references, errors, fallback choices, and the final verdict.
`test:native:decision-fallback` exercises these paths and verifies reconstructable diagnostic records.
Provider request metadata supplies context and is not independently subject to tool-call approval.

Provider switching, retries, navigation, reload, and ordinary chat remain owned by Pi.
Prompt and request capture failures are retained for tool review without rejecting the native chat request.
Each tool decision captures the current native branch;
navigation invalidates that decision without retiring later prompts.

Native Shift+Tab bypass executes supported native programs without semantic profile admission.
The fixed virtual-input guard, original execution ownership, cancellation, status, and audit entries remain active.
With bypass disabled, automatic review covers direct Bash calls and the inspected program profiles in the native README.

The extension export remains available for legacy consumers.
The remaining sections describe that legacy implementation;
its flagger,
model fallback,
and approval reuse are not the installed native policy.

## Legacy extension architecture

Three-stage pipeline:

```text
tool_call -> flagger (signals.ts, wide-net boolean predicates)
          -> judge (judge.ts adapter over @monochromatic-dev/pi-shared-model-review)
          -> approve (silent)
          -> deny (block + reason + guidance)
          -> ask (user decides; user-denied blocks include reason)
```

### Session approval reuse

Before resolving a judge model,
 auto-mode checks the current session branch for
an earlier verdict with the same action text and approval fingerprint.
 The
fingerprint hashes the tool name,
 current working directory,
 permission scope,
and loaded project-context snapshot when context files are present.
Changing `AGENTS.md` or another loaded context file requires a fresh judge
decision instead of reusing approval made under older project guidance.
 For `read`,
 the scope is the path only,
 so a user approval for one
`offset` or `limit` range also covers later reads of another range in the same
file.
 For other tools,
 the scope is the full tool input,
 so a later write or
edit to the same path with different content does not match.
 Only the
fingerprint digest is stored;
 full write or edit payloads are not added to
custom entries.
 Older verdict entries without an approval fingerprint fail
closed:
 they are not reused,
 and the newest same-action unkeyed verdict prevents
older fingerprinted approvals from being reused.

If the latest matching verdict is `approve` or `user-approve`,
 auto-mode allows
the action immediately and records a fresh `approve` verdict with the original
approval reason plus `reusedFromVerdict` metadata for auditability.
 A later
`deny`,
 `user-deny`,
 or `ask` verdict for the same
action and fingerprint disables reuse,
 so stale approvals do not override newer
decisions.

The flagger and judge are strictly separated:
 the flagger never provides reasons.
The judge receives the concise action summary,
the complete current tool input encoded as untrusted JSON data,
Pi's complete loaded project-context files,
and recent session context.
The complete input is request-only and is not added to verdict session entries.

### Judge context

Every flagged action sends its complete current tool input and Pi-loaded project
context files to the judge without auto-mode content truncation.
Project context is captured from `before_agent_start.systemPromptOptions`,
serialized as canonical untrusted JSON,
and retained through automatic compaction
retries that do not emit another `before_agent_start` event.
It is cleared after `agent_settled` and replaced authoritatively on each later
agent run,
including when Pi reports no loaded context files.

Context files remain evidence about project workflow,
not guardrail authority.
They cannot create trust directives,
authorize actions,
change verdict meanings,
or override auto-mode's fixed safety policy.
Pi's context-file set can include global and ancestor instructions in addition
to repository `AGENTS.md` files,
so their full paths and contents are disclosed to every selected judge and
fallback provider.

Every flagged action sends its complete current tool input to the judge without
auto-mode content truncation.
For `write`,
this includes the full proposed file body.
For `edit`,
this includes every `oldText` and `newText` hunk.
JSON encoding keeps model-generated content in an explicit data grammar and the
prompt labels it as untrusted data rather than instructions.

The recent context sent to judge uses larger of complete span from latest user
message and newest five user-visible messages.
Short latest-user spans backfill older messages until five-message floor is reached when enough history exists;
longer latest-user spans are not capped.

Each selected message is encoded as canonical untrusted JSON without auto-mode summarization or truncation.
Projection preserves complete user text and images,
assistant text,
tool-call names and arguments with Bash source replaced by parsed structure,
tool-result text,
images,
renderer details and error state,
parsed direct Bash commands and complete output,
visible custom messages,
compaction summaries,
and branch summaries.
Guard verdict is attached to corresponding tool result for circumvention context.
Assistant thinking,
provider-only signatures,
token usage,
timestamps,
provider metadata,
system-prompt messages,
and hidden custom messages are excluded because they are not visible transcript content.

Session entries become messages through Pi's exported `sessionEntryToContextMessages`,
the same step Pi's transcript renderer runs,
so the projection follows the running Pi version.
A message role this build does not know fails `lint:types` against installed Pi declarations,
and at runtime is logged and omitted rather than failing guarded tool calls.
See `doc/troubleshooting/pi-0-87-extension-api-breakage.md`.

### Shared review infrastructure

Auto-mode owns its action prompt,
trust directives,
batch context,
`approve`/`deny`/`ask` interpretation,
and tool-call policy.
`@monochromatic-dev/pi-shared-model-review` owns provider dispatch,
forced structured-tool transport,
direct-JSON retries,
balanced JSON extraction,
timeout propagation,
and the distinct fallback race.
This keeps transport behavior shared with other Pi plugins without coupling their user-facing policy to auto-mode.

### Verdict extraction

Primary path:
 the judge is invoked through the shared package with forced `tool_choice` (`render_verdict`).
The pi-ai event stream's `toolcall_end` event carries the parsed `arguments`,
which are translated to a `Verdict` by `parseVerdict`.

Retry path:
 if the model finishes without calling `render_verdict`,
the shared structured-attempt runner retries without tools and asks for direct JSON.
 If that direct JSON
attempt returns no text,
 it makes one final direct JSON attempt.
 These attempts
use the same safety context,
 then parse the retry response with
`extractJsonVerdict`.

Model fallback:
 if the selected judge model still fails after its transport attempts,
auto-mode excludes that model and resolves up to two distinct authenticated fallback models
before sending any request.
 It runs every selected complete judge attempt concurrently;
the first valid structured verdict settles the guard decision.
 When only one fallback model can be selected,
that model runs alone.
 A rejected contender does not settle the race while another contender can still return a verdict.
 This is an availability fallback,
 not consensus,
so the winning fallback is timing-dependent when models disagree.
 After the first selected model fails,
automatic selection supplies the fallback contenders.
 If no fallback model can be selected or all complete attempts fail,
auto-mode asks the user as before.

No-content model health is tracked per canonical provider/model identity for the
current Pi session.
One logical judge call includes the forced-tool request and its bounded direct-JSON retries.
A call counts as wholly empty only when all of those responses contain no tool call or text.
When a model's three most recent logical calls are all wholly empty,
automatic primary and fallback selection temporarily excludes that model.
Any other recorded outcome breaks the consecutive empty window.
The session-start boundary clears this in-memory history,
so the blocklist never changes model configuration or persists into another session.

The shared parser first tries `JSON.parse(text)` for whole-text output,
then falls back to a balanced-brace scan that respects string-literal escapes.
A `"reason"` field containing `{` or `}` therefore does not skew object boundaries.

When a transport fallback fires,
 the shared review logger records the contract violation so an operator can see it.
 The primary contract remains "MUST call the
tool";
 the retry exists so an unexpected provider response degrades
gracefully rather than throwing.

## Bug fixes vs upstream pi-safeguard

<table>
<thead>
<tr>
<th>Aspect</th>
<th>pi-safeguard</th>
<th>pi-auto-mode</th>
</tr>
</thead>
<tbody>
<tr>
<td>`isSystemPath` check</td>
<td>Present (causes `/var/home` false positive)</td>
<td>Removed</td>
</tr>
<tr>
<td>Budget model API</td>
<td>Calls removed `getApiKey()`</td>
<td>Calls `getApiKeyAndHeaders()`</td>
</tr>
<tr>
<td>Error handling in evaluate</td>
<td>Bare `catch` swallows TypeErrors</td>
<td>Logs error, falls back to ask-user</td>
</tr>
<tr>
<td>Budget model</td>
<td>Separate `pi-budget-model` package</td>
<td>Inlined `budget-model.ts`</td>
</tr>
<tr>
<td>Bash parser</td>
<td>`@aliou/sh` (UNLICENSED, three gap bugs)</td>
<td>`unbash` (ISC) plus targeted extraction</td>
</tr>
<tr>
<td>`--` separator</td>
<td>Not handled in `hasFlag()`</td>
<td>Handled</td>
</tr>
<tr>
<td>Judge response format</td>
<td>Free-text JSON</td>
<td>Forced tool-calling, direct-JSON retry, logged text parser fallback</td>
</tr>
</tbody>
</table>

## Judge model selection

Automatic judge selection ranks every candidate in Pi's effective model scope
by local speed-name heuristic:
`highspeed` or `high-speed` > `fast` > `luna` > `flash` or `spark` > `terra` >
`turbo` > `nano` > `mini` > `haiku` > `lite` or `light` > no signal.
When no speed signal separates candidates,
 selection falls back to input cost.

### Effective scoped model set

Automatic selection only considers Pi's effective scoped models:

1. Nonempty live scope exposed by Pi's extension context.
2. Startup `--models` patterns.
3. Merged global and project `enabledModels` settings.
4. Authenticated registry models when Pi has no scope restriction.

The fixed strategy ranks candidates from every provider in that set.
Pi SDK sessions,
including subagents,
can expose an empty live cycle list without forbidding models.
That list falls through to argv and configured settings;
explicit restrictions that match nothing still fail closed.

### Headless subagents

Judge approval works without an interactive UI.
Judge denial still blocks the tool.
If manual approval is required but the child has no UI,
auto-mode returns the blocked action and reason with instructions to report them to its parent.
It does not repeatedly direct the child to an unavailable approval dialog.

Bypass,
trust directives,
and exact-action approvals remain session-local.
The parent's bypass banner does not disable child guards.
See the [subagent scope diagnosis](../../../../doc/troubleshooting/pi-auto-mode-subagent-model-scope.md)
for the reproduction and verification boundaries.

## Skill read allowlist

Before each agent run,
 auto-mode reads Pi's loaded skill metadata from
`before_agent_start` and allows `read` tool access to every loaded skill
directory.
 This prevents guard prompts when the model loads `SKILL.md` or
referenced files from global,
 project,
 or package skills.

Auto-mode also allows `read` tool access to existing files under current
`~/temp/agent` and historical `/tmp/agent` compatibility roots when each
directory is owned by the current process user,
has no group or other permission bits,
 and resolves without symlinks below canonical account home.
The account home is canonicalized before deriving `~/temp/agent`,
so operating-system aliases such as `/home/user` to `/var/home/user` preserve account identity.
A symlink at `temp`,
`agent`,
or any descendant still cannot escape canonical scratch boundary.
Agents should place new third-party source clones under `~/temp/agent` when they
need repeated inspection outside the current project.
The allowlist uses canonical filesystem paths,
 so symlinks that resolve outside
either root still go through the normal signal and judge pipeline.

For Bash tool calls,
a positive whole-shell proof bypasses normal path judging for modeled source-inspection commands.
Current modeled forms cover ripgrep without external helper options,
GNU find without mutation,
subprocess,
named-file output,
or link-following actions,
output-only `sort`,
`paste`,
and `printf`,
and `git -C <repo> tag --points-at <object>`.
Literal `for` loop values carry path provenance into body commands,
so repeated Git inspection across scratch repositories remains provable.
Every path must canonically remain under Pi's current working directory or a private agent scratch root.
Unknown commands,
unproven expansions,
writable redirects,
secret-looking paths,
and mutating option forms continue through normal judge handling.
Unquoted pathname,
brace,
tilde,
and extended-glob expansion syntax fails closed.
Quoted patterns used by `find` predicates and ripgrep arguments remain literal.

Both private roots remain trusted for existing non-secret helper paths.
Running an inspected helper script from either root
does not trigger a location-only prompt
when existing credential-handoff policy applies.
Bash calls still flag destructive commands,
secret-looking paths inside either root,
and paths outside trusted roots.

When a Bash command passes a secret-looking environment variable to a trusted
script or interpreter command under either root,
 auto-mode permits `grep` to
source that key from a project-local `.env` file.
 This covers image-diff or
model-check helpers such as
`GEMINI_API_KEY="$KEY" node "${HOME}/temp/agent/check.mjs"`.
 It does not allow
arbitrary secret files,
 unrelated dotenv reads,
 home dotfiles,
 direct network
commands with secret parameter references,
 or untrusted script paths.

Auto-mode also allows `read` tool access to existing files in linked git
worktrees attached to the current repository.
`@monochromatic-dev/git-executable-core` selects real Git with common-platform-path priority and caches successful executable
resolution.
The worktree list remains fresh for each read event,
and each candidate root is classified with `rev-parse` so the main worktree is not added to this cross-worktree
allowlist.

The allowlist preserves secret-path checks.
 `write` and `edit` calls targeting
skill directories,
 linked worktrees,
 or either agent scratch root still go through
the normal signal and judge pipeline.

## Virtual input hard guard

Before bypass,
flagger,
or judge handling,
a fixed guard parses Bash commands and blocks caller-scoped ydotool execution.
ydotool sends key-down and key-up as separate datagrams.
If injected input cancels its own Pi Bash caller,
the persistent virtual device can retain the key-down until ydotoold restarts.

The guard covers statically visible direct executable paths,
generic forwarding commands,
and direct or wrapper-nested inline shell programs.
Allowlisted inspection commands can mention ydotool as data.
Commands with `-exec`-style capabilities fail closed instead.
Malformed shell containing ydotool fails closed.

The guard cannot identify executable names hidden behind shell variables,
commands loaded from script files or standard input,
runtime-generated command-substitution output,
configuration that later spawns ydotool,
or custom interpreter code that spawns it.
`AGENTS.md` rule `VKI` remains the policy backstop for those forms.
The guard exposes no live-input bypass.
A separately deployed broker must own release events behind a narrow API.
See
[`doc/troubleshooting/ydotool-interrupted-key-release.md`](../../../../doc/troubleshooting/ydotool-interrupted-key-release.md).

## Migration clause lowering

`lowerEstablishedClause` maps code-admitted clause relationships into directive facts,
authorization prerequisites,
and independent missing premises.
It does not authenticate a model response,
source authority,
operation coverage,
or approval sufficiency.
Unknown or unsupported meanings remain broader than uncertainty among supported effects.
A satisfied prerequisite grants no permission;
an unmet prerequisite cannot override a prohibition.

The private native integration links the maintained exports for these deterministic computations.
The original request,
source,
judgment,
and human-approval owners remain required before any result can authorize execution.

## Migration original prerequisite evidence

`captureAuthorizationInputs` collects independent satisfaction and channel facts
through a factory fixed by trusted bootstrap.
Each original record must belong to the same judgment,
clause,
request-bound source,
and request.
Owner methods and primitive facts remain subject to freshness checks;
the first eligibility failure permanently retires the collection.
Borrowed owners and records remain mutable.

`MISSING_AUTHORIZATION_EVIDENCE` identifies a clause without captured facts.
It grants no permission and does not satisfy an approval requirement.
`captureAuthorizationOwnerFactory` validates configured callables;
native bootstrap handles absent configuration before calling it.
The compiler admits clause meaning separately from these facts.
The private native consumer now delegates to the maintained artifact.
This evidence API does not supply a production fact owner or install the migrated host.

## Migration query observation

`createNativeQueryObservation` retains the first unexpected input or output observation failure.
Native adapters supply synchronous receivers and forward original frames.
The helper does not authenticate those frames,
assign source authority,
or prove complete coverage.
A callback catching an observer failure cannot restore that observation lifetime.
Native reporting failures preserve their original value when there was no observer failure;
otherwise an aggregate retains the observer,
callback,
and reporting failures.
Borrowed frames,
callbacks,
and thrown values remain mutable.

The private query adapter delegates failure retention to this maintained export.
Original argument-frame construction,
query result identity,
source capture,
and native error reporting remain the adapter's responsibilities.
This does not replace the installed judge or supply a maintained host launcher.

## Migration original source contributions

`consumeSourceAssociations` accounts for individual original serialized contributions,
using the existing request receipt,
source views,
carriers,
run publications,
active ancestors,
and separately retained message origins.
Copied references and equal content cannot substitute for original membership.
Its owned result containers are frozen;
borrowed sources and publications remain untouched.

The consumer and `PolicyInputBindingError` now have maintained implementations.
The private native adapter delegates through the existing artifact link.
Build,
type checks,
package units,
native serializer scenarios,
and action-policy regressions have passed.
Lint and final delivery verification remain in progress.
These partial links establish neither complete host receipt nor authority,
scope,
delegation,
or permission.
The governing-domain hold remains.

## Migration same-source incorporation

`selectUnitAdoptionPairs` preserves the supported structural profile:
a source must contain exactly a tagged adopter and a whole quoted target,
with at most 64 source opportunities.
Selection alone establishes neither incorporation meaning nor authority.

`consumeUnitAdoptions` checks original pair,
member,
source-projection,
authority,
and request-scope references.
Adoption-only meaning cannot override a conflicting primitive interpretation of the complete adopter.
Target meaning is independently admitted before lowering with the original registered priority.
A target prerequisite remains a constraint,
not permission supplied by incorporation.
Independent implementation-fact gaps remain outside this consumer.

The private native compiler now delegates selection and consumption to maintained exports.
Build,
types,
package units,
changed-file lint,
the native incorporation scenarios,
and action-policy regressions passed.
Original source membership and target meaning remain separate requirements.
This does not close the governing domain or install the migrated host.

## Migration heading-context interpretation

`consumeHeadingContexts` retains original candidate and evidence references for source-local heading groups.
It checks the original question kind,
model attribution,
content relationship,
and synchronous Boolean code acceptance.
Native integration still authenticates the complete batch and revalidates dependencies after callbacks.
Equal text in independent groups does not merge their identities.

Admitted `none` concerns only the selected group's independent heading content.
Potential or unknown content remains separate from directive meaning,
source authority,
and governing-domain completeness.
The consumer produces no directive or permission.
It freezes owned results,
not borrowed candidates,
evidence,
or callbacks.
Build,
types,
package units,
changed-file lint,
all native heading scenarios,
and action-policy regressions passed.
The private native compiler now calls the exact maintained export.
This does not close the governing domain or install the migrated host.

## Migration semantic admission configuration

`copySemanticAdmissionPolicy` copies an explicitly configured bootstrap declaration.
The native adapter handles absent configuration before calling it and fixes each implemented definition ID.
Only model,
definition,
program profile,
and acceptance callback fields are allowed.
Facts,
coverage,
qualification,
and verdict fields are rejected.
Accessors are not invoked;
proxy configurations reject before reflection.
Borrowed callbacks remain unfrozen.

`inspectSemanticAdmissionScope` matches direct Bash commands or original program inspections to the captured profile
and requires clause candidates only for clause-selecting consumers.
Direct Bash admission requires the original native `bash` operation with a string `command`.
The assessment receives the complete command and every operation in its prepared group.
An uncertain answer still requires scoped confirmation;
headless consumers withhold execution when no approval host is available.
`test:native:direct-bash` covers permission, prohibition, unknown answers, and a mixed Bash/program group.
The native adapter fixes the expected model from its actual wire contract,
not an action argument.
A matching scope selects a consumer;
it neither invokes the acceptance callback nor establishes semantic truth or permission.
No production acceptance criterion is installed by these helpers.
Build,
types,
package units,
changed-file lint,
all native compiler scenarios,
and the action-policy regression task passed.
The native model binding and absent-policy behavior remain unchanged.

## Migration original request interpretation

`consumeRequestMaterial` checks the selected question,
original request identity,
source-bound request identity,
serving-model attribution,
and synchronous Boolean policy acceptance.
Native callers still authenticate complete-batch membership and revalidate the original owners after callbacks.
The interpretation retains the exact request and evidence references without freezing borrowed objects.
It does not parse or re-encode the request,
assign authority,
or lower directives.

`retainRequestMaterialObligations` preserves existing source-domain and unrelated premises.
Admitted `potential-instruction` adds an independent `request-material-instruction-use` obligation;
unknown or unadmitted content keeps an interpretation obligation.
Admitted `none` does not certify source completeness or grant execution permission.
The private compiler delegates these consumers to exact maintained exports.
Built-artifact units,
type checking,
changed-file lint,
the native request scenarios,
and the action-policy regressions passed.
Native domain contrasts still retain the source-domain hold;
this port does not establish domain closure or install the migrated host.

## Migration native hook lifetime

`installNativePolicyHooks` owns slot installation,
reverse-order removal,
and rollback for the migration runtime.
The native integration still authenticates the original Agent,
Session,
request,
judgment,
and registration capabilities before calling this helper.
It does not establish source coverage or authorize execution.

An installation failure preserves its original value;
independent cleanup and restoration failures remain in ordered aggregates.
Incomplete cleanup keeps the executor slot occupied.
Successful removers are not repeated,
and failed removers remain available for retry.
Detachment requires completed Session disposal and an idle Agent.
The invalidated observer remains on the stream after detachment.
Borrowed slots and callbacks are not frozen.

The private native runtime now delegates this bookkeeping through the existing maintained-artifact link.
Built-artifact units,
type checking,
changed-file lint,
and the native cutover passed.
A disposable built-artifact omission confirms that removing the disposal gate allows premature detachment.
This is not a maintained host launcher or installed judge cutover.

## Migration input accounting

`assertOriginalInputBudget` validates resource and Session receipt counters,
then checks that their combined arithmetic remains exactly representable.
On 2026-10-15 the user approved removal of arbitrary instruction-capture size and count ceilings,
superseding the 128-occurrence limit adopted on 2026-10-14.
Normal configured prompts had exceeded several independently imposed fixture limits.
It preserves the original counter profile without invoking accessors,
reflecting on proxies,
or freezing borrowed records.
The counters must come from fixed native readers;
valid shapes and arithmetic establish neither ownership nor governing-domain completeness.
Complete resource text,
callback inputs,
tool metadata,
source histories,
and source indexes no longer have fixture-derived capacity ceilings.
The iterative data copier rejects cycles explicitly and preserves supported shared metadata by occurrence.
Provider request limits and executable-program admission constraints remain separate.

This maintained helper is part of the ongoing migration.
Its export does not activate the private native runtime or replace the installed judge pipeline.

## Logging

The package uses tagged loggers from `@monochromatic-dev/logger-core`.
The root tag is `auto-mode`;
 each module composes a deeper tag
(`auto-mode -> evaluate`,
 `auto-mode -> judge`,
 ...).
 At function entry
and at branch decisions the log lines describe the flow so operators
can audit verdicts without instrumenting the code further.

## Commands

- `/guard <directive>`:
   add a session-scoped trust directive
- `/guard reset`:
   clear all trust directives
- `/guard`:
   list active trust directives

## Bypass mode

`Shift+Tab` toggles bypass mode when pi has no built-in binding on that key.
Stock pi binds `Shift+Tab` to `app.thinking.cycle`,
 so users who want the
bypass shortcut must unbind or remap that action in `~/.pi/agent/keybindings.json`:

```json
{
  "app.thinking.cycle": []
}
```

While bypass is enabled,
 auto-mode allows tool calls without flagger or judge
evaluation.
The fixed virtual-input guard remains active.
 The footer shows `auto-mode: bypass`,
 toggles are written as
`auto-mode:bypass` session entries,
 and each tool call allowed while bypass is
active is written as a bypass audit entry.
 Press `Shift+Tab` again to restore
guardrail checks.

## Tools

- `propose_trust`:
   request permission for something the guardrail blocked.
  If the proposed rule exactly matches an active session trust directive,
  auto-mode accepts it without a UI prompt.
   New or reset-cleared rules still
  prompt the user.
