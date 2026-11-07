# agent-browser 0.36.0: a pre-reload offline check does not verify the reloaded document

## Symptom

The revised export harness set offline mode and observed `navigator.onLine === false`.
After navigation or reload,
its next check observed `true`.
Processes `proc_b5e4` and `proc_7dfc` failed Node assertions rather than reporting an application exception:

```text
AssertionError [ERR_ASSERTION]: Offline state must be measured after this file reload
true !== false
```

The earlier `set offline on` followed by an unchecked reload did not prove the intended condition held afterward.
That probe is not retained as proof of offline loading.
This does not establish that requests were actually allowed after reload;
the browser's reported online value and request blocking are distinct evidence.

## Source and diagnosis boundary

Inspected release:
`v0.36.0`,
commit `eb05921bad874cd2a1b4fa5d1149f1ed26576cae`.
Clone:
`~/temp/agent/agent-browser-quality-explorer-2026-09-06`.

`cli/src/native/network.rs:28` implements `set_offline` by sending:

```rust
// cli/src/native/network.rs:35
"Network.emulateNetworkConditions",
Some(json!({
    "offline": offline,
    "latency": 0,
    "downloadThroughput": -1,
    "uploadThroughput": -1,
})),
```

This forwards the supplied `offline` boolean and the shown numeric parameters.
`cli/src/native/actions.rs:5826` implements the native reload path;
it sends this separate operation through the active page session:

```rust
// cli/src/native/actions.rs
.send_command_no_params("Page.reload", Some(&session_id))
```

Those excerpts identify the invoked operations.
They do not explain why the measured online value changed.
No universal persistence rule or upstream emulation bug is claimed.

## Verification

Measured browser:
headless Chromium `149.0.7827.54` with agent-browser `0.36.0`.
The failed sequence opened a local reference export,
called `set offline on`,
reloaded it,
and inspected `navigator.onLine`.
The same after-state assumption also failed when moving from an export back to the lesson.

Working catalog:

- A loopback canary page loads directly and displays `Network canary available`.
- A fresh browser launched through the rejecting proxy instead displays `Offline fixture rejected HTTP`.
- The canary request count does not increase on that proxied navigation;
  the proxy's rejected-request count does increase.
- The actual reference,
  learner file,
  foundation downloads,
  and edited boundary-string download
  load and run in that same proxy-restricted browser.
  Process `proc_f17b` passed this complete export verification.

Failing catalog:

- Inspecting offline state only before a later reload or navigation.
- Treating source self-containment or a successful `set` response as the only runtime offline proof.

## Verified workaround

Use a fresh browser profile with HTTP proxy configuration present from launch,
not a media or network assumption made before navigation.
The local fixture is deliberately non-forwarding:
HTTP receives status `503`,
and CONNECT receives a `503` response followed by connection closure.

Runnable local sources:

- `~/temp/agent/promises-revision/offline-fixture.mjs`,
  launched by `mise run ui:offline-fixture`.
- `~/temp/agent/promises-revision/verify-exports.mjs`,
  run by `mise run test:exports`.

The driver reads the fixture's generated loopback endpoints,
uses `--proxy <endpoint> --proxy-bypass '<-loopback>'` on every command in that browser,
and proves the canary difference before opening the lesson.
The fixture binds only to `127.0.0.1` and never forwards to another server.
The verification profile is newly created rather than reusing cached user resources.

Tradeoffs:
this verifies local-file operation in a browser whose HTTP denial was positively tested.
CONNECT rejection is implemented in the proxy,
but no separate HTTPS-canary result was recorded.
It is not an operating-system network-disconnection test,
not a test of every possible network transport,
and not a fix for `navigator.onLine` or CDP emulation persistence.
The lesson's original CSP and opaque-frame sandbox remain in place.

## What does not work

- Weakening the lesson's CSP or granting its previews same-origin access to make testing easier.
- Interpreting `navigator.onLine` alone as a complete request-blocking test.
- Reusing the failed after-navigation assertion as positive evidence in later reports.

## Upstream filing decision

1.  Fault:
    not established;
    the observed value changed,
    but request behavior across reload was not isolated.
2.  Fixability:
    no upstream defect or required patch has been identified.
3.  Supported use:
    the CLI exposes offline and reload operations;
    their observed composition needs further diagnosis.
4.  Contribution policy:
    not evaluated for a filing because no upstream defect is claimed.
5.  Direction:
    not evaluated;
    no maintainer-intent claim is made.
6.  Prototype:
    the verified change is the consumer's proxy-backed harness,
    not an upstream patch.

The skill-relative `.out-of-scope/` directory was absent.
No issue,
PR,
or comment is proposed.
There is nothing to add upstream from this bounded observation without isolating the missing request evidence.
