import { createConsoleSink, } from './sink/console.ts';
import { createIndexedDbSink, } from './sink/indexed-db.ts';
import { createLocalStorageSink, } from './sink/local-storage.ts';
import { createSessionStorageSink, } from './sink/session-storage.ts';

import type { Sink, } from './types.ts';

/**
 Builds the default sink backends the platform-neutral artifact attempts, in
 priority order. Chosen at bundle time: `package.json` maps `#default-sinks`
 to this module under the `default` condition, so every non-Node resolution
 (browsers, Deno, Bun, workers) inlines this list without a runtime
 platform probe. Called on the default logger's first use, never at import,
 so a runtime that forbids timers and I/O in global scope (Cloudflare
 Workers, issue #493) constructs and verifies its sinks inside the handler
 that logs first. Each runtime keeps only the sinks whose `verify` confirms
 platform probe. Each runtime keeps only the sinks whose `verify` confirms
 its backend: {@link createConsoleSink} everywhere,
 {@link createIndexedDbSink} in browsers, {@link createSessionStorageSink}
 wherever web storage round-trips (browsers, Deno),
 {@link createLocalStorageSink} wherever `localStorage` round-trips
 (browsers, Deno). The noop sink is intentionally absent: the console sink
 verifies wherever `console` and `queueMicrotask` exist, so the default
 logger has a backend in every supported runtime, and a custom
 `createLogger` whose sinks all fail verification surfaces the "No logging
 backends available" error instead of silently discarding. The file sink
 is absent because its static `node:fs/promises` import cannot load outside
 Node; it ships through the `./node` subpath and the Node default list
 instead. The OPFS sink is exported from `./browser` but not a default:
 its stream stages writes until a close that a crash never performs, so
 IndexedDB holds the persistent-browser slot; see `DECISIONS.md`.

 @returns Fresh default sinks in priority order.

 @example
 ```ts
 const { logger } = createLogger({ sinks: createDefaultSinks() });
 ```
 */
export function createDefaultSinks(): readonly Sink[] {
  return [
    createConsoleSink(),
    createIndexedDbSink(),
    createSessionStorageSink(),
    createLocalStorageSink(),
  ];
}
