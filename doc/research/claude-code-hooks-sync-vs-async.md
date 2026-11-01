# Are Claude Code hooks synchronous or asynchronous?

All web sources in this note were fetched on 2026-11-01.

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
> hooks block Claude's execution until they complete
> ([run hooks in the background][run-in-background]).

The same blocking waits carry decisions.
Exit code 2 from a `PreToolUse` hook blocks the tool call,
from `UserPromptSubmit` blocks the prompt,
and from `Stop` prevents Claude from stopping;
the reference documents that in [Exit code 2 behavior per event][exit-code-2].

Those three events use a hook's output as a decision.
`PreToolUse` runs "before processing the tool call" ([PreToolUse][pretooluse]);
`UserPromptSubmit` "blocks model processing until it completes"
([UserPromptSubmit][userpromptsubmit]);
`Stop` decides whether Claude continues ([Stop][stop]).
An async handler cannot provide that decision,
so a gating hook on these events must stay synchronous.

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
> 12 of those run at once and the rest start as running ones finish
> ([hook handler fields][hook-handler-fields]).

On the default path,
the extracted bundle agrees:
the runner starts every matched handler and waits for all of them before the hook call returns.
Parallel execution shortens wall time;
it does not let Claude continue past the event.

## Opt-in asynchronous command hooks

Two per-handler fields change the behavior.
Both exist only on `type: "command"` handlers
([command hook fields][command-hook-fields]):

- `"async": true` runs the handler in the background without blocking.
  The reference:
  "Add `"async": true` to a command hook's configuration to run it in the background without blocking Claude.
  This field is only available on `type: "command"` hooks."
- `"asyncRewake": true` also runs in the background,
  but wakes Claude when the handler exits with code 2.
  The field description says it "runs in the background and wakes Claude on exit code 2",
  and that the handler's stderr,
  or stdout when stderr is empty,
  "is shown to Claude as a system reminder".

Background output is delivered late.
After the background process exits,
Claude Code delivers the `additionalContext` and `systemMessage` fields on the next conversation turn.
Unlike a synchronous handler's `systemMessage`,
neither field is shown to you.
If the session is idle,
the output waits for the next user interaction,
except that an `asyncRewake` handler exiting with code 2 wakes Claude immediately ([limitations][async-limitations]).

Async handlers cannot control the action:
`decision`,
`permissionDecision`,
and `continue` have no effect,
because the action they would have controlled has already completed
([run hooks in the background][run-in-background]).
Timeout handling differs too.
Claude Code does not enforce `timeout` on an `async: true` handler,
but still enforces it on `asyncRewake` ([configure an async hook][configure-async]).

The 2.1.316 bundle shows the backgrounding branch:

```js
// Identifiers renamed; log strings verbatim.
if (hook.async || (hook.asyncRewake && canBackground) && !forceSync) {
  log('Hooks: Config-based async hook, backgrounding process ...');
  return { stdout: '', stderr: '', output: '', status: 0, backgrounded: true };
}
```

The bundle also contains a runtime marker.
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

## Events with their own timing rules

Some events have their own timing rules even when their handlers are synchronous:

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
cannot undo that action even though they are still awaited by default
([run hooks in the background][run-in-background]).

## Version nuance

In the fetched text,
neither the reference nor the changelog names the release that introduced
`async` or `asyncRewake`
([hooks reference][hooks-reference],
[changelog][changelog]).
The reference documents both without an "added in" note,
so the introducing releases cannot be read off these sources.

An earlier `## 2.1.19` entry says
"Fixed backgrounded hook commands not returning early,
potentially causing the session to wait on a process that was intentionally backgrounded"
([changelog][changelog]).
That wording is ambiguous:
it does not say whether the command used `async: true`
or merely started a shell background process,
so it cannot establish the introduction either.

The changelog's earliest explicit mention of async hooks is under `## 2.1.23`:
"Fixed pending async hooks not being cancelled when headless streaming sessions ended"
([changelog][changelog]).
That is a fix,
not an introduction,
but it shows async hooks existed by that release.
The earliest mention of `asyncRewake` is under `## 2.1.287`:
"Fixed hooks configured with `asyncRewake` waking Claude over and over"
([changelog][changelog]).
That entry is also a fix.

Other releases changed documented async behavior:

- `## 2.1.75`:
  "Suppressed async hook completion messages by default
  (visible with `--verbose` or transcript mode)" ([changelog][changelog]).
- `## 2.1.295`:
  the reference says
  "Before v2.1.295,
  Claude Code added the same text to the conversation again on every resume",
  and states the multiline rule:
  "Spanning several lines requires Claude Code v2.1.295 or later"
  ([limitations][async-limitations],
  [how async hooks execute][async-execute]).

The fetched changelog's newest entry is `## 2.1.316`
([changelog][changelog]).
So async hooks existed by v2.1.23,
`asyncRewake` is referenced by v2.1.287,
and v2.1.295 fixed two async output behaviors,
but the introducing releases are unresolved from these sources.

## Plugin mods and Agent SDK callbacks

These are different hook systems.
Their handlers are callbacks rather than JSON-configured settings hooks,
so they deserve a separate answer.

A mod registers a JavaScript event handler with `on(eventName, handler)`.
The handlers are async functions,
except `turn.step` and `process.spawn`,
which are async generators,
and they form a middleware chain
([mods reference][mods-reference]):
"`next(e)` calls the next handler ... and it resolves to the result"
([mods events][mods-events]).
A handler can await work before it calls `next`,
and the tool call stays pending until it does ([mods events][mods-events]).
The reference gives a hook 10 seconds of its own execution time,
or 50 milliseconds for `prompt.edit`,
and says Claude Code skips a hook that exceeds a time limit
([mods limits][mods-limits]).
The documented model is an awaited chain,
not a background opt-in
([mods events][mods-events],
[mods reference][mods-reference]).

