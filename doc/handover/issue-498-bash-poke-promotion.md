# Issue #498 bash-poke promotion handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `@monochromatic-dev/module-logger` is now `@monochromatic-dev/logger-core`.

## Authority and state

The user asked to resolve Aquaticat/Monochromatic#498 and invoked the grilling skill.
Four rounds settled the design,
and the user accepted the round 4 proposals as written,
so implementation proceeded.
The package is implemented,
 linted,
 type-checked,
 tested,
 and verified through a
real Pi TUI;
 see "Verification evidence".

The working tree also carries unrelated concurrent changes that this task must
not stage or revert:
 `mise.lock`,
 several `doc/audit`,
 `doc/planning`,
 and
`doc/troubleshooting` files,
 `package/dev-script/file-enforcer/README.md`,
 and
untracked `package/music-player/design/questions` evidence.

The issue carries `ready-for-agent`,
which authorizes fix and commit under the issue-tracker skill.
The closing commit body carries `Closes #498`.

## Cross-model caution

Rounds 1 and 2 ran on a different model,
and the user switched to a larger model mid-conversation while asking for care.
Every load-bearing pi fact recorded here was re-verified by reading the installed 0.85.1 distribution after that switch.
One round 2 claim was overstated and is corrected under "Message delivery".

## Verified pi 0.85.1 facts

Cited paths are relative to
`node_modules/.pnpm/@earendil-works+pi-coding-agent@0.85.1_supports-color@10.2.2/node_modules/@earendil-works/pi-coding-agent`.
The TUI and runtime facts come from the minified bundle `dist/bundle/chunks/chunk-JVUZSMYM.js`,
so they are cited by symbol name rather than line number.

### Execution takeover

- `dist/core/extensions/types.d.ts` declares `UserBashEvent` with `command`,
   `excludeFromContext`,
   and `cwd`,
  and `UserBashEventResult` with optional `operations` or `result`.
- TUI `handleBashCommand` renders a returned `result` once through `BashExecutionComponent`,
  records it with `recordBashResult`,
  and returns without awaiting anything,
  so an extension-owned child process keeps running detached.
- The same handler passes a returned `operations` into `await this.session.executeBash(...)`,
  which streams chunks and blocks until exit.
- Background semantics and pi-rendered streaming are therefore mutually exclusive through this event.

### Escape does not cancel an extension-owned job

- TUI `onEscape` branches in order:
   streaming restores queued messages and aborts;
  otherwise `session.isBashRunning` calls `session.abortBash()`;
  otherwise bash mode clears the editor;
  otherwise the double-escape action runs.
- `isBashRunning` is `_bashAbortControllers.size > 0`,
  and `executeBash` is the only place that adds a controller.
- Returning `result` never calls `executeBash`,
  so no controller exists and Escape cannot reach a job this extension spawned.
- This answers the user's round 2 Q15 question:
   Escape does not interrupt it for free.

### Message delivery

- Runtime `sendCustomMessage` behaves as follows.
  `deliverAs: "nextTurn"` queues for the next user prompt and never triggers.
  While streaming with `triggerTurn !== false`,
  `deliverAs: "followUp"` queues through `agent.followUp` and anything else steers.
  Otherwise,
   a truthy `triggerTurn` awaits `_runAgentPrompt`,
  and a falsy one appends or pends without a turn.
- So `{ triggerTurn: true, deliverAs: "followUp" }` fires immediately when idle
  and queues behind current work while streaming.
- Correction to the round 2 framing:
   converting a custom message to LLM context yields
  `{ role: "user", content }`,
  and `details` are excluded.
  The gain over `sendUserMessage` is transcript rendering,
  the `custom` session role,
  and structured renderer data,
  not model-side attribution.
  The model still receives a user-role message either way.

### Extension surface limits

