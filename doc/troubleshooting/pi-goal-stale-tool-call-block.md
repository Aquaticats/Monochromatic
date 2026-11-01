# `@narumitw/pi-goal@0.12.0` blocks every Pi tool after an aborted goal turn, preventing recovery

## Symptom

An active `/goal` turn that ends with `stopReason: "aborted"` transitions to paused state.
The next tool call can then fail with:

```text
Blocked stale /goal tool call after the goal was paused or interrupted.
```

The failure is not limited to `goal_complete` or another goal-owned action.
The installed `0.12.0` handler returns the same block decision for:

- `read`
- `bash`
- `edit`
- any unrelated custom tool

A crash or interruption can therefore leave the next agent turn unable to inspect files,
inspect process state,
or repair the original failure.
The user must submit fresh non-extension input,
run `/goal clear`,
or otherwise reactivate the goal before ordinary tools work again.

The incident that led to
[issue #360](https://github.com/Aquaticat/Monochromatic/issues/360)
required human `/goal clear` intervention after the first blocked result prevented autonomous recovery.

## Root cause

### One module-level latch governs every tool

The installed npm source declares one mutable latch beside all other extension runtime state.
It is not keyed by tool name,
tool call,
goal generation,
or continuation marker.

```typescript
// extensions/pi-goal/src/goal.ts:122-128 at v0.12.0
let activeGoal: ActiveGoal | undefined;
let completionStatusTimer: NodeJS.Timeout | undefined;
let extensionApi: ExtensionAPI | undefined;
let continuationPending: ContinuationPending | undefined;
let goalRecovery: GoalRecovery | undefined;
let staleGoalToolCallsBlocked = false;
const cancelledContinuationMarkers = new Set<string>();
```

Source:
`extensions/pi-goal/src/goal.ts:122-128` at tag `v0.12.0`,
commit `345e291a5fb307a1afc785effb6563b4d827b2cc`.
The installed file's SHA-256,
`429491a073d4d0648dddb5044075349bae849baaf87fb8cacbce03baf647bb41`,
exactly matches the tagged source.

### An aborted or terminal errored run pauses the goal

The `agent_end` handler distinguishes retryable provider or compaction recovery.
Every other aborted or errored final assistant message calls `pauseGoalAfterAgentEnd`.

```typescript
// extensions/pi-goal/src/goal.ts:345-366 at v0.12.0
if (finalAssistant?.stopReason === "aborted" || finalAssistant?.stopReason === "error") {
  if (isRetryableGoalInterruption(finalAssistant)) {
    goalRecovery = {
      goalId,
      kind: isGoalContextOverflow(finalAssistant) ? "compaction_retry" : "provider_retry",
    };
    cancelContinuationPending();
    persistGoal(activeGoal);
    updateStatus(ctx, activeGoal);
    return;
  }
  clearGoalRecoveryForGoal(goalId);
  pauseGoalAfterAgentEnd(ctx, activeGoal, finalAssistant);
  return;
}
```

Source:
`extensions/pi-goal/src/goal.ts:345-366` at `v0.12.0`.

### Pausing sets the global latch

`pauseGoalAfterAgentEnd` cancels goal continuation,
sets the stale-tool latch,
requests an abort,
and persists paused state.

```typescript
// extensions/pi-goal/src/goal.ts:574-590 at v0.12.0
function pauseGoalAfterAgentEnd(
  ctx: StatusContext,
  goal: ActiveGoal,
  assistant: AssistantMessageLike,
) {
  cancelContinuationPending();
  blockStaleGoalToolCalls();
  abortCurrentTurn(ctx);
  activeGoal = transitionGoal(goal, "paused");
  persistGoal(activeGoal);
  updateStatus(ctx, activeGoal);

  const reason = assistant.stopReason === "aborted" ? "interruption" : "agent error";
  const details = assistant.errorMessage ? ` (${truncateNotification(assistant.errorMessage)})` : "";
  ctx.ui.notify(`Goal paused after ${reason}${details}. Run /goal resume to continue.`, "warning");
}
```

The helper performs an unconditional assignment:

```typescript
// extensions/pi-goal/src/goal.ts:831-836 at v0.12.0
function blockStaleGoalToolCalls() {
  staleGoalToolCallsBlocked = true;
}

function clearStaleGoalToolCallBlock() {
  staleGoalToolCallsBlocked = false;
}
```

Sources:
`extensions/pi-goal/src/goal.ts:574-590` and
`extensions/pi-goal/src/goal.ts:831-836` at `v0.12.0`.

### The `tool_call` handler ignores tool identity

The registered handler takes no event parameter.
Once the latch is true and the goal is paused,
it blocks every tool call with the same result.

```typescript
// extensions/pi-goal/src/goal.ts:321-331 at v0.12.0
pi.on("tool_call", () => {
  if (!staleGoalToolCallsBlocked) return;
  if (!activeGoal || activeGoal.status !== "paused") {
    clearStaleGoalToolCallBlock();
    return;
  }
  return {
    block: true,
    reason: "Blocked stale /goal tool call after the goal was paused or interrupted.",
  };
});
```

Source:
`extensions/pi-goal/src/goal.ts:321-331` at `v0.12.0`.

Because the handler never reads `event.toolName`,
it cannot distinguish stale goal completion from recovery work.
The first blocked result also prevents the agent from using a tool to discover why the goal paused.

### Only fresh user input or a goal command releases it

The input handler clears the latch only for non-extension input.
Extension-authored continuation input does not release it.

```typescript
// extensions/pi-goal/src/goal.ts:312-319 at v0.12.0
pi.on("input", (event) => {
  if (event.source === "extension") {
    if (consumeCancelledContinuationPrompt(event.text)) return { action: "handled" as const };
    return;
  }
  clearGoalRecovery();
  clearStaleGoalToolCallBlock();
});
```

`/goal clear` also delegates to `clearActiveGoal`,
which clears the latch and persisted state.

```typescript
// extensions/pi-goal/src/goal.ts:1054-1061 at v0.12.0
function clearActiveGoal(ctx: StatusContext) {
  cancelContinuationPending();
  clearGoalRecovery();
  clearStaleGoalToolCallBlock();
  activeGoal = undefined;
  clearPersistedGoal(ctx.cwd);
  ctx.ui.setStatus(STATUS_KEY, undefined);
}
```

Sources:
`extensions/pi-goal/src/goal.ts:312-319` and
`extensions/pi-goal/src/goal.ts:1054-1061` at `v0.12.0`.

### The related compaction diagnosis is not this cause

Upstream issue
[`narumiruna/pi-extensions#124`](https://github.com/narumiruna/pi-extensions/issues/124)
reported the same blocker text after auto-compaction.
PR
[`narumiruna/pi-extensions#125`](https://github.com/narumiruna/pi-extensions/pull/125)
added compaction-aware retry classification.

That repair does not disprove this interruption path.
The reproduction here uses no compaction.
It supplies an ordinary aborted assistant message,
which directly pauses the goal and sets the latch.

The earlier idea that goal state was merely forgotten during compaction is therefore incomplete for this incident.
The active goal is present,
and `agent_end` deliberately transitions it to paused state before the global handler blocks tools.

### The behavior remains in the latest inspected release

Tag `v0.15.1`,
commit `fcb8d15ec5d63604309b69f083870f2d5b5a5979`,
still registers a global `tool_call` handler.

```typescript
// extensions/pi-goal/src/goal.ts:492-500 at v0.15.1
if (!runtime.staleGoalToolCallsBlocked) return;
if (!runtime.activeGoal || !blocksStaleGoalToolCalls(runtime.activeGoal.status)) {
  clearStaleGoalToolCallBlock();
  return;
}
return {
  block: true,
  reason: "Blocked stale /goal tool call after the goal stopped or was interrupted.",
};
```

Source:
`extensions/pi-goal/src/goal.ts:492-500` at `v0.15.1`.

The `v0.15.1` README describes this as intentional behavior:

```markdown
<!-- extensions/pi-goal/README.md:117-119 at v0.15.1 -->
## 🛑 Interruption and queued-input behavior

A user pause or aborted turn produces `paused`; a terminal provider/account quota error produces
`usage_limited`; another non-retryable agent error produces `blocked`. Each stopped transition cancels
pending continuation intent or delivery, aborts stale work when applicable, and blocks stale tool calls
until the next non-goal user prompt, successful reactivation/replacement, or `/goal clear`.
```

Source:
`extensions/pi-goal/README.md:117-119` at `v0.15.1`.

## Verification

### Versions and source identity

Verified on 2026-07-15 with:

- Node `26.5.0`.
- Pi coding agent `0.80.6`.
- Installed `@narumitw/pi-goal` `0.12.0`.
- Upstream tag `v0.12.0` at commit
  `345e291a5fb307a1afc785effb6563b4d827b2cc`.
- Installed and tagged `goal.ts` SHA-256
  `429491a073d4d0648dddb5044075349bae849baaf87fb8cacbce03baf647bb41`.

The upstream repository was cloned read-only with `gh repo clone` into
`/var/home/user/temp/agent/pi-extensions-2026-07-15/`.
The verified remotes and refs were:

- Origin `https://github.com/narumiruna/pi-extensions.git`.
- `v0.12.0` commit `345e291a5fb307a1afc785effb6563b4d827b2cc`.
- `v0.15.1` commit `fcb8d15ec5d63604309b69f083870f2d5b5a5979`.

No upstream file was modified.
Tag contents were read with `git show`,
and the clone remained an inspection-only third-party source checkout.
The README and source excerpts for `v0.15.1` were extracted directly from that tag,
not inferred from the later main-branch checkout.

The harness uses Pi's real TypeScript extension loader with an injected runtime and disposable agent directory.
It does not read or write real Pi sessions or settings.

### Reproduction harness

Save this as `~/temp/agent/pi-goal-reproduce.mjs`:

```javascript
// ~/temp/agent/pi-goal-reproduce.mjs
import { pathToFileURL } from 'node:url';

const repoRoot = process.env.REPO_ROOT;
const source = process.env.PI_GOAL_SOURCE;
const agentDir = process.env.REPRO_AGENT_DIR;
if (!repoRoot || !source || !agentDir) {
  throw new Error('REPO_ROOT, PI_GOAL_SOURCE, and REPRO_AGENT_DIR are required');
}
process.env.PI_CODING_AGENT_DIR = agentDir;
const piPackage = `${repoRoot}/node_modules/.pnpm/@earendil-works+pi-coding-agent@0.80.6/node_modules/@earendil-works/pi-coding-agent/dist`;
const { createEventBus } = await import(pathToFileURL(`${piPackage}/index.js`).href);
const {
  createExtensionRuntime,
  loadExtensions,
} = await import(pathToFileURL(`${piPackage}/core/extensions/loader.js`).href);

const runtime = createExtensionRuntime();
runtime.appendEntry = () => {};
runtime.sendMessage = () => {};
runtime.sendUserMessage = () => {};
runtime.setSessionName = () => {};
runtime.getSessionName = () => undefined;
runtime.setLabel = () => {};
runtime.getActiveTools = () => [];
runtime.getAllTools = () => [];
runtime.setActiveTools = () => {};
runtime.getCommands = () => [];
runtime.setModel = async () => false;
runtime.getThinkingLevel = () => 'off';
runtime.setThinkingLevel = () => {};

const loaded = await loadExtensions([source], process.cwd(), createEventBus(), runtime);
if (loaded.errors.length > 0) throw new Error(JSON.stringify(loaded.errors));
const extension = loaded.extensions.at(0);
if (!extension) throw new Error('pi-goal extension not loaded');
const handler = (name) => {
  const matches = extension.handlers.get(name) ?? [];
  if (matches.length !== 1) throw new Error(`expected one ${name} handler`);
  return matches[0];
};
const context = {
  cwd: process.cwd(),
  mode: 'tui',
  hasUI: true,
  isIdle: () => true,
  hasPendingMessages: () => false,
  abort: () => {},
  sessionManager: { getBranch: () => [], getEntries: () => [] },
  ui: {
    confirm: async () => true,
    notify: () => {},
    setStatus: () => {},
  },
};
const command = extension.commands.get('goal');
if (!command) throw new Error('/goal command not loaded');
await handler('session_start')({ type: 'session_start', reason: 'startup' }, context);
const toolCall = handler('tool_call');
const decision = async (toolName) =>
  (await toolCall({ type: 'tool_call', toolName, toolCallId: toolName, input: {} }, context))
    ?.reason ?? 'allowed';
const abortGoal = async () => {
  await handler('agent_end')({
    type: 'agent_end',
    messages: [{
      role: 'assistant',
      content: [],
      api: 'openai-responses',
      provider: 'fixture',
      model: 'fixture',
      usage: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        totalTokens: 0,
        cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
      },
      stopReason: 'aborted',
      timestamp: Date.now(),
    }],
  }, context);
};

await command.handler('reproduce stale blocker', context);
const beforeAbort = await decision('read');
await abortGoal();
const afterAbort = Object.fromEntries(
  await Promise.all(['read', 'bash', 'edit', 'custom_fixture'].map(async (name) => [name, await decision(name)])),
);
await command.handler('replacement goal', context);
const afterReplacement = await decision('read');
await abortGoal();
await command.handler('resume', context);
const afterResume = await decision('read');
await abortGoal();
await handler('input')({ type: 'input', source: 'interactive', text: 'inspect state' }, context);
const afterHumanInput = await decision('read');
await abortGoal();
await command.handler('clear', context);
const afterClear = await decision('read');
console.log(JSON.stringify({
  beforeAbort,
  afterAbort,
  afterReplacement,
  afterResume,
  afterHumanInput,
  afterClear,
}, null, 2));
```

Run it with a disposable agent directory and explicit local paths:

```bash
mkdir --parents "${HOME}/temp/agent/pi-goal-reproduction"
chmod 700 "${HOME}/temp/agent/pi-goal-reproduction"
REPO_ROOT="${PWD}" \
PI_GOAL_SOURCE="${HOME}/.pi/agent/npm/node_modules/@narumitw/pi-goal/src/goal.ts" \
REPRO_AGENT_DIR="${HOME}/temp/agent/pi-goal-reproduction" \
node "${HOME}/temp/agent/pi-goal-reproduce.mjs"
```

Observed output:

```json
{
  "beforeAbort": "allowed",
  "afterAbort": {
    "read": "Blocked stale /goal tool call after the goal was paused or interrupted.",
    "bash": "Blocked stale /goal tool call after the goal was paused or interrupted.",
    "edit": "Blocked stale /goal tool call after the goal was paused or interrupted.",
    "custom_fixture": "Blocked stale /goal tool call after the goal was paused or interrupted."
  },
  "afterReplacement": "allowed",
  "afterResume": "allowed",
  "afterHumanInput": "allowed",
  "afterClear": "allowed"
}
```

### Patterns that work cleanly

- `read` before interruption returns no block decision.
- `read` after goal replacement returns no block decision.
- `read` after `/goal resume` returns no block decision.
- `read` after a non-extension interactive input returns no block decision.
- `read` after `/goal clear` returns no block decision.

### Patterns that fail with the stale-goal diagnostic

After one aborted active goal run:

- `read` is blocked.
- `bash` is blocked.
- `edit` is blocked.
- unrelated `custom_fixture` is blocked.

The identical failure across built-in and custom names confirms that tool identity does not scope the latch.

## Verified workarounds

### Submit fresh non-extension input

A non-extension interactive input clears the latch.
The harness verifies that `read` is allowed immediately after this event.
RPC event behavior was not exercised and is not claimed here.

Tradeoffs:

- It requires another human or external input event.
- It does not restore autonomous recovery after a crash.
- The goal remains paused,
  so goal continuation does not resume automatically.
- The fresh prompt may start a model turn before the user has inspected the failure.

### Replace the goal

Starting a replacement goal clears recovery state and the stale-tool latch before installing the new active goal.
The harness verifies that `read` is allowed after replacement.

Tradeoffs:

- It discards the old active objective.
- It immediately starts different goal work.
- It does not provide a neutral state-inspection window.

### Run `/goal resume`

`/goal resume` clears recovery state and the stale-tool latch,
rotates the goal identifier,
and resumes automatic goal work.
The harness verifies that `read` is allowed after resume.

Tradeoffs:

- It deliberately restarts the automation that the interruption stopped.
- It is unsuitable when the user wants ordinary recovery work without goal enforcement.
- It rotates the stale completion guard identifier.

### Run `/goal clear`

`/goal clear` clears active goal state,
pending continuation tracking,
recovery state,
the stale-tool latch,
and the footer.
The harness verifies that `read` is allowed afterwards.

Tradeoffs:

- It discards the active objective from current goal state.
- It requires human command access.
- It does not preserve autonomous goal continuation.
- A later goal must be started again with a new identifier.

The repository-owned replacement planned in
[`doc/planning/pi-goal-stop-hook.md`](../planning/pi-goal-stop-hook.md)
is the durable consumer-side fix,
but it is not yet implemented or verified and is therefore not listed as a verified workaround.

## Inferred operational escape hatch

Removing `npm:@narumitw/pi-goal` from global Pi packages and reloading Pi
should prevent this extension from registering its `tool_call` handler.
This follows Pi package-loading behavior,
but no real settings mutation or reload was run during this investigation.
Treat it as an unverified operational escape hatch.

Tradeoffs:

- `/goal` and `goal_complete` disappear entirely.
- Existing active goal state is no longer managed by the extension.
- It is not feature parity.

## What does not work

### Retrying the blocked tool

The handler does not consume or clear the latch when it blocks a tool.
Retrying `read`,
`bash`,
or another tool without intervening user input or a goal command returns the same diagnostic.

### Switching to another tool name

The handler never reads `event.toolName`.
The harness confirms that a custom tool is blocked exactly like built-in tools.
Tool substitution cannot recover the session.

### Waiting for another extension callback

The latch is ordinary mutable state with no timer.
There is no delayed release path.
Waiting does not change it.

### Relying on the compaction repair

Upstream PR #125 protects recognized Pi-owned compaction retry paths.
It does not remove the global stopped-goal blocker.
An ordinary aborted turn still reaches `pauseGoalAfterAgentEnd` in `0.12.0`,
and `0.15.1` still documents and implements the global block.

### Editing installed third-party source

Editing
`~/.pi/agent/npm/node_modules/@narumitw/pi-goal/src/goal.ts`
would be overwritten by package update or reinstall and bypasses the package-management boundary.
No local workaround should depend on modifying this third-party installation.

## Upstream filing decision

### Out-of-scope check

The repository's `.out-of-scope/` directory was checked on 2026-07-15.
No entry covers `@narumitw/pi-goal`,
Pi goal extensions,
or this tool-blocking behavior.

### Duplicate search

GitHub issue and pull-request searches covered these terms across open and closed state:

- exact stale-tool diagnostic
- `pi-goal interruption tool block`
- `paused tools blocked goal`
- `stale tool calls goal paused`
- `goal clear tool blocked`

The only matching tracker item was closed issue
[`#124`](https://github.com/narumiruna/pi-extensions/issues/124),
fixed by merged PR
[`#125`](https://github.com/narumiruna/pi-extensions/pull/125).
That thread covers the compaction variant but not the reproduced ordinary-abort path.
The reproduction and current-release source trace would be additive to that thread.

### Constraint check

#### Constraint 1: Is it really upstream's fault?

Yes for the observed global block.
The extension installs an unscoped `tool_call` handler and returns the block before unrelated tools execute.

#### Constraint 2: Can upstream fix it?

Yes.
Upstream could remove the stopped-goal tool handler or scope stale rejection to goal-owned completion behavior.
Pi itself does not require the global latch.

#### Constraint 3: Are they supporting this use case?

No.
The `v0.15.1` README explicitly advertises blocking stale tool calls after paused,
blocked,
or usage-limited transitions.
The desired rule that unrelated recovery tools always remain available conflicts with that documented policy.

#### Constraint 4: Would the repository welcome a contribution?

Soft yes.
Issues are enabled,
the root README documents local development,
and PR #125 changed this path with tests.
No `CONTRIBUTING.md`,
issue template,
pull-request template,
or AI-assistance ban was found in the inspected repository.

#### Constraint 5: Will they likely fix it?

No current positive signal.
The latest inspected release still documents the block as intended behavior and tests the stopped-goal policy.
There is no maintainer statement accepting the broader unrelated-tool requirement.

#### Constraint 6: Have we prototyped a minimal upstream-compatible fix?

No.
Constraints three and five do not hold,
so the troubleshooting policy's automatic upstream prototype requirement does not trigger.
The planned repository-owned replacement intentionally chooses different product semantics
rather than patching the third-party package in place.

The upstream contribution gate does not pass.
Do not post the draft as-is.

### Additive comment draft for issue #124

The related issue already exists,
so retain an additive comment rather than a duplicate new issue.
This draft is not eligible for posting because the contribution gate fails.
Issue #124's compaction defect was already fixed,
and the remaining aborted-turn behavior is a different product-policy disagreement:

~~~md
I reproduced a broader path than overflow compaction against `@narumitw/pi-goal@0.12.0`.
An active goal followed by an assistant message with `stopReason: "aborted"` calls
`pauseGoalAfterAgentEnd`, sets `staleGoalToolCallsBlocked`, and then blocks `read`,
`bash`, `edit`, and an unrelated custom tool with the same stale-goal diagnostic.
No compaction is involved.

The installed source exactly matches tag `v0.12.0` at commit
`345e291a5fb307a1afc785effb6563b4d827b2cc`.
The global stopped-goal block also remains in `v0.15.1` at
`extensions/pi-goal/src/goal.ts:478-500`.

A minimal harness using Pi's extension loader observes:

```json
{
  "beforeAbort": "allowed",
  "afterAbort": {
    "read": "Blocked stale /goal tool call after the goal was paused or interrupted.",
    "bash": "Blocked stale /goal tool call after the goal was paused or interrupted.",
    "edit": "Blocked stale /goal tool call after the goal was paused or interrupted.",
    "custom_fixture": "Blocked stale /goal tool call after the goal was paused or interrupted."
  },
  "afterHumanInput": "allowed",
  "afterClear": "allowed"
}
```

PR #125 protects Pi-owned compaction retries,
but it does not cover this ordinary aborted-turn path.
The broader recovery-safe behavior would require scoping stale rejection to goal-owned completion
rather than blocking every tool while the goal is stopped.
~~~
