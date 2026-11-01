# @monochromatic-dev/css-core-edit

Byte-preserving CSS read,
 edit,
 and write utility.
Free-function API over an immutable `CssEditState`;
a stylesheet-structure CST over the
[`@csstools/css-tokenizer`](https://github.com/csstools/postcss-plugins/tree/main/packages/css-tokenizer)
spec tokenizer,
 with comments and formatting kept as first-class token data.

Member of the workspace's format-editing family
(`jsonc-edit`,
 `toml-edit`):
immutable state,
 every edit returns a fresh tree,
unchanged nodes shared by reference.

## Why this exists

The 2026-07 parser survey
(recorded in [doc/troubleshooting/css-tooling.md](../../../doc/troubleshooting/css-tooling.md))
found no established pure-JS CSS parser that is simultaneously
lossless,
 nesting-correct,
 and maintained:
css-tree corrupts relaxed nesting and drops comments,
`@adobe/css-tools` throws on custom-property block values,
`@projectwallace/css-parser` cannot emit CSS,
and postcss carries process-global reach-ins and a 2013-era mutable AST.
The `@csstools` tokenizer is spec-exact and byte-lossless but deliberately headless;
this package supplies the head:
the CSS Syntax Module Level 3 section 5 structure layer.

## Design

- **Byte fidelity**:
  tokens keep their source representation;
  `stringifyCss` over an unedited state reproduces the input byte-exactly.
  Comments and whitespace live in the tree as `trivia` nodes.
- **Unified block contents**:
  every block accepts declarations,
   nested rules
  (`&` and relaxed),
   and at-rules,
  so unknown at-rules like `@mixin` parse structurally.
  Declaration-versus-rule classification follows the spec's
  restart-as-rule step
  (`span:hover { ... }` reclassifies;
   `--x: { ... };` stays a declaration).
- **Strict**:
  malformed input throws a positioned `CssParseError`
  instead of producing recovery nodes;
   a build pipeline wants loud failures.
- **Immutable transforms**:
  `transformNodes`/`transformStylesheet` rebuild bottom-up under a visitor;
  a visitor returns a node (keep or replace) or a node array
  (splice;
   empty removes).
  Untouched subtrees keep reference identity.
- **No value parsing**:
  preludes and declaration values stay token slices.
  Consumers needing token-level detail use the re-exported guards
  (`isTokenString`,
   `isTokenURL`,
   `isTokenIdent`) and `tokenData`.

## Usage

```ts
import {
  asCssSource,
  isCssAtRule,
  parseCss,
  stringifyCss,
  transformStylesheet,
} from '@monochromatic-dev/css-core-edit';

const state = parseCss({ source: asCssSource('.btn { @apply --card; }',), },);

const withoutApplies = transformStylesheet({
  root: state.root,
  visit: (node,) => (isCssAtRule(node,) && (node.name === 'apply')) ? [] : node,
  pruneTriviaBeforeRemoved: true,
},);

stringifyCss({ state, },); // byte-identical to the input
```

## Node kinds

- `stylesheet`:
   root,
   ordered children
- `atRule`:
   at-keyword token,
   unescaped `name`,
   prelude token slice,
  optional `block` or `semicolonToken`
- `rule`:
   selector prelude token slice plus `block`
- `block`:
   `{`/`}` tokens plus unified children
- `declaration`:
   raw token run including any trailing `;`
- `trivia`:
   whitespace,
   comments,
   CDO/CDC

## Sidecars

- [`css-edit.fuzz`](../core-edit.fuzz/README.md):
  fast-check properties (byte round-trip,
   totality,
   structural sharing)
  plus a postcss differential oracle
- [`css-edit.bench`](../core-edit.bench/README.md):
  mitata benchmark against postcss and css-tree
- [`css-edit.conformance`](../core-edit.conformance/README.md):
  curated css-parsing-tests-style corpus with context amplification

## Testing

```bash
mise run //package/css/core-edit:buildAndTest
```
