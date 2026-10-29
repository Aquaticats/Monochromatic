# @monochromatic-dev/pnpm-override-shim-ungap-structured-clone

Ready to publish.

API-compatible shim for [`@ungap/structured-clone`](https://www.npmjs.com/package/@ungap/structured-clone).
Its `index.cjs` is a single line:
 `module.exports = globalThis.structuredClone`,
 identical to what the upstream package's default export resolves to
 on every runtime where native `structuredClone` exists
 (Node 17+, Bun, all modern browsers per the Firefox ESR 140 baseline
 in `PHILOSOPHY.browser-support.md`).

`pnpm-workspace.yaml` substitutes the deprecated upstream package with this shim
via `overrides: { '@ungap/structured-clone': 'link:package/pnpm-override/shim-ungap-structured-clone' }`.
The substitution clears the published `Potential CWE-502 - Update to 1.3.1 or higher` deprecation
while keeping `mdast-util-to-hast`'s and `rehype-autolink-headings`'s
`import structuredClone from '@ungap/structured-clone'`
(and the `structuredClone(value)` call sites in `lib/state.js`,
 `lib/footer.js`,
 and `lib/index.js`) working unchanged.

The upstream is a ponyfill that exists for environments without native `structuredClone`.
 No supported environment in this workspace falls into that category,
 so re-exporting the global is API-equivalent.
 Only the default export is consumed by either transitive parent;
 named exports (`serialize`,
 `deserialize`) are intentionally omitted and can be added if a future transitive dep reaches in.

See `TROUBLESHOOTING.dependencies.md` for the audit trail
and `doc/dependency-blocklist.md` for when to reach for a shim package
versus a generic stub or removal.
