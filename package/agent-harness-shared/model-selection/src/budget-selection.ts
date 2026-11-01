/**
 Fast judge-model strategy selection with injected auth callbacks.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  NoBudgetModelError,
  toBudgetModelCandidate,
} from './budget-report.ts';
import {
  compareModelSpeed,
  findFastestInMajorVersions,
} from './speed-ranking.ts';
import {
  type BudgetModel,
  type BudgetModelCandidate,
  type BudgetModelSelectionOptions,
  type ModelPricing,
  NO_AUTH,
} from './types.ts';

/**
 Sentinel returned by {@link findFastestCandidate} when no provider yields a
 candidate (empty registry). A `unique symbol`; narrowed with
 `=== NO_CANDIDATE`. Exported because {@link findFastestCandidate} is public.
 */
export const NO_CANDIDATE: unique symbol = Symbol('model selection fast candidate absent after version filtering',);

//region Internal types

/**
 Options shared by concrete budget-model strategy helpers.
 */
type BudgetStrategyOptions<TModel extends ModelPricing,> = Omit<
  BudgetModelSelectionOptions<TModel>,
  'strategy'
  >;

//endregion Internal types

//region Public API

/**
 Find the fastest authenticated judge model for configured strategy.
 
 @param options - active model, model list, strategy, major versions, and auth callbacks
 
 @returns selected budget model with auth
 
 @mutates options - strategy helpers invoke supplied `resolveAuth` and `hasConfiguredAuth` callbacks
 
 @throws {@link NoBudgetModelError} when no suitable model is found
 
 @example
 ```typescript
 const budget = await selectBudgetModel({
   activeModel,
   allModels,
   strategy,
   majorVersions,
   resolveAuth,
   hasConfiguredAuth,
 });
 ```
 */
export async function selectBudgetModel<TModel extends ModelPricing,>(
  options: BudgetModelSelectionOptions<TModel>,
): Promise<BudgetModel<TModel>> {
  if (options.strategy
    === 'any-provider') {
    return await findAnyProvider({
      activeModel: options.activeModel,
      allModels: options.allModels,
      majorVersions: options.majorVersions,
      resolveAuth: options.resolveAuth,
      hasConfiguredAuth: options.hasConfiguredAuth,
    },);
  }
  return await findSameProvider({
    activeModel: options.activeModel,
    allModels: options.allModels,
    majorVersions: options.majorVersions,
    resolveAuth: options.resolveAuth,
    hasConfiguredAuth: options.hasConfiguredAuth,
  },);
}

/**
 Find the single fastest model across all providers for error context.
 
 @param allModels - registry model list
 
 @param majorVersions - major-version families to search
 
 @param hasConfiguredAuth - host auth predicate for reporting
 
 @returns fastest candidate when present
 
 @mutates hasConfiguredAuth - invokes supplied auth-availability predicate for selected model
 
 @example
 ```typescript
 const candidate = findFastestCandidate({ allModels, majorVersions, hasConfiguredAuth });
 ```
 */
export function findFastestCandidate<TModel extends ModelPricing,>(
  {
    allModels,
    majorVersions,
    hasConfiguredAuth,
  }: ForeignBorrowed<Readonly<{
    allModels: readonly TModel[];
    majorVersions: number;
    hasConfiguredAuth: (options: { readonly model: TModel; }) => boolean;
  }>>,
): BudgetModelCandidate | typeof NO_CANDIDATE {
  /**
   Provider name to its list of models.
   */
  const byProvider = groupModelsByProvider(allModels,);

  /**
   Fastest per-provider head for every provider that yielded a candidate.
   */
  const providerHeads: {
    readonly model: TModel;
    readonly provider: string;
  }[] = [];
  for (const [provider, models,] of byProvider) {
    /**
     Per-provider candidates already sorted by speed, cost, then version.
     */
    const firstCandidate = findFastestInMajorVersions({
      models,
      majorVersions,
    },)
      .at(0,);
    if (firstCandidate !== undefined) {
      providerHeads.push({
        model: firstCandidate,
        provider,
      },);
    }
  }

  /**
   Overall fastest provider head by speed heuristic.
   */
  const best = providerHeads.toSorted(function bySpeed(
    left,
    right,
  ) {
    return compareModelSpeed({
      left: left.model,
      right: right.model,
    },);
  },)
    .at(0,);

  if (best === undefined)
    return NO_CANDIDATE;
  return toBudgetModelCandidate({
    model: best.model,
    hasConfiguredAuth: hasConfiguredAuth({ model: best.model, },),
  },);
}

/**
 Backwards-compatible name for {@link findFastestCandidate}.
 
 @deprecated Use {@link findFastestCandidate}; automatic judge selection now
 ranks by speed heuristic before cost.
 
 @example
 ```typescript
 findCheapestCandidate({ allModels, majorVersions: 1, hasConfiguredAuth });
 ```
 */
export const findCheapestCandidate: typeof findFastestCandidate = findFastestCandidate;

//endregion Public API

//region Same-provider strategy

/**
 Find fastest model in the same provider as active model.
 
 @param activeModel - active model used for provider reference
 
 @param allModels - registry model list
 
 @param majorVersions - major-version families to search
 
 @param resolveAuth - host auth resolver
 
 @param hasConfiguredAuth - host auth predicate for reporting
 
 @returns selected budget model
 
 @mutates resolveAuth - invokes supplied async auth resolver for ranked candidates
 
 @mutates hasConfiguredAuth - invokes supplied auth-availability predicate for error context
 
 @throws {@link NoBudgetModelError} when no suitable same-provider model is found
 */
