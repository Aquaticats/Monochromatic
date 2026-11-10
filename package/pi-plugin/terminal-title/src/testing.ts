/**
 Built entry the unit tests import.

 The package entry registers this pi extension's event handlers when it loads, so a
 test cannot import it. This module re-exports the title builder the tests drive, so
 they load what this package builds rather than sibling source, as
 `test-import/require-eventual-artifact` requires.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`, beside
 the extension bundle. Nothing in this package imports it, and pi never loads it.

 @module
 */

//region Title building

/**
 The prefix every title carries, and the two lookups that compose a title body.

 @internal
 */
export {
  TITLE_PREFIX,
  titleForEvent,
  titleForTool,
} from './title-builder.ts';

//endregion Title building
