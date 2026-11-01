# Sinon 22.1.0 formats called symbol spies with a TypeError

## Symptom

The namespace sync-removal test reached a false assertion about a spy on `FileHandle[Symbol.asyncDispose]`.
Instead of the expected assertion error,
Sinon diagnostic formatting threw:

```text
TypeError: Cannot convert a Symbol value to a string
```

The actual failure was retained in
`namespace-guard-proof-20260912/r3/native-sync-promise-awaited-test.out`.
The stack passes through Sinon's call formatter,
`sinon-chai` and the repository test matcher.
That exception was not counted as a successful guard-removal detection.

A standalone probe also reproduces the error through `sinon.assert.callCount`,
without the repository harness or `sinon-chai`.
The measured distinction is whether the symbol-named spy already has recorded calls.

## Source trace

The installed distribution is Sinon `22.1.0`.
Source was read from tag `v22.1.0`,
commit `ab289e92cdd76caf8cec2b0a8c9a391283e6c9df`,
in `/var/home/user/temp/agent/sinon-symbol-diagnostic-20260912`.
The npm metadata endpoint also returned `22.1.0` during this investigation.
GitHub's latest-release endpoint returned an old `v4.2.2` release and was not used as current npm-version evidence.

`src/sinon/util/core/wrap-method.js:240` preserves the property key as the display name:

```js
// src/sinon/util/core/wrap-method.js
extend.nonEnum(target, {
    displayName: property,
    wrappedMethod: wrappedMethods[i],
```

`src/sinon/proxy.js:23` delegates string conversion to `functionToString`:

```js
// src/sinon/proxy.js
const proxyApi = {
    toString: functionToString,
```

`src/sinon/util/core/function-to-string.js:27` can return that symbol unchanged:

```js
// src/sinon/util/core/function-to-string.js
return this.displayName || "sinon fake";
```

Recorded-call formatting reaches the proxy conversion in `src/sinon/proxy-call.js:205`:

