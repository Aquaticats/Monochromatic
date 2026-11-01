/**
 In-memory {@link ObjectStore} for unit tests. Exported from the package entry as `\@internal` so tests import it through the built artifact.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type {
  ObjectStore,
  StoredObject,
  StoredObjectHead,
} from './store.ts';

/**
 Memory store plus a view of its contents for assertions.
 */
export type MemoryObjectStore = ObjectStore & {
  /**
   Stored bytes keyed by oid.
   */
  readonly objects: Map<string, Uint8Array<ArrayBuffer>>;
};

/**
 Build an in-memory store, optionally seeded.

 @param initial - objects present before the test starts, keyed by oid

 @returns store whose `objects` map reflects every put

 @example
 ```ts
 const store = createMemoryObjectStore({ [oid]: new TextEncoder().encode('png bytes') });
 ```
 */
export function createMemoryObjectStore(initial: Readonly<Record<string, Uint8Array<ArrayBuffer>>> = {},): MemoryObjectStore {
  /**
   Backing map; seeded from `initial`.
   */
  const objects = new Map<string, Uint8Array<ArrayBuffer>>(Object.entries(initial,),);
  return {
    objects,
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors Cloudflare R2Bucket.head through the ObjectStore contract
    head(key: string,): Promise<StoredObjectHead | null> {
      /**
       Seeded or stored bytes for `key`.
       */
      const bytes = objects.get(key,);
      return Promise.resolve(bytes === undefined ? null : { size: bytes.byteLength, },);
    },
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors Cloudflare R2Bucket.get through the ObjectStore contract
    get(key: string,): Promise<StoredObject | null> {
      /**
       Seeded or stored bytes for `key`.
       */
      const bytes = objects.get(key,);
      if (bytes === undefined) {
        return Promise.resolve(null,);
      }
      return Promise.resolve({
        size: bytes.byteLength,
        body: nonNullishOrThrow(new Response(bytes,).body,),
      },);
    },
    async put(
      key: string,
      body: ReadableStream<Uint8Array>,
    ): Promise<void> {
      /**
       Body drained into memory.
       */
      const bytes = new Uint8Array(await new Response(body,).arrayBuffer(),);
      objects.set(
        key,
        bytes,
      );
    },
  };
}
