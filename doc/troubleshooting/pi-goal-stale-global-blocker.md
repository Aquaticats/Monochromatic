# @narumitw/pi-goal 0.12.0 pause or interruption blocks unrelated Pi tool calls

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

A Pi session loaded with `@narumitw/pi-goal` 0.12.0 can reject `read`,
`bash`,
`edit`,
`write`,
or any other tool after `/goal pause` or a non-retryable aborted or errored goal turn.
The tool result is:

```text
Blocked stale /goal tool call after the goal was paused or interrupted.
```

The diagnostic names a stale `/goal` call,
but the handler applies to every Pi tool call.
A `read` call and an unrelated custom tool receive the same block result.

This repository replaces that package with `@monochromatic-dev/pi-goal`.
The repository-owned extension never registers a goal-state `tool_call` handler.

The observed global configuration already omits `npm:@narumitw/pi-goal`.
`pi list` on 2026-07-17 showed neither the retired npm package nor the repository replacement.
An unreferenced 0.12.0 package directory remains under the global Pi npm directory,
but that directory alone does not make the package active.
Pi documents that `pi list` reports packages from settings in
`node_modules/@earendil-works/pi-coding-agent/docs/packages.md:20-43`.

## Root cause

### Pause enables one process-wide stale-tool flag

The installed 0.12.0 source calls `blockStaleGoalToolCalls()` before it changes an active goal to `paused`.
Installed package file `@narumitw/pi-goal/src/goal.ts:428-443` contains:

```ts
function pauseGoal(ctx: StatusContext) {
  // ...
  cancelContinuationPending();
  blockStaleGoalToolCalls();
  abortCurrentTurn(ctx);
  activeGoal = transitionGoal(activeGoal, "paused");
  persistGoal(activeGoal);
  // ...
}
```

A non-retryable aborted or errored turn reaches the same flag.
Installed package file `@narumitw/pi-goal/src/goal.ts:342-365` delegates to
`pauseGoalAfterAgentEnd()`,
and lines 574 to 588 set the flag before persisting the paused state:

```ts
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
  // ...
}
```

### The handler does not distinguish goal tools from unrelated tools

The registered handler accepts no event parameter and therefore never checks `event.toolName`.
Installed package file `@narumitw/pi-goal/src/goal.ts:321-330` contains:

```ts
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

Because the callback ignores the tool name,
its block result applies to built-in and custom tools alike.
The flag is in module memory,
so deleting a persisted state file does not alter an already loaded extension instance.

### Some user input and clear paths remove the flag

The 0.12.0 input handler clears the flag for non-extension input at
installed package file `@narumitw/pi-goal/src/goal.ts:312-319`:

```ts
pi.on("input", (event) => {
  if (event.source === "extension") {
    // ...
    return;
  }
  clearGoalRecovery();
  clearStaleGoalToolCallBlock();
});
```

`clearActiveGoal()` also clears it at lines 1054 to 1060.
Those escape paths explain why the symptom can disappear after another human message,
`/goal clear`,
`/goal resume`,
or a fresh extension runtime even though the package is still configured.
They do not remove the global handler.

### Source inspected on 2026-07-17 still intentionally owns a blocker

The source inspected at commit `2142918467385daf4afbdadadce52527761264d1`
is `@narumitw/pi-goal` 0.17.0.
Its `extensions/pi-goal/src/goal.ts:643-673` still registers `tool_call` and returns:

```ts
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

The upstream package has evolved,
but retaining a stopped-goal blocker is an intentional product policy there.
The repository-owned replacement has a different invariant:
goal state must never block unrelated tools.

## Verification

### Versions and source identities

- Installed retired package:
   `@narumitw/pi-goal` 0.12.0.
- Installed Pi host used by this repository:
   `@earendil-works/pi-coding-agent` 0.80.8.
- Upstream source inspected on 2026-07-17:
   tag `v0.17.0`,
  commit `2142918467385daf4afbdadadce52527761264d1`.
- Repository replacement verification commit:
   `2acc8e76f`.

### Minimal 0.12.0 reproduction

Node refuses built-in TypeScript stripping directly below `node_modules`,
so the reproduction copies only the installed source into a disposable directory
and resolves its peer dependencies from this checkout.
Run from the repository root:

