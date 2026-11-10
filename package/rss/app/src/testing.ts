/**
 Built entry the unit tests import.

 The test harness loads the artifact this package builds rather than sibling
 source, so this module re-exports exactly the behavior the tests drive. The
 server entry stays out: importing `src/index.ts` starts an HTTP listener, and
 a test must not bind a port.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`,
 copying `src/client.css` beside it because the asset module reads that file
 relative to its own URL.

 @module
 */

/**
 Feed fetching, parsing, and date sorting.

 @internal
 */
export { getSortedFeeds, } from './feed.ts';

/**
 Item normalization and the page renderer for one item.

 @internal
 */
export { itemToFeed, } from './html-item.ts';

/**
 The ignore endpoint, whose request-body contract rejects a bad body before it
 reaches disk.

 @internal
 */
export { ignore, } from './handler.ts';

/**
 Item extraction, date coercion, and sorting.

 @internal
 */
export { getSortedItems, } from './item.ts';

/**
 OPML text fetching, discarding unreachable sources.

 @internal
 */
export { getOPMLTexts, } from './opml-text.ts';

/**
 OPML parsing and outline URL validation.

 @internal
 */
export { getOutlinesFromOpmls, } from './outline.ts';
