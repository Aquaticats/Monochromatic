# agent-browser 0.36.0: verify forced-color media in the applying CDP session

## Symptom

A lesson check applied `Emulation.setEmulatedMedia` through a separate CDP connection,
closed that connection,
then used `agent-browser eval` to inspect `matchMedia("(forced-colors: active)").matches`.
The assertion failed with `false !== true` in process `proc_381e`.

Both operations targeted the lesson's main document.
This was not the previously documented iframe-selection incident.

## Source and observation boundary

The cause of the cross-connection result is not established.
Do not attribute it to a wrong iframe or claim a universal emulation-persistence rule.

Agent-browser commit `eb05921bad874cd2a1b4fa5d1149f1ed26576cae`,
`cli/src/native/browser.rs:1835`,
forwards media overrides through its active CDP session:

```rust
// Upstream cli/src/native/browser.rs
let session_id = self.active_session_id()?;
```

The call at `cli/src/native/browser.rs:1854` uses:

```rust
// Upstream cli/src/native/browser.rs
.send_command(
    "Emulation.setEmulatedMedia",
    Some(params),
    Some(session_id),
)
```

The running Chromium `149.0.7827.54` protocol descriptor at `/json/protocol`
also describes this operation as emulating media types or features for CSS media queries.
Its feature records contain `name` and `value` strings.
That descriptor does not establish what another client will observe after detachment.

## Verification

The local helper is `~/temp/agent/promises-revision/cdp-page.mjs`.
It attaches to exactly one page URL,
applies the command,
and can evaluate a predicate through the same attached session before detaching.

Working catalog:

- Set the `forced-colors` feature to `active` and inspect it in that session.
- In the same inspection,
  verify that colored mirrors are hidden
  and the native textareas have a non-transparent foreground.
- `mise run test:presentation`,
  from `~/temp/agent/promises-revision/`,
  passed this positive control in processes `proc_ee8b` and `proc_cc0f`.

Failing catalog:

- The original separate-client inspection after the applying helper had closed
  returned false and did not test the intended forced-color rendering state.

## Verified workaround

Pass the media predicate and computed-style checks to the helper's optional final expression argument.
Assert its returned `inspection` value instead of assuming the next CLI evaluation shares the override.

Tradeoff:
this proves rendering under the configured media state in the applying session.
It does not prove persistence across client detachment or subsequent automation commands.

## What does not work

- Treating a successful protocol response alone as proof that a later screenshot used that media state.
- Reclassifying the failure as an iframe bug without checking the target URL and session.
- Removing the native-editor fallback rather than exercising the condition which selects it.

## Upstream filing decision

Nothing is proposed for upstream filing.

1.  Upstream fault:
    unproven;
    the original assertion did not inspect the applying session.
2.  Fixability:
    no upstream change is identified.
3.  Supported use:
    the running protocol exposes media emulation;
    cross-client persistence is not established.
4.  Contribution policy:
    not assessed because no upstream contribution is proposed.
5.  Maintainer intent:
    not assessed;
    no willingness or impossibility claim is made.
6.  Prototype:
    the verified change is confined to our inspection helper and test.

The skill-relative `.out-of-scope/` directory was absent.
No duplicate search or filing was performed because there is no established upstream defect or additive report.