- `ExtensionContext` in `dist/core/extensions/types.d.ts` exposes `ui`,
   `mode`,
   `hasUI`,
   `cwd`,
  `sessionManager`,
   `modelRegistry`,
   `model`,
   `scopedModels`,
   `thinkingLevel`,
  `isIdle()`,
   `isProjectTrusted()`,
   `signal`,
   `abort()`,
   `hasPendingMessages()`,
  `shutdown()`,
   `getContextUsage()`,
   `compact()`,
   and `getSystemPrompt()`.
- It exposes no settings accessor.
  A search for `Setting` in that declaration file returns nothing,
  so pi's `shellPath` and `shellCommandPrefix` settings are unreachable from an extension.
- `pi.events` is a generic extension-to-extension bus.
  Pi publishes no session events to it,
  so `auto_retry_start` and `auto_retry_end` are unobservable from an extension.
- `ctx.ui.setWidget(key, lines, options)` defaults to `placement: "aboveEditor"`.
- `ctx.ui.setStatus(key, text)` writes footer status text and clears on `undefined`.
- `ctx.ui.onTerminalInput(handler)` receives raw terminal input in interactive mode
  and may return `{ consume: true }`.
- `pi.registerShortcut(keyId, { description, handler })` exists,
  and `docs/keybindings.md` lists the defaults a new binding must avoid.
- `dist/core/tools/bash.d.ts` documents `createLocalBashOperations()`,
  and `dist/core/tools/truncate.d.ts` documents `truncateHead`,
   `truncateTail`,
   and `formatSize`.
  Both are rejected by decision,
   not by capability.
- `AssistantMessage` in `@earendil-works/pi-agent-core` `dist/types.d.ts`
  carries `stopReason`,
   whose values include `"error"`,
   plus `errorMessage`.
  Recorded for completeness only,
   because the retry ladder is out of scope.

### Shell resolution and unavailable pi helpers

- `dist/utils/shell.d.ts` documents pi's resolution order:
  an explicit `shellPath` setting,
  then Git Bash or `bash` on Windows,
  then `/bin/bash`,
   `bash` on `PATH`,
   and finally `sh` on Unix.
  Pi does not consult `$SHELL`.
- The package root exports only `getShellConfig` and `getPowerShellConfig` from that module.
- `killProcessTree`,
   `sanitizeBinaryOutput`,
   `trackDetachedChildPid`,
  and `killTrackedDetachedChildren` are declared there but not exported from `dist/index.d.ts`,
  so an extension cannot reach them through the public entry.
- Two consequences for this package.
  Process-tree termination on cancel is ours to implement.
  Output sanitization is ours too:
  pi sanitizes chunks before its own component renders them,
  and unsanitized control characters or lone surrogates can crash width measurement,
  so anything this package renders into a widget or a message must be cleaned first.

### Stale contexts and job lifecycle

- `SessionShutdownEvent.reason` in `dist/core/extensions/types.d.ts` is
  `"quit" | "reload" | "new" | "resume" | "fork"`,
  so every lifecycle transition that can orphan a background job is observable.
- `AgentSession.reload()` in `dist/core/agent-session.js` emits `session_shutdown` with reason `reload`,
  then calls `oldRunner.invalidate()` before building a new runtime.
  The dispose path invalidates the runner the same way.
- `assertActive()` in `dist/core/extensions/runner.js` throws `new Error(this.staleMessage)` once invalidated,
  and the stale message names `ctx.newSession()`,
   `ctx.fork()`,
   `ctx.switchSession()`,
   and `ctx.reload()`.
- Consequence:
   a job that finishes after any of those transitions throws when it calls `pi.sendMessage`
  or touches a captured `ctx.ui`.
  Delivery must be guarded,
   logged through the tagged logger,
   and treated as undeliverable,
  and the job registry must be cleared on `session_shutdown`.
- This machine reports `TERM_PROGRAM=ghostty` at version 1.3.1,
  which implements the Kitty keyboard protocol,
  so `ctrl+shift+<key>` bindings registered through `pi.registerShortcut` are deliverable here.