```bash
mkdir --parents /tmp/agent/pi-goal-stale-repro-20260717
chmod 700 /tmp/agent/pi-goal-stale-repro-20260717
cp /var/home/user/.pi/agent/npm/node_modules/@narumitw/pi-goal/src/goal.ts \
  /tmp/agent/pi-goal-stale-repro-20260717/goal.ts
ln --symbolic /var/home/user/Monochromatic/package/pi-plugin/goal/node_modules \
  /tmp/agent/pi-goal-stale-repro-20260717/node_modules
node --input-type=module-typescript -e '
const handlers = new Map();
let command;
const pi = {
  appendEntry() {},
  on(name, handler) { handlers.set(name, handler); },
  registerCommand(_name, definition) { command = definition; },
  registerTool() {},
  async sendUserMessage() {},
};
const context = {
  cwd: "/tmp/pi-goal-stale-reproduction",
  isIdle() { return true; },
  hasPendingMessages() { return false; },
  abort() {},
  sessionManager: { getBranch() { return []; } },
  ui: {
    async confirm() { return true; },
    notify() {},
    setStatus() {},
  },
};
const imported = await import(
  "file:///tmp/agent/pi-goal-stale-repro-20260717/goal.ts"
);
imported.default(pi);
await command.handler("reproduce stale blocker", context);
await command.handler("pause", context);
const result = await handlers.get("tool_call")(
  { type: "tool_call", toolName: "read", toolCallId: "probe", input: {} },
  context,
);
console.log(JSON.stringify(result));
'
```

Observed output:

```json
{"block":true,"reason":"Blocked stale /goal tool call after the goal was paused or interrupted."}
```

### Clean catalog

These paths return no stale-tool block in 0.12.0:

- A tool call before `/goal pause` returns no block because the flag is clear.
- A tool call after non-extension user input clears the flag.
- A tool call after `/goal clear` removes active goal state and clears the flag.
- A tool call after `/goal resume` clears the flag before starting the new goal instance.

The repository replacement additionally keeps unrelated tools usable
while the goal remains active after abort and error.
Its real `AgentSession` verifier runs `read`,
`bash`,
`edit`,
`write`,
and a custom tool after abort,
a settled model error,
replacement,
and clear.
`package/pi-plugin/goal/src/pi-runtime-verifier-tools.ts:324-438` contains those consumer-boundary assertions.

### Failing catalog

These 0.12.0 paths set the same process-wide block:

- `/goal pause`,
  followed by any built-in or custom tool.
- A non-retryable assistant `stopReason: "aborted"`,
  followed by any built-in or custom tool before an input path clears the flag.
- A non-retryable assistant `stopReason: "error"`,
  followed by any built-in or custom tool before an input path clears the flag.

The handler ignores tool identity,
so there is no separate error variant for `read`,
`bash`,
`edit`,
`write`,
or a custom tool.

### Repository replacement verification

Run:

```bash
mise run //package/pi-plugin/goal:verify:extension
mise run //package/pi-plugin/goal:verify:pi-runtime
```

The built verifier rejects any registered `tool_call` handler at
`package/pi-plugin/goal/src/mise.verify-extension.ts:220-222`.
The real runtime verifier reported success for:

```text
manifest discovery, lifecycle continuation, abort, compaction, clear, branch reconstruction
noninteractive reviewer exhaustion, fake reviewer denial and approval
tools read, bash, edit, write, verification_echo after abort, error, and clear
```

## Verified workarounds

### Migrate to the repository-owned package

First inspect package settings:

```bash
pi list
```

For the observed state,
the retired source is already absent.
Skip removal and install only the repository package.

On a machine where the retired source is listed,
close running Pi processes and back up `~/.pi/agent/settings.json`.
Then remove only that source while Pi is stopped:

```bash
pi remove npm:@narumitw/pi-goal
```

Install the repository package globally:

```bash
pi install /var/home/user/Monochromatic/package/pi-plugin/goal
```

Run `pi list` again,
then restart Pi.
Pi documents global `install` and `remove` behavior in
`node_modules/@earendil-works/pi-coding-agent/docs/packages.md:20-43`.

