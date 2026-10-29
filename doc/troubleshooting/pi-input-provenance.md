# Pi 0.87.1 input labels and copied entries do not establish human authority

## Symptom and relevance

An auto-mode authorization collector must not equate a user-role transcript message with human authorization.
Pi 0.87.1 distinguishes input channels during its input event,
but ordinary message construction does not retain that source field.
A programmatic prompt without an explicit source defaults to `interactive`.
Input transforms can also alter text while retaining the channel label.

This is relevant to the axiom migration because models may match an effect against a trusted human request,
but cannot establish who authored a message from its prose or role.
The user already requires this distinction.
No production collector or SDK patch has been implemented.

## Source identity

Installed package:
 `@earendil-works/pi-coding-agent@0.87.1`.
Upstream repository: <https://github.com/earendil-works/pi>.
Tag `v0.87.1` resolves to `f07218c4d4bbc12bef056a7058c3dd49dfe41abe`.
Read-only clone:
 `~/temp/agent/pi-input-provenance-2026-09-26`.
The cloned license is MIT.

The probe uses inspected compiled method bodies from the installed package,
not an SDK startup command.
Copied `dist/core/agent-session.js` SHA-256:
`5ebfae51db5a900596145159428e7cb57d195af9d54a28f41d4ac8ff1bfd5729`.
Copied `dist/core/extensions/runner.js` SHA-256:
`67c7ca2d24197ff46cb5f0a49d7c19a76825ab7c66bc942015a484b550396441`.
Source paths in this document are relative to the pinned clone.
No upstream source was changed.

The installed extension,
SDK,
message-type,
and session-format documentation and input-transform example were read.
The upstream input-event and session-context-edit tests were inspected,
not executed as complete suites.

## Root cause trace

### Source is supplied to input handlers, not automatically authenticated

`packages/coding-agent/src/core/agent-session.ts:1633-1639`
passes the caller option or a default to input handlers:

```typescript
// packages/coding-agent/src/core/agent-session.ts:1633-1639, selected arguments
const processedInput = await this._runInputHandlers(
  text,
  options?.images,
  options?.source ?? "interactive",
  this.isStreaming ? options?.streamingBehavior : undefined,
);
```

The source tag describes the supplied channel.
Its default is not proof that a person typed the submitted string.
A trusted host may establish that separately at its own input interface;
the model must not infer it from the label.

The official extension submission path at `agent-session.ts:2029-2035`
explicitly supplies `source: "extension"`:

```typescript
// packages/coding-agent/src/core/agent-session.ts:2029-2035
await this.prompt(text, {
  expandPromptTemplates: options?.expandPromptTemplates ?? false,
  streamingBehavior: options?.deliverAs,
  images,
  source: "extension",
});
```

That input still becomes a user-role message.
A source distinction in the transient event does not imply a source distinction in the message object.

### Message construction omits the source field

At `agent-session.ts:1724-1728`,
the constructed message contains role,
content,
and time:

```typescript
// packages/coding-agent/src/core/agent-session.ts:1724-1728
messages.push({
  role: "user",
  content: userContent,
  timestamp: Date.now(),
});
```

The provider-facing `UserMessage` declaration at `packages/ai/src/types.ts:509`
also has no source member.
The installed session-format documentation defines message entries as storing `AgentMessage`.
This does not rule out a separate application-owned provenance ledger;
it rules out recovering a unique input source from this constructed message alone.

With a fixed clock and identical text,
the actual inspected method produced indistinguishable message objects
for explicit `interactive` and `extension` inputs.
The callback-source positive control still distinguished them.
The same loss was observed through the queued extension follow-up method.
No real session file was written by this probe.

### Transforms retain the input channel label

`packages/coding-agent/src/core/extensions/runner.ts:1412-1457`
chains input handlers.
Each receives the current transformed text and the same source parameter:

```typescript
// packages/coding-agent/src/core/extensions/runner.ts:1424-1436, selected statements
const event: InputEvent = {
  type: "input",
  text: currentText,
  images: currentImages,
  source,
  streamingBehavior,
};
currentText = result.text;
currentImages = result.images ?? currentImages;
```

In the offline control,
an earlier handler appended `AUTOMATED_ADDITION`.
A later handler received that modified body with source still `interactive`,
and the altered text entered the constructed user message.
A late collector must not certify the transformed body as untouched human text merely from this label.

