/**
 The dispatch loop's pool table: an id the table carries, an id it lacks, and a key only its
 prototype chain carries.
 @module
 */

import type {
  Api,
  AssistantMessageEvent,
  Model,
} from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  dispatchRequest,
  PoolDispatchError,
  type Member,
  type RoutingSession,
} from '../dist/final/node/index.mjs';
import {
  immediate,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

//region Fixtures for a caller-supplied pool table

/**
 Provider id of the table's own member; no real provider answers it.
 */
const CUSTOM_PROVIDER = 'dispatch-table-probe';

/**
 Pool id the caller-supplied table carries, which no package pool has.
 */
const CUSTOM_POOL_ID = 'custom-probe';

/**
 Member the caller-supplied table names.
 */
const CUSTOM_MEMBER: Member = {
  provider: CUSTOM_PROVIDER,
  id: 'answers',
};

/**
 Catalog record this suite's registry resolves for {@link CUSTOM_MEMBER}.
 */
const CUSTOM_MODEL: Model<Api> = {
  ...CUSTOM_MEMBER,
  name: CUSTOM_MEMBER.id,
  api: CUSTOM_PROVIDER,
  baseUrl: '',
  input: ['text',],
  reasoning: false,
  cost: {
    input: 0,
    output: 0,
    cacheRead: 0,
    cacheWrite: 0,
  },
  contextWindow: 16_000,
  maxTokens: 256,
};

/**
 Object prototype keys a pool table inherits, which are not pools.
 */
const INHERITED_KEYS = [
  'constructor',
  'toString',
  '__proto__',
] as const;

/**
 Selected pool model, carrying only the id the dispatch loop reads.
 */
function poolSelection(id: string,): Model<Api> {
  return {
    ...CUSTOM_MODEL,
    id,
  };
}

/**
 Build a session whose registry resolves one member out of its catalog and streams its answer.

 @param models - catalog records the registry resolves

 @param calls - physical models the registry was asked to stream, in order

 @returns session the dispatch loop runs against

 @example
 ```ts
 const session = sessionOver({ models: [CUSTOM_MODEL], calls: [] });
 ```
 */
function sessionOver({
  models,
  calls,
}: {
  readonly models: readonly Model<Api>[];
  readonly calls: Model<Api>[];
},): RoutingSession {
  return {
    registry: {
      find: function findInCatalog(
        provider,
        id,
      ) {
        for (const model of models) {
          if ((model.provider === provider) && (model.id === id))
            return model;
        }
        return undefined;
      },
      getApiKeyAndHeaders: function keylessMember() {
        return Promise.resolve({
          ok: true,
          apiKey: 'dispatch-table-key',
        },);
      },
      streamSimple: function streamMember(model,) {
        calls.push(model,);
        return immediate({ model, },);
      },
    },
    members: new Map(),
  };
}

/**
 Collect the events one request produces through the dispatch loop.

 @param pools - pool table the request dispatches against

 @param id - pool id the request selects

 @param calls - physical models the session's registry streamed, in order

 @returns every event the request exposed

 @example
 ```ts
 const events = await routedEvents({ pools, id: CUSTOM_POOL_ID, calls: [] });
 ```
 */
async function routedEvents({
  pools,
  id,
  calls,
}: {
  readonly pools: Readonly<Record<string, readonly Member[]>>;
  readonly id: string;
  readonly calls: Model<Api>[];
},): Promise<AssistantMessageEvent[]> {
  const events: AssistantMessageEvent[] = [];
  const session = sessionOver({
    models: [CUSTOM_MODEL,],
    calls,
  },);
  for await (const event of dispatchRequest({
    session,
    model: poolSelection(id,),
    context: TRANSCRIPT,
    pools,
  },))
    events.push(event,);
  return events;
}

/**
 Error the pool raised for one unroutable id.

 @param pools - pool table the request dispatches against

 @param id - pool id the request selects

 @returns the thrown value, once the request ends

 @example
 ```ts
 const error = await routingFailure({ pools: {}, id: 'missing' });
 ```
 */
async function routingFailure({
  pools,
  id,
}: {
  readonly pools: Readonly<Record<string, readonly Member[]>>;
  readonly id: string;
},): Promise<unknown> {
  try {
    await routedEvents({
      pools,
      id,
      calls: [],
    },);
  } catch (error) {
    return error;
  }
  return undefined;
}

/**
 Narrow one unroutable id's failure to the error the pool documents.

 @param error - value the request threw

 @returns the pool's own error, or a failure naming what was thrown instead

 @example
 ```ts
 const failure = poolFailure({ error });
 ```
 */
function poolFailure({ error, }: { readonly error: unknown; },): PoolDispatchError {
  if (error instanceof PoolDispatchError)
    return error;
  throw new Error(`Expected PoolDispatchError, got ${String(error)}`,);
}

//endregion Fixtures for a caller-supplied pool table

//region Cases

/**
 One case per pool id the table does not carry, computed so each key is named once.
 */
const INHERITED_CASES = INHERITED_KEYS.map(function inheritedKey(key,) {
  return it({
    name: `refuses the inherited key ${key} without reading it as a pool`,
    fn: async function inheritedKeyIsNotAPool() {
      const error = await routingFailure({
        pools: {},
        id: key,
      },);
      expect(poolFailure({ error, },).message,).toContain(`Unknown rate-limit-fallback model: ${key}`,);
    },
  },);
},);

await describe({
  name: 'dispatch request pool tables',
  children: [
    it({
      name: 'routes to the member a caller-supplied table carries',
      fn: async function customTableRoutes() {
        /**
         Physical models the session's registry streamed, in order.
         */
        const calls: Model<Api>[] = [];
        const events = await routedEvents({
          pools: { [CUSTOM_POOL_ID]: [CUSTOM_MEMBER,], },
          id: CUSTOM_POOL_ID,
          calls,
        },);
        expect(calls,).toHaveLength(1,);
        expect(calls[0]
          ?.id,).toBe(CUSTOM_MEMBER.id,);
        expect(events.at(-1,)
          ?.type,).toBe('done',);
      },
    },),
    it({
      name: 'refuses a pool id no table carries',
      fn: async function unknownPoolId() {
        const error = await routingFailure({
          pools: { [CUSTOM_POOL_ID]: [CUSTOM_MEMBER,], },
          id: 'missing-pool',
        },);
        expect(poolFailure({ error, },).message,).toContain('Unknown rate-limit-fallback model: missing-pool',);
      },
    },),
    ...INHERITED_CASES,
  ],
},);
