# Electron 43: sandboxed preload scripts cannot be ESM, so an all-ESM package needs one deliberate CommonJS island bundled as .cjs

With `sandbox: true` (this repo's required renderer posture),
 Electron's preload runtime does
not load ES modules:
 sandboxed preloads run in a restricted environment that supports only
CommonJS with a polyfilled `require` exposing a small module set.
An all-ESM TypeScript package therefore needs its preload bundled separately to CJS,
 or the
bridge never installs and the renderer sees no exposed API.

Encountered while building `package/file-manager/variant-electron` (the repo's prior
Electron package,
 `electron-counter`,
 avoided preloads entirely,
 so this was the repo's first
contact with the constraint).

## Symptom

If the preload is emitted as ESM (`.mjs`,
 or `.js` under `"type": "module"`),
 the renderer's
`globalThis.fileManagerBridge` is undefined and the app's status line shows the boot failure
from the missing-bridge guard;
 Electron logs a preload error naming the module format.

## Root cause

Documented Electron behavior:
 sandboxed preloads run in a limited context without Node's ESM
loader ("Preload scripts" and "Process sandboxing" in Electron's docs state sandboxed preloads
are CommonJS with a limited `require` polyfill;
 ESM preloads additionally require an `.mjs`
path and unsandboxed renderers).
The constraint is Electron's sandbox architecture,
 not a packaging accident.

## Verification

Environment:
 electron 43.1.0 (pnpm store),
 `webPreferences: { sandbox: true,
contextIsolation: true, nodeIntegration: false }`.

Works cleanly (shipped):
 a dedicated tsdown config bundling `src/preload.ts` to
`dist/app/preload.cjs` (`format: 'cjs'`,
 `fixedExtension: true`),
 with
`webPreferences.preload` pointing at the `.cjs` file;
 the bridge installs and the boundary
test's IPC-backed steps pass.

Note the sibling footgun when adding this second config:
 tsdown's default clean deletes the
main bundle from the shared outDir;
 see `tsdown-shared-outdir-clean.md`.

## Verified workarounds

- Bundle the preload to CJS as its own build step (above).
   Tradeoff:
   one CJS island in an
  otherwise ESM package,
   and preload code must avoid ESM-only constructs that do not bundle
  down (none needed here;
   the preload only touches `contextBridge`/`ipcRenderer`).
- Not taken:
   `sandbox: false` to unlock ESM preloads.
   Tradeoff:
   gives up the sandbox,
   which
  this repo's security posture treats as non-negotiable for renderers.

## What does not work

- Pointing `preload` at an `.mjs`/ESM file with the sandbox on:
   the preload does not run as a
  module in the sandboxed context.

## Upstream filing decision

`.out-of-scope/` was checked:
 no Electron exemption exists.

1. Really upstream's fault?
    No;
    documented architectural restriction of the sandbox.
2. Can upstream fix it?
    It is a stated design constraint,
    not a defect.
3. Supported use case?
    Yes;
    the docs describe exactly the CJS-preload-with-sandbox pattern.
4. Would the repo welcome the contribution?
    Not evaluated;
    constraint 1 fails.
5. Will they likely fix it?
    Tracked as an Electron design topic,
    not something this repo needs.
6. Prototyped minimal fix?
    Not applicable;
    supported pattern used.

Decision:
 nothing to file.