- `SHELL` is `/bin/bash` and `/bin/bash` exists,
  so the shell-resolution question is about correctness elsewhere rather than about this machine.

### Terminal input interception

- `TUI.handleTerminalInput` in `@earendil-works/pi-tui` `dist/tui.js` runs `inputListeners`
  before the focused component,
  and a listener returning `{ consume: true }` stops all further processing.
  An extension can therefore preempt pi's own Escape handling.
- `ctx.ui.onTerminalInput` maps to `addExtensionTerminalInputListener`,
  which registers into that same listener set.
- `StdinBuffer` in `dist/stdin-buffer.js` accumulates partial sequences
  and holds a buffer equal to a lone ESC for `escapeTimeoutMs` before flushing it.
  `resolveEscapeTimeoutMs` defaults to 10 ms,
  uses 100 ms when `SSH_CONNECTION` or `SSH_TTY` is set,
  and honors `PI_TUI_ESC_TIMEOUT`.
- Because listeners sit downstream of that buffer,
  a delivered chunk is already a complete sequence:
  alt combinations arrive as one `ESC` plus byte chunk,
  and a chunk that is only an escape is a genuine lone Escape.
  This package needs no disambiguation timeout of its own.
- Pi pushes the Kitty keyboard protocol,
   so `matchesKey` in `dist/keys.js`
  accepts a bare `\x1b`,
  the Kitty functional form for codepoint 27 with no modifier,
  and the modifyOtherKeys form.
  An owned matcher must accept at least the bare and Kitty forms.
- This retires the round 3 objection that Escape interception would need
  a reimplemented `PI_TUI_ESC_TIMEOUT` window.

### Process exit and orphaned jobs

- The TUI `shutdown()` path ends in `process.exit(0)`,
  and the signal path exits with 129 or 143,
  so pi never waits on this package's children at quit.
- Those paths call `killTrackedDetachedChildren()`,
  which only covers pids pi tracked itself.
  The tracking helpers are not exported,
  so jobs spawned here are never killed by pi.
- Consequence under the v0.x decision to skip lifecycle work:
  a job still running when pi exits becomes an orphan,
  and its poke is lost because the extension runtime is gone.

### Host tooling for verification

- `tmux` 3.7c is installed at `/usr/bin/tmux`.
  `script` and `socat` exist;
   `expect` and `node-pty` do not.
- `pi --help` confirms `--extension`,
   `--no-extensions`,
   `--no-session`,
  `--no-context-files`,
   `--no-skills`,
   `--mode rpc`,
   and `--offline`.
- `docs/environment-variables.md` documents `PI_CODING_AGENT_DIR`
  as the config-directory override whose default is `~/.pi/agent`.
- `dist/modes/rpc/rpc-mode.js` emits `user_bash` for its `bash` command,
  which gives an integration path that needs no TUI.
- `nano-spawn` 2.1.0 is a catalog dependency in `pnpm-workspace.yaml`
  and is already used by `package/pi-plugin/agent-settled-notification`.
  Its `Subprocess` type exposes `nodeChildProcess`,
  so detached spawning and process-group termination are ours to implement.
- Sibling package shape,
   from `package/pi-plugin/agent-settled-notification`:
  `private: true`,
  `pi.extensions` pointing at `dist/final/node/index.mjs`,
  `exports["./ts"]` pointing at source,
  mise task stubs extending root tasks,
  and `verify:extension` plus `verify:pi-runtime` scripts under `src/`.
- `~/.pi/agent/settings.json` lists packages by absolute directory,
  and `~/.pi/agent/extensions/bash-poke.ts` is still auto-discovered there,
  so promotion deletes that file and adds the package directory.
- No `poke` or `bash-poke` match appears in `forbidden-strings.append.txt`,
  `forbidden-strings.append.local.txt`,
  or the `.cache/forbidden-strings*` rule files.