### Throwing from an input collector does not veto prompt delivery

The same method catches handler exceptions,
reports them through `emitError`,
and continues its handler loop.
If no handler changes or handles the input,
it returns `continue`.
The offline error control recorded one collector error and still constructed one user message.

A collector therefore needs an explicit unresolved-origin state that the guarded-action policy checks.
Do not rely on an input-handler exception to prevent later actions.
A missing witness must not silently promote the new text or stale authorization.
This is a proposed consumer requirement,
not a verified collector implementation.

### Registered commands run before input callbacks

At `packages/coding-agent/src/core/agent-session.ts:1615-1623`,
registered extension commands are dispatched before the source-bearing input event:

```typescript
// packages/coding-agent/src/core/agent-session.ts, selected statements
if (expandPromptTemplates && text.startsWith("/")) {
  const handled = await this._tryExecuteExtensionCommand(text);
  if (handled) {
    preflightResult?.(true);
    return;
  }
}
```

The current project handler at
`package/pi-plugin/auto-mode/src/guard-command.ts:91-111`
appends a null reset or a text directive without checking `ctx.hasUI` or receiving input-origin evidence:

```typescript
// package/pi-plugin/auto-mode/src/guard-command.ts, selected append calls
if (trimmed === 'reset') {
  pi.appendEntry(TRUST_ENTRY_TYPE, null);
  ctx.ui.notify('Trust directives cleared for this session.');
  return Promise.resolve();
}
pi.appendEntry(TRUST_ENTRY_TYPE, trimmed);
```

An isolated actual-method composition confirmed that
`prompt('/guard Allow reading /fixture/project/example.txt.', { source: 'rpc' })`
reached the registered handler and appended its directive with `hasUI: false`,
without invoking the input callback.
Ordinary text reached that callback in the positive control.
This is a programmatic SDK-method probe,
not a tested live RPC client exploit or a claim that ordinary model tools can call this method.
An input-callback-only collector therefore does not cover this grant-creation route.
A string-shaped trust entry cannot by itself distinguish this route from an explicit UI confirmation.

### Branch reset and fork identity are separate from human confirmation

Current project `package/pi-plugin/auto-mode/src/context.ts:136-148`
projects directives from the active branch only:

```typescript
// package/pi-plugin/auto-mode/src/context.ts, selected projection
for (const entry of ctx.sessionManager.getBranch()) {
  if (isTrustEntry(entry)) {
    if (entry.data === null)
      directives.length = 0;
    else
      directives.push(entry.data);
  }
}
```

Pi's `packages/coding-agent/src/core/session-manager.ts:1467-1477`
walks parents from the current leaf,
and `:1572-1577` changes that leaf without deleting entries:

```typescript
// packages/coding-agent/src/core/session-manager.ts:1572-1577
branch(branchFromId: string): void {
  if (!this.byId.has(branchFromId)) {
    throw new Error(`Entry ${branchFromId} not found`);
  }
  this.leafId = branchFromId;
}
```

The method probe reset an active grant,
moved to the grant entry before the reset,
and observed the directive again.
Returning to the reset entry cleared it again.
This verifies branch-local projection on the fixture,
not the desired migration revocation policy.

At `session-manager.ts:1625-1675`,
`createBranchedSession()` copies the selected path under a new session ID,
retaining non-label entry IDs:

```typescript
// packages/coding-agent/src/core/session-manager.ts, selected statements
const path = this.getBranch(leafId);
pathWithoutLabels.push(
  entry.type === "compaction"
    ? {
        ...entry,
        parentId: pathParentId,
        firstKeptEntryId:
          entry.firstKeptEntryId === entry.id
            ? entry.id
            : (replacementByLabelId.get(entry.firstKeptEntryId) ?? entry.firstKeptEntryId),
      }
    : { ...entry, parentId: pathParentId },
);
const newSessionId = createSessionId();
```

The in-memory fork control copied the grant's original entry ID and text into the new session.
A bare entry ID is consequently not sufficient to bind authority to an original session.
Verified lineage and allowed grant lifetime are separate requirements.
No policy choosing whether ordinary grants should inherit into forks was implemented.

### The existing editor owner returns an answer, not an approval-scope witness

