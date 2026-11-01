# @monochromatic-dev/css-core-edit.bench

Parse plus stringify benchmark for
[`@monochromatic-dev/css-core-edit`](../core-edit/README.md)
against postcss and css-tree,
on a synthetic stylesheet shaped like this repository's real CSS
(custom properties,
 `&` nesting,
 media queries,
 adversarial strings).

Non-runtime sidecar,
 mirroring `jsonc-edit.bench`;
 timings via mitata.

Caveat when reading numbers:
the three parsers do different amounts of `work.css`-edit keeps bytes and produces token slices (lossless),
postcss builds a full node tree with raws (lossless-ish),
css-tree builds a detailed value-level AST and emits minified output (lossy).

## Running

```bash
mise run //package/css/core-edit.bench:bench
```
