# Clack 1.7.0 note formatter snapshots fail on Node 22.18.0 because nested styleText resets outer color

## Symptom

Running Clack 1.7.0's default test suite on supported Node 22.18.0 reports four failures from
`packages/prompts/test/note.test.ts`:

```text
Test Files  1 failed | 17 passed (18)
Tests  4 failed | 593 passed (597)
```

Vitest identifies each failure as a snapshot mismatch.
The expected text reopens red after nested cyan formatting with ANSI code `31m`.
The received text instead emits foreground reset code `39m`,
so trailing formatter text loses its outer color.

The input,
multiselect,
and confirmation suites still pass.
The failure is specific to the `note()` formatter tests that nest `node:util` `styleText()` calls.

## Root cause

### Clack's formatter test requires nested outer-style restoration

[`packages/prompts/test/note.test.ts:70-87`][clack-note-test]
creates cyan inner text inside red outer text:

```ts
format: (line) => styleText('red', `* ${styleText('cyan', line)} *`),
```

The checked-in snapshot expects the red style to resume after the cyan close code.
Clack's [`packages/prompts/src/note.ts:24-28`][clack-note-source]
invokes that caller formatter before and after wrapping:

```ts
const maxWidthFormat = wrapMsg.map(format).reduce((sum, ln) => Math.max(stringWidth(ln), sum), 0);
const wrapWidth = width - (maxWidthFormat - maxWidthNormal);
return wrapAnsi(message, wrapWidth, opts);
```

The formatter's ANSI sequence is therefore produced by Node before Clack measures and renders it.

### Node 22.18.0 does not restore outer nested styles

This minimal call on Node 22.18.0 emits `39m` after cyan:

```js
styleText('red', `* ${styleText('cyan', 'x', { validateStream: false })} *`, {
  validateStream: false,
});
```

Observed JSON encoding:

```text
"\u001b[31m* \u001b[36mx\u001b[39m *\u001b[39m"
```

Node's fix,
[`util: respect nested formats in styleText`][node-nested-fix],
changed `lib/util.js` to replace an inner closing code with the active outer opening code when more text follows:

```js
if (offset + match.length < text.length) {
  return `${escapeStyleCode(code[0])}`;
}
return match;
```

Node 22.19.0 includes that change.
The same probe then emits red reopening code `31m`,
matching Clack's snapshot:

```text
"\u001b[31m* \u001b[36mx\u001b[31m *\u001b[39m"
```

### Clack's declared runtime floor predates the Node fix

Both published package manifests declare Node `>=20.12.0`.
[`packages/prompts/package.json:49-52`][clack-manifest]
contains:

```json
"engines": {
  "node": ">= 20.12.0"
}
```

The repository's `.nvmrc` and Volta configuration select Node 20.18.1.
Those declarations admit runtimes without nested `styleText()` restoration,
while the snapshot requires the later behavior.
Current Clack `main` still declares the same floor.

## Verification

### Versions and environment

- Clack:
  `@clack/prompts` 1.7.0,
  source commit `dc5bce8aae84a57b5863124adfaa839c1db1fa23`.
- Failing runtime:
  Node 22.18.0,
  container digest `sha256:752ea8a2f758c34002a0461bd9f1cee4f9a3c36d48494586f60ffce1fc708e0e`.
- Passing runtime:
  Node 22.19.0,
  container digest `sha256:cff78eb5aa1cf27dc2b6aeea9d31366415a43e9a9ea0ddec00d780b2b66fad0f`.
- Bounds:
  2 GiB memory,
  two CPUs,
  256 processes,
  no credentials,
  and no network during tests.

The failing suite command was:

```bash
pnpm run build
pnpm run types
pnpm run test
```

A focused reproduction is:

```bash
pnpm --dir packages/prompts exec vitest run test/note.test.ts
```

### Failing behavior catalog

On Node 22.18.0:

- formatted long note with CI rendering;
- formatted wide-character note with CI rendering;
- formatted long note with ordinary rendering;
- formatted wide-character note with ordinary rendering.

Every case expects outer red restoration and receives a foreground reset.

### Working behavior catalog

On Node 22.18.0:

- all 148 `@clack/core` tests pass;
- all 98 targeted high-level text,
  multiselect,
  and confirmation tests pass;
- plain note formatting without nested styles passes.

On Node 22.19.0:

- all 18 focused note tests pass;
- the nested `styleText()` positive control emits outer red restoration.

A Node 26.7.0 positive control emits the same restored sequence as Node 22.19.0.

## Verified workarounds

### Run Clack on Node 22.19.0 or newer

Node 22.19.0 contains the nested-format fix.
The focused Clack note suite passes unchanged on that exact runtime.

Tradeoff:
this drops Clack's declared Node 20 support and earlier Node 22 minors.

### Avoid nested `styleText()` calls in note formatters

A formatter can apply one style to its complete line
or use a coloring implementation that restores nested outer styles independently of Node's built-in behavior.