The first-party `ask-user-question` package is version `0.0.1`.
Its `src/tool.ts:144` accepts an unused `_toolCallId`;
`src/tool.ts:165` sends only working directory and cancellation to the requester:

```ts
// package/pi-plugin/ask-user-question/src/tool.ts
const outcome = await requestAnswer({
  cwd: ctx.cwd,
  ...(signal === undefined ? {} : { signal, }),
},);
```

Startup update on 2026-10-04:
the helper now runs from a request-private snapshot instead of the installed `dist/final/node/answer-helper.mjs` path.
On Linux,
the requester uses its live procfs executable when accessible.
These lifetime protections do not change the answer-authentication or approval-scope conclusions.
See [the startup investigation](pi-ask-user-question-startup.md).

The helper authenticates its channel before running the editor.
`package/pi-plugin/ask-user-question/src/helper-core.ts:61` writes the request token;
line 76 then invokes the editor:

```ts
// package/pi-plugin/ask-user-question/src/helper-core.ts
socket.write(`${request.token}\n`,);
const status = await runEditor({
  answerPath: request.answerPath,
  editorCommand: request.editorCommand,
  signal: controller.signal,
},);
```

This links a helper to its pending request,
not an answer to an independently retained approval scope.
`package/pi-plugin/ask-user-question/src/request-external-answer.ts:260`
reads the raw file after completion and normalizes one final editor line ending:

```ts
// package/pi-plugin/ask-user-question/src/request-external-answer.ts
const rawAnswer = await readWorkspaceAnswer({ workspace, },);
const answer = normalizeEditorAnswer({ text: rawAnswer, },);
```

Under Q23 A,
the concrete host confirmation workflow may be trusted without defending against same-account interference.
That assumption does not make role labels,
helper tokens,
or copied answers into original scope-bound confirmations.
The retained writer/response/scope binding is still required.

## Verification

Private harness:
 `~/temp/agent/pi-input-provenance-probe-2026-09-26`.
It extracts the inspected compiled methods using checked unique boundaries
and executes those bodies in a VM with explicit host-service doubles.
It does not import the full SDK modules,
load extensions from the real home,
request provider credentials,
or call a model.

```sh
# Private method-level fixture harness.
cd -- "${HOME}/temp/agent/pi-input-provenance-probe-2026-09-26"
mise --no-env --no-hooks run build
mise --no-env --no-hooks run probe
```

Base image:
 `bbc51c187ec813fd7c6a49afd22c15efdfe969b8c3a9a9cd49193c8a03908984`.
Probe image:
 `104fe7d55d74ad816462178c6ee0f2e239f5daefd037b50f1c0ebae5df354019`.
The Containerfile uses COPY only.
Runtime bounds are 2 GiB memory including swap allowance,
2 CPUs,
64 PIDs,
256 file descriptors,
60 seconds,
and a 256 MiB Node heap.
No network,
host mounts,
real credentials,
or writable host state is present.
Process `proc_9199` exited 0.

### Working controls

- Explicit input source values are visible to input callbacks.
- Extension submission supplies the extension source tag.
- Normal prompt delivery and queued delivery produce user-role messages.
- Input transforms reach later handlers and the constructed message.
- Input-handler errors reach the error callback.

### Rejected provenance assumptions

- User role identifies a human author.
- `interactive` identifies a human even when supplied by a programmatic/defaulted call.
- The source field is automatically retained in the ordinary message object.
- A transformed body inherits human authorship from its channel label.
- Throwing from the input collector stops the prompt.

The source-method checks pass because they reproduce these counterexamples.
They do not verify a live TUI/RPC session,
persisted JSONL,
a deployed origin collector,
or the auto-mode consumer interface.
Those remain implementation qualification work.

### Command and lifecycle probe

The additional private harness is
`~/temp/agent/pi-trust-lifecycle-probe-2026-09-26`.
Scratch commits `a81fa9b` and `e508b17` freeze the inspected composition and source mapping;
`83e9762` retains the result.
Process `proc_ec6a` completed build and probe with exit 0.
Image:
`d24811aa4ade5e0a2cf7ab1daedc00b6c30d70dc4c079404858a745e562ae7e5`.
Result SHA-256:
`76d454148c75ced5f6f6a0de916990bdc3177c106d3de8b86ec4e1ab7619bc8b`.

