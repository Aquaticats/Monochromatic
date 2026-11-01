# Are Claude Code hooks synchronous or asynchronous?

Fetched 2026-11-01.

Short answer:
hooks are synchronous by default.
When an event fires,
Claude Code waits for the event's matching hook handlers to finish before it continues,
and a handler's exit code or JSON output can block the action.
Matching handlers for one event run in parallel with each other,
but the event as a whole still waits.
Command hooks can opt out per handler with `"async": true` or `"asyncRewake": true`,
and some hook events are asynchronous by design.

## Verification basis

- Installed Claude Code:
  v2.1.316,
  native binary at `/var/home/user/.local/share/claude/versions/2.1.316`.
- Documentation:
  [hooks reference][hooks-reference] and [hooks guide][hooks-guide],
  fetched 2026-11-01.
  The reference page's metadata reports a modification date of 2026-10-29.
- Source:
  the minified JavaScript embedded in the native binary,
  extracted with tweakcc 4.4.0's `extractClaudeJsModulesFromNativeInstallation` export,
  the engine behind `tweakcc unpack`.
  Identifiers in that bundle are mangled,
  so the evidence below is string literals and control flow.

## Default: blocking

The reference states the default directly:

> By default,
> hooks block Claude's execution until they complete.

The same blocking waits carry decisions.
Exit code 2 from a `PreToolUse` hook blocks the tool call,
from `UserPromptSubmit` blocks the prompt,
and from `Stop` prevents Claude from stopping;
the reference documents that in [Exit code 2 behavior per event][exit-code-2].

A synchronous handler is bounded by its timeout.
The defaults are 600 seconds for `command`,
`http`,
and `mcp_tool` handlers,
30 seconds for `prompt`,
and 60 for `agent`.
`UserPromptSubmit`,
`PreModelSwitch`,
and `PostModelSwitch` lower the first group to 30 seconds,
`MessageDisplay` to 10 seconds,
and `SessionEnd` handlers share a 1.5-second budget ([common fields][common-fields]).

## Parallel is not asynchronous

The reference:

> When an event fires,
> its matching hooks run in parallel.
> If more than 12 of them are prompt or agent hooks,
> 12 of those run at once and the rest start as running ones finish.

The extracted bundle agrees:
the runner starts every matched handler and awaits all of them before the hook call returns.
Parallel execution shortens wall time;
it does not let Claude continue past the event.

## Opt-in asynchronous command hooks

Two per-handler fields change the behavior.
Both exist only on `type: "command"` handlers:

- `"async": true` runs the handler in the background without blocking.
  The reference:
  "Add `"async": true` to a command hook's configuration to run it in the background without blocking Claude.
  This field is only available on `type: "command"` hooks."
- `"asyncRewake": true` also runs in the background,
  but wakes Claude when the handler exits with code 2.
  The field description says it "runs in the background and wakes Claude on exit code 2";
  the bundled schema description adds "Implies async".

Background output is delivered late.
After the background process exits,
Claude Code delivers the `additionalContext` and `systemMessage` fields on the next conversation turn.
If the session is idle,
the output waits for the next user interaction,
except that an `asyncRewake` handler exiting with code 2 wakes Claude immediately ([limitations][async-limitations]).

Async handlers cannot control the action:
`decision`,
`permissionDecision`,
and `continue` have no effect,
because the action they would have controlled has already completed.
Timeout handling differs too.
Claude Code does not enforce `timeout` on an `async: true` handler,
but still enforces it on `asyncRewake` ([configure an async hook][configure-async]).

The 2.1.316 bundle shows the decision after the command process is spawned:

```js
// Identifiers renamed; log strings verbatim.
if (hook.async || (hook.asyncRewake && canBackground) && !forceSync) {
  log('Hooks: Config-based async hook, backgrounding process ...');
  return { stdout: '', stderr: '', output: '', status: 0, backgrounded: true };
}
```

A handler can also turn asynchronous at run time.
When the first stdout line is JSON containing `"async": true`,
the runner logs `Hooks: Detected async hook, backgrounding process ...` and stops waiting.
The public documentation does not describe that marker;
treat it as an implementation detail.

One more implementation detail lines up with a documented behavior:
at an interactive start or resume,
command hooks from project settings on `SessionStart` are rewritten internally to `async: true`.
The documentation states the observable boundary:
"SessionStart hooks run in the background.
You can type right away... Claude's first response still waits for the hooks to finish,
so their context reaches Claude."
Switching conversations with `/resume` inside a session waits for those hooks instead ([SessionStart][sessionstart]).

## Events that are asynchronous by design

Some events are off the blocking path even when their handlers are synchronous:

- `SessionStart` at launch runs its hooks in the background,
  but Claude's first response still waits for them ([SessionStart][sessionstart]).
- `InstructionsLoaded`:
  "It runs asynchronously for observability purposes" ([InstructionsLoaded][instructionsloaded]).
- `DirectoryAdded`:
  "Claude Code doesn't wait for the hook:
  the add completes immediately,
  and the hook runs in the background with the 600-second default timeout" ([DirectoryAdded][directoryadded]).

Events that observe a completed action,
such as `PostToolUse`,
cannot undo that action even though they are still awaited by default.

## Practical consequence

A hook that must influence what Claude does next,
such as denying a tool call or stopping a turn,
has to stay synchronous;
only the synchronous path can return a decision.
A hook that only reports,
such as running tests after an edit,
can use `"async": true` on a `PostToolUse` handler and let Claude keep working,
at the cost of delivering the result one turn later.

## Sources

- [Hooks reference][hooks-reference],
  fetched 2026-11-01.
- [Automate actions with hooks][hooks-guide],
  fetched 2026-11-01.
- Claude Code v2.1.316 native binary,
  extracted JavaScript bundle,
  using tweakcc 4.4.0.

[hooks-reference]: https://code.claude.com/docs/en/hooks.md
[hooks-guide]: https://code.claude.com/docs/en/hooks-guide.md
[exit-code-2]: https://code.claude.com/docs/en/hooks.md#exit-code-2-behavior-per-event
[common-fields]: https://code.claude.com/docs/en/hooks.md#common-fields
[configure-async]: https://code.claude.com/docs/en/hooks.md#configure-an-async-hook
[async-limitations]: https://code.claude.com/docs/en/hooks.md#limitations
[sessionstart]: https://code.claude.com/docs/en/hooks.md#sessionstart
[instructionsloaded]: https://code.claude.com/docs/en/hooks.md#instructionsloaded
[directoryadded]: https://code.claude.com/docs/en/hooks.md#directoryadded