## Settled decisions

- `!` stays intercepted and background;
   `!!` stays native and hidden from context.
- Live progress is extension-owned,
   not pi-rendered streaming.
- No retry ladder and no usage-limit awareness.
  The extension may be used to wait for a limit reset,
  but it must not know the problem.
- Poke delivery uses `pi.sendMessage` with a registered renderer,
  `triggerTurn: true`,
  and `deliverAs: "followUp"`,
  subject to the attribution correction recorded above.
- This package owns as much as possible:
  its own spawn,
  output accumulation,
  truncation,
  overflow spooling,
  and process-group kill.
  `pi.exec`,
   `createLocalBashOperations()`,
   `truncateHead`,
   and `truncateTail` are out.
- All slash commands are cut,
   so `/poke` does not survive.
- Concurrency stays unrestricted,
   with one poke per finished command.
- Verification includes unit tests over every exported path,
  `verify:extension`,
  `verify:pi-runtime`,
  a committed provider-free fake-terminal check,
  and a documented manual real-provider step.
- Settings live in a global JSON file under `~/.pi/agent/extensions/`,
  and every arbitrary magic number must earn a key there.
- Cancellation emulates Claude Code:
  a lone Escape cancels running jobs,
  implemented through `ctx.ui.onTerminalInput`.
  No slash command and no separate shortcut key.
- Shell resolution is owned here and mirrors pi's documented order,
  `/bin/bash` then `bash` on `PATH` then `sh`,
  with no `shell` settings key.
- Truncation is middle-out:
  the head and the tail are kept and the middle is elided.
- Accepted defaults from round 3:
  `pokeInstruction` `"continue"`,
  `maxPokeChars` 8000,
  `maxPokeLines` 2000,
  `cancelShortcut`,
  and `progressWidget` true.
  The `shell` and `cancelShortcut` keys are now superseded
  by the two decisions recorded directly above.
- No features or fixes are built for session-transition states in v0.x:
  no `session_shutdown` subscription,
  no kill-on-quit,
  no session-switch handling.

## Rejected directions

- Returning `operations` to recover streaming,
  because it re-blocks the editor and removes the reason the extension exists.
- A new prefix intercepted through the `input` event to leave native `!` alone.
- Retry ladder variants,
   including error-kind gating and 429 observation.
- Pi's execution and truncation helpers.
- A slash-command list and cancel surface.

## Round 4 answers

The user accepted every round 4 proposal as written.

- Escape is observed,
   never consumed,
  and cancels only when the agent is idle
  and `ctx.ui.getEditorText()` does not start with `!`.
- Escape cancels every running job.
- Middle-out keeps head and tail with `pokeHeadChars` 2000 and `pokeTailChars` 6000,
  an elision marker stating the elided character count,
  and no separate line cap.
  This retracts the round 3 `maxPokeChars` and `maxPokeLines` proposal.
- Overflow is spooled to `<os.tmpdir()>/pi-bash-poke/`,
  the path is cited in the poke content and carried in `details`,
  and there is no retention policy in v0.x.
- Settings keys are `pokeInstruction`,
  `pokeHeadChars`,
  `pokeTailChars`,
  `progressWidget`,
  `progressTailLines`,
  `progressRefreshMs`,
  `progressTickMs`,
  and `killGraceMs`,
  with unknown keys rejected.
  `progressTickMs` was added during implementation,
   because a silent job never
  produced the output chunks that drove redraws and its elapsed clock froze.
- Spawning is detached,
  and cancel signals the child's process group,
  which is safe precisely because the child leads that group.

## Accepted design

Package `package/pi-plugin/bash-poke`,
named `@monochromatic-dev/pi-plugin-bash-poke`,
private,
with `pi.extensions` pointing at `./dist/final/node/index.mjs`
and `exports["./ts"]` pointing at source,
mirroring `package/pi-plugin/agent-settled-notification`.

Runtime behavior:

