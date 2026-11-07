# agent-browser 0.36.0: a Print click is not a completed print cycle

## Symptom

A Promise-lesson verification script passed PDF generation and content checks,
then failed a disclosure-restoration assertion after clicking the page's Print button.
The assertion reported every disclosure as open when its saved state was closed.

The combined counter contained `before: 2` and `after: 1`.
That was initially misdiagnosed as a PDF restoration bug.
The events belonged to separate actions:
PDF generation completed one cycle,
then the Print click began another.

A synchronous automation call which clicks Print can also remain pending while the native dialog is open.
A command timeout does not establish that the click was never delivered.

## Root cause of the false diagnostic

The harness asserted post-print state before the Print-button cycle had ended.
It did not wait for cancellation or completion of that cycle.

The failing local reproduction is `~/temp/agent/promises-revision/probe-print.mjs`:
its `button` variant clicks `#print-lesson`,
reads the current disclosure state,
and immediately compares that state with the pre-print state.
The PDF variant uses the PDF command instead and passes.

The tools invoke different operations.
In agent-browser commit `eb05921bad874cd2a1b4fa5d1149f1ed26576cae`,
`cli/src/native/actions.rs:7145` implements `handle_pdf`.
Its call at `cli/src/native/actions.rs:7156` is:

```rust
// Upstream cli/src/native/actions.rs
.send_command("Page.printToPDF", Some(params), Some(&session_id))
```

The lesson's Print button instead invokes `window.print()`.
[MDN documents `Window.print()`][window-print] as opening the print dialog and blocking while it is open.
Those are different boundaries to verify.

The original restoration code was not proven defective.
An attempted idempotence patch did not change the failing test result
and was removed after identifying the incorrect assertion boundary.
Do not restore that patch as a remedy for this incident.

## Verification

Measured tools:

- `agent-browser 0.36.0`.
- Headless Chromium `149.0.7827.54`,
  revision `42ce21f112d0b2ba486bbb5cadb225f78b4ddb97`.
- Native Helium reports Chromium `152.0.7977.82`,
  revision `d04cdb24d67b081f6cf80200ffc5233f44b61109`.

The browser versions and modes differ.
These probes do not establish that headless mode alone causes the observed event difference.

Working catalog:

- `mise run probe:print`,
  from `~/temp/agent/promises-revision/`,
  generates a PDF and verifies restored disclosures.
  Process `proc_207a` passed with one before/after pair.
- `mise run test:presentation` checks every inventoried source,
  workshop task,
  hint,
  worked comparison,
  current draft,
  preview text,
  setting,
  and note
  against extracted text from an actual PDF.
- `mise run test:native-print` uses the page's Print button in an isolated native Helium profile,
  finds its real `chrome://print/` page,
  and activates that page's Cancel control.
  It waits for `afterprint` and checks restoration of a mixture of open and closed disclosures.
  Processes `proc_ee8b` and `proc_cc0f` passed.

Failing catalog:

- `mise run probe:print-button` deliberately retains the invalid immediate-restoration assertion.
  Process `proc_4a24` observed `beforeprint` without a completed cycle and failed that assertion.
- Waiting synchronously for the initiating native Print click before attempting to dismiss the dialog
  leaves the dismissal action unable to begin.
  The click was delivered despite the command timeout;
  the separate CDP target list showed `chrome://print/`.

## Verified workaround

Separate PDF-content verification from native-dialog lifecycle verification.
For the native path,
start the click without blocking the verification driver,
inspect the real print-preview target through a separate CDP connection,
wait until both Cancel and Print/Save are enabled,
and activate Cancel.
Only then await the initiating click and assert post-print state.

The runnable implementation is
`~/temp/agent/promises-revision/verify-native-print.mjs`.
The native browser is launched by `mise run ui:print` in an off-screen Cage compositor
with a disposable profile.
No real printer job is submitted.