async function findSameProvider<TModel extends ModelPricing,>(
  {
    activeModel,
    allModels,
    majorVersions,
    resolveAuth,
    hasConfiguredAuth,
  }: BudgetStrategyOptions<TModel>,
): Promise<BudgetModel<TModel>> {
  /**
   Active provider name used to filter registry.
   */
  const activeProvider = activeModel.provider;
  /**
   Subset of all models sharing active provider.
   */
  const providerModels = allModels.filter(function sameProvider(model,) {
    return model.provider
      === activeProvider;
  },);

  /**
   Lazily find fastest candidate across all providers.
   
   @returns error-context object carrying fastestOverall only when found
   */
  function fastestOverallContext(): {
    readonly fastestOverall?: BudgetModelCandidate;
  } {
    /**
     Fastest cross-provider candidate result.
     */
    const candidate = findFastestCandidate({
      allModels,
      majorVersions,
      hasConfiguredAuth,
    },);
    return candidate === NO_CANDIDATE
      ? {}
      : { fastestOverall: candidate, };
  }

  if (providerModels.length
    === 0) {
    throw new NoBudgetModelError(
      `no models found for provider "${activeProvider}"`,
      fastestOverallContext(),
    );
  }

  /**
   Same-provider candidates ranked by speed, cost, then version.
   */
  const candidates = findFastestInMajorVersions({
    models: providerModels,
    majorVersions,
  },);

  if (candidates.length
    === 0) {
    throw new NoBudgetModelError(
      `no versioned models found for provider "${activeProvider}"`,
      fastestOverallContext(),
    );
  }

  /**
   Fastest same-provider candidate.
   */
  const fastestCandidate = candidates.at(0,);
  if (fastestCandidate === undefined) {
    throw new NoBudgetModelError(
      `no candidates available for provider "${activeProvider}"`,
    );
  }

  for (const candidate of candidates) {
    /* oxlint-disable no-await-in-loop -- sequential auth walk must stop at first successful candidate. */
    /**
     Resolved auth for current candidate.
     */
    const auth = await resolveAuth({ model: candidate, },);
    /* oxlint-enable no-await-in-loop */
    if (auth !== NO_AUTH) {
      return {
        model: candidate,
        auth,
      };
    }
  }

  /**
   Same-provider report row after every candidate failed auth.
   */
  const sameProvider = toBudgetModelCandidate({
    model: fastestCandidate,
    hasConfiguredAuth: hasConfiguredAuth({ model: fastestCandidate, },),
  },);
  throw new NoBudgetModelError(
    `no API key available for fastest models in provider "${activeProvider}"`,
    {
      sameProvider,
      ...fastestOverallContext(),
    },
  );
}

//endregion Same-provider strategy

//region Any-provider strategy

/**
 Find fastest model across all providers.
 
 @param allModels - registry model list
 
 @param majorVersions - major-version families to search
 
 @param resolveAuth - host auth resolver
 
 @returns selected budget model
 
 @mutates resolveAuth - invokes supplied async auth resolver for ranked candidates
 
 @throws {@link NoBudgetModelError} when no suitable model is found across any provider
 */
async function findAnyProvider<TModel extends ModelPricing,>(
  {
    allModels,
    majorVersions,
    resolveAuth,
  }: BudgetStrategyOptions<TModel>,
): Promise<BudgetModel<TModel>> {
  /**
   Provider name to its list of models.
   */
  const byProvider = groupModelsByProvider(allModels,);

  /**
   Flat union of every provider's top candidates.
   */
  const allCandidates: TModel[] = [];
  for (const [, models,] of byProvider) {
    allCandidates.push(...findFastestInMajorVersions({
      models,
      majorVersions,
    },),);
  }
  /**
   Cross-provider candidates sorted by speed heuristic.
   */
  const sortedCandidates = allCandidates.toSorted(function bySpeed(
    left,
    right,
  ) {
    return compareModelSpeed({
      left,
      right,
    },);
  },);

  for (const model of sortedCandidates) {
    /* oxlint-disable no-await-in-loop -- sequential auth walk must stop at first successful candidate. */
    /**
     Resolved auth for current candidate.
     */
    const auth = await resolveAuth({ model, },);
    /* oxlint-enable no-await-in-loop */
    if (auth !== NO_AUTH) {
      return {
        model,
        auth,
      };
    }
  }

  throw new NoBudgetModelError(
    'no fast judge models with API keys found across any provider',
  );
}

//endregion Any-provider strategy

//region Helpers

/**
 Group models by provider.
 
 @param models - models to group
 
 @returns provider to model list map
 */
function groupModelsByProvider<TModel extends ModelPricing,>(
  models: readonly TModel[],
): Map<string, TModel[]> {
  /**
   Provider name to models.
   */
  const byProvider = new Map<string, TModel[]>();
  for (const model of models) {
    /**
     Provider name keying grouping map.
     */
    const { provider, } = model;
    if (!byProvider.has(provider,)) {
      byProvider.set(
        provider,
        [],
      );
    }
    /**
     Bucket current model goes into.
     */
    const list = byProvider.get(provider,);
    if (list !== undefined)
      list.push(model,);
  }
  return byProvider;
}

export { budgetModelSlug, } from './budget-report.ts';

//endregion Helpers
