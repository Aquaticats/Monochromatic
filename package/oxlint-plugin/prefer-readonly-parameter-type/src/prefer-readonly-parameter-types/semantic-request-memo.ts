/**
 Per-snapshot memo of TypeScript 7 synchronous API requests.

 A snapshot of the native server is immutable, so a question put to it twice gets the same answer twice.
 The rules put the same question many times: measured over one cold lint of the 36 `translation-repair`
 packages, 67.1 percent of 2.86 million requests repeated one already made on the same snapshot
 (`doc/planning/prefer-readonly-parameter-type-shared-analysis.md`, section "Cold time after landing").
 This module answers those from memory instead of crossing the pipe to the native child again.

 The memo is installed on one API client instance, never on the shared client class, because each
 native child numbers its snapshots independently and an entry is only valid for the child that made it.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { version as typescriptVersion, } from 'typescript';
import type { API, } from 'typescript/sync';

import { SemanticBridgeError, } from './semantic-bridge-error.ts';

/**
 Package logger for request memo lifecycle.
 */
const l = tagged({ tag: 'semantic-request-memo', },);

/**
 Request that tells the native server a snapshot is no longer held.
 */
const RELEASE_REQUEST = 'release';

/**
 Sentinel for a request whose parameters name no snapshot.
 */
const NO_SNAPSHOT: unique symbol = Symbol('request names no snapshot',);

/**
 Request names whose answer is a pure function of snapshot, project, and parameters.

 Each was read in the native server (`internal/api/session.go` at `microsoft/typescript-go` commit
 `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`, the `gitHead` of the installed `typescript@7.0.2`).
 Every handler resolves its snapshot and project, looks handles up in registries
 that live exactly as long as the snapshot, asks the persistent API checker a read-only question, and
 answers with handles into those same registries. The synchronous client turns every answer into an
 object through `getOrCreateSymbol`, `getOrCreateType` or `getOrCreateSignature`, which return the object
 already registered for an id and ignore the later answer, so replaying the first answer changes nothing
 a caller can observe.

 The registration those handlers do is the identity-keyed kind: the same checker object always gets the same
 id in a registry owned by the snapshot, and the entry stays until the snapshot is released. A handler that
 opens a project, registers something per request, or answers from a language service that takes its own
 checker does not qualify.

 The last six entries were added on 2026-10-19 (UTC) at that revision, each after a cold lint of the 36
 `translation-repair` packages showed it repeating a request on one snapshot (requests that reached the
 server in one lint, repeats among them): `getIndexInfosOfType` 28,175 and 10,017, `isArrayType` 24,879 and
 6,770, `getAliasedSymbol` 6,700 and 5,198, `getNonNullableType` 1,710 and 402, `getBaseConstraintOfType`
 699 and 531, `getExportsOfModule` 153 and 57. In a second lint every repeated answer of the six kinds (9,788,
 6,541, 5,198, 402, 531 and 57 repeats) was compared with the first answer to the same request: none differed
 in anything the client keeps. Only 299 `getIndexInfosOfType` and 18 `getBaseConstraintOfType` answers
 differed in raw text, in the `objectFlags` bits the checker sets lazily on a type, which the client ignores
 for an id it has already registered.

 It is an allow-list so that a request added by a later TypeScript release is sent every time until
 someone has read its handler. Left out on purpose:
 - `updateSnapshot` and `release` change which snapshots exist.
 - `initialize`, `parseConfigFile` and the profiling requests name no snapshot.
 - `getDefaultProjectForFile` and `getSourceFileNames` return values the client hands to callers as they
 arrive, so a hit would give every caller the same array (119 of 176 `getSourceFileNames` requests repeated).
 - `getSourceFile` and `getSourceFileMetadata` already have client-side caches and show no repeats.
 - `getSymbolsAtLocations`, `getSymbolOfType`, `getAliasSymbolOfType`, `getParametersOfSignature`,
 `getAliasTypeArgumentsOfType` and `isTupleType` meet the rule, but none of their 26,437, 12,092, 4,431,
 2,615, 17 and 3 requests repeated on a snapshot, so storing their answers would cost memory and save nothing.
 Add one when a workload repeats it.
 - `typeToString` returns a rendering, not a handle: the printer resolves type members lazily while it
 prints (`internal/checker/printer.go`), and no check showed the text independent of earlier queries
 (4,266 of 4,338 requests repeated, 0.07 s).
 - `isTypeAssignableTo` is a relation check run against the checker's relation caches, and no check showed
 its answer independent of earlier queries (64 of 92 requests repeated).
 - `getSignatureUsages` is answered by a language service that takes its own checker and walks every source
 file, not by the persistent API checker the others use (12 of 97 requests repeated).
 */
