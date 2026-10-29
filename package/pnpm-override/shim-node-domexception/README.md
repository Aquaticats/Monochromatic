# @monochromatic-dev/pnpm-override-shim-node-domexception

Ready to publish.

API-compatible shim for [`node-domexception`](https://www.npmjs.com/package/node-domexception).
Its `index.cjs` is a single line:
 `module.exports = globalThis.DOMException`,
 identical to the final line of `node-domexception@1.0.0/index.js` after the Node-<17 fallback block runs.

`pnpm-workspace.yaml` substitutes the deprecated upstream package with this shim
via `overrides: { 'node-domexception': 'link:package/pnpm-override/shim-node-domexception' }`.
The substitution silences pnpm's install-time deprecation warning
while keeping `fetch-blob`'s `import DOMException from 'node-domexception'`
(and its `new DOMException(...)` throw site) working unchanged on Node 17+ and Bun.

See `TROUBLESHOOTING.dependencies.md` for the audit trail
and `doc/dependency-blocklist.md` for when to reach for a shim package
versus a generic stub or removal.
