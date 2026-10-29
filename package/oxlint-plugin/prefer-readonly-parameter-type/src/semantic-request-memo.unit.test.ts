import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MEMOIZED_SNAPSHOT_REQUESTS,
  memoizeSnapshotRequests,
  type SnapshotRequestClient,
} from '../dist/final/node/index.mjs';

/** One request that reached the stub standing in for the native server. */
type ServerRequest = {
  readonly method: string;
  readonly params: unknown;
};

/** Stub client plus the requests that reached it. */
type CountingClient = {
  readonly client: SnapshotRequestClient;
  readonly reached: readonly ServerRequest[];
};

/**
 Builds a client standing in for the native server, answering each request with a fresh object.

 @param fail - Methods that throw instead of answering.

 @param empty - Methods that answer with nothing.

 @param falsy - Methods that answer with the boolean `false`.

 @returns the client and the live list of requests that reached it.
 */
function countingClient({
  fail = [],
  empty = [],
  falsy = [],
}: {
  readonly fail?: readonly string[];
  readonly empty?: readonly string[];
  readonly falsy?: readonly string[];
},): CountingClient {
  /** Requests that reached the stub, appended as they arrive. */
  const reached: ServerRequest[] = [];
  return {
    reached,
    client: {
      apiRequest: function answer(method: string, params?: unknown,): unknown {
        reached.push({
          method,
          params,
        },);
        if (fail.includes(method,))
          throw new Error(`stub server refused ${method}`,);
        if (empty.includes(method,))
          return undefined;
        if (falsy.includes(method,))
          return false;
        return { answeredRequest: reached.length, };
      },
    },
  };
}

/**
 Counts the requests of one method that reached the stub.

 @param reached - Requests that reached the stub.

 @param method - Request name to count.

 @returns how many requests of that name arrived.
 */
function reachedCount({
  reached,
  method,
}: {
  readonly reached: readonly ServerRequest[];
  readonly method: string;
},): number {
  return reached.filter(function isMethod(request,): boolean {
    return request.method === method;
  },).length;
}

/** Hot request every case below repeats. */
const LOCATION_REQUEST = 'getTypeAtLocation';

/**
 Requests added to the allow-list after a cold lint of the `translation-repair` packages showed each one repeating
 on a snapshot, with the parameters the synchronous client sends for each.
 */
const ADDED_REQUESTS: readonly (readonly [method: string, params: Readonly<Record<string, unknown>>,])[] = [
  ['getAliasedSymbol', { snapshot: 1, project: 'p', symbol: 'symbol-a', },],
  ['getBaseConstraintOfType', { snapshot: 1, project: 'p', type: 'type-a', },],
  ['getExportsOfModule', { snapshot: 1, project: 'p', symbol: 'symbol-a', },],
  ['getIndexInfosOfType', { snapshot: 1, project: 'p', type: 'type-a', },],
  ['getNonNullableType', { snapshot: 1, project: 'p', type: 'type-a', },],
  ['isArrayType', { snapshot: 1, project: 'p', type: 'type-a', },],
];

