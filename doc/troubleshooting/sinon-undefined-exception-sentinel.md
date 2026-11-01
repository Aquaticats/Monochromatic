# Sinon 22.1.0 drops an injected undefined throw before a logger test can observe it

## Symptom

The logger callback-observation test injects `undefined` through
`sinon.stub().throws(function factory() { return undefined; })`.
The callback appears to return normally,
so its observer correctly records no abnormal completion.
The test then fails:

```text
# main-merge-package-checks-nV45Z9/module-logger-test-unit.out
AssertionError: expected [] to deeply equal [ 'warn' ]
```

This is a fixture problem,
not a failure of the observer's catch boundary.
The observer and test source are unchanged from the repair branch's pre-merge commit
`ea3087d9c477b133df2b15bcaa401987d42bd78a`.
It is separate from Sinon's symbol-named spy diagnostic behavior.

## Root cause

The inspected Sinon source is commit
`ab289e92cdd76caf8cec2b0a8c9a391283e6c9df`,
matching the session's `22.1.0` source checkout.
In that checkout,
`src/sinon/behavior.js:163` invokes the supplied factory and throws its result:

```js
// Sinon src/sinon/behavior.js
} else if (this.exceptionCreator) {
    this.exception = this.exceptionCreator();
    this.exceptionCreator = undefined;
    throw this.exception;
}
```

The surrounding proxy catches that value at `src/sinon/proxy-invoke.js:74`:

```js
// Sinon src/sinon/proxy-invoke.js
} catch (e) {
    exception = e;
}
```

Its rethrow condition at `src/sinon/proxy-invoke.js:104` uses the value as the presence test:

```js
// Sinon src/sinon/proxy-invoke.js
if (exception !== undefined) {
    throw exception;
}
```

A thrown `undefined` and no exception therefore take the same return path.
The installed generated module has the corresponding condition at
`node_modules/.pnpm/sinon@22.1.0/node_modules/sinon/lib/sinon/proxy-invoke.js:110`.
No installed dependency or source checkout is patched.

## Verification

The fixed-input probe `probe-sinon-undefined-20260914.mts` in private agent scratch
runs on Node `26.8.2` and writes `sinon-undefined-probe-20260914.json`.
It records:

- Native suspended-generator throw:
  `threw: true`,
  `undefinedValue: true`.
- Sinon factory-backed stub:
  `threw: false`,
  with one recorded invocation.

The logger tests also distinguish ordinary Error,
string,
Symbol,
null,
false and revoked Error values.
The initial run passes those cases but fails the undefined fixture.
After replacing the fixture,
`main-merge-package-checks-MELqFK` passes the complete logger unit task,
including its native undefined positive control.
The later `main-merge-package-checks-63k9ap` run also passes the logger build,
types,
complete unit task and read-only lint after the merged conventions are applied.
The local workaround has not been reviewed;
[repository issue 509](https://github.com/Aquaticat/Monochromatic/issues/509) records that status.
This is bounded fixture evidence,
not a claim about every Sinon assertion or future version.

## Verified remedy

Use native abrupt completion for arbitrary foreign values,
with a separate invocation spy that does not wrap the throwing operation:

```ts
// package/logger/core/src/observe-callbacks.unit.test.ts
function* faultCarrier(): Generator<undefined> {
  yield;
}

const entered = sinon.spy(function enteredCallback() {});
function foreignFailure(): void {
  entered();
  const carrier = faultCarrier();
  carrier.throw(fault);
}
```

The package test uses `caught` to prove the native control throws the exact supplied value
before offering the callback to `observeLoggerCallbacks`.
The invocation counter observes entry only;
it cannot replace the exception with Sinon's own sentinel.

Tradeoff:
this does not test Sinon as the throwing intermediary.
It tests the logger's actual contract against a native foreign throw,
which is the intended boundary.
No logger message,
error name or cause is needed to construct that observation.

## What does not work

- Returning `undefined` from a Sinon exception factory does not establish an escaped undefined throw.
- A Sinon invocation count proves invocation,
  not exceptional completion.
- Changing the observer to report a failure when a callback returns normally would encode the fixture defect
  into production behavior.

## Upstream filing decision

[Issue 2471][issue] already reports this behavior.
[PR 2763][pull] already describes the same value-versus-presence cause,
a separate `didThrow` record and broader falsy-value assertion implications.
Both bodies and available comments were read through GitHub's API.

1.  Upstream behavior:
    reproduced at the proxy boundary,
    independently of the merge.
2.  Fixability:
    the existing PR supplies a concrete candidate;
    no fix-duration estimate is made here.
3.  Supported use:
    the issue quotes the assertion contract for any thrown exception.
4.  Contribution policy:
    no new contribution is proposed or authorized by this repository merge task.
5.  Maintainer intent:
    the issue and PR remain open in the inspected response.
    Mixed issue labels are not treated as a decision about the PR.
6.  Prototype:
    this session verifies the consumer-side fixture remedy,
    not the existing upstream patch.

Nothing is filed or drafted for posting.
The existing issue and PR already carry the relevant root cause and candidate fix;
a duplicate report or a bare confirmation would add no needed technical explanation.

[issue]: https://github.com/sinonjs/sinon/issues/2471
[pull]: https://github.com/sinonjs/sinon/pull/2763
