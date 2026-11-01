# @monochromatic-dev/css-core-edit.fuzz

Property-based fuzz campaign for
[`@monochromatic-dev/css-core-edit`](../core-edit/README.md).

Non-runtime sidecar,
 mirroring `jsonc-edit.fuzz`:
the runtime package's `src` stays pure production code
while the fuzz generators,
 properties,
 and campaign task live here.

## Properties

- **Byte round-trip**:
  every generated structurally-valid document
  (declarations,
   custom properties,
   `&` and relaxed nesting,
  statement and block at-rules,
   adversarial strings,
   varied trivia)
  parses and stringifies byte-identically.
- **Totality**:
  an arbitrary string either parses (and then must round-trip byte-exactly)
  or throws `CssParseError`;
   no other failure mode exists.
- **Structural sharing**:
  a keep-everything transform returns the identical root reference.
- **postcss differential oracle**:
  postcss accepts every generated document css-edit accepts,
  and reproduces the same bytes from its own CST.

## Running

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/css/core-edit.fuzz:<task>`.
The unit suite runs each property 200 times by default.
Refreeze the baseline after intentionally changing reachable runtime functions.

Property files import css-edit source through its `/ts` subpath,
so no build step is needed.

## Coverage gate

`fuzz:coverage` runs the deterministic `src/coverage-driver.ts` under
`NODE_V8_COVERAGE` and gates the covered-function count per runtime source
file against the frozen `coverage-baseline.json`
(mirroring `jsonc-edit.fuzz`'s gate).
It measures the runtime package's `src/` reachability,
 not this sidecar:
its `SOURCE_MARKER` targets `package/css/core-edit/src`.
The frozen baseline reaches every function in every runtime file,
so any change that makes a function unreachable from the public API fails
the check.
