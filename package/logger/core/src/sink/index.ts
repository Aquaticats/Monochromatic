// Cross-platform factories only. The file sink lives behind the `./node`
// subpath (`src/node.ts`) and the IndexedDB and OPFS sinks behind `./browser`
// (`src/browser.ts`), so the root entry never pulls `node:fs` into the
// neutral artifact nor browser-only storage code into the Node artifact.
export { createConsoleSink, } from './console.ts';
export { createLocalStorageSink, } from './local-storage.ts';
export { createNoopSink, } from './noop.ts';
export { createSessionStorageSink, } from './session-storage.ts';