Pi stores a local package path relative to your home directory,
so the entry written to `~/.pi/agent/settings.json` reads
`../../../../var/home/user/Monochromatic/package/pi-plugin/goal`
rather than the absolute path given on the command line.
Measured 2026-10-02 on pi 1.0.0 while installing
`package/pi-plugin/model-retirement`,
whose stored entry took that form beside an identically stored `openai-fast` entry.
The form resolves correctly,
and `pi remove` accepts the absolute path anyway.
A scripted installer that diffs settings before and after must therefore expect the
relative form in the `packages` array,
not the argument it passed.

Tradeoff:
the private package points at this checkout,
so the checkout and built artifact must remain available.
Completion review can use another configured model provider,
incur provider cost,
and send bounded post-goal evidence to that provider.

### Clear the old in-memory blocker before migration

With 0.12.0 still loaded,
`/goal clear`,
`/goal resume`,
or non-extension user input clears the in-memory stale flag.

Tradeoff:
this is temporary.
The retired extension remains loaded,
keeps its broader command surface,
and can set the flag again after another pause or interruption.

### Restart Pi after changing settings

`pi remove` changes settings used by future resource discovery.
An already running process can still hold the old module state until process restart.

Tradeoff:
restart interrupts the current interactive workflow.
Do not use restart as a substitute for removing the retired package from settings.

## What does not work

### Treating an unreferenced package directory as active configuration

The directory
`/var/home/user/.pi/agent/npm/node_modules/@narumitw/pi-goal`
still exists locally,
but `pi list` and `/var/home/user/.pi/agent/settings.json` contain no retired package entry.
Deleting this directory is unnecessary for migration and does not prove which packages Pi loads.
Use settings and `pi list` as the authority.

### Installing both packages

Keeping both sources can register duplicate `/goal` commands and `goal_complete` tools.
It also leaves the old global `tool_call` handler available.
Remove the npm source before enabling the repository source.

### Installing in project settings

`pi install -l` writes to `.pi/settings.json`.
That would make a private user workflow part of project configuration
and conflicts with this repository's package-free project settings policy.
Use global settings.

### Deleting only persisted goal state

The block flag and handler live in the loaded module.
Deleting persisted goal state cannot unregister a handler from the current process.
Use the supported clear path for immediate recovery,
then remove the package from settings and restart Pi.

### Importing the installed TypeScript source directly with Node

The first reproduction attempt imported the `.ts` file under `node_modules` directly.
Node 26.5.0 rejected it with:

```text
ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING
```

Copying the source to the private disposable directory avoids that Node restriction without editing third-party source.

## Upstream filing decision

The `.out-of-scope/` directory contains no exemption for Pi goal packages or unrelated-tool blocking.

Duplicate search found closed upstream issue
[narumiruna/pi-extensions#124](https://github.com/narumiruna/pi-extensions/issues/124),
`Unable to run tools after /goal compaction`,
and merged pull request
[narumiruna/pi-extensions#133](https://github.com/narumiruna/pi-extensions/pull/133),
`fix(pi-goal): release stale tool block on clear`.
The issue has no comments.
The pull request records clear-path self-healing and tests.

The filing constraints resolve as follows:

1.  **Upstream fault:
    ** No for this repository's requirement.
    Upstream 0.17.0 source inspected on 2026-07-17 intentionally retains stopped-goal blocking,
    while this repository deliberately forbids any goal-state blocker.
2.  **Upstream can fix it:
    ** Yes technically,
    by narrowing or removing the handler,
    but that would change upstream policy.
3.  **Supported use case:
    ** Partly.
    Upstream supports autonomous goals,
    but its current source and tests support a different stopped-goal policy.
4.  **Contribution welcome:
    ** No explicit external-contribution or AI-assisted-contribution ban was found
    in `README.md` or `.github/`.
5.  **Likely upstream action:
    ** No evidence supports removing the intentional current policy.
    The maintainer already fixed the distinct stale-clear bug in pull request 133.
6.  **Compatible prototype:
    ** The repository-owned replacement is implemented and verified,
    but it is a separate product contract rather than an upstream patch.

Constraints one and five do not pass,
so no upstream patch prototype or new issue is warranted.
Issue 124 and pull request 133 already contain the relevant stale-clear finding.
There is no additive comment to post.