Tradeoff:
callers lose mixed nested colors or add another formatting dependency.
This does not make Clack's own supported-runtime test matrix coherent.

### Raise Clack's declared Node floor

A disposable prototype changes `.nvmrc`,
Volta,
and both package engine declarations to Node 22.19.0.
The patch is recorded in
[`clack-note-nested-styletext-node-floor.patch`](clack-note-nested-styletext-node-floor.patch).
The focused note suite then passes on Node 22.19.0.

Tradeoff:
raising the floor is a package compatibility decision rather than a local test-only correction.

## What does not work

### Updating the snapshots on Node 22.18.0

Accepting `39m` records the lost outer color as expected output.
It hides the supported-runtime mismatch rather than restoring formatting.

### Treating the four failures as random snapshot drift

All four diffs replace the same expected `31m` reopening code with `39m`.
The standalone Node probe reproduces that exact transition.

### Using only Clack's current CI runtime as the compatibility claim

Clack CI uses unpinned Node 22 through its reusable workflow.
A current Node 22 runner includes the fix,
but the package engine range and contributor configuration admit older runtimes.
Testing only the newest minor cannot verify the published floor.

## Upstream filing decision

No `.out-of-scope/` entry covers Clack,
Node `styleText()`,
or terminal prompt snapshots.
Open and closed Clack Issues and pull requests were searched for:

- `styleText nested Node 22.18 snapshot note`;
- `minimum node styleText nested format`.

No duplicate was found.

1.  **Is it really upstream's fault?**
    Yes.
    Node's historical behavior caused the color loss,
    but Clack's package range declares affected runtimes while its snapshots require the fixed behavior.
2.  **Can upstream fix it?**
    Yes.
    Clack can raise its runtime floor,
    revise its formatter contract,
    or avoid depending on nested built-in formatting behavior.
3.  **Are they supporting this use case?**
    Yes.
    `note()` explicitly accepts a formatter,
    and the upstream suite asserts nested colored formatter output.
4.  **Would the repo welcome our contribution?**
    Yes.
    `CONTRIBUTING.md` requests reproductions,
    environment details,
    tests,
    and pull requests.
    Its pull request template permits AI-generated code when disclosed.
    No policy forbids a human-verified issue report.
5.  **Will they likely fix it?**
    Plausible.
    Maintenance and release activity are current,
    and the relevant formatter tests remain active.
    No maintainer rejection or stated non-goal was found.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Yes.
    The recorded patch aligns package engines and contributor runtimes with Node 22.19.0.
    The focused suite fails before the runtime-floor change on Node 22.18.0
    and passes under the selected floor on Node 22.19.0.

All constraints pass.
The following draft is technically fileable,
but it has not been posted because external publication was not authorized.

~~~md
Title: Clack 1.7.0 note formatter snapshots fail on supported Node 22.18.0

Labels: bug

## Environment

- OS: Linux x86_64
- Node: 22.18.0
- Package: `@clack/prompts`
- Package version: 1.7.0
- Source: `dc5bce8aae84a57b5863124adfaa839c1db1fa23`

## Description

`pnpm test` reports four `packages/prompts/test/note.test.ts` snapshot failures on Node 22.18.0.
The expected output reopens outer red with `31m` after nested cyan.
Node 22.18.0 emits foreground reset `39m` instead.

Both package manifests currently declare Node `>=20.12.0`,
and the repository selects Node 20.18.1.
The snapshots require Node's later nested-`styleText()` behavior,
which Node 22.19.0 includes.

## Reproduction

```js
import { styleText } from 'node:util';

console.log(JSON.stringify(
  styleText('red', `* ${styleText('cyan', 'x', { validateStream: false })} *`, {
    validateStream: false,
  }),
));
```

Node 22.18.0:

```text
"\u001b[31m* \u001b[36mx\u001b[39m *\u001b[39m"
```

Node 22.19.0:

```text
"\u001b[31m* \u001b[36mx\u001b[31m *\u001b[39m"
```

Focused Clack command:

```bash
pnpm --dir packages/prompts exec vitest run test/note.test.ts
```

It fails four snapshots on Node 22.18.0 and passes all 18 tests on Node 22.19.0.

## Suggested fix

Align `.nvmrc`,
root Volta configuration,
and both package `engines.node` values on Node 22.19.0 or newer.
A verified four-file patch is available.
If retaining Node 20 is required,
the formatter contract or implementation needs a different compatibility strategy instead of snapshot updates.
~~~

[clack-manifest]: https://github.com/bombshell-dev/clack/blob/dc5bce8/packages/prompts/package.json#L49-L52
[clack-note-source]: https://github.com/bombshell-dev/clack/blob/dc5bce8/packages/prompts/src/note.ts#L24-L28
[clack-note-test]: https://github.com/bombshell-dev/clack/blob/dc5bce8/packages/prompts/test/note.test.ts#L70-L87
[node-nested-fix]: https://github.com/nodejs/node/pull/59098