The verified commands from that directory were:

```sh
# ~/temp/agent/pi-trust-lifecycle-probe-2026-09-26
mise --no-env --no-hooks run verify:sources
mise --no-env --no-hooks run build
mise --no-env --no-hooks run probe
```

Bounds are 2 GiB RAM,
2 CPUs,
64 PIDs,
256 descriptors,
60 seconds,
and a 256 MiB Node heap,
with no added swap,
network,
credentials,
host mounts,
or writable filesystem.
Registry lookup,
UI notification,
append,
index rebuilding,
ID generation,
and filesystem-facing helpers are controlled doubles.
Actual inspected prompt,
command-dispatch,
branch,
fork,
and project trust functions execute.
No model or real session is used.

The repository's `git-policy-cli` emitted
`final-newline/noncanonical-final-newline`
when freezing copied SDK files.
`verify-sources.mjs` subsequently verified that only final LF bytes differed,
recording original and baked hashes in `source-manifest.json`.
Installed `agent-session.js` retained its original hash recorded in the source identity section;
the baked copy is
`e691e6ef07e44a59c8a7add242bea8e0fc7e78e6cd38bc06f8e94c90a16db8c4`.
Installed `session-manager.js` is
`d365ffb5a189915c3af93953daf751bff45fe46222b05c426f8d8b845946bebf`;
the baked copy is
`290281d71337e18a1fd5fffeee1a4079eba66c8b5ce5bc2e0a6a732678518bc8`.
No method body changed.
Node emitted its `stripTypeScriptTypes` experimental-feature warning;
that built-in transform is used only by this disposable research harness.

The positive controls verify ordinary input interception,
reset on its containing branch,
and clearing again when returning to that branch.
Counterexamples reject these assumptions:
all grant commands pass the input collector,
`hasUI: false` prevents the current command's append,
a reset removes its grant from every branch,
and copied entry identity implies the same session-scoped authority.
The probe does not verify live TUI/RPC interaction,
persistent session copying,
a production collector,
or a finalizer.

### Actual requester, helper, and scripted-editor return path

The private `auto-mode-consumer-contract.mDLkyNoP` repository records this new check under
`contract/human-origin/helper-correlated/`.
The successful controller was invoked with `mise --no-env --no-hooks run run` in that directory.
Do not rerun its create-new epoch;
retain `result.json`,
`run.stdout.txt`,
`run.stderr.txt`,
and `frozen.json`.

The existing requester launched the actual built helper,
which launched a separate synthetic Node editor.
Only terminal launch was injected.
No desktop interaction or genuine human response occurred.
The reused SDK-input image supplied its existing dependencies;
no SDK image was rebuilt and no session or model was created.
Bounds were 2 GiB memory,
2 CPUs,
no extra swap,
64 PIDs,
256 file descriptors,
a read-only nonroot container,
128 MiB disposable temporary storage,
isolated loopback only,
and a 60-second container limit.

The admitted first-party artifacts were:

- `index.mjs`:
   SHA-256 `8e576f31650abb5a9e14ed335fc72b84f0c0af09d35842fc98897358857bec61`.
- `answer-helper.mjs`:
   SHA-256 `1cbd92f41372effbfd2cc6a23370d5dd8257798a0a4850908e228083c4675bd9`.

Source traces are separately hashed;
no source-to-bundle reproducibility claim follows.

Working controls:

- Multiline input preserved its internal CRLF and removed one final CRLF.
- Blank input returned cancellation.
- Both actual helper children exited with code zero and empty stderr.
- Both disposable answer workspaces were removed after settlement.
- The intact authentication guard rejected the wrong candidate and selected the valid candidate's distinct status.

Rejected assumptions:

- Returned answer/status fields do not contain the original question,
  tool-call identity,
  accepted scope,
  or approval witness.
- A separate guard-omitted artifact selected the wrong candidate's `cancelled` status,
  failing the same settled-result assertion.
  Its valid follow-up received `ECONNREFUSED`;
  that error was recorded but was not the proof that the wrong token won.
- Client socket closure is not the channel-settlement assertion.
  A frozen but unexecuted predecessor used that inference and was retained without dispatch.
