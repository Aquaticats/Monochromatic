# Codex 0.166.0 waits for command results unless its execution cells yield

## Purpose and applied change: 2026-11-03T04:31:57-05:00

The user requested:
"Make codex commands always run in the background,
like what we did for Claude Code."
The clarification was:
"Stopping codex from unnecessarily awaiting is the only goal!"

The intended behavior is to start commands,
continue independent work immediately,
and collect results when a dependent step needs them or before completion.
Immediately calling a blocking result collector defeats that purpose.

The native configuration API installed this user-level configuration in `$HOME/.codex/config.toml`:

```toml
# $HOME/.codex/config.toml
[features]
code_mode_only = true

[features.code_mode]
enabled = true
default_exec_yield_time_ms = 0
```

The edit retained the existing feature entries and other settings.
`config/batchWrite` returned `status: "ok"` with `reloadUserConfig: true`.
A subsequent `config/read` confirmed the values in the user configuration layer.
Backups and the API receipt are under `$HOME/temp/agent/codex-background-install/`.

`$HOME/.codex/AGENTS.md` now requires explicit zero-wait shell cells,
immediate independent progress,
and result collection only for dependencies or completion.
The fallback uses `exec_command` with `yield_time_ms: 1` when code mode is unavailable.
The global instruction file was empty before this change.

## Symptom and precedent

Codex's execution cell normally waits before returning control to the agent.
That permits unnecessary serial waiting even when independent work is available.

