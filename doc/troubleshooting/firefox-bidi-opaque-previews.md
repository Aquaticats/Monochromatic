# Firefox ESR 140.15.0: verify opaque previews without changing their sandbox

## Trigger and environment

The Promise lesson requires Firefox ESR 140 compatibility,
including editable JavaScript executed inside opaque iframes.
The verification must reach each current preview without adding `allow-same-origin` or weakening its CSP.

`command -v firefox` returned status 1 in the shell used for this check.
That was a PATH lookup result,
not evidence that Firefox was absent.
`~/.local/share/applications/firefox-esr.desktop:7` identified the installed executable:

```text
Exec=/home/user/.local/opt/firefox-esr/firefox %u
```

Invoking that executable with `--version` reported `Mozilla Firefox 140.15.0esr`.
Its `--help` listed `--headless`,
`--new-instance`,
`--profile`,
and `--remote-debugging-port`.

## Verified control route

Mozilla documents a direct WebDriver BiDi connection using `--remote-debugging-port 0`.
Firefox reports the selected loopback address;
the WebSocket endpoint adds `/session`.
A `session.new` request with `capabilities: {}` creates the session.
See [the connection guide][connection] and [session creation][session].

The test uses a fresh profile and an explicit new instance.
It enables neither remote hosts/origins nor system access.
[Mozilla's security guidance][security] describes those boundaries.
The remote interface is privileged test instrumentation,
not a new capability granted to page scripts.

### Source trace

Read-only clone:
`/var/home/user/temp/agent/promises-revision/firefox-esr140-source`.
The `esr140` checkout identifies commit `82076abb4c58a40cbd5e772b2c41887444aa9510`.
It was retrieved with `gh repo clone`,
`--depth 1`,
`--filter=tree:0`,
and `--no-checkout`.
`git show` extracted the relevant file without modifying the third-party working tree.

In [that source revision][source],
`remote/webdriver-bidi/modules/root/script.sys.mjs:522` defines `evaluate`:

```js
// remote/webdriver-bidi/modules/root/script.sys.mjs:522
async evaluate(options = {}) {
  const {
    awaitPromise,
    expression: source,
    resultOwnership = lazy.OwnershipModel.None,
    serializationOptions,
    target = {},
    userActivation = false,
  } = options;
```

From line 549,
it resolves the requested context or realm and forwards the expression:

```js
// remote/webdriver-bidi/modules/root/script.sys.mjs:549
const { contextId, realmId, sandbox } = this.#assertTarget(target);
const context = await this.#getContextFromTarget({ contextId, realmId });
const serializationOptionsWithDefaults =
  lazy.setDefaultAndAssertSerializationOptions(serializationOptions);
const evaluationResult = await this._forwardToWindowGlobal(
  "evaluateExpression",
  context.id,
  {
    awaitPromise,
    expression: source,
    realmId,
    resultOwnership,
    sandbox,
    serializationOptions: serializationOptionsWithDefaults,
    userActivation,
  }
);
```

The consumer locates the lesson realm and each exact current preview title through `script.getRealms`.
It evaluates with `target: { realm }` and `awaitPromise: true`.
JSON serialization inside the evaluated expression avoids treating protocol remote values
as ordinary JavaScript objects.
The iframe's browser security properties are unchanged.

## Reproduction and verification

Scratch authoring directory:
`/var/home/user/temp/agent/promises-revision`.
Run its `ui:firefox-review` mise task as a managed background process.
Wait for `FIREFOX_REVIEW_READY`,
then run its `test:firefox-review` task.
The endpoint is recorded in `firefox-endpoint.txt`;
never reuse it after stopping that browser.

The implementation is in `launch-firefox.mjs`,
`firefox-bidi.mjs`,
and `verify-firefox-review.mjs`.
The client gives protocol requests a 20-second deadline and rejects pending requests on socket failure.
Rendered-state checks have their own 10-second bound.
The verifier ends its BiDi session;
the browser process and owned profile are cleaned up separately.

Working catalog from `proc_a4b4`,
which passed in 26 seconds:

- Pre-aborted `wait` returns a rejected Promise with the same reason.
- Ordered Send/Stop cancellation and the separate real deadline path.
- Ownership's worked comparison preserves controllers after partial completion and Stop followed by a fresh send.
- Capstone success,
  permanent failure,
  retries,
  and timeout exhaustion.
- Capstone and reference backoff examples cancel A while B continues.
- The sandbox remains exactly `allow-scripts allow-downloads`.

Negative catalog:

- PATH-only lookup does not locate this installation;
  the desktop entry supplies its actual executable path.
- The intentionally broken ownership starter reports zero pending sends while one held reply still exists.
  The worked comparison reports one,
  establishing that the same probe detects the ownership difference.

This is DOM/event and semantic coverage,
not Firefox keyboard,
native-print-dialog,
or full visual coverage.
Pointer-input and native-print verification were performed separately in Chromium/Helium.

## Wait for content fit, not only the first height report

After the workshop gained automatic height allocation,
Node's assertion in the verifier's `load()` function failed in `proc_7b59` with
`{"content":1288,"viewport":1007}`.
The first accepted height report had arrived,
but it was not evidence that all subsequent layout changes had finished.

`proc_241b` read the same unchanged frame later and measured `{"content":1288,"viewport":1288}`.
No Firefox or lesson implementation change was made for this discrepancy.
The verifier now keeps its bounded wait until the actual child viewport fits the measured content,
then asserts the geometry.
`proc_f316` passed the complete Firefox check twice with that condition.
The preserved fixed-height preview is a separate negative control and still fails its content-fit assertion.

## Boundaries and rejected approaches

Do not make the iframe same-origin just to read it from its parent.
That would test a different isolation boundary.
Do not select a preview only by its URL:
each run uses an exact title and new realm.
Do not connect to the user's everyday Firefox profile.
No global browser preferences,
installed Firefox files,
or third-party source files were changed.

## Upstream filing decision

Nothing to add upstream:
this investigation found a supported control route,
not a Firefox defect.

1.  Upstream fault:
    no. The installation lookup and consumer verification route are local concerns.
2.  Fixability:
    no upstream fix is requested.
3.  Supported use:
    the connection/session documentation and quoted source describe this route.
4.  Contribution welcome:
    not investigated because no contribution is proposed.
5.  Likelihood of a fix:
    not applicable without a proposed upstream defect.
6.  Prototype:
    the consumer-side harness exercises the documented route;
    no upstream patch is needed.

No upstream issue or comment draft was produced,
so no duplicate-report search or filing was initiated.

[connection]: https://developer.mozilla.org/en-US/docs/Web/WebDriver/How_to/Create_BiDi_connection
[session]: https://developer.mozilla.org/en-US/docs/Web/WebDriver/Reference/BiDi/Modules/session/new
[security]: https://firefox-source-docs.mozilla.org/remote/Security.html
[source]: https://github.com/mozilla-firefox/firefox/tree/82076abb4c58a40cbd5e772b2c41887444aa9510