export const MEMOIZED_SNAPSHOT_REQUESTS: ReadonlySet<string> = new Set<string>([
  'getResolvedSignature',
  'getPropertiesOfType',
  'getShorthandAssignmentValueSymbol',
  'getSignaturesOfType',
  'getSymbolAtLocation',
  'getTypeArguments',
  'getTypeAtLocation',
  'getTypeOfSymbolAtLocation',
  'getTypesOfType',
  'resolveName',
  // Read and added on 2026-10-19 (UTC), after a cold family lint showed each one repeating on a snapshot.
  'getAliasedSymbol',
  'getBaseConstraintOfType',
  'getExportsOfModule',
  'getIndexInfosOfType',
  'getNonNullableType',
  'isArrayType',
],);

/**
 Most responses kept for one snapshot before the oldest is dropped.

 Chosen from a replay of the ordered request streams of the 36 `translation-repair` lint processes
 through this memo's rules (allow-list, one store per snapshot, oldest first out, drop on release):
 across the 2.69 million memoizable requests, the most answers any process held at once was 136,050
 (`corpus-pass`). At 131,072 only that process evicts, 4,998 answers, and the hit rate stays at
 70.99 percent, the same as with no bound. At 65,536 the family evicts 160,893 answers and loses
 0.02 percentage points; at 8,192 it evicts 559,516 and loses 0.44. A power of two just under the
 observed peak buys the whole hit rate and still caps the memory a package larger than any measured
 one can make the memo hold.

 The bound applies to each snapshot. The bridge keeps one live snapshot and, while it replaces it,
 one more, so a lint process holds a small multiple of it at most.

 Size in bytes, measured on 2026-10-19 (UTC) by serializing each distinct request key and its answer in one
 cold lint of the same 36 packages: the sixteen memoized request kinds stored 823,446 distinct entries of
 561 bytes on average (440.7 MiB serialized across the 36 processes). `corpus-pass`, the largest, stored
 141,367 entries and 75.1 MiB. The average per request kind runs from 149 bytes (`isArrayType`) to 2.8 KiB
 (`getPropertiesOfType`). With the six added kinds that process peaks at 141,220 entries in one store and
 evicts 10,167 at this bound; the other 35 do not evict, and the eviction costs 22 of the 2,060 repeats
 that process could still answer from memory. The heap held by the parsed answers was not measured and
 is larger than the serialized size.
 */
export const MAXIMUM_MEMOIZED_RESPONSES_PER_SNAPSHOT = 131_072;

/**
 Counters describing what a memo has served.

 `entries` is a size and the others are running totals.
 */
export type SnapshotRequestMemoStats = {
  readonly hits: number;
  readonly misses: number;
  readonly evictions: number;
  readonly releases: number;
  readonly entries: number;
};

/**
 Surface of the synchronous API client the memo wraps.
 */
export type SnapshotRequestClient = {
  apiRequest: (
    method: string,
    params?: unknown,
  ) => unknown;
};

/**
 Handle on an installed memo.
 */