- `user_bash` with `excludeFromContext` false is taken over.
  A detached job starts and the handler immediately returns
  `{ result: { output: PENDING_NOTE, exitCode: undefined, cancelled: false, truncated: false } }`.
- `user_bash` with `excludeFromContext` true returns undefined,
  leaving `!!` to native pi handling.
- Each job spawns through this package's own shell resolution,
  `/bin/bash` then `bash` on `PATH` then `sh`,
  with `detached: true`,
  captures stdout and stderr,
  sanitizes control characters and lone surrogates,
  and spools the complete output to `<os.tmpdir()>/pi-bash-poke/`.
- Concurrency is unrestricted and each finished job produces exactly one poke.
- While `progressWidget` is true,
  a widget above the editor shows one line per running job,
  throttled to `progressRefreshMs`,
  carrying at most `progressTailLines` output lines,
  and is cleared when no jobs remain.
- On exit the output is middle-out truncated,
  then sent as a custom message with `customType` `bash-poke`,
  `display: true`,
  content shaped `[bash finished] $ cmd (exit N)` plus the fenced output plus `pokeInstruction`,
  `details` carrying command,
  exit code,
  cancelled flag,
  truncation state,
  elided character count,
  and spool path,
  and options `{ triggerTurn: true, deliverAs: "followUp" }`.
- A renderer registered for `bash-poke` draws the labelled card from `details` and content.
- Cancellation comes from a `ctx.ui.onTerminalInput` listener
  that matches a lone Escape as a bare `\x1b` or the Kitty form `\x1b[27u`,
  never consumes it,
  and cancels every running job when the agent is idle
  and the editor text does not start with `!`.
  Cancel signals the process group with SIGTERM,
  then SIGKILL after `killGraceMs`.
  Cancelled jobs do not poke.
- Message delivery is wrapped so a stale-context throw is logged
  through the tagged logger rather than escaping as an unhandled rejection.
- There are no slash commands,
  no registered shortcuts,
  no retry ladder,
  no usage-limit awareness,
  and no session-lifecycle handling in v0.x.
- Settings load from `~/.pi/agent/extensions/pi-bash-poke.json`
  with the defaults recorded in the round 4 answers,
  and unknown keys are rejected with a diagnostic.
- Logging uses a tagged logger named `pi-bash-poke`
  with per-function inner tags.

Verification:

- Unit tests over every exported code path.
- `verify:extension` drives the built bundle through a fake `ExtensionAPI`.
- `verify:pi-runtime` loads the built package through pi discovery.
- A committed provider-free check drives a real pi TUI inside `tmux`
  against an isolated `PI_CODING_AGENT_DIR`
  with `--no-extensions -e`,
  guarded by an environment flag so it cannot recurse.
- The README documents the manual real-provider step.

Activation,
 which the user performs:

- Delete `~/.pi/agent/extensions/bash-poke.ts`.
- Add the package directory to `~/.pi/agent/settings.json` under `packages`.
- Run `/reload`,
  which the agent cannot invoke from inside a turn.
## Recorded without asking

These follow from settled answers and the naming policy,
so they are adopted rather than put to the user.

- Config file `~/.pi/agent/extensions/pi-bash-poke.json`,
  mirroring `pi-guardrail.json` and `pi-search-fetch.json`.
- Logger tag `pi-bash-poke` through `@monochromatic-dev/module-logger`.
- `customType` value `bash-poke`,
  which is the renderer key and appears in session entries.
- Neither `poke` nor `bash-poke` collides with the forbidden-strings appendices or the rule cache.
- The pending placeholder recorded in context stays a constant rather than a setting.

## Verification evidence

Every claim below was observed,
 not inferred.

### Automated suites

- `mise run //package/pi-plugin/bash-poke:lint` passes with zero oxlint findings
  and a clean type check.