Tradeoffs:
this native-dialog driver is Chromium/Helium-specific and depends on the observed WebUI controls.
It verifies cancellation,
not a physical printer.
PDF text verification is separate from visual inspection of rendered PDF pages.

### Target registration is not control readiness

During the shop-opening revision,
`proc_3299` passed PDF-content checks but failed in
`~/temp/agent/promises-revision/verify-native-print.mjs`:

```text
AssertionError [ERR_ASSERTION]:
assert.ok(controls.some(button => button.text === 'Cancel' && !button.disabled))
```

The driver's `printTarget()` only waited for a `chrome://print/` target to exist.
It then immediately asserted that Cancel was available,
although it already waited for Print/Save to become enabled.
The failing assertion does not distinguish an absent control from a disabled control.

A later inspection of that same owned print target reported `document.readyState` as `complete`,
with both Cancel and Print enabled.
The driver now waits for both conditions within its existing bounded readiness loop.
The corrected full suite passed as `proc_e6a1`,
including real Cancel and disclosure restoration.
No lesson restoration code or browser implementation was changed for this assertion.

One cleanup invocation clicked Cancel synchronously through `cdp-page.mjs`.
The evaluation returned `Cancelled owned failed verification dialog`,
then the helper's explicit `Target.detachFromTarget` request failed:

```text
{"code":-32602,"message":"No session with given id"}
```

A subsequent target-list inspection found zero print targets and one lesson target:
cancellation had happened despite the cleanup command's nonzero exit.
Do not repeat Cancel against that ended target.
The native verifier retains its separate scheduled Cancel action,
then waits for the completed print cycle rather than treating an evaluation response as completion.
This is a caller/cleanup boundary observation,
not a new agent-browser defect.

### Ended-browser cleanup

After the owned Cage/browser process was stopped,
closing the agent-browser session while still supplying its ended `--cdp` endpoint failed in `proc_315e`:

```text
CDP WebSocket connect failed: IO error: Connection refused (os error 111)
```

The connection-error wording comes from `cli/src/native/cdp/client.rs:121`
in the inspected agent-browser source:

```rust
// cli/src/native/cdp/client.rs:121
let (ws_stream, _) =
    tokio_tungstenite::connect_async_with_config(request, Some(ws_config), false)
        .await
        .map_err(|e| format!("CDP WebSocket connect failed: {}", e))?;
```

Closing that disposable controller without the ended endpoint passed as `proc_5e54`:

```bash
# End only the owned print-verification session after its external browser has ended.
agent-browser --session promises-print-ui --allow-file-access close
```

A subsequent session list contained only `promises-revised-present`.
The ended print and export profiles were then removed;
the user-facing browser was retained.
This cleanup sequence is not advice to omit launch/CDP options during ongoing page verification.
It is not evidence of a print failure or an upstream defect.

## What does not work

- Combining event counts across PDF generation and a subsequent Print click.
- Treating a submitted Print click as a completed print cycle.
- Treating command timeout as proof that Print was never invoked.
- Changing lesson restoration logic to satisfy an assertion made before cancellation.
- Removing browser interactivity to avoid testing the print boundary.

## Upstream filing decision

No upstream issue or contribution is proposed.

1.  Upstream fault:
    not established;
    the demonstrated error was in the verification assertion.
2.  Upstream fixability:
    no upstream fix is identified or needed for that assertion.
3.  Supported use:
    the CLI exposes PDF generation;
    `window.print()` separately opens native UI.
4.  Contribution policy:
    not assessed because no upstream contribution is proposed.
5.  Maintainer intent:
    not assessed;
    no claim about willingness to change print behavior.
6.  Upstream prototype:
    inapplicable;
    the working change is in our verification driver.

The skill-relative `.out-of-scope/` directory was absent.
No duplicate search or filing was performed because no upstream defect or additive report is claimed.
There is nothing to file from this investigation.

[window-print]: https://developer.mozilla.org/en-US/docs/Web/API/Window/print