export type SnapshotRequestMemo = {
  readonly stats: () => SnapshotRequestMemoStats;
};

/**
 Reads the snapshot a request is addressed to.

 @param params - Request parameters as the client sends them.

 @returns snapshot id, or {@link NO_SNAPSHOT} when the parameters carry none.

 @example
 ```ts
 snapshotOf({ snapshot: 3, project: 'p', }); // 3
 ```
 */
function snapshotOf(params: unknown,): number | string | typeof NO_SNAPSHOT {
  if (((typeof params) !== 'object') || (params === null))
    return NO_SNAPSHOT;
  if (!('snapshot' in params))
    return NO_SNAPSHOT;
  /**
   Snapshot id as the client wrote it.
   */
  const { snapshot, } = params;
  if ((typeof snapshot) === 'number')
    return snapshot;
  if ((typeof snapshot) === 'string')
    return snapshot;
  return NO_SNAPSHOT;
}

/**
 Wraps a client so repeated snapshot requests are answered from memory.

 @param client - Client whose request operation is replaced.

 @param maximumResponses - Most responses kept per snapshot; the oldest is dropped past it.

 @returns statistics handle for the installed memo.

 @throws {@link RangeError} when the bound is not a positive integer.

 @mutates client - Replaces its request operation with one that consults the memo first and forwards with `Reflect.apply`.

 @example
 ```ts
 memoizeSnapshotRequests({ client });
 ```
 */
export function memoizeSnapshotRequests({
  client,
  maximumResponses = MAXIMUM_MEMOIZED_RESPONSES_PER_SNAPSHOT,
}: {
  readonly client: SnapshotRequestClient;
  readonly maximumResponses?: number;
},): SnapshotRequestMemo {
  /**
   Function-tagged memo lifecycle logger.
   */
  const rl = tagged({
    tag: memoizeSnapshotRequests.name,
    l,
  },);
  if ((!Number.isSafeInteger(maximumResponses,)) || (maximumResponses < 1)) {
    throw new RangeError(
      `Request memo bound must be a positive integer, received ${String(maximumResponses,)}.`,
    );
  }
  /**
   Request operation the memo falls back to on a miss.
   */
  const forward = client.apiRequest;
  /**
   Responses by snapshot, each in the order it was first stored.
   */
  const responsesBySnapshot = new Map<number | string, Map<string, unknown>>();
  /**
   Running totals read back through the statistics handle.
   */
  const totals = {
    hits: 0,
    misses: 0,
    evictions: 0,
    releases: 0,
  };
  /**
   Starts the response store of a snapshot the memo has not seen.

   @param snapshot - Snapshot the first memoized request is addressed to.

   @returns the empty store, already registered.
   */
  function startResponses(snapshot: number | string,): Map<string, unknown> {
    /**
     Empty store registered under the snapshot.
     */
    const created = new Map<string, unknown>();
    responsesBySnapshot.set(
      snapshot,
      created,
    );
    return created;
  }
  client.apiRequest = function memoizedApiRequest(
    method: string,
    params?: unknown,
  ): unknown {
    if (method === RELEASE_REQUEST) {
      /**
       Snapshot being given back to the native server.
       */
      const released = snapshotOf(params,);
      // Dropped on every release: another handle on the same snapshot id loses its entries, which costs
      // requests and never correctness, while keeping them could outlive the server's registries.
      if ((released !== NO_SNAPSHOT) && responsesBySnapshot.delete(released,)) {
        totals.releases += 1;
        rl.debug(`dropped memo for released snapshot ${String(released,)}`,);
      }
      return Reflect.apply(
        forward,
        client,
        [
          method,
          params,
        ],
      );
    }
    if (!MEMOIZED_SNAPSHOT_REQUESTS.has(method,))
      return Reflect.apply(
        forward,
        client,
        [
          method,
          params,
        ],
      );
    /**
     Snapshot this request is addressed to.
     */
    const snapshot = snapshotOf(params,);
    if (snapshot === NO_SNAPSHOT)
      return Reflect.apply(
        forward,
        client,
        [
          method,
          params,
        ],
      );
    /**
     Responses already held for that snapshot, created on its first memoized request.
     */
    const held = responsesBySnapshot.get(snapshot,) ?? startResponses(snapshot,);
    /**
     Method and parameters exactly as they would cross the pipe.
     */
    const key = `${method}\0${JSON.stringify(params,)}`;
    /**
     Earlier answer, absent on first ask and ambiguous with an empty answer.
     */
    const earlier = held.get(key,);
    if ((earlier !== undefined) || held.has(key,)) {
      totals.hits += 1;
      return earlier;
    }
    totals.misses += 1;
    /**
     Answer from the native server, which throws for a refused request before anything is stored.
     */
    const answer = Reflect.apply(
      forward,
      client,
      [
        method,
        params,
      ]
    );
    if (held.size >= maximumResponses) {
      /**
       Key stored first, which a Map yields first.
       */
      const oldest = held.keys()
        .next();
      if (oldest.done !== true)
        held.delete(oldest.value,);
      totals.evictions += 1;
      if (totals.evictions === 1) {
        rl.debug(`snapshot ${String(snapshot,)} reached the bound of ${String(maximumResponses,)} responses; oldest are dropped from now on`,);
      }
    }
    held.set(
      key,
      answer,
    );
    return answer;
  };
  rl.debug(`memoizing ${String(MEMOIZED_SNAPSHOT_REQUESTS.size,)} request kinds, at most ${String(maximumResponses,)} responses per snapshot`,);
  return {
    stats: function memoStats(): SnapshotRequestMemoStats {
      return {
        ...totals,
        entries: [...responsesBySnapshot.values(),]
          .reduce(
          function addSize(
            sum,
            responses,
          ): number {
            return sum + responses.size;
          },
          0,
          ),
      };
    },
  };
}

