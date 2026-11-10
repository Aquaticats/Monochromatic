# @monochromatic-dev/pnpm-override-shim-readable-stream

API-compatible shim for [`readable-stream@3.x`](https://www.npmjs.com/package/readable-stream)
 and [`readable-stream@4.x`](https://www.npmjs.com/package/readable-stream).
Its `index.cjs` re-exports `node:stream`
 with the same end-state shape upstream's READABLE_STREAM=disable branch produces (default export is `Stream.Readable`
 with every `node:stream` property defined as an own property on it,
 plus `.Stream` pointing at the module itself),
 and the `exports` map serves each `lib/_stream_*.js` deep path from a `lib/_stream_*.cjs` file that re-exports
 the matching class from `node:stream`,
 so deep imports such as `require('readable-stream/lib/_stream_writable.js')`,
the shape `winston-transport`'s `modern.js` used,
continue to
  resolve.
The package declares `"type": "module"` so its unit test loads as ESM,
 and every file a consumer loads (`index.cjs`,
 `lib/*.cjs`) stays CommonJS by its extension.
The shim uses `Object.defineProperty` instead of upstream's `Object.assign`
 because Node 22+'s `Stream.Readable` inherits a getter-only `promises` accessor from the legacy `Stream` prototype,
 which makes the upstream pattern throw `TypeError: Attempted to assign to readonly property` at module load;
 see `doc/decision/readable-stream-shim.md` for the full diagnosis.

Node builds a CommonJS module's ESM named exports from static `module.exports.<name> =` writes alone,
 so that copying loop by itself left `import { Readable } from 'readable-stream'` failing at link time
 while `require('readable-stream').Readable` answered.
`index.cjs` repeats every copied name in the form the detector reads,
 after the loop that owns the property descriptors;
 `index.unit.test.ts` links one static import per name `node:stream` exposes,
 so a Node release that adds a name to `node:stream` fails the suite until the entry declares it too.

`pnpm-workspace.yaml` substitutes both upstream versions
 with this shim via `overrides: { 'readable-stream': 'link:package/pnpm-override/shim-readable-stream' }`.
The substitution removes 3,005 LOC of duplicate stream machinery from `node_modules`.

No workspace package requests `readable-stream` today:
 `pnpm why readable-stream` resolves nothing,
 so the override is dormant defense in depth.
The three consumer chains the audit of 2026 enumerated left the workspace with their packages:
 winston and winston-transport through `package-deprecated/mcp/nvim`,
 and isomorphic-git through `package-paused/webapp-forge/server` and `package-paused/webapp-forge/stress`,
 neither of which matches the `package/*/*` workspace glob.
A future dependency that pulls `readable-stream` back in resolves to this shim without a manifest change.

See `TROUBLESHOOTING.dependencies.md` for the audit trail and `doc/dependency-blocklist.md` for when to reach
 for a shim package versus a generic stub or removal.