```js
// src/sinon/proxy-call.js
let callStr = this.proxy ? `${String(this.proxy)}(` : "";
```

The `%C` formatter invokes call stringification for each recorded call,
`src/sinon/spy-formatters.js:129`:

```js
// src/sinon/spy-formatters.js
for (let i = 0, l = spyInstance.callCount; i < l; ++i) {
    let stringifiedCall = `    ${spyInstance.getCall(i).toString()}`;
```

The historical fix from [issue 1640][issue] and [pull request 1642][fix] is already present
at `src/sinon/proxy.js:131`:

```js
// src/sinon/proxy.js
return String(formatter(spyInstance, args));
```

That outer conversion cannot run successfully when the formatter first throws inside `String(this.proxy)`.
The historical uncalled-spy case works in the measured distribution;
reapplying the historical patch does not address this called-spy path.

## Verification

The standalone probe runs against the installed ESM distribution:

```bash
# Run from the translation-repair worktree
node /var/home/user/temp/agent/probe-sinon-symbol-diagnostic-20260912.mts
```

`sinon-symbol-diagnostic-probe-20260912.json` records Node `26.8.2` and these catalogs:

- String-named spy:
  a wrong call count throws the ordinary Sinon `AssertError`.
- Uncalled symbol-named spies:
  `sinon.assert.called` throws the ordinary `AssertError`.
- Correct counts:
  assertions return successfully for string,
  named symbol,
  anonymous symbol,
  registered symbol and `Symbol.asyncDispose` keys.
- Called symbol-named spies:
  both `%C` formatting and a wrong call count throw the quoted `TypeError`.
- Primitive count comparison:
  Node's numeric assertion reports its ordinary `AssertionError` for each wrong count.

The actual package control uses `expect(disposal.callCount).toBe(0)`.
The full R11 source suite passes,
and the final namespace removal proof detects a missing sync await through
`AssertionError: expected 1 to equal +0`,
not through diagnostic formatting.

## Verified consumer workaround

Compare the primitive count rather than asking the Sinon matcher to render a symbol-named spy:

```ts
// package/module/translation-repair/src/preparation-attempt-order.unit.test.ts (removed before 2026-10-18)
expect(disposal.callCount).toBe(0);
```

This preserves the false-condition assertion and avoids the formatting path.
It sacrifices the matcher's rendered call history;
it does not change the spy,
its recorded calls or production file operations.
No installed dependency was patched.

## Disposable source prototype

A fresh clone at `/var/home/user/temp/agent/sinon-prototype.licfEjmZ`
was verified against the same origin and commit before this change:

```diff
--- a/src/sinon/util/core/function-to-string.js
+++ b/src/sinon/util/core/function-to-string.js
@@ -27,1 +27,1 @@
-    return this.displayName || "sinon fake";
+    return String(this.displayName || "sinon fake");
```

The exact original and patched source functions were installed on fresh proxies from the published distribution.
This exercises the deciding function through the real call formatter and public assertion API.
It is not a rebuilt upstream distribution or a completed upstream browser/contract suite.

The no-network prototype container used 2 GiB RAM,
2 CPUs and 512 PIDs.
Only the Node installation,
original/patched functions,
published bundle and owned probe/output paths were mounted.
No corpus or credential directories were mounted.

`sinon-symbol-prototype-output-f5Gmzg/report.json` records 90 cases:

- Published,
  original-source-hook and patched-source-hook variants.
- String,
  named/anonymous/registered symbol and `Symbol.asyncDispose` properties.
- Zero,
  one and two recorded calls.
- Returning and throwing target functions.

Each patched false assertion produces `AssertError`;
correct-count assertions return,
and call text renders.
The published and original-source variants retain the measured symbol-formatting failures.
A separately named display-name override also passes.
The recorded memory peak is 25640960 bytes;
no OOM or PID-limit event occurred.
The manifest binds all mounted source bytes.

## What does not work

- A passing call-count assertion never exercises its failure-message formatter.
- A zero-call symbol test does not exercise recorded-call stringification.
  That is the distinction from the historical regression test.
- Counting any nonzero test exit as guard detection would have accepted the formatting exception.
- The historical outer `String(formatter(...))` fix is already installed.
  It does not repair a formatter that throws before returning.
- Worker-only Node checks do not establish the Node version running a Mise test.
  That separate harness issue is recorded in
  [Node command contexts](node-lookup-in-agent-command-context.md).

## Upstream filing decision

- Upstream fault:
  reproduced through public Sinon assertions without the repository matcher.
  The source function promises a string representation but returns a symbol on the affected path.
- Fixability:
  the disposable function-boundary prototype corrects the measured paths.
- Supported use:
  `test/src/assert-test.js:2299` explicitly covers symbol method names;
  `COMPATIBILITY.md` includes maintained Node runtimes.
- Contribution policy:
  `CONTRIBUTING.md`,
  `README.md`,
  `CODE_OF_CONDUCT.md`,
  bug/PR templates and `.github/` policy searches were inspected.
  They invite reproduced bug reports and code contributions.
  No AI-assisted-submission prohibition was found in those files or the targeted tracker searches.
- Maintainer direction:
  the complete issue 1640 and PR 1642 discussion was read.
  Maintainers welcomed the comparable fix and released it in `4.1.4`.
  No contrary direction was found;
  that historical response is not a promise about a new contribution.
- Prototype:
  source-level integration controls pass,
  with the full upstream suite explicitly unverified.

The `.out-of-scope/` inventory contains no matched Sinon exemption.
Searches for symbol diagnostics found the existing issue and merged fix;
this is additive evidence about a called-spy path,
not a reason to file another copy of the historical report.
A current default-branch source read still returned the unchanged display-name value.
No external issue,
comment or pull request was sent.
External posting still requires authorization.

### Local additive-comment draft for issue 1640

~~~md
AI-assisted investigation with agent-run reproductions and source-level prototype checks;
no human-verification claim.

Sinon 22.1.0 on Node 26.8.2 still has a called-spy variant of this diagnostic failure.
The fix from PR 1642 is present,
and the original uncalled-spy case now reports AssertError correctly.

Using the installed ESM distribution:

```js
import sinon from './node_modules/sinon/pkg/sinon-esm.js';
const key = Symbol('method');
const object = { [key]() {} };
const spy = sinon.spy(object, key);
object[key]();
sinon.assert.callCount(spy, 0);
```

The result is TypeError: Cannot convert a Symbol value to a string,
rather than AssertError.
`spy.printf('%C')` reproduces it directly.

The call path is `spy-formatters.js`'s `%C` formatter,
then `proxy-call.js`'s `String(this.proxy)`.
The proxy delegates to `function-to-string.js`,
whose display-name fallback returns the symbol unchanged.
The existing outer `String(formatter(...))` cannot catch that earlier conversion failure.

A disposable prototype changes the fallback to
`return String(this.displayName || "sinon fake")`.
Exact original/patched source functions were exercised on published proxies across string/symbol keys,
zero/one/two calls and returning/throwing targets.
The patched false assertions produce AssertError and successful assertions still return.
This was not a rebuilt distribution or full upstream browser/contract run.

The consumer workaround is to assert the primitive `spy.callCount`,
avoiding spy call-history formatting.
~~~

[issue]: https://github.com/sinonjs/sinon/issues/1640
[fix]: https://github.com/sinonjs/sinon/pull/1642