/**
 Tests whether an unstable client value exposes the request operation the memo replaces.

 @param value - Candidate nested client.

 @returns whether value carries a callable `apiRequest`.

 @example
 ```ts
 isSnapshotRequestClient({ apiRequest() {} }); // true
 ```
 */
function isSnapshotRequestClient(value: unknown,): value is SnapshotRequestClient {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('apiRequest' in value)
    && ((typeof value.apiRequest) === 'function');
}

/**
 Installs the memo on the client of a TypeScript 7 synchronous API.

 @param api - API whose client the memo wraps.

 @param maximumResponses - Most responses kept per snapshot.

 @returns statistics handle for the installed memo.

 @throws {@link SemanticBridgeError} when the pinned client shape changed.

 @mutates api - Its nested client receives the memoized request operation; `Reflect.apply` reaches the original.

 @example
 ```ts
 memoizeApiSnapshotRequests({ api });
 ```
 */
export function memoizeApiSnapshotRequests({
  api,
  maximumResponses,
}: {
  readonly api: API;
  readonly maximumResponses?: number;
},): SnapshotRequestMemo {
  /**
   Public object view used for guarded internal-property traversal.
   */
  const apiObject: object = api;
  if (!('client' in apiObject)) {
    throw new SemanticBridgeError({
      reason: 'api-unavailable',
      message: `TypeScript ${typescriptVersion} synchronous API omitted its request client.`,
    },);
  }
  /**
   Sync client hidden behind unstable API implementation.
   */
  const { client, } = apiObject;
  if (!isSnapshotRequestClient(client,)) {
    throw new SemanticBridgeError({
      reason: 'api-unavailable',
      message: `TypeScript ${typescriptVersion} synchronous API client omitted its request operation.`,
    },);
  }
  return memoizeSnapshotRequests(maximumResponses === undefined
    ? { client, }
    : {
      client,
      maximumResponses,
    },);
}