- The owned preparer's initial statement-shaped guard anchor did not match the deployed ternary expression.
  Its failure and original artifacts were preserved;
  a separate correction supplied the omission artifact.
  This was not an upstream defect.

The outer stderr retained the expected warning:

```text
rejected unauthenticated answer helper connection
```

No bare shutdown error or logger-internal error was accepted.
No human approval was created.
A genuine confirmation capture and production-writer admission remained unqualified by that helper check.

### Private observed-owner scope and capture mechanics

The separate `contract/human-origin/observed-owner/verification-result.json` passed ten predeclared cases
with nine actual helper/scripted-editor launches.
Its injected launcher returned after child spawn,
not after helper exit.
A private requester copy presents a code-authored scope before launch
and captures the original decoded document before normalization and workspace deletion.
The source and production bundle are unchanged;
reversing its observer edits restores the retained deployed bytes.
Callback shape does not authenticate a human.

Untouched and erased forms cancelled;
fixture approval and denial tokens retained only mechanical status.
Changed scope,
replayed forms,
spoofed roles,
and capture/presentation failures published no approval.
Editor cancellation manufactured no raw response.
All workspaces were removed.
Owned-handle,
input/size,
encoded-delimiter,
changed-prefix,
and separate scope-guard omission controls passed.
Every scripted outcome retained `humanOrigin: "not-established"`.
Child stderr was empty;
no session,
fetch,
external model call,
human approval,
or production change occurred.
The freeze is
`8cacba0c2bd55e9b0b87224de35be319f51b3b83a3d5695ee302babd743318a5`.
Do not replay this completed phase.

The real default terminal launcher remains nonblocking.
This observer test qualifies capture mechanics across startup-only ordering,
not desktop detachment,
physical-human origin,
or production writer coverage.
The separately frozen genuine host test uses actual Ghostty and Nano with a disposable scope.
Its launch guard checks the concrete default-spawn inputs;
original response and diagnostics stay under ignored private storage.
No genuine response has yet been claimed.

### Standalone genuine controller exits before response capture

The separately frozen genuine controller `proc_d6b3` reached its default terminal spawn boundary,
then its child exited with code `13`.
Node emitted `Detected unsettled top-level await`.
The outer controller emitted
`AssertionError [ERR_ASSERTION]: Genuine confirmation did not finalize; private diagnostics retained; no replay`.
Its private pre-spawn receipt exists;
original-capture and final-confirmation receipts do not.
Read-only owned-process inspection found a Ghostty process and the remaining private answer workspace.
No raw editor document was read or exported.
Missing capture does not establish that no human typed into the document.

#### Root cause

The existing owner intentionally unrefs its listener and detached terminal child:

```typescript
// package/pi-plugin/ask-user-question/src/answer-channel.ts:107
server.unref();
```

```typescript
// package/terminal-exec/cli/src/launch.ts:122
child.unref();
```

The private standalone bootstrap awaited the confirmation without another referenced handle:

```javascript
// Private contract/human-origin/live-confirmation/live.mjs:84
const observation = await captureConfirmation({ owner, presentation, cwd: privateRoot, editorCommand: inputs.editorCommand, beforePresent, afterCapture });
```

