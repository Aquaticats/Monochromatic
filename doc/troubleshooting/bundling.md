# tsdown client bundles preserve `node:` dynamic imports inside try-catch, leaking CORS errors to the browser console

## Symptom

A browser running a client bundle produced by
`config-tsdown/.client.ts` logs CORS errors to the console even
though the app continues to work:

```text
Access to script at 'node:fs/promises' from origin 'http://localhost:4400'
has been blocked by CORS policy: Cross origin requests are only supported
for protocol schemes: chrome, chrome-extension, chrome-untrusted, data,
http, https, isolated-app.

node:fs/promises:1  Failed to load resource: net::ERR_FAILED
```

The application's logger logs fall through to other sinks;
 the file
sink's `import('node:fs/promises')` rejection is caught by the
sink's try/catch.
 The user-visible behaviour is correct but the
console is polluted,
 masking real diagnostics.

Discovered when `module-es`'s logger file-sink ended up in a
browser bundle because the default factory statically imports every
sink.

## Root cause

The client tsdown config (`config-tsdown/.client.ts`) declares
`deps.alwaysBundle: [/^@monochromatic-dev\//]` so workspace
packages are inlined into the output rather than left as external
imports.
 When an inlined module contains a dynamic
`import('node:fs/promises')` (even inside a try/catch),
 the bundler
preserves the dynamic import as-is in the emitted JS.

At runtime,
 the browser executes the dynamic import and attempts to
fetch `node:fs/promises` as a URL.
 The `node:` scheme is not in the
browser's list of fetchable protocols,
 so the network layer rejects
the request with a CORS error.
 Crucially,
 the browser logs the
network-level error to the console **before** the rejection reaches
the JavaScript promise;
 the try/catch sees the rejection and
recovers,
 but the console log is already written.

Key insight:
 **try/catch around a dynamic `import()` catches the
JavaScript error,
 but cannot suppress the browser's network-level
error log for the failed fetch.
**

Affected pattern:
 any module that

1.  Is statically imported (directly or transitively) into a
    client-side entry point.
2.  Contains dynamic `import('node:...')` calls,
     even inside
    try/catch.

The module-es logger file sink (`t sink/t file`) was the first
instance.
 It uses `await import('node:fs/promises')` inside
`verify()` and `$()`,
 both wrapped in try/catch.
 The default
logger factory statically imports all sinks including the file
sink,
 so the file sink ends up in any client bundle that uses the
logger.

## Verification

Version under test:

- tsdown bundler as pinned via `@monochromatic-dev/config-tsdown`
  at workspace HEAD
- Chromium 126+ (the error wording above is Chrome's;
   Firefox and
  Safari emit equivalent errors)

Reproduce:

```ts
// module-a.ts (workspace package)
export async function verify(): Promise<boolean> {
  try {
    const { stat, } = await import('node:fs/promises');
    await stat('/tmp/x',);
    return true;
  }
  catch {
    return false;
  }
}
```

Statically import `module-a` from a client entry point,
 bundle with
`config-tsdown/.client.ts`,
 load the page,
 and observe the CORS
error in the browser console even though `verify()` returns
`false` (no thrown exception in user code).

## Verified workaround: gate the dynamic import with a synchronous environment check

```ts
export async function verify(): Promise<boolean> {
  if (typeof globalThis.process === 'undefined'
    || typeof globalThis.process.versions?.node === 'undefined')
  {
    return false;
  }

  // Only reached in Node.js
  const { stat, } = await import('node:fs/promises');
  await stat('/tmp/x',);
  return true;
}
```

The synchronous check runs **before** the `import()` expression,
so the browser never starts the fetch.
 The function returns `false`
early without touching `node:fs/promises`.

Tradeoff:
 the module stays in the bundle (since static analysis
cannot prove the dynamic import is unreachable),
 so bundle size
includes the file-sink code even on browser builds.
 Acceptable
because the file sink is small;
 if size mattered,
 the alternative
is to split the logger factory by environment and never import
file-sink statically on the client.

## What does not work

- **Try/catch alone**:
   catches the JS rejection,
   but the browser
  has already logged the network-level error.
   The console pollution
  remains.
- **Marking `node:fs/promises` as external in the client config**:
  the bundler leaves the import as-is in the output and the browser
  still attempts the fetch at runtime.
   External-marking only
  affects whether the bundler inlines the module;
   it does not stop
  the browser from executing the import.
- **Tree-shaking**:
   dynamic `import()` expressions inside functions
  are not statically analysable,
   so bundlers cannot eliminate them.
  The static chain "client entry -> logger factory -> file sink
  module" guarantees the file-sink module is included even when its
  exports are unused.
- **Polyfilling `node:` schemes**:
   there is no portable polyfill
  for `node:fs/promises` in the browser;
   even rollup's
  `@rollup/plugin-node-polyfills` does not handle the `node:`
  prefix uniformly across bundlers.

## Prevention rules (workspace convention)

When writing modules that may be bundled for both Node.
js and
browser targets:

- Guard every `node:` dynamic import with a synchronous environment
  check (`globalThis.process.versions?.node`) before the `import()`
  expression.
- Do not rely on dynamic-import failure as the sole detection
  mechanism:
   it works logically but pollutes the console.
- If a module's `node:` imports are not needed in browser builds at
  all,
   keep them out of the static import graph entirely (split the
  module so the browser entry never reaches the node-only branch).

## Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** No. tsdown is correctly
    preserving the dynamic import the source asked for.
     Browsers are
    correctly refusing to fetch the `node:` scheme.
     Neither side
    misbehaves.
2.  **Can upstream fix it?
    ** They could heuristically warn when a
    client-target bundle contains `import('node:...')`,
     but the
    warning would have to permit legitimate cases (server bundles,
    isomorphic libraries) and the heuristic is noisy.
3.  **Are they supporting this use case?
    ** Both tsdown and the
    browser are doing what they are configured to do.
     The "support"
    here is the documentation that explains the constraint.
4.  **Will they likely fix it?
    ** N/A.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report.
 The fix lives at our boundary (the
synchronous environment check before each `node:` dynamic import).
