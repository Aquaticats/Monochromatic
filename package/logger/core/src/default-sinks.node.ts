import { createConsoleSink, } from './sink/console.ts';
import { createFileSink, } from './sink/file.ts';
import { createLocalStorageSink, } from './sink/local-storage.ts';
import { createSessionStorageSink, } from './sink/session-storage.ts';

import type { Sink, } from './types.ts';

/**
 Builds the default sink backends the Node artifact attempts, in priority
 order. Chosen at bundle time: `package.json` maps `#default-sinks` to this
 module under the `node` condition, so `logger.ts` inlines this list without
 a runtime platform probe. Called on the default logger's first use, never
 at import, so no sink is constructed in global scope (issue #493). Each
 runtime keeps only the sinks whose `verify`
 confirms its backend: {@link createConsoleSink} everywhere,
 {@link createSessionStorageSink} wherever web storage round-trips (Node
 22+, Deno), {@link createLocalStorageSink} wherever `localStorage`
 round-trips (Deno, Node launched with `--localstorage-file`), and
 {@link createFileSink} in the ancestor project or writable user state. The
 noop sink is intentionally absent: the console sink verifies wherever
 `console` and `queueMicrotask` exist, so the default logger has a backend
 in every supported runtime, and a custom `createLogger` whose sinks all
 fail verification surfaces the "No logging backends available" error
 instead of silently discarding. The IndexedDB and OPFS sinks are absent
 because Node exposes neither `indexedDB` nor `navigator.storage`; they
 ship through the `./browser` subpath instead, keeping their probes and
 code out of this artifact.

 @returns Fresh default sinks in priority order.

 @example
 ```ts
 const { logger } = createLogger({ sinks: createDefaultSinks() });
 ```
 */
export function createDefaultSinks(): readonly Sink[] {
  return [
    createConsoleSink(),
    createSessionStorageSink(),
    createLocalStorageSink(),
    createFileSink(),
  ];
}
