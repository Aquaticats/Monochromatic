# agent-browser 0.36.0 frame selection does not redirect native eval into the practice frame

## Symptom

During verification of `doc/planning/promises-teaching.local.html`,
its sandboxed `#work-preview` iframe rendered correctly in the accessibility snapshot.
Its greeting ran when clicked.
The verification harness nevertheless failed at these separate calls:

- `frame '#work-preview'` returned `Frame not found`,
  including on a repeated probe.
- Selecting the snapshot's iframe reference succeeded.
  A subsequent `eval 'document.querySelector("#result").textContent'` returned:
  `Evaluation error: TypeError: Cannot read properties of null (reading 'textContent')`.

The lesson's main document has no `#result` element.
The practice frame does.
The harness incorrectly assumed frame selection changed the target of native `eval`.
It was not evidence that the learner's code or the iframe failed to render.

## Source trace

Installed binary:
`agent-browser 0.36.0`.
Inspected release source:
`v0.36.0`,
commit `eb05921bad874cd2a1b4fa5d1149f1ed26576cae`.
The existing read-only clone is `~/temp/agent/agent-browser-quality-explorer-2026-09-06`.
Paths and line numbers in this section are relative to that clone.

`cli/src/native/actions.rs:5027` defines `handle_evaluate`.
Its Chromium path calls the browser manager directly at line 5045:

```rust
// cli/src/native/actions.rs:5045
let result = mgr.evaluate(script, None).await?;
```

It does not pass `state.active_frame_id`.
`cli/src/native/browser.rs:1206` selects the active page session:

```rust
// cli/src/native/browser.rs:1206
pub async fn evaluate(&self, script: &str, _args: Option<Value>) -> Result<Value, String> {
    let session_id = self.active_session_id()?.to_string();
```

The method sends `Runtime.evaluate` to that session,
without a frame-specific execution-context identifier.

Frame-reference selection is a different path.
`cli/src/native/actions.rs:8813` records the resolved frame ID:

```rust
// cli/src/native/actions.rs:8813
state.active_frame_id = Some(frame_id.to_string());
```

Element operations can use that selection.
In the observed reproduction,
`get text '#result'` returned `Ready`,
then `click '#send'` ran the child greeting.
Native `eval` did not use the same scope.

The CSS-frame failure is separately bounded evidence.
The CSS path at `cli/src/native/actions.rs:8819` reads the element's name,
ID,
or source,
then searches the frame tree by that returned string:

```rust
// cli/src/native/actions.rs:8823
return el.name || el.id || el.src || null;
```

```rust
// cli/src/native/actions.rs:8831
if let Some(frame_id) = find_frame(frame_tree, Some(frame_name), None) {
```

The precise frame-tree mismatch was not captured.
Do not claim that all CSS iframe selection fails,
or attribute the native-eval scope to that separate lookup failure.

## Verification

The local lesson is intentionally ignored by Git at the user's request.
The observed invocation sequence used its live artifact:

```bash
# Run from /var/home/user/Monochromatic.
agent-browser --session promises-lesson-verify --allow-file-access open \
  file:///var/home/user/Monochromatic/doc/planning/promises-teaching.local.html
agent-browser --session promises-lesson-verify --allow-file-access snapshot --interactive
agent-browser --session promises-lesson-verify --allow-file-access frame '#work-preview'
```

Select the iframe reference from the current snapshot,
not a copied reference number from this document.
The observed reference was `@e48`:

```bash
# References are snapshot-specific; this records the actual reproduction.
agent-browser --session promises-lesson-verify --allow-file-access frame @e48
agent-browser --session promises-lesson-verify --allow-file-access get text '#result'
agent-browser --session promises-lesson-verify --allow-file-access click '#send'
agent-browser --session promises-lesson-verify --allow-file-access eval \
  'document.querySelector("#result").textContent'
```

Working catalog:

- The snapshot included the child heading,
  name input,
  Send,
  and Stop.
- Frame-reference selection succeeded.
- Child element selection returned `Ready`;
  clicking Send ran the greeting.
- Direct CDP evaluation in the iframe target returned
  `{"title":"Promise practice","result":"Hello Ada"}`.

Failing catalog:

- CSS frame selection returned `Frame not found` twice for this input.
- Native `eval` after reference selection still evaluated against the page session.
- The first verification run stopped before workshop assertions because of the frame lookup.
  That was a harness failure,
  not a failed teaching interaction.

## Verified workaround

Keep ordinary lesson interactions in agent-browser.
For evaluation inside the opaque practice frame,
use the browser's CDP endpoint and explicitly attach to its iframe target:

1.  Obtain `cdpUrl` with `agent-browser get cdp-url` using the same session and launch flags.
2.  Call `Target.getTargets` and select the owned `iframe` target whose URL is `about:srcdoc`.
3.  Call `Target.attachToTarget` with `flatten: true`.
4.  Send `Runtime.evaluate` through the returned `sessionId`,
    with `returnByValue: true` and `awaitPromise: true`.
5.  Detach that target and close the helper socket.

The tested helper is `~/temp/agent/promises-frame-eval.ts`.
The caller is `~/temp/agent/verify-promises-lesson.ts`.
Their browser belongs only to this verification run.
The helper expects one owned `about:srcdoc` iframe;
it is not suitable as-is for a shared browser with several matching frames.
The tradeoff is an additional protocol bridge and explicit target ownership.
The lesson's sandbox and network restrictions remain intact.

## What does not work

- Treating the successful main-page load as proof of a successful child evaluation.
- Assuming native `eval` follows the frame selected for element operations.
- Removing the iframe sandbox to accommodate the harness.
  This was not attempted because it would change the artifact's isolation boundary.
- Reusing old iframe references after replacing the preview.

## Upstream filing artifact

No upstream issue,
comment,
or patch was prepared or sent.
This records a verified consumer-side workaround,
not a proposal to change upstream evaluation semantics.

### Upstream filing decision

1.  Upstream fault:
    not established as a contract violation;
    the harness assumed a shared scope that the inspected evaluator does not implement.
2.  Fixability:
    no upstream semantic change was designed.
3.  Supported combination:
    individual frame and eval commands exist;
    a promise that their scopes compose was not established.
4.  Contribution policy:
    not evaluated because no contribution is proposed.
5.  Upstream direction:
    not evaluated;
    no tracker claim or filing is made.
6.  Prototype:
    the consumer bridge was exercised;
    no upstream patch was attempted because the fault and supported-contract gates are unresolved.

Upstream triage remains outside this lesson-delivery work.
Re-evaluate these gates and search for duplicates before proposing a filing.
