# Pi goal with pi-processes 0.9.4: live background work caused automatic continuation turns

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

With an active repository-owned `/goal`,
the main model could call `process` with `action: "start"`,
finish its current run while that process remained live,
and immediately receive another goal-owned continuation turn.

No error string was emitted.
The visible symptom was a `Goal continuation` message appearing before the background process reached
`exited` or `killed`.
The same behavior applied when `process list` reported `running`,
`terminating`,
or `terminate_timeout`.

The intended behavior is different:
the goal remains active,
but a live process makes that settlement ineligible for a goal-owned continuation.
The process extension's own alert preferences decide whether a completion or failure starts a later turn.

## Root cause

The pre-fix goal lifecycle handled every non-aborted `agent_settled` event through `settleCurrentGoal`.
It checked pending Pi messages,
but had no process-liveness input.
At baseline commit `39ffdb81a27e6e45352ba11a9a219ef4ee1c0609`,
`package/pi-plugin/goal/src/lifecycle.ts:330-337,435-445` contained:

```ts
function settleCurrentGoal(context: ForeignBorrowed<ExtensionContext>,): void {
  applyTransition({
    transition: settleGoal({
      controller,
      marker: services.createId(),
      timestamp: services.now(),
      hasPendingMessages: context.hasPendingMessages(),
    },),
    context,
  },);
}

pi.on(
  'agent_settled',
  function continueActiveGoal(_event, context,) {
    if (settledRunWasAborted) {
      settledRunWasAborted = false;
      return;
    }
    settleCurrentGoal(context,);
  },
);
```

`@aliou/pi-processes` already exposes enough data at Pi's extension boundary to maintain runtime-local
liveness without polling or importing its private manager.
A successful start result includes the complete process snapshot in
`src/tools/actions/start.ts:191-198` at commit
`eb523640a02aa90e2c1f665aba62efa53ed88be1`:

```ts
return {
  content: [{ type: "text", text: message }],
  details: {
    action: "start",
    success: true,
    message,
    process: proc,
  },
};
```

The same source defines live states in `src/constants/types.ts:14-25`:

```ts
export type ProcessStatus =
  | "running"
  | "terminating"
  | "terminate_timeout"
  | "exited"
  | "killed";

export const LIVE_STATUSES: ReadonlySet<ProcessStatus> = new Set([
  "running",
  "terminating",
  "terminate_timeout",
]);
```

Process identities are monotonic within one manager runtime.
`src/manager.ts:52-54,190-192` stores one counter and increments it for each start:

```ts
export class ProcessManager {
  private processes: Map<string, ManagedProcess> = new Map();
  private counter = 0;

  // Inside start(...)
  const id = `proc_${++this.counter}`;
}
```

When a process becomes terminal,
`src/manager.ts:121-130` emits `process_ended`.
`src/hooks/process-end.ts:47-66` converts that event into an `ad-process:update` custom message carrying
`kind: "lifecycle"`,
`processId`,
and terminal status,
regardless of whether its configured `triggerTurn` value is true:

```ts
const details: ProcessUpdateDetails = {
  kind: "lifecycle",
  processId: info.id,
  processName: info.name,
  command: info.command,
  status: info.status as "exited" | "killed",
  exitCode: info.exitCode,
  success: info.success ?? false,
  runtime,
};

safeSendMessage(
  pi,
  {
    customType: MESSAGE_TYPE_PROCESS_UPDATE,
    content: message,
    display: true,
    details,
  },
  { triggerTurn: triggerAgentTurn },
);
```

Pi 0.80.10 emits `message_end` even for a non-triggering custom message.
The installed
`@earendil-works/pi-coding-agent/dist/core/agent-session.js:1092-1099`
path appends that message and emits both message events:

```js
else if (options?.triggerTurn) {
    await this._runAgentPrompt(appMessage);
}
else {
    this.agent.state.messages.push(appMessage);
    this.sessionManager.appendCustomMessageEntry(message.customType, message.content, message.display, message.details);
    this._emit({ type: "message_start", message: appMessage });
    this._emit({ type: "message_end", message: appMessage });
}
```

The cause was therefore local policy omission,
not missing upstream process data.
The goal extension did not observe the public process result and lifecycle message shapes before deciding
whether to continue.

## Verification

Versions and source identities:

- Repository baseline:
   `39ffdb81a27e6e45352ba11a9a219ef4ee1c0609`.
- Installed Pi coding agent:
   `0.80.10`.
- Installed `@aliou/pi-processes`:
   `0.9.4`.
- Audited `@aliou/pi-processes` source commit:
  `eb523640a02aa90e2c1f665aba62efa53ed88be1`.

The regression test in
`package/pi-plugin/goal/src/lifecycle.unit.test.ts`
can be copied into a disposable worktree at the baseline commit:

```bash
mkdir --parents /tmp/agent
chmod 700 /tmp/agent
git worktree add /tmp/agent/goal-process-baseline \
  39ffdb81a27e6e45352ba11a9a219ef4ee1c0609
ln --symbolic "$PWD/node_modules" /tmp/agent/goal-process-baseline/node_modules
ln --symbolic "$PWD/package/pi-plugin/goal/node_modules" \
  /tmp/agent/goal-process-baseline/package/pi-plugin/goal/node_modules
cp package/pi-plugin/goal/src/lifecycle.unit.test.ts \
  /tmp/agent/goal-process-baseline/package/pi-plugin/goal/src/lifecycle.unit.test.ts
mise trust /tmp/agent/goal-process-baseline/mise.toml
cd /tmp/agent/goal-process-baseline
mise run //package/pi-plugin/goal:build
mise run //package/pi-plugin/goal:test:unit
```

The baseline run fails both process-policy cases.
The direct start case reports that the goal appended two events rather than one while `proc_1` was still
`running`:

```text
[suppresses continuation while observed background processes remain live] FAIL
expected [...] to have a length of 1 but got 2
```

The list-reconciliation case likewise reports two goal messages rather than one while a listed process
remained live.

Run the fixed artifact tests from the repository root:

```bash
mise run //package/pi-plugin/goal:build
mise run //package/pi-plugin/goal:lint:types
mise run //package/pi-plugin/goal:lint:oxlint
mise run //package/pi-plugin/goal:test:unit
```

Known-working catalog after the fix:

- A successful start snapshot with `status: "running"` suppresses goal continuation,
  including when the process started before goal activation.
- List snapshots with `terminating` or `terminate_timeout` suppress goal continuation.
- An `exited` snapshot in the same list is not treated as live.
- `ad-process:update` lifecycle messages remove `exited` and `killed` process identities.
- Terminal identities are retained as runtime-local tombstones,
  so a delayed start result cannot resurrect a process that already ended.
- Ending one process does not continue while another observed process remains live.
- After every observed process ends,
a later otherwise-eligible settlement can continue the active goal.

Known-failing catalog at the baseline commit:

- `running` start results were not observed and settlement continued immediately.
- `terminating` and `terminate_timeout` list results were not observed and settlement continued immediately.
- Process lifecycle updates could not affect goal policy because no goal-owned `message_end` observer existed.

## Verified workarounds

The implemented consumer-side workaround is
`package/pi-plugin/goal/src/background-process-monitor.ts`.
It passively observes successful `process` start and list results,
then removes terminal identities from `ad-process:update` lifecycle messages.
Terminal messages also tombstone process identities for the runtime,
which makes terminal-before-start-result delivery safe under the manager's source-backed monotonic identity contract.
`package/pi-plugin/goal/src/lifecycle.ts:276-278` now exits before the continuation transition when that
runtime-local monitor reports live work.

Tradeoffs:

- The integration intentionally relies on `@aliou/pi-processes` public tool name and result/message shapes.
  A future upstream rename requires a corresponding consumer update.
- The state is runtime-local.
  It does not infer live work from old session history,
  which avoids treating processes killed during restart or reload as still live.
- A process configured not to trigger an alert can finish without starting a model turn.
  The goal remains active and receives its usual prompt on the next independently started turn.

As a temporary operator workaround on an unfixed build,
`/goal clear` prevents automatic goal continuation.
Tradeoff:
the objective is no longer active and no longer receives goal prompt or completion review.

## What does not work

- `ctx.hasPendingMessages()` does not represent operating-system process liveness.
  The baseline already checked it and still continued.
- Setting `alertOnSuccess: false` does not solve the goal behavior.
  That flag controls the process extension's own completion turn,
  while the baseline goal independently emitted a turn at `agent_settled`.
- Scanning all historical session entries for the last process start is unsafe.
  Pi-processes kills runtime-owned processes during session shutdown,
  but old start results remain in persisted history.
- Polling `process list` from the model would spend turns to decide whether another turn should exist.
  It also contradicts the process extension's notification-based workflow.
- A `tool_call` blocker would prevent work rather than suppress only goal-owned continuation,
  and would violate the goal package's unrelated-tool policy.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers this local goal policy integration.
GitHub issue and pull request searches across open and closed `aliou/pi-processes` items found no match for
`event bus process state extension integration`.

The filing constraints do not pass:

1.  **Is it really upstream's fault?**
     No.
    Pi-processes already supplies start snapshots,
    list snapshots,
    live status names,
    and terminal lifecycle messages.
    The repository-owned goal extension ignored those surfaces.
2.  **Can upstream fix it?**
     Upstream could add another event-bus API,
    but that would not change the goal's continuation policy by itself.
    The complete fix belongs at the goal consumer boundary.
3.  **Are they supporting this use case?**
     Partly.
    `README.md` and `CONTRIBUTING.md` support agent-managed background processes and configurable alerts,
    but they do not promise coordination with arbitrary third-party stop hooks.
4.  **Would the repository welcome our contribution?**
     No blocking policy was found.
    `CONTRIBUTING.md` documents normal lint and type-check workflows and contains no AI-assistance ban.
    This does not overcome the failed fault and ownership constraints.
5.  **Will they likely fix it?**
     Not applicable as a goal-package bug.
    Tracker searches found no maintainer position on this separate consumer policy.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
     A minimal consumer fix is
    implemented and tested in this repository.
    No upstream patch is appropriate because changing pi-processes is unnecessary.

Nothing should be filed upstream.
There is no additive issue or comment for `aliou/pi-processes` because the source audit confirms its existing
result and lifecycle surfaces are sufficient.
