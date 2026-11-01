/**
 Browser-only entry (`\@monochromatic-dev/logger-core/browser`).

 Ships the sinks whose backends exist only in browsers (`indexedDB`,
 `navigator.storage`). Built only by `rolldown.browser.config.ts`, so the
 Node artifact carries no dead browser-storage probes. The IndexedDB sink is
 also part of the neutral default list (see `default-sinks.neutral.ts`);
 the OPFS sink is opt-in only, see `DECISIONS.md`.

 @module
 */

export { createIndexedDbSink, } from './sink/indexed-db.ts';
export { createOpfsSink, } from './sink/opfs.ts';