[Node's current ESM documentation](https://nodejs.org/api/esm.html#top-level-await)
specifies exit status `13` for an unresolved top-level await.
The private stderr matched that diagnostic.
The no-desktop contrast controls support a missing standalone bootstrap liveness owner,
not a Node/Pi defect or SDK-session failure.
The prior source-review clearance is retracted in a separate corrective record:
it checked publication ordering but missed process survival.
A launcher returning after spawn may still retain a child or pipe;
that is not equivalent to detached/unreferenced children with ignored stdio.

#### Verification and bounded remedy

`contract/human-origin/liveness-controls/result.json` records `proc_4663` on Node `v26.10.0`.
Unreferenced completion and a separate reference-lease omission exited `13`.
Referenced completion,
caught cancellation,
and caught failure exited naturally with code zero.
No desktop window,
model,
session,
or human response was created.
The original genuine attempt was not replayed.
The caught cases do not exercise a rejection escaping from the leased callback.
Separate `contract/human-origin/liveness-rejection-controls/result.json` records successful `proc_dba6` controls:
escaping asynchronous rejection,
synchronous throw,
returned-value propagation,
and cleanup omission.
The first cases exited naturally after their exact value/error assertions;
the cleanup omission reached its bounded stop,
proving the reference would otherwise remain.
These are inert bootstrap checks,
not a genuine workflow repair claim.

Production update on 2026-10-04:
commit `c443026a6` keeps the ask-user channel listener referenced during detached startup.
A separate disposable regression now reproduces the unreferenced requester exit,
and the real-terminal verifier checks submission and cancellation after the production change.
See [the startup investigation](pi-ask-user-question-startup.md).
This does not replay or qualify any private genuine-confirmation epoch described here.

The historical private bootstrap remedy owns one event-loop reference through the callback:

```javascript
// Private contract/human-origin/liveness-controls/lease.mjs:4
export async function withReferencedLoopLease({ run }) {
  const lease = setInterval(() => {}, 1000);
  try { return await run(); }
  finally { clearInterval(lease); }
}
```

This timer supplies liveness only,
not cancellation,
a deadline,
human authentication,
or a production grant.
Lease integration,
request cancellation,
a declared human-response deadline,
and controller-loss checks remain before a new separately authorized genuine epoch.
The consumed attempt and potentially sensitive workspace remain private evidence.
Do not infer approval from its inactive window or reopen it as a replay.
Detached terminal cleanup remains a separate unestablished result.

#### Rejected approaches and upstream filing

Do not reinterpret exit `13` as a denial,
a human approval,
or evidence of no physical input.
Do not repeat the consumed controller or replace its original response with fabricated provenance.
The source-level clearance is historical evidence,
not present qualification.
No upstream filing is justified:
upstream fault is not established,
the remedy belongs to the owned bootstrap,
and no supported-API defect,
maintainer position,
or upstream patch target is asserted.
A future external filing needs its own contribution,
duplicate,
and demonstrated-fix checks.

### Held control driver waits on a different close boundary

The no-desktop `proc_299d` driver failed with Node `v26.10.0` exit `13` while awaiting child `close`,
not while awaiting a genuine response.
Its first control cases settled;
the explicit IPC-disconnect case retained its expected non-content child marker,
but the driver did not retain that child's exit status.
The failed epoch and its partial artifacts remain unchanged.
No genuine window,
response,
or grant was created.

The inspected Node release source has a separate IPC close-count path:

```javascript
// Node v26.10.0 lib/internal/child_process.js:501 to 504
if (stream.ipc) {
  this._closesNeeded++;
  continue;
}
```

```javascript
// Node v26.10.0 lib/internal/child_process.js:690 to 698, EOF branch
this.buffering = false;
target.disconnect();
channel.onread = nop;
channel.close();
target.channel = null;
maybeClose(target);
```

```javascript
// Node v26.10.0 lib/internal/child_process.js:984 to 988, explicit disconnect finish
if (fired) return;
fired = true;
channel.close();
target.emit('disconnect');
```

The inspected explicit-disconnect finish does not call the EOF close counter.
`maybeClose()` at `lib/internal/child_process.js:1152` emits child `close` only when its counts match.
The private source copy is `live-held-confirmation/node-v26-child-process-source.js`,
retrieved from the Node `v26.10.0` release path.
This trace explains why awaiting process exit and awaiting IPC/stdio closure are different obligations;
it is not a complete Node supported-API conformance or native-build attestation.
Do not merge this control-driver incident with the original unreferenced listener/terminal incident.

The separate `controls-exit/` correction retained the completed prefix without new children,
then used the actual process `exit` event for the unresolved case and unopened suffix.
`proc_4d23` passed those controls,
including natural controller-loss settlement and the expected monotonic-deadline guard-omission failure.
The workaround relies on file-backed diagnostic streams;
it does not qualify arbitrary piped-stream draining or detached-window termination.
Replaying the failed driver or assuming generic abort controls prove requester/helper cleanup does not work.
Actual requester/helper cancellation is a separate incident with retained failure and private guard controls,
not proof supplied by these generic checks.
The read-only clone `~/temp/agent/node-child-close-2026-09-29` is pinned to
`151845ab90d3926ceb36eedf1eade09619c3adc9`.
Its `doc/api/child_process.md:1471` states that child `close` follows process termination and closed stdio,
and promises emission after `exit` or failed-spawn `error`.
That published contract is relevant evidence for a separate Node conformance investigation;
no runtime-core patch or general conformance verdict is claimed here.

#### Upstream filing decision for the close wait

- Upstream fault is not classified by this owned fixture-driver correction.
- No architectural impossibility or fix-difficulty claim is made.
- The exact supported-API contract still needs a separate conformance assessment.
- Contribution-policy assessment is not part of the current fixture qualification.
- Maintainer willingness and duplicate-tracker assessment have not been performed.
- No Node fix prototype is claimed;
  the verified workaround changes the owned driver's settlement boundary.

No upstream report or fileable draft is prepared from this limited incident.
Any external contribution requires a separate source-clone,
contract,
duplicate,
contribution-policy,
and tested-fix assessment.

### Actual helper reports an error after normal originating-request cancellation

The actual no-desktop requester test `proc_c105` stopped its real inert editor after `SIGTERM`,
removed the requester workspace,
and captured or published no response.
The unchanged helper nevertheless exited with code one,
and its private stderr contained `AbortError`.
No genuine window or original response was read.
This is separate from both Node exit-13 incidents.

The first-party editor owner waits for exit then throws its aborted signal:

```typescript
// package/pi-plugin/ask-user-question/src/editor-process.ts:74 to 94, selected statements
using abortSubscription = addAbortListener(signal, function abortEditor(): void {
  child.kill();
});
const exit = await once(child, 'exit');
signal.throwIfAborted();
```

The first-party helper aborts that signal when the originating socket closes,
then logs and rethrows the caught value when that socket is destroyed:

```typescript
// package/pi-plugin/ask-user-question/src/helper-core.ts:65 to 93, selected statements
const controller = new AbortController();
socket.once('close', function abortOnPiDisconnect(): void {
  controller.abort();
});
// Existing runEditor call receives controller.signal.
catch (error: unknown) {
  l.error(`answer helper failed: ${String(error)}`);
  if (socket.destroyed) throw error;
}
```

A separately owned private helper copy adds a narrowly matched expected-stop path:

```javascript
// Private requester-stop-clean/answer-helper-clean.mjs, expanded equivalent guard
if (controller.signal.aborted && socket.destroyed && error === controller.signal.reason) {
  l.info(`originating request closed; editor cancellation completed: ${String(error)}`);
  return;
}
```

The caught value is still logged.
Only the helper-owned cancellation reason after origin closure is consumed;
unmatched errors retain the existing error log and propagation/completion behavior.
No production source or deployed bundle was changed.
`git-policy-cli` added a canonical final LF to the private helper copy,
emitting `final-newline/noncanonical-final-newline`.
Guard removal alone is therefore not exact byte reversal;
removing the guard and reversing that separate final-LF normalization restores the unchanged original.
The executed post-commit hash is retained in `requester-stop-clean/started.json`.
No genuine streams were changed.

`proc_4e89` passed real requester/private-helper interruption,
controller-loss,
and deadline cases.
Their inert Node editors terminated,
workspaces were removed,
helper exits were zero with empty stderr,
and no response was captured or published.
The exact original helper served as the guard-omitted control and reproduced exit one with `AbortError`.
`proc_d680` checked the extracted helper method's unmatched-error,
normal-submission,
and expected-stop branches with inert stubs.
Those source-method checks are not actual helper/desktop parity.
The failure epoch was not replayed.

The workaround changes the private helper's expected-stop branch only.
It does not prove actual Ghostty/Nano termination,
classify physical-human origin,
or qualify a production grant writer.
The frozen unlaunched held epoch retains the original helper and must not dispatch.
A new separately prepared clean-helper epoch is required before genuine interaction.
Inferring helper cleanup from generic lifecycle abort success does not work.
Filtering the original shutdown stderr instead of fixing its owner is rejected.

#### Filing decision for helper shutdown

The affected helper is owned by this repository,
not an external upstream.
No Node/Pi upstream report or contribution is justified by this first-party failure.
Production repair remains outside the authorized documentation/private-qualification scope.
The retained failure,
private guarded-copy test,
unchanged-helper omission,
and source-method limits are the local filing artifact.

### New genuine fixture retains original response and accepted scope

The user-authorized clean epoch `proc_7c5f` completed through the actual default Ghostty/Nano workflow.
Its validated summary reports `approved-fixture`,
private original-document capture,
accepted fixture scope binding,
and temporary answer-workspace removal.
No external model call or production grant was created.
This is a private Q23-A workflow witness,
not proof of physical-person identity or comprehension.
The original failed epoch remains untouched and was not reopened or replayed.

The positive parser path explicitly constructs its accepted scope and digest fields,
so the successful comparison is not two missing values.
The actual writer owns the presentation/capture transaction;
a recorded workflow string or summary boolean alone would not establish that binding.
Protected local reconciliation `proc_5cce` checks the private original UTF-8 bytes,
nonce/presentation,
exact accepted scope,
frozen identities,
deadline,
and finalization.
It emits only fixed non-content metadata.
The successful initial reader `proc_3cd6` did not protect its assertion-error details;
a separate reader/controller was added rather than replaying it.
All worker stdout/stderr in the protected reconciliation stay on ignored private fds.

One-shot exact-argument inspection `proc_5517` found no matching request/answer processes,
and no remaining new answer workspace/file.
This null does not independently attest GUI surface termination or other producer coverage.
No unrelated process was signalled.
Task #68 closes only the private configured-writer/original-response/exact-fixture-scope gate.
Production writers,
other input producers,
lifecycle/finalization,
semantic qualification,
handback,
and replacement parity remain separate.

A proposed policy clarification belongs with the existing data-sharing guidance,
without changing `AGENTS.md` in this task:
private-data processors should capture worker stdout/stderr privately and project fixed non-content fields.
Assertion exceptions can include private expected/actual values;
successful output projection does not qualify failure-path privacy.
The existing media inspection/masking requirements should remain unchanged.

## Proposed containment and unverified remedies

Treat conversational evidence without a verified witness as non-authorizing.
Explicit UI-confirmed grants and exact-action approvals remain separate authority paths.
Do not discard those paths solely because conversational origin is unresolved.

A future origin collector must bind the original admitted content to its actual session entry and branch.
Channel,
content digest,
entry identity,
and human confirmation are different fields with different meanings.
A content hash alone is not a unique message identity.
Prefer metadata references to existing session entries over duplicate transcript capture.
No new ongoing transcript capture is authorized.

Context edits can replace projected content while preserving entry role and metadata,
as described in the inspected session-format documentation and tests.
Authorization must not transfer to rewritten text merely because its original entry has a trusted witness.
The original witnessed content and later projected context need separate treatment.
This path has not yet received a local runtime probe.

Every grant writer needs its own admitted authority witness;
merely adding an input callback cannot cover registered commands.
Copied entries need original-session identity and explicit lifetime/lineage treatment.
Revocation scope and fork inheritance are design choices to confirm,
not facts supplied by a model or implied by copied text.
Qualified legacy prose matching remains eligible once its human authority is established.

No collector workaround is declared verified.
The proposed design can ask when its required authority evidence is absent,
while preserving the accepted semantic-effect policy from Q13 B.
Origin qualification is not a replacement code-proof requirement for script effects.

## What does not work

- Asking the model whether text was written by a human.
- Promoting user-role messages,
  summaries,
  tool output,
  project context,
  or automated continuations into grants.
- Assuming the defaulted source tag is an authenticated identity.
- Binding authority only to transformed text or a projected role.
- Declaring a production provenance solution from this isolated method test.

## Upstream filing artifact

No upstream issue or comment is drafted or filed.
Input transformations and provider-facing message shapes are documented host behavior,
not an established upstream defect.
No patch is proposed.

### Upstream filing decision

1.  Upstream fault:
     not established.
    The authorization consumer needs a stronger witness than the documented message shape provides.
2.  Fixability:
     no impossibility claim.
    Application-owned metadata and explicit approval interfaces remain possible integration paths.
3.  Supported use:
     extension input events and custom session state are documented;
    a complete human-authorization collector is not supplied by these tests.
4.  Contribution policy:
     not evaluated because no upstream contribution is proposed.
5.  Maintainer willingness:
     not evaluated;
    no request was sent.
6.  Fix prototype:
     none.
    The method-level counterexample harness is not a provenance fix.

Any future filing requires the exclusion,
contribution,
duplicate-search,
and tested-fix checks before a draft.
