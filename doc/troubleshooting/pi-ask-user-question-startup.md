# Pi ask-user-question 0.0.1 detached startup can lose launch inputs

## Symptom and incident boundary

[Issue #581][issue] reports intermittent file-not-found failures in the detached answer terminal.
The reporter cannot reproduce them on demand.
On 2026-10-04,
the reporter supplied this additional diagnostic:

```text
Requested executable not found. Please verify the command is on
the PATH and try again.
```

This exact string is present in the installed Ghostty 1.3.1 binary.
Ghostty tag `v1.3.1`,
commit `332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28`,
emits it after its command-execution call returns `error.FileNotFound`:

```zig
// Ghostty src/Command.zig:228, 237 to 240 at v1.3.1.
const err = posix.execvpeZ(self.path, argsZ, envp);
// Selected error branch:
error.FileNotFound => stderr.print(
    \\Requested executable not found. Please verify the command is on
    \\the PATH and try again.
```

This identifies the emitting tool and failed operation,
not the missing pathname in every historical occurrence.
The message alone does not establish a PATH configuration problem.

Local Pi logs contain requests removed about 30 seconds after launch,
including calls on 2026-09-28,
2026-09-29,
and 2026-10-01.
Those agree with the startup deadline but do not identify why the helper failed to connect.
The reproduced cases in this document remain separate from those historical incidents.

## Follow-up incident on 2026-10-08

The reporter supplied `Screenshot_20261008_064047.png`.
Node 26.10.0 reports `MODULE_NOT_FOUND` for the request-private helper,
not the installed bundle:

```text
Error: Cannot find module '/tmp/pi-ask-user-question-Vh8iRc/answer-helper.mjs'
```

The matching local log is `node_modules/.monochromatic/2026-10-08T02-36-56.899Z.log.jsonl`:

- Line 31 creates the workspace at timestamp `1791447393573`.
- Line 32 prepares the helper at `1791447393574`.
- Lines 60 and 61 dispatch the terminal command at `1791447393579`.
- Line 62 closes the channel at `1791447423577`.
- Line 63 removes the workspace at `1791447423579`.

The removal occurs exactly 30 seconds after launch dispatch.
The matching Pi session's `ask_user_question` tool result at `2026-10-08T08:17:03.579Z`
reports `The answer helper did not connect within 30 seconds.`
Its timestamp is also `1791447423579`,
confirming deadline expiry rather than an inferred user cancellation.
The screenshot establishes that Node subsequently attempted the deleted path.
This is evidence for the lifetime gap explicitly left open by the previous fix.
It is not evidence that a package rebuild removed this helper.

A separate confirmed defect was in the pre-fix `package/terminal-exec/cli/src/build-command.ts`:
it filtered every token beginning with `--gtk-single-instance`,
including the user's explicit `--gtk-single-instance=false`.
The log retains that flag through tokenization but omits it from the final launch vector.
The user explicitly authorized preserving the flag and changing the launcher on 2026-10-08.
Its contribution to the delayed startup is not established.

The existing [locked-session investigation](ghostty-locked-session-no-command.md)
documents why elapsed wall time is not a reliable terminal-start failure signal.
The source at Ghostty `v1.3.1`,
`src/apprt/gtk/class/surface.zig:3224`,
still says initialization waits for the first resize;
line 3344 calls `self.initSurface()` from that path.
No contemporaneous screen-lock state was captured for the screenshot incident.
Do not assign the historical delay to the screen locker without that evidence.

### Current request lifetime

The fix preserves the separate default-terminal editor workflow and model blocking.
It does not replace the editor with a modal dialog or add a persistent broker.

`package/pi-plugin/ask-user-question/src/answer-channel-auth.ts:114`
uses a disposable notice timer rather than a cancellation deadline:

```ts
// package/pi-plugin/ask-user-question/src/answer-channel-auth.ts
using notice = setTimeout(
  function reportPendingStartup(): void {
    l.warn('The answer terminal has not connected yet. The question remains active; unlock or reveal the desktop, inspect the terminal, or cancel the tool in Pi.');
  },
  HELPER_START_NOTICE_MS,
);
```

The startup listener remains referenced.
The 30-second notice neither deletes the workspace nor lets the model continue.
A terminal that never executes its command now requires cancellation in Pi.
This is intentional:
wall-clock delay does not establish that a desktop launch failed.
The separate 5-second authentication deadline begins only after a candidate socket connects.
A regression exposed Node's `ERR_STREAM_PREMATURE_CLOSE` when that deadline destroyed an idle candidate;
commit `724fb299a` contains that failure within the candidate and continues accepting helpers.
Independent review identified the corresponding reset case;
its new regression reproduced `ECONNRESET` expiring the question.
Commit `c7b16bd39` rejects all failed unauthenticated candidates while preserving caller cancellation.

`package/pi-plugin/ask-user-question/src/request-external-answer.ts:249`
launches inline JavaScript with paths as separate argument tokens:

```ts
// package/pi-plugin/ask-user-question/src/request-external-answer.ts
'--input-type=module',
'--eval',
ANSWER_BOOTSTRAP,
helperPath,
'--request',
workspace.requestPath,
```

`package/pi-plugin/ask-user-question/src/helper-bootstrap.ts:15`
loads the self-contained snapshot into memory before importing it:

```js
// package/pi-plugin/ask-user-question/src/helper-bootstrap.ts, embedded bootstrap
const source = await readFile(helperPath);
await import('data:text/javascript;base64,' + source.toString('base64'));
```

Only `ENOENT` naming the exact helper or request path means that the request ended.
Malformed request JSON and other loading failures remain errors.
No token or answer enters argv.
Reading the bundle also avoids interpreting a valid filesystem name as a module URL.
The quoted-path fixture exposed `ERR_INVALID_MODULE_SPECIFIER` with a backslash filename;
commit `2e0df35a7` changed the bootstrap from file-URL import to byte loading.
The fixture additionally exercises quotes,
a newline,
percent and hash characters,
and shell separators.
The final `answer-helper.mjs` was inspected for module-location dependencies:
its imports name only Node built-ins and it contains no `import.meta` references.
The imported module therefore does not need its former filesystem location.

`package/pi-plugin/ask-user-question/src/answer-channel.ts:157`
sends `HELPER_READY` only after authentication and a cancellation check.
`package/pi-plugin/ask-user-question/src/helper-connection.ts:156`
requires the complete acknowledgement before returning to editor startup:

```ts
// package/pi-plugin/ask-user-question/src/helper-connection.ts
if (state.acknowledgement === HELPER_READY) {
  signal.throwIfAborted();
  rl.debug('requester accepted answer helper');
  socket.resume();
  return;
}
```

The channel owns every connected socket,
including unauthenticated candidates.
The helper owns its connection and terminates the editor when the requester disconnects.
Expected cancellation exits without a raw shutdown stack trace.
`request-external-answer.ts:268` observes launcher and channel together:

```ts
// package/pi-plugin/ask-user-question/src/request-external-answer.ts
const [, completion,] = await Promise.all([
  launchAnswerTerminal(),
  completionTask,
]);
```

Cancellation therefore settles the tool even if the launcher has not returned yet.
A later launcher rejection remains observed.
A later successful launch receives the cancellation-aware bootstrap.
Commit `269d21002` additionally opens the cancellation registry scope before terminal resolution.
A regression demonstrated that shutdown during that first asynchronous operation could otherwise be forgotten,
allowing an already-cancelled request to open a terminal.

`package/terminal-exec/cli/src/build-command.ts:122`
now retains the parsed desktop-entry arguments directly:

```ts
// package/terminal-exec/cli/src/build-command.ts
...terminal.execTokens,
```

This preserves both explicit `false` and explicit `true` values.
The terminal's single-instance behavior belongs to its configuration,
not a silent policy override in this library.
Preserving the flag is verified independently of the cause of the historical startup delay.

### Follow-up verification evidence

Commit `668d2781d` introduced delayed-start,
late-after-cancellation,
and terminal-flag regressions before the implementation.
The initial harness attempt failed at setup because a test dependency was unavailable and only one package had rebuilt.
After correcting the fixture and rebuilding each package,
the old implementation failed as intended:

- At 31 seconds the request had already settled instead of remaining answerable.
- Both caller and session cancellation followed by late execution reproduced the screenshot's `MODULE_NOT_FOUND`.
- The command builder removed `--gtk-single-instance=false`,
  `--gtk-single-instance=true`,
  the bare flag,
  and even an unrelated longer prefix.
- An unrelated terminal option remained present as the positive control.

The implemented path passes the real delayed-start test:
a helper launched after 31 seconds returns the exact multiline answer.
This test delays execution at the injected launcher boundary and then runs the production argv directly.
It reproduces the requester's delayed-execution boundary,
not an actual locked-desktop launch.
The real Ghostty verifiers separately cover ordinary desktop startup.
Lifecycle tests exercise cancellation during a pending launcher,
active editor termination with captured stderr,
an unavailable editor,
missing versus malformed request files,
and idle authentication candidate cleanup.
Acknowledgement tests exercise complete,
fragmented,
missing,
and invalid replies from a fixture requester.

A fresh Pi 1.0.4 TUI process was also exercised under a private PTY,
with isolated `PI_CODING_AGENT_DIR`,
no discovered extensions,
offline mode,
no model request,
and an explicitly guarded test extension.
That extension captures and registers the production tool,
then invokes its real `execute` callback with the host's TUI context and a scripted editor.
It verifies multiline answer details and empty-answer cancellation through the actual default Ghostty terminal.
It does not simulate a model response or human keyboard entry.
The disposable driver and extension are retained under `${HOME}/temp/agent/ask-host-*.ts`.
No user editor configuration or existing session was modified.

The package `verify:terminal` and `verify:extension` tasks also passed after the lifecycle change.
The acknowledgement guard was also removed in a disposable copied bundle.
The silent-requester regression then failed because an editor started without acknowledgement;
restoring the original bundle made the same tests pass.
Neither the installed artifact nor the working source was mutated by this guard test.

Final post-bootstrap validation is recorded in [Completion on 2026-10-08](#completion-on-2026-10-08).

### Follow-up limits and rejected approaches

- Increasing the destructive timeout would only move the same race.
- Retaining every cancelled workspace would accumulate private request data without preserving a live consumer.
- A permanent broker is unnecessary while the answer belongs to one live Pi tool call.
- A cancelled late terminal exits cleanly;
  its message can disappear as the terminal closes and is not promised as a persistent notification.
- Killing Pi before the terminal executes can invalidate Linux's `/proc/<pid>/exe` runtime path.
  This change preserves the existing protection against runtime unlink while Pi is alive;
  it does not make questions survive requester process death.
- No screen-lock state was captured for the reported request,
  and no test deliberately locked the user's desktop.
- Real desktop verification covers Linux with Ghostty,
  not Windows or macOS.

## Historical fixes from 2026-10-04

The source excerpts and line numbers in this section describe the earlier implementation.
The 2026-10-08 follow-up supersedes its startup deadline and `AnswerHelperStartupTimeoutError` export.
The runtime-retention and private helper-snapshot protections remain in place.

### Installed runtime disappears while requester remains alive

The old requester passed `process.execPath` directly to the detached terminal.
A disposable consumer copies Node,
starts that copy,
unlinks it,
and invokes the actual built requester.
Before the fix,
its helper launch fails with `spawn <fixture>/node ENOENT`.

On Linux,
`package/pi-plugin/ask-user-question/src/helper-launch.ts:94`
now selects the executable retained by the live requester:

```ts
// package/pi-plugin/ask-user-question/src/helper-launch.ts
const liveExecutable = `/proc/${String(pid,)}/exe`;
await access(liveExecutable, constants.X_OK);
return liveExecutable;
```

When procfs is inaccessible,
or on another platform,
the original runtime path is checked before launch.
Missing runtime access raises `AnswerLaunchError` in Pi with a restart or file-access remediation.
The procfs protection is Linux-specific;
the fallback check cannot prevent a later unlink on another platform.
No alternate executable is selected from PATH.

### Installed helper disappears after terminal launch is prepared

A separate disposable consumer removes its copied installed helper immediately before executing the prepared command.
Before the fix,
Node reports `MODULE_NOT_FOUND` for `answer-helper.mjs`.
This demonstrates dependence on a mutable installation path,
not that a particular historical build deleted that path.

`package/pi-plugin/ask-user-question/src/helper-launch.ts:160`
copies the self-contained helper into the private request workspace before terminal launch:

```ts
// package/pi-plugin/ask-user-question/src/helper-launch.ts
const helperPath = join(workspace.directory, 'answer-helper.mjs');
await copyFile(sourcePath, helperPath, constants.COPYFILE_EXCL);
await chmod(helperPath, PRIVATE_FILE_MODE);
```

The helper has mode `0600` inside the existing private request directory.
The regression verifies its location and mode,
executes the real copied helper and scripted editor,
and checks the returned multiline answer.
A missing source bundle fails in Pi before the terminal opens.

`package/pi-plugin/ask-user-question/src/request-external-answer.ts:211`
prepares that snapshot,
and the launch uses `runtimePath` and the request-owned `helperPath`:

```ts
// package/pi-plugin/ask-user-question/src/request-external-answer.ts:240
command: [
  runtimePath,
  helperPath,
  '--request',
  workspace.requestPath,
],
```

The helper snapshot executes outside the fixture installation's dependency tree.
The built helper's observed imports are Node built-ins only.
This matters because copying a bundle with external package imports would not be sufficient.

### Requester exits before detached startup finishes

The initial real-terminal verifier exited with Node status `13`:

```text
Warning: Detected unsettled top-level await
```

The terminal subsequently attempted to execute a procfs path whose owning process no longer existed.
A regression consumer whose launcher returns at process spawn,
then unreferences its child,
reproduced the requester exit independently of Ghostty.
Depending on scheduling,
the child encountered an unavailable executable or `ECONNREFUSED` after the requester exited.

The previous `server.unref()` immediately after listening was removed from
`package/pi-plugin/ask-user-question/src/answer-channel.ts`.
The listener remains referenced until authentication,
deadline,
or disposal.
This keeps an otherwise idle requester alive during the startup it is awaiting.

### Startup timeout looked like user cancellation

The previous channel deadline surfaced as `The operation was aborted`.
`package/pi-plugin/ask-user-question/src/answer-channel-auth.ts:175`
now translates only a failure caused by its own deadline:

```ts
// package/pi-plugin/ask-user-question/src/answer-channel-auth.ts
if (deadlineSignal.aborted && (startupSignal.reason === deadlineSignal.reason)
  && ((error === deadlineSignal.reason) || (Error.isError(error) && (error.cause === deadlineSignal.reason))))
  throw new AnswerHelperStartupTimeoutError(error);
```

Caller aborts retain their cancellation error.
Protocol failures are not relabeled merely because the deadline also fired.
The startup diagnostic states the elapsed deadline and asks for the detached-terminal error,
without asserting an unobserved cause.

## Verification

Environment:
Node `26.10.0`,
Pi `1.0.2`,
Ghostty `1.3.1`,
Linux.

Regression commit `afd470f3f` failed on both missing launch dependencies while its normal positive control passed.
Commit `04467c4d9` added the detached-start positive-lifetime regression;
it failed with requester status `13` before `c443026a6` retained the listener reference.
The tests operate on copied artifacts and a disposable runtime,
not the installed runtime or package output.

```sh
# Repository root.
mise run //package/pi-plugin/ask-user-question:build
mise run //package/pi-plugin/ask-user-question:test:unit
mise run //package/pi-plugin/ask-user-question:lint:types
mise run //package/pi-plugin/ask-user-question:lint:oxlint
mise run //package/pi-plugin/ask-user-question:verify:extension
mise run //package/pi-plugin/ask-user-question:verify:terminal
```

The terminal task opens the configured default terminal with a scripted editor.
It checks multiline submission and empty-answer cancellation without keyboard injection or changing editor settings.
It must return actual answers;
terminal process startup alone is not a pass.

### Working cases

- Normal helper execution and exact multiline return.
- Installed helper removed after preparation.
- Copied running Node executable unlinked on Linux.
- Launcher returns at spawn without retaining the child.
- Original runtime fallback when procfs is unavailable.
- Caller abort remains cancellation rather than startup timeout.

### Deliberate failure cases

- Missing helper before preparation raises `AnswerLaunchError` without opening a terminal.
- Missing original runtime without procfs raises `AnswerLaunchError` with its underlying `ENOENT`.
- Helper never connects:
  startup wait raises `AnswerHelperStartupTimeoutError` at the real 30-second deadline.
- Historical intermittent terminal failures remain unclassified until their failing launch path is captured.

## Verified workarounds and limits

The Linux live-executable path and request-owned helper snapshot are covered by actual child-process regressions.
They retain the current interpreter rather than silently changing Node versions.
Their tradeoff is dependence on the requester remaining alive;
forcefully terminating Pi is still cancellation,
not a supported way to leave an answer editor running.

The snapshot is deleted with the request workspace.
A terminal that starts only after cancellation or the startup deadline can still find an expired request missing.
This is not protected by copying the helper.
Do not infer that the historical incident was fixed merely because these launch-dependency tests pass.

## What does not work

- Treating every generic abort in Pi's session history as a user cancellation.
- Assuming the current package build task pre-cleans the output directory.
  The current task invokes Rolldown directly;
  the old shared-config comment about task-owned pre-cleaning is not evidence that it happens.
- Adding an artificial keepalive timer only to the verification harness.
  The requester itself must retain the lifetime of the startup it awaits.
- Changing editor-command validation to hide a faulty verification argument.
  The first cancellation verifier passed an empty argument,
  which correctly triggered `Answer helper request editorCommand must be a nonempty string array.`
  Both reporter-provided screenshots from 2026-10-04 10:12 and 10:13 show that same verifier failure.
  The verifier now JSON-encodes its answer argument so even the empty answer has a nonempty command token.
- Publishing the supplied desktop screenshots unchanged.
  They contain unrelated background content through terminal transparency;
  only their diagnostic text is recorded here.

Moving the helper does not change the logger's search root.
`package/module/logger/src/sink/file.ts:174` starts from the working directory:

```ts
// package/module/logger/src/sink/file.ts
const nodeModulesDir = await findNodeModulesUp({
  cwd: process.cwd(),
  stat,
  dirname,
  join,
});
```

A helper failing before logger initialization may still leave only terminal output.
Missing log entries are therefore not proof that a helper ran successfully.

## Upstream filing decision

No external filing is proposed.
The existing repository issue is [#581][issue].
Ghostty's diagnostic correctly reports an execution failure in the reproduced case.

1.  Upstream fault:
    not established;
    reproduced defects are in the first-party launch lifetime and dependency handling.
2.  Upstream fixability:
    no upstream change is needed for the demonstrated first-party fixes.
3.  Supported use case:
    the verification exercises Ghostty's ordinary command launch,
    not an unsupported extension protocol.
4.  Contribution policy:
    not evaluated because no upstream contribution is proposed.
5.  Likelihood of upstream action:
    not evaluated;
    no upstream defect has been established.
6.  Compatible prototype:
    first-party fixes and regressions are committed;
    no Ghostty patch was made.

## Final verification and tracking

On 2026-10-04,
all package verification commands in the Verification section passed.
Oxlint reported `Found 0 warnings and 0 errors.`
The regression output included every startup scenario,
including `detached-start`,
and both deadline and caller-abort cases.
The real Ghostty task returned:

```text
Detached terminal verified: answered
Detached terminal verified: cancelled
```

The extension verifier confirmed sequential registration,
shutdown cleanup registration,
and the helper artifact.
Markdown was rendered with installed Marked `18.0.11` and inspected.
Independent review found no remaining correctness blocker in the reproduced launch paths.

The built output is intentionally ignored by `.gitignore:53` (`dist/`),
not tracked;
source commits were auto-pushed and local artifacts rebuilt.
The source fixes are `1db6774ac`,
`744e1d128`,
`d0808a498`,
and `c443026a6`;
subsequent fixture and documentation commits complete their verification.

On 2026-10-04,
#581 was closed at the reporter's explicit request with the comment `cannot reproduce`.
This supersedes the investigation's initial recommendation to keep it open.
No historical timeout was retroactively attributed to a particular missing file.
Use a fresh Pi process to exercise the rebuilt extension rather than an already loaded bundle.

## Completion on 2026-10-08

The final implementation includes `5596ad743`,
`724fb299a`,
`2e0df35a7`,
`269d21002`,
and `c7b16bd39`.
Both affected packages were rebuilt locally.
Their type checks,
Oxlint checks,
and complete package test suites passed;
Oxlint reported `Found 0 warnings and 0 errors.` for each package.

```sh
# Repository root, reproducible package checks.
mise run //package/terminal-exec/cli:build
mise run //package/terminal-exec/cli:lint
mise run //package/terminal-exec/cli:test:unit
mise run //package/pi-plugin/ask-user-question:build
mise run //package/pi-plugin/ask-user-question:lint
mise run //package/pi-plugin/ask-user-question:test:unit
mise run //package/pi-plugin/ask-user-question:verify:terminal
mise run //package/pi-plugin/ask-user-question:verify:extension
```

The final real-terminal launch log contains the user's exact `--gtk-single-instance=false` token.
Consumer-boundary results after the final implementation were:

```text
Detached terminal verified: answered
Detached terminal verified: cancelled
ask-user-question extension verified: sequential question tool, shutdown cleanup, and detached helper artifact
Real Pi TUI host verified: answered
Real Pi TUI host verified: cancelled
```

The final package suite includes the 31-second delayed-start regression,
late-after-caller and session cancellation,
shutdown during terminal resolution,
idle and reset candidates,
active-editor cancellation with empty stderr,
acknowledgement framing,
and quoted launch paths.
Tests deliberately exercising operational failures still emit their expected tagged diagnostics.
No claim is made that the whole test runner's stderr is empty.

The updated package READMEs and this investigation were rendered with Marked `18.0.11` and inspected.
The terminal README's pre-existing broken fences were repaired so its configuration section renders as prose.
Independent review informed the reset-candidate regression and the helper's module-location inspection.

Existing Pi processes retain their already-loaded extension module.
A fresh Pi process loads the rebuilt implementation.
No editor setting,
terminal preference,
provider configuration,
or existing user session was changed.
Unrelated concurrent working-tree changes were left untouched.

[issue]: https://github.com/Aquaticat/Monochromatic/issues/581
