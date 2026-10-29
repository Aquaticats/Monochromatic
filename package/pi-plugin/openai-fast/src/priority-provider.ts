/**
 Keyless priority targets leave each original OpenAI provider untouched. @module
 */
import type {
  Api,
  ApiStreamOptions,
  Model,
  StreamOptions,
  Provider,
  StreamFunction,
  TranscriptContext,
} from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  createKeylessAuth,
  noAvailableTargets,
} from './keyless-auth.ts';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { PriorityApi, } from './constants.ts';
import type { OriginalModelLookup, } from './original-dispatch-types.ts';
import {
  isPriorityTarget,
  priorityTarget,
  resolvePriorityBase,
} from './priority-target.ts';
import {
  streamPriority,
  streamSimplePriority,
} from './priority-stream.ts';

export {
  isPriorityTarget,
  priorityTarget,
  resolvePriorityBase,
} from './priority-target.ts';

/**
 Module logger excludes request content and authentication.
 */
const moduleLogger = tagged({ tag: 'pi-plugin-openai-fast.priority-provider', },);

//region Adapter provider

/**
 Source metadata and the original runtime dispatch boundary.
 */
export type PriorityProviderOptions = {
  readonly provider: ForeignHostCapability<Provider>;
  readonly getProvider?: () => ForeignHostCapability<Provider>;
  readonly lookup: OriginalModelLookup;
  readonly isConfigured: (signal: ForeignBorrowed<AbortSignal>) => Promise<boolean>;
  readonly dispatch: StreamFunction<PriorityApi>;
  readonly onCatalog: (models: readonly Model<Api>[]) => void;
};

/**
 Add keyless physical targets without registering over the original provider.
 
 @param provider - bootstrap metadata source, never an authentication source
 
 @param getProvider - live original-provider lookup after session initialization
 
 @param lookup - current upstream model lookup

 @param isConfigured - fresh native availability check for the original provider
 
 @param dispatch - original registry dispatch for native auth and header precedence
 
 @param onCatalog - guarded companion registration when catalog reads expose changes
 
 @returns adapter whose targets are excluded from normal availability lists
 
 @mutates provider - invokes metadata accessors and explicitly delegated ordinary streams
 
 @mutates getProvider - invokes live catalog capability
 
 @mutates lookup - priority dispatch invokes original-model lookup

 @mutates isConfigured - adapter readiness checks invoke original-provider availability
 
 @mutates dispatch - priority dispatch invokes native stream capability
 
 @mutates onCatalog - catalog reads invoke virtual registration

 @example
 ```ts
 const adapter = createPriorityProvider({ provider, lookup, isConfigured, dispatch, onCatalog });
 ```
 */
export function createPriorityProvider({
  provider,
  getProvider,
  lookup,
  isConfigured,
  dispatch,
  onCatalog,
}: PriorityProviderOptions,): Provider {
  /**
   Adapter logger retains the module boundary through catalog callbacks.
   */
  const l = tagged({
    tag: createPriorityProvider.name,
    l: moduleLogger,
  },);

  /**
   Read only the original provider, never recursively enumerate this adapter.

   @returns current physical original models
   */
  function originalModels(): readonly Model<Api>[] {
    /**
     Catalog-read logger never receives model headers or authentication.
     */
    const ml = tagged({
      tag: originalModels.name,
      l,
    },);
    /**
     Source lookup targets only the unchanged original provider.
     */
    const source = getProvider?.() ?? provider;
    /**
     Virtual routers cannot be routed to as physical bases.
     */
    const models = source.getModels()
      .filter(function physicalModel(model: ForeignBorrowed<Model<Api>>,) {
        return model.api !== 'pi-virtual';
      },);
    onCatalog(models,);
    ml.trace(`read ${models.length} original ${provider.id} models`,);
    return models;
  }

  /**
   Derive targets from current original metadata without stale snapshots.

   @returns current local priority targets
   */
  function getModels(): readonly Model<Api>[] {
    return originalModels()
      .map(function createTarget(model: ForeignBorrowed<Model<Api>>,) {
        return priorityTarget(model,);
      },);
  }

  l.debug('creating keyless priority adapter',);
  return {
    id: `${provider.id}-fast`,
    name: `${provider.name} Fast`,
    auth: createKeylessAuth(isConfigured,),
    getModels,
    getAllModels: getModels,
    filterModels: noAvailableTargets,
    filterAllModels: noAvailableTargets,
    stream: function stream<TApi extends Api>(
      model: Model<TApi>,
      context: TranscriptContext,
      options?: ApiStreamOptions<TApi>,
    ) {
      if (!isPriorityTarget(model,))
        return provider.stream(
          model,
          context,
          options,
        );
      return streamPriority({
        model: resolvePriorityBase({
          model,
          lookup,
        },),
        context,
        ...(options === undefined ? {} : { options, }),
        stream: dispatch,
      },);
    },
    streamSimple: function streamSimple(
      model,
      context,
      options,
    ) {
      if (!isPriorityTarget(model,))
        return provider.streamSimple(
          model,
          context,
          options,
        );
      return streamSimplePriority({
        model: resolvePriorityBase({
          model,
          lookup,
        },),
        context,
        ...(options === undefined ? {} : { options, }),
        stream: dispatch,
      },);
    },
    refreshModels: async function refreshModels(context,) {
      // Yield before catalog synchronization to avoid synchronous refresh re-entry.
      await Promise.resolve();
      /**
       Refresh logger records companion synchronization without another provider's I/O.
       */
      const rl = tagged({
        tag: refreshModels.name,
        l,
      },);
      if (context.signal
        .aborted)
        return;
      originalModels();
      rl.debug('synchronized companions without refreshing another provider under the adapter identity',);
    },
  };
}

//endregion