await describe({
  name: memoizeSnapshotRequests.name,
  concurrency: 1,
  children: [
    it({
      name: 'sends a repeated request on one snapshot to the server once',
      fn: async () => {
        const { client, reached, } = countingClient({},);
        memoizeSnapshotRequests({ client, },);
        const params = { snapshot: 1, project: 'p', location: 'node-a', };
        const first = client.apiRequest(LOCATION_REQUEST, params,);
        const second = client.apiRequest(LOCATION_REQUEST, { ...params, },);

        expect(reachedCount({ reached, method: LOCATION_REQUEST, },),).toBe(1,);
        expect(second,).toBe(first,);
      },
    },),
    ...ADDED_REQUESTS.map(function mapAdded([method, params,],) {
      return it({
        name: `sends a repeated ${method} request on one snapshot to the server once`,
        fn: async () => {
          const { client, reached, } = countingClient({},);
          memoizeSnapshotRequests({ client, },);
          const first = client.apiRequest(method, params,);
          const second = client.apiRequest(method, { ...params, },);
          client.apiRequest(method, { ...params, snapshot: 2, },);
          client.apiRequest(method, { ...params, project: 'q', },);

          expect(MEMOIZED_SNAPSHOT_REQUESTS.has(method,),).toBe(true,);
          // One reach for snapshot 1, one for snapshot 2, one for the other project.
          expect(reachedCount({ reached, method, },),).toBe(3,);
          expect(second,).toBe(first,);
        },
      },);
    },),
    it({
      name: 'remembers the false answer of isArrayType as an answer',
      fn: async () => {
        const { client, reached, } = countingClient({ falsy: ['isArrayType',], },);
        memoizeSnapshotRequests({ client, },);
        const params = { snapshot: 1, project: 'p', type: 'type-a', };

        expect(client.apiRequest('isArrayType', params,),).toBe(false,);
        expect(client.apiRequest('isArrayType', { ...params, },),).toBe(false,);
        expect(reachedCount({ reached, method: 'isArrayType', },),).toBe(1,);
      },
    },),
    it({
      name: 'sends the same request on a new snapshot to the server again',
      fn: async () => {
        const { client, reached, } = countingClient({},);
        memoizeSnapshotRequests({ client, },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 1, project: 'p', location: 'node-a', },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 2, project: 'p', location: 'node-a', },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 2, project: 'p', location: 'node-a', },);

        expect(reachedCount({ reached, method: LOCATION_REQUEST, },),).toBe(2,);
      },
    },),
    it({
      name: 'keeps requests that differ in project or parameters apart',
      fn: async () => {
        const { client, reached, } = countingClient({},);
        memoizeSnapshotRequests({ client, },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 1, project: 'p', location: 'node-a', },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 1, project: 'q', location: 'node-a', },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 1, project: 'p', location: 'node-b', },);
        client.apiRequest('getSymbolAtLocation', { snapshot: 1, project: 'p', location: 'node-a', },);

        expect(reached.length,).toBe(4,);
      },
    },),
    it({
      name: 'always sends a request outside the allow-list to the server',
      fn: async () => {
        const { client, reached, } = countingClient({},);
        memoizeSnapshotRequests({ client, },);
        /** State-changing, snapshot-scoped, and unknown requests alike. */
        const unmemoized = [
          'updateSnapshot',
          'getDefaultProjectForFile',
          'getSourceFileNames',
          'getSignatureUsages',
          'isTypeAssignableTo',
          'typeToString',
          'aFutureMethod',
        ];
        for (const method of unmemoized) {
          client.apiRequest(method, { snapshot: 1, project: 'p', },);
          client.apiRequest(method, { snapshot: 1, project: 'p', },);
        }

        expect(reached.length,).toBe(unmemoized.length * 2,);
        expect(unmemoized.filter(function isMemoized(method,): boolean {
          return MEMOIZED_SNAPSHOT_REQUESTS.has(method,);
        },),).toEqual([],);
      },
    },),
    it({
      name: 'sends every release to the server and drops that snapshot entries',
      fn: async () => {
        const { client, reached, } = countingClient({},);
        const memo = memoizeSnapshotRequests({ client, },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 1, project: 'p', location: 'node-a', },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 2, project: 'p', location: 'node-a', },);
        client.apiRequest('release', { snapshot: 1, },);
        client.apiRequest('release', { snapshot: 1, },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 1, project: 'p', location: 'node-a', },);
        client.apiRequest(LOCATION_REQUEST, { snapshot: 2, project: 'p', location: 'node-a', },);

        expect(reachedCount({ reached, method: 'release', },),).toBe(2,);
        // Snapshot 1 was asked again after its release, snapshot 2 kept its entry.
        expect(reachedCount({ reached, method: LOCATION_REQUEST, },),).toBe(3,);
        expect(memo.stats().releases,).toBe(1,);
      },
    },),
    it({
      name: 'evicts the oldest response once the bound is reached',
      fn: async () => {
        const { client, reached, } = countingClient({},);
        const memo = memoizeSnapshotRequests({ client, maximumResponses: 2, },);
        /**
         Requests one location on the only snapshot.

         @param location - Node the request names.
         */
        const ask = (location: string,): unknown => client.apiRequest(LOCATION_REQUEST, { snapshot: 1, project: 'p', location, },);
        ask('a',);
        ask('b',);
        ask('c',);
        // Entry a was the oldest when c arrived, so asking for it again reaches the server, and evicts b.
        ask('a',);
        // Entry a was just stored, so it is still held.
        ask('a',);

        expect(reached.length,).toBe(4,);
        expect(memo.stats().evictions,).toBe(2,);
        expect(memo.stats().entries,).toBe(2,);
      },
    },),
    it({
      name: 'does not remember a request the server refused',
      fn: async () => {
        const { client, reached, } = countingClient({ fail: [LOCATION_REQUEST,], },);
        memoizeSnapshotRequests({ client, },);
        const params = { snapshot: 1, project: 'p', location: 'node-a', };

        expect(() => client.apiRequest(LOCATION_REQUEST, params,),).toThrow('stub server refused',);
        expect(() => client.apiRequest(LOCATION_REQUEST, params,),).toThrow('stub server refused',);
        expect(reached.length,).toBe(2,);
      },
    },),
    it({
      name: 'remembers an empty answer as an answer',
      fn: async () => {
        const { client, reached, } = countingClient({ empty: [LOCATION_REQUEST,], },);
        memoizeSnapshotRequests({ client, },);
        const params = { snapshot: 1, project: 'p', location: 'node-a', };

        expect(client.apiRequest(LOCATION_REQUEST, params,),).toBeUndefined();
        expect(client.apiRequest(LOCATION_REQUEST, params,),).toBeUndefined();
        expect(reached.length,).toBe(1,);
      },
    },),
    it({
      name: 'sends a request that names no snapshot to the server every time',
      fn: async () => {
        const { client, reached, } = countingClient({},);
        memoizeSnapshotRequests({ client, },);
        client.apiRequest(LOCATION_REQUEST, { project: 'p', location: 'node-a', },);
        client.apiRequest(LOCATION_REQUEST, { project: 'p', location: 'node-a', },);
        client.apiRequest(LOCATION_REQUEST,);
        client.apiRequest(LOCATION_REQUEST,);

        expect(reached.length,).toBe(4,);
      },
    },),
  ],
},);