Agent SDK callback hooks are functions as well ([SDK hooks][sdk-hooks]).
The SDK reference states:
"By default,
the agent waits for your hook to return before proceeding"
([asynchronous output][sdk-async]).
A callback can instead return an async output,
`{ "async": true, "asyncTimeout": 30000 }` in TypeScript,
which tells the agent to continue immediately.
Async output "can't block,
modify,
or inject context into the operation
since the agent has already moved on"
([asynchronous output][sdk-async]).

## What this does not mean

- Hooks are not fire-and-forget by default.
  The event waits for its matching handlers
  ([run hooks in the background][run-in-background]).
- Matching handlers run in parallel,
  but that does not let Claude continue.
  The event still waits for them
  ([hook handler fields][hook-handler-fields]).
- `"async": true` is not a general hook field.
  It is documented only for `type: "command"` handlers,
  so it is not a documented option for HTTP,
  MCP tool,
  prompt,
  or agent handlers
  ([configure an async hook][configure-async]).
- An asynchronous handler cannot return a decision.
  Its `decision`,
  `permissionDecision`,
  and `continue` fields have no effect;
  an `asyncRewake` exit code 2 can wake Claude,
  but it is not a blocking decision.
  A `PreToolUse`,
  `UserPromptSubmit`,
  or `Stop` hook that must decide
  has to stay synchronous
  ([run hooks in the background][run-in-background],
  [configure an async hook][configure-async]).
- A shell `&` is not the async option.
  A synchronous handler's output is read only until its process exits,
  and a deny printed later is ignored
  ([hooks that start a background process][bg-process]).
- Async work does not outlive a `-p` session.
  Claude Code kills a still-running async hook at teardown
  and records `cancelled`
  ([run hooks in the background][run-in-background]).

## Practical consequence

A hook that must influence what Claude does next,
such as denying a tool call or stopping a turn,
has to stay synchronous;
only the synchronous path can return a decision
([run hooks in the background][run-in-background]).
A hook that only reports,
such as running tests after an edit,
can use `"async": true` on a `PostToolUse` handler and let Claude keep working,
at the cost of delivering the result one turn later
([hooks guide, how hooks work][guide-work]).

## Measured delivery timing

The "one turn later" cost was measured directly on 2026-11-01,
against the installed v2.1.316.

The probe registered a `UserPromptSubmit` command hook
that injected `PROBE-NONCE-4f3a` as `additionalContext`,
then asked `claude -p` which nonce it sees,
both from a scratch settings file with the `haiku` model.

- Positive control,
  synchronous hook:
  the model answered `PROBE-NONCE-4f3a`.
- Async arm,
  `"async": true`:
  the model answered `NO-NONCE`,
  while a side-effect marker file proved the hook process had run.

So an async handler's `additionalContext` does not reach the turn in
progress.
A hook whose job is same-turn context (a time stamp,
a correction reminder,
child-session results) must stay synchronous.
The marker arm matters:
without it,
"no nonce" would also fit a hook
that never ran.

## Sources

- [Hooks reference][hooks-reference],
  fetched 2026-11-01.
- [Automate actions with hooks][hooks-guide],
  fetched 2026-11-01.
- [Claude Code changelog][changelog],
  fetched 2026-11-01.
- [Mods events][mods-events] and the [mods reference limits][mods-limits],
  fetched 2026-11-01.
- [Agent SDK hooks][sdk-hooks] and [asynchronous output][sdk-async],
  fetched 2026-11-01.
- Claude Code v2.1.316 native binary,
  extracted JavaScript bundle,
  using tweakcc 4.4.0.

[hooks-reference]: https://code.claude.com/docs/en/hooks
[hooks-guide]: https://code.claude.com/docs/en/hooks-guide
[run-in-background]: https://code.claude.com/docs/en/hooks#run-hooks-in-the-background
[hook-handler-fields]: https://code.claude.com/docs/en/hooks#hook-handler-fields
[exit-code-2]: https://code.claude.com/docs/en/hooks#exit-code-2-behavior-per-event
[common-fields]: https://code.claude.com/docs/en/hooks#common-fields
[configure-async]: https://code.claude.com/docs/en/hooks#configure-an-async-hook
[command-hook-fields]: https://code.claude.com/docs/en/hooks#command-hook-fields
[async-execute]: https://code.claude.com/docs/en/hooks#how-async-hooks-execute
[async-limitations]: https://code.claude.com/docs/en/hooks#limitations
[bg-process]: https://code.claude.com/docs/en/hooks#hooks-that-start-a-background-process
[sessionstart]: https://code.claude.com/docs/en/hooks#sessionstart
[instructionsloaded]: https://code.claude.com/docs/en/hooks#instructionsloaded
[directoryadded]: https://code.claude.com/docs/en/hooks#directoryadded
[pretooluse]: https://code.claude.com/docs/en/hooks#pretooluse
[userpromptsubmit]: https://code.claude.com/docs/en/hooks#userpromptsubmit
[stop]: https://code.claude.com/docs/en/hooks#stop
[guide-work]: https://code.claude.com/docs/en/hooks-guide#how-hooks-work
[changelog]: https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md
[mods-events]: https://code.claude.com/docs/en/plugins/mods/events#how-a-hook-handles-an-event
[mods-reference]: https://code.claude.com/docs/en/plugins/mods/reference
[mods-limits]: https://code.claude.com/docs/en/plugins/mods/reference#limits
[sdk-hooks]: https://code.claude.com/docs/en/agent-sdk/hooks
[sdk-async]: https://code.claude.com/docs/en/agent-sdk/hooks#asynchronous-output