Claude Code's implementation is documented in
[`bash-output-filter/README.md`](../../package/claude-code-plugin/bash-output-filter/README.md#force-background).
It forces `run_in_background` through a Bash hook,
with exceptions for subagents and bare numeric sleeps.
Codex's existing Bash integration consumes only a command-string rewrite,
as recorded in
[`codex-shell-worker-limits.md`](../decision/codex-shell-worker-limits.md).
The applied Codex configuration instead backgrounds the execution cell containing the command.
It also affects other operations run inside those cells.

## Root cause and the meaning of await

At upstream commit `6ff1e1ca3b30fc8b914720681fcf5fe62e95f856`,
[`codex-rs/core/src/tools/code_mode/mod.rs:128`][default-source]
uses the configured delay only when the call omits its own delay:

```rust
// codex-rs/core/src/tools/code_mode/mod.rs:134
request
    .yield_time_ms
    .get_or_insert(self.default_exec_yield_time_ms);
```

Consequently,
the setting is a default,
not an enforced ceiling.
The instruction explicitly selects zero on shell calls to cover existing sessions
and avoid overriding the default with a longer wait.
An instruction is still not equivalent to Claude's runtime hook enforcement.

JavaScript `await` inside a background cell keeps that cell's work alive.
It does not require the agent to wait for the cell.
The agent receives the running cell identifier and can do other work.
Calling `functions.wait` is the separate operation that collects the cell's result.

The [tool contract in `codex-rs/code-mode-protocol/src/description.rs:33`][lifetime-source] states:

```text
# codex-rs/code-mode-protocol/src/description.rs:33
When the JS code is fully evaluated, the isolate's lifetime ends and unawaited promises are silently discarded.
```

Removing the inner `await` is therefore not the way to keep commands running.
The initial instruction,
"keep the cell alive by awaiting each command",
failed to distinguish cell lifetime from the agent's decision to wait.
It was replaced with explicit instructions to continue independent work
and defer result collection until needed.

## Verification

The running app-server executable and the tested bundle were Codex `0.166.0`.
The installed CLI also reported `0.166.0`.
The older launcher at `/usr/lib/chatgpt/resources/codex` reported `0.159.0-alpha.12.1`;
that launcher was not used for the passing host probe.

A disposable home and local deterministic Responses endpoint exercised the installed app-server.
No model provider was called.
The probe ran with `CPUQuota=200%`,
`MemoryMax=2G`,
`TasksMax=512`,
and `IOWeight=10`.
Its bounded fixture sent this command through a code cell:

```sh
# Disposable host fixture command
sleep 2 && printf background-probe-complete && exit 7
```

The observed cases were:

- A `30000` millisecond default returned the completed command result directly.
- A zero default returned `Script running with cell ID 1` before completion.
  Collecting the cell returned `background-probe-complete` and exit status `7`.
- An explicit `10000` millisecond call override returned the completed result directly,
  despite the configured zero default.
  This demonstrates the limit of configuration-only enforcement.

The runnable local probe is `$HOME/temp/agent/codex-background-probe.ts`.
Its passing evidence is `$HOME/temp/agent/codex-background-BRjx1y/results.json`.
The fixture constructs isolated homes,
has a recursion guard and a deadline,
and uses the exact app-server bundle when given its executable path.
This is a behavior check,
not a timing benchmark or a model-compliance evaluation.

In the configured live session,
explicit zero-wait cells returned running identifiers immediately.
A real shell call still passed through the resource hook,
reported membership in `piwork3285273.slice/codex`,
and returned its output successfully.
Both upstream source selections were opened and their quoted text confirmed with `agent-browser`.

The configuration record and its rendering task were committed as `8da30c2753`.
The scoped `lint:monochromatic-lint` task passed.
The `docs:render:background` task passed,
and its rendered headings,
paragraphs,
links,
and code blocks were inspected.
The browser verification session was closed.

## Verified workaround and limits

The installed configuration gives newly initialized code-mode sessions a zero-wait default.
The explicit call directive also works in an already-running session:

```js
// @exec: {"yield_time_ms": 0}
// functions.exec input
text(await tools.exec_command({
  cmd: "printf background-probe-complete",
  workdir: "/absolute/working/directory",
}));
```

Continue independent work after the running cell identifier arrives.
Collect the output and exit status when a dependency needs them or before declaring completion.
If a nested command returns its own session identifier,
collect that command separately through `write_stdin`.

The already-running code-mode session retained its previous default after configuration reload:
a directive-free delay probe still waited for completion.
Explicit zero-wait calls work immediately;
a new session is needed to rely on the default alone.
The session was not restarted during this task.

## What does not work

- Treating `run_in_background` or `yield_time_ms` as a Bash hook rewrite field.
  The inspected Codex Bash integration rewrites command text only.
- Calling the result collector immediately when independent work remains.
  This recreates the serial waiting the user wants removed.
- Dropping the inner JavaScript `await`.
  This ends the cell before its pending promises have completed.
- Treating zero as a hard cap.
  The explicit-override fixture demonstrates that a longer per-call wait still wins.
- Treating the initial failed fixtures as command-backgrounding evidence.
  Both reported `code-mode host closed its stdout` before running the command
  under the initial `512M` memory and `64` task limits.
  The passing fixture used the app bundle and larger limits.
  These observations do not isolate the startup failure's cause.

## Upstream filing decision

`.out-of-scope/codex-harness.md` permits requested integration work.
No upstream filing is proposed for this configuration change.

- Upstream fault:
  the observed default and explicit override follow the inspected implementation.
- Fixability:
  a runtime ceiling would require a different contract;
  the applied native default addresses ordinary unnecessary waiting.
- Supported use:
  the source defines the default setting and the execution-cell lifecycle.
- Contribution acceptance:
  not assessed because no upstream change is proposed.
- Maintainer interest:
  not established.
- Prototype:
  the native configuration passed the host fixture;
  no upstream patch was created.

The query `gh search issues --repo openai/codex '"yield_time_ms" "hook"'`
returned no matches during this investigation.
That narrow result does not establish the absence of broader background-execution discussions.
There is no new upstream issue or comment to draft for the applied configuration.

[default-source]: https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/core/src/tools/code_mode/mod.rs#L128-L145
[lifetime-source]: https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/code-mode-protocol/src/description.rs#L29-L33