- `mise run //package/pi-plugin/bash-poke:test:unit` passes.
- The same task with `--all` adds the fake-terminal suite,
   which also passes:
  a nested Pi inside `tmux` shows the placeholder,
  the `bash-poke exit 0` card,
  the fenced output,
  and the instruction;
  `!!` produces Pi's own output with no card;
  and a lone Escape cancels a `sleep 300` with the notice and no poke.
- `src/discovery.unit.test.ts` loads the built package through Pi's own
  `discoverAndLoadExtensions`,
  confirms one extension with no errors,
  the two event handlers,
  the renderer,
  and zero commands or shortcuts,
  and shows the unbound-runtime poke being caught and logged rather than escaping.
- The process-group claim is guarded by a real grandchild:
   a job runs
  `sleep 300 & printf %s "$!"; wait`,
  and after cancel the printed pid no longer exists.

### Positive control

The `!!` passthrough guard in `src/register.ts` was temporarily disabled,
the package rebuilt,
and the suites rerun.
All three layers failed as intended:
the wiring unit case,
the discovery case,
and the fake-terminal case,
 which reported that a poke card appeared for a hidden
command.
The guard was committed before the mutation was reverted,
 so no uncommitted work
was at risk.

### Live probes

- A nested Pi started with `-e <built bundle>` and an isolated
  `PI_CODING_AGENT_DIR` showed the placeholder,
  the card,
  Pi's own `!!` execution,
  the progress row,
  the cancellation notice,
  and no surviving `sleep` process.
- A second nested Pi loaded the package through `packages` in a temporary
  `settings.json` instead of `-e`,
  which is the real activation path,
  and produced the same card.
- Both probes were provider-free,
   so each poke turn ended with
  `Error: Unknown provider: unknown` after the card had already rendered.

### Defects the probes found

- The progress row froze at `(0s)` for a silent job,
   because redraws were driven
  only by output chunks.
  Fixed by an elapsed-time ticker armed while jobs are displayed and disarmed
  when none remain,
   configured by `progressTickMs`.
- The progress row survived cancellation,
   because nothing refreshed after the
  job left the registry.
  The same ticker fixes it;
   a re-probe showed the row gone after Escape.
- Routine lifecycle records were logged at info,
   and the logger's console sink
  prints info into the TUI transcript.
  They moved to debug,
   which the sink hides unless verbose logging is requested.
  A re-probe showed a clean transcript.
- After the fixes a re-probe showed the elapsed clock advancing,
   `(2s)` then
  `(7s)` across an eight second window.

## Post-acceptance change

After using the reloaded extension,
the user asked for the trailing `continue` instruction to be eliminated.

- `DEFAULT_POKE_INSTRUCTION` is now the empty string,
  so `buildPokeContent` appends no instruction line and the card ends at its
  closing fence.
- The `pokeInstruction` setting key is retained,
  because deleting an accepted key is a wider change than the request and an
  empty default already removes the instruction from every poke.
- Rationale for future readers:
   a poke triggers its turn through `triggerTurn`
  and `deliverAs`,
  so no imperative is needed to make the model respond.
- The fake-terminal assertion that checked the pane contained
  `constants.DEFAULT_POKE_INSTRUCTION` was replaced.
  With an empty default that check is vacuously true,
  since every string contains the empty string,
  so the case now asserts that no trimmed pane line equals `continue`.
- Poke output is also trimmed of surrounding whitespace before fencing.
  The user's first live poke showed a blank line inside the fence,
  which was the command's own trailing newline,
  and a whitespace-only run now reports `(no output)` instead of an empty block.
  The spool file keeps the exact captured bytes.

## Next action

- Run `/reload` in Pi after any further change to this package,
  because the agent cannot reload from inside a turn.
- The activation edits are done:
   the stopgap moved to
  `~/temp/agent/bash-poke-stopgap-2026-09-18.ts`,
  and the package path added to `packages` in `~/.pi/agent/settings.json`.
- The README's manual real-model check remains the only step no automated suite
  covers.
