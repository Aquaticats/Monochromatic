/**
 Node build entry the unit tests import.

 `scratch-root.ts` stays out of `index.ts` because the neutral build must load in
 browsers, where `node:` imports do not exist, so a test cannot reach `scratchRoot`
 through the package entry. This module re-exports it, so the suite that checks it
 loads what this package builds rather than sibling source, as
 `test-import/require-eventual-artifact` requires.

 Node-only: `rolldown.node.config.ts` alone builds it, to
 `dist/final/node/testing.mjs`.

 @module
 */

//region Scratch root

/**
 Directory module tests create their fixtures in, never the system temporary one.

 @internal
 */
export { scratchRoot, } from './scratch-root.ts';

//endregion Scratch root
