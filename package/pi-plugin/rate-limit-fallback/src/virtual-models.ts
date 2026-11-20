/**
 Native Pi virtual models whose routers send every request to one of the package's pools.
 @module
 */

import {
  getSupportedThinkingLevels,
  type Api,
  type Model,
  type ModelThinkingLevel,
} from '@earendil-works/pi-ai';
import type {
  ExtensionVirtualModel,
  ModelRegistry,
  ModelRoute,
} from '@earendil-works/pi-coding-agent';
import {
  POOL_DEFINITIONS,
  poolEntryFor,
  PROVIDER_ID,
  type MemberLimitsReader,
  type PoolDefinition,
} from './model-pools.ts';

//region The route

/**
 Catalog lookup a router reads, the one registry operation it needs.
 */
type PoolCatalog = Readonly<Pick<ModelRegistry, 'find'>>;

/**
 One request's fields the router reads: Pi's chosen thinking level, not the transcript.
 */
type RouteView = Readonly<{ thinkingLevel: ModelThinkingLevel; }>;

/**
 What one routing call may read from the session.
 */
type RouteContextView = Readonly<{ modelRegistry: PoolCatalog; }>;

/**
 Router Pi calls before each request made with the virtual model.
 */
type PoolRouter = (
  request: RouteView,
  ctx: RouteContextView,
) => ModelRoute;

/**
 One virtual model as this package defines it, before Pi stores it in the catalog.

 Project-owned so the router's narrow reads are the definition's own signature, and Pi's fuller
 request and context types are assignable to them.
 */
export type PoolVirtualModel = Readonly<{
  provider: string;
  id: string;
  name: string;
  contextWindow: number;
  maxTokens: number;
  input: Model<Api>['input'];
  thinkingLevels: readonly ModelThinkingLevel[];
  route: PoolRouter;
}>;

/**
 Failure before a request can reach its pool.
 */
export class PoolCatalogError extends Error {
  /**
   Distinguishes a missing pool model from a failure inside a member.
   */
  override readonly name = 'PoolCatalogError';
}

/**
 Resolve the physical pool model Pi's catalog holds for one pool.

 @param registry - session registry the router reads

 @param poolId - physical model id of the pool

 @returns the catalog entry Pi dispatches through

 @throws PoolCatalogError when the catalog holds no such model

 @example
 ```ts
 poolModelFrom({ registry: ctx.modelRegistry, poolId: 'writer-pool' });
 ```
 */
function poolModelFrom({
  registry,
  poolId,
}: {
  readonly registry: PoolCatalog;
  readonly poolId: string;
},): Model<Api> {
  /**
   Registry object the lookup belongs to.
   */
  const owner = registry;
  /**
   Catalog lookup bound to its own registry: `ModelRegistry.find` takes two strings, and a bare
   `find(` call with two arguments reads like an array callback reference to the repository's
   array-method rules. `src/index.ts` binds the same lookup for the same reason.
   */
  const findModel = owner
    .find
    .bind(owner,);
  /**
   Catalog record Pi holds for the pool.
   */
  const found = findModel(
    PROVIDER_ID,
    poolId,
  );
  if (found === undefined)
    throw new PoolCatalogError(`Pi's catalog holds no ${PROVIDER_ID}/${poolId} model, so this request has no pool to route to.`,);
  return found;
}

/**
 Route every request, including retries and compaction summaries, to one pool.

 A retry after a failed member still starts at the pool's head, which is where the pool begins every
 request; the pool, not the router, decides which member answers.

 @param poolId - physical model id of the pool

 @returns router Pi calls before each request made with the virtual model

 @example
 ```ts
 const route = poolRoute({ poolId: 'writer-pool' });
 ```
 */
function poolRoute({ poolId, }: { readonly poolId: string; },): PoolRouter {
  return function route(
    request: RouteView,
    ctx: RouteContextView,
  ): ModelRoute {
    return {
      // The thinking level stays Pi's decision: Pi clamps it to the pool model, and the pool clamps it
      // again to the member that answers.
      model: poolModelFrom({
        registry: ctx.modelRegistry,
        poolId,
      },),
      thinkingLevel: request.thinkingLevel,
    };
  };
}

//endregion The route

//region Registration

/**
 Capabilities the virtual models need from Pi's extension API.
 */
export type VirtualModelsHost = {
  /**
   Register one selectable virtual model, replacing one of the same provider and id.
   */
  registerVirtualModel: (definition: ExtensionVirtualModel) => void;
};

/**
 Definitions of the virtual models Pi lists for this package.

 Each pool appears twice in Pi's catalog: as a physical pool model the router dispatches through, and
 as the public virtual model a session selects. The declared window and output limit are the
 smallest every registered member of that pool reports, so Pi's compaction threshold fits them all;
 Pi applies the answering member's own limits from the next request on.

 @param readMemberLimits - registry reader, or undefined before a session binds

 @returns writer and reviewer virtual model definitions

 @example
 ```ts
 virtualModelDefinitions({ readMemberLimits });
 ```
 */
export function virtualModelDefinitions({
  readMemberLimits,
}: {
  readonly readMemberLimits?: MemberLimitsReader;
} = {},): readonly PoolVirtualModel[] {
  return POOL_DEFINITIONS.map(function definition(pool: PoolDefinition,): PoolVirtualModel {
    /**
     The pool model this virtual model routes to, whose limits and thinking levels it mirrors.
     */
    const poolModel = poolEntryFor({
      pool,
      ...(readMemberLimits === undefined ? {} : { readMemberLimits, }),
    },);
    return {
      provider: PROVIDER_ID,
      id: pool.name,
      name: poolModel.name,
      contextWindow: poolModel.contextWindow,
      maxTokens: poolModel.maxTokens,
      input: poolModel.input,
      // Pi reads a virtual model's levels from its own list, so the list is the pool's, derived
      // rather than restated.
      thinkingLevels: getSupportedThinkingLevels(poolModel,),
      route: poolRoute({ poolId: pool.modelId, },),
    };
  },);
}

/**
 Register the package's virtual models with Pi.

 @param host - Pi's extension API for this session

 @param readMemberLimits - registry reader, or undefined before a session binds

 @example
 ```ts
 registerVirtualModels({ host: pi, readMemberLimits: memberLimitsReader({ registry }) });
 ```
 */
export function registerVirtualModels({
  host,
  readMemberLimits,
}: {
  readonly host: VirtualModelsHost;
  readonly readMemberLimits?: MemberLimitsReader;
},): void {
  /**
   Definitions this registration asks Pi to list, with the reader only when one is supplied.
   */
  const definitions = readMemberLimits === undefined
    ? virtualModelDefinitions()
    : virtualModelDefinitions({ readMemberLimits, });
  for (const definition of definitions)
    host.registerVirtualModel(definition,);
}

//endregion Registration
