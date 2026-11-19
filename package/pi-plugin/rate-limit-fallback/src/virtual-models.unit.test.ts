/**
 The shipped route: native Pi virtual models whose routers send every request to a pool.
 @module
 */

import {
  getSupportedThinkingLevels,
  type Api,
  type Model,
  type ModelThinkingLevel,
} from '@earendil-works/pi-ai';
import type { ExtensionVirtualModel, } from '@earendil-works/pi-coding-agent';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  PoolCatalogError,
  registerVirtualModels,
  virtualModelDefinitions,
  type PoolVirtualModel,
} from '../dist/final/node/index.mjs';
import { PHYSICAL_MODELS, } from './stand-in-pi-fixture.ts';
import { WRITER_TARGETS, } from './member-order-fixture.ts';

/**
 Public model id of the writer pool's virtual model, the name Pi's selection carries.
 */
const WRITER_VIRTUAL_ID = 'writer';

/**
 Public model id of the reviewer pool's virtual model.
 */
const REVIEWER_VIRTUAL_ID = 'reviewer';

/**
 Physical model id of the writer pool inside the package; the router's target.
 */
const WRITER_POOL_ID = 'writer-pool';

/**
 Thinking level a request asks for, whose meaning belongs to Pi and the router.
 */
const REQUESTED_LEVEL: ModelThinkingLevel = 'high';

/**
 Window a stand-in member reports for the limit cases.
 */
const MEMBER_WINDOW = 40_000;

/**
 Output limit a stand-in member reports for the limit cases.
 */
const MEMBER_OUTPUT = 8_000;

/**
 Window Pi itself falls back to for a model that declares none
 (`@earendil-works/pi-coding-agent` dist/core/provider-composer.js, `modelFromJson`).
 */
const PROVISIONAL_WINDOW = 128_000;

/**
 Output limit Pi falls back to beside that window.
 */
const PROVISIONAL_OUTPUT = 16_384;

/**
 First stand-in catalog record, which the capability cases mirror.

 @returns the record, or a failure naming the fixture that lost it
 */
function firstMember(): Model<Api> {
  /**
   Record the stand-in catalog opens with.
   */
  const [first,] = PHYSICAL_MODELS;
  if (first === undefined)
    throw new Error('Expected a stand-in catalog record',);
  return first;
}

/**
 Registered definitions one registration call produced, keyed by model id.

 @param definitions - definitions the package asked Pi to register

 @returns definitions keyed by id, for a case that reads more than one
 */
function byId(registered: readonly PoolVirtualModel[],): Map<string, PoolVirtualModel> {
  return new Map(registered.map(definition => [definition.id, definition,],),);
}

/**
 Virtual model definitions the package registers, with no member limits reader.

 @returns definitions keyed by model id
 */
function definitions(): Map<string, PoolVirtualModel> {
  return byId(virtualModelDefinitions(),);
}

/**
 Catalog lookup over one list of models, as Pi's registry exposes it.

 @param models - catalog records the lookup resolves

 @returns resolver the router may call
 */
function registryOver(models: readonly Model<Api>[],) {
  return {
    find(provider: string, id: string,) {
      return models.find(model => (model.provider === provider) && (model.id === id),);
    },
  };
}

/**
 The error one router raised instead of routing, narrowed to an `Error`.

 @param writer - virtual model whose router is asked

 @param models - catalog records the router may resolve

 @returns the failure the router raised

 @throws Error when the router routed instead of refusing
 */
function routeFailure({
  writer,
  models,
}: {
  readonly writer: PoolVirtualModel;
  readonly models: readonly Model<Api>[];
},): Error {
  try {
    writer.route({ thinkingLevel: REQUESTED_LEVEL, }, { modelRegistry: registryOver(models,), },);
  }
  catch (error) {
    if (Error.isError(error,))
      return error;
    throw error;
  }
  throw new Error('The router routed although the catalog holds no pool model',);
}

await describe({
  name: 'virtual models',
  children: [
    it({
      name: 'registers one virtual model per pool under the ids Pi selections use',
      fn: async function registeredIds() {
        const registered: ExtensionVirtualModel[] = [];
        registerVirtualModels({
          host: {
            registerVirtualModel(definition,) { registered.push(definition,); },
          },
        },);
        expect(registered.map(definition => definition.id,),).toEqual([WRITER_VIRTUAL_ID, REVIEWER_VIRTUAL_ID,],);
        expect(registered.map(definition => definition.provider,),).toEqual(['rate-limit-fallback', 'rate-limit-fallback',],);
      },
    },),
    it({
      name: 'offers the thinking levels its pool model supports and text input alone',
      fn: async function declaredCapabilities() {
        const writer = definitions().get(WRITER_VIRTUAL_ID,);
        if (writer === undefined)
          throw new Error('The package registered no writer virtual model',);
        // The pool entry declares reasoning, so Pi's own rule for a model with no thinking map
        // applies to it: every level through high.
        expect([...writer.thinkingLevels,],).toEqual(getSupportedThinkingLevels({
          ...firstMember(),
          reasoning: true,
        },),);
        expect(writer.thinkingLevels,).toContain(REQUESTED_LEVEL,);
        expect(writer.input,).toEqual(['text',],);
      },
    },),
    it({
      name: 'routes every request to the pool model Pi’s catalog holds',
      fn: async function routesToPool() {
        const writer = definitions().get(WRITER_VIRTUAL_ID,);
        if (writer === undefined)
          throw new Error('The package registered no writer virtual model',);
        const pool = { ...firstMember(), provider: 'rate-limit-fallback', id: WRITER_POOL_ID, };
        const route = writer.route({ thinkingLevel: REQUESTED_LEVEL, }, { modelRegistry: registryOver([pool,],), },);
        expect(route.model,).toBe(pool,);
        expect(route.thinkingLevel,).toBe(REQUESTED_LEVEL,);
      },
    },),
    it({
      name: 'refuses to route when Pi’s catalog holds no pool model',
      fn: async function missingPool() {
        const writer = definitions().get(WRITER_VIRTUAL_ID,);
        if (writer === undefined)
          throw new Error('The package registered no writer virtual model',);
        /**
         The failure the router raised instead of routing.
         */
        const caught = routeFailure({ writer, models: [], },);
        expect(caught,).toBeInstanceOf(PoolCatalogError,);
        expect(caught.message,).toContain('rate-limit-fallback/writer-pool',);
      },
    },),
    it({
      name: 'declares the smallest limits its pool’s registered members report',
      fn: async function declaredLimits() {
        const withLimits = byId(virtualModelDefinitions({
          readMemberLimits(provider, id,) {
            if ((provider !== WRITER_TARGETS[1].provider) || (id !== WRITER_TARGETS[1].id))
              return { contextWindow: 0, maxTokens: 0, };
            return { contextWindow: MEMBER_WINDOW, maxTokens: MEMBER_OUTPUT, };
          },
        },),);
        const writer = withLimits.get(WRITER_VIRTUAL_ID,);
        expect(writer?.contextWindow,).toBe(MEMBER_WINDOW,);
        expect(writer?.maxTokens,).toBe(MEMBER_OUTPUT,);
      },
    },),
    it({
      name: 'declares Pi’s own fallback limits before any member is registered',
      fn: async function provisionalLimits() {
        const writer = definitions().get(WRITER_VIRTUAL_ID,);
        expect(writer?.contextWindow,).toBe(PROVISIONAL_WINDOW,);
        expect(writer?.maxTokens,).toBe(PROVISIONAL_OUTPUT,);
      },
    },),
  ],
},);
