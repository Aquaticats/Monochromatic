/**
 Translation from a live registry read into per-provider filtering plans.

 Two mechanisms are available and the planner picks per provider. A provider another
 extension registered through a configuration is re-registered as a configuration, because
 pi merges defined values over the previous registration and so preserves that owner's
 `apiKey`, `oauth`, and `streamSimple`. Any other provider, meaning a native registration
 or a builtin, is wrapped instead: its composed `Provider` object is spread and only its
 model listing methods are replaced, which needs no endpoint metadata and loses no model
 field.

 `ModelRegistry.getAll()` reports chat models only, which is why image and classifier
 models are read separately and passed through untouched on the configuration path.

 @module
 */

import type {
  ProviderConfig,
  ProviderModelConfig,
} from '@earendil-works/pi-coding-agent';
import type {
  Api,
  ClassifierApi,
  ClassifierModel,
  ImageApi,
  ImageModel,
  Model,
  Provider,
} from '@earendil-works/pi-ai';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  buildRetiredIndex,
  isRetired,
} from './retired-index.ts';
import {
  planGap,
  REGISTERABLE,
  toModelConfig,
} from './registration-gap.ts';
import { wrapProvider, } from './provider-wrapper.ts';
import {
  decideRetirements,
  type AbstentionCounts,
  type CatalogEntry,
  type Retirement,
} from './retirement-rule.ts';

//region Types

/**
 Incumbent extension configuration for one provider.

 Modelled as a discriminant rather than an optional value because pi returns `undefined`
 both for a builtin provider and for a provider another extension registered as a native
 object, and the planner needs to tell "has a configuration" from "has none".
 */
export type IncumbentConfig = {
  /**
   Whether pi returned a configuration for this provider.
   */
  readonly kind: 'present' | 'absent';
  /**
   Configuration pi returned, present only when the kind is `present`.
   */
  readonly config?: ForeignBorrowed<ProviderConfig>;
};

/**
 Composed provider object for one provider.
 */
export type ComposedProvider = {
  /**
   Whether the registry returned a composed provider.
   */
  readonly kind: 'present' | 'absent';
  /**
   Composed provider, present only when the kind is `present`.
   */
  readonly provider?: ForeignBorrowed<Provider>;
};

/**
 Registry reads this extension needs, injected so tests can fake them.

 Every model is marked {@link ForeignBorrowed} because model objects are owned by pi's
 registry: this package reads them and spreads them into configurations, and never mutates
 them.
 */
export type CatalogRead = {
  /**
   Chat models the registry knows, in registry order.
   */
  readonly readChatModels: () => readonly ForeignBorrowed<Model<Api>>[];
  /**
   Image models one provider serves.
   */
  readonly readImageModels: (
    provider: string,
  ) => readonly ForeignBorrowed<ImageModel<ImageApi>>[];
  /**
   Classifier models one provider serves.
   */
  readonly readClassifierModels: (
    provider: string,
  ) => readonly ForeignBorrowed<ClassifierModel<ClassifierApi>>[];
  /**
   Incumbent configuration one extension registered for a provider.
   */
  readonly readProviderConfig: (provider: string,) => IncumbentConfig;
  /**
   Composed provider object the registry holds for a provider.
   */
  readonly readComposedProvider: (provider: string,) => ComposedProvider;
};

/**
 Plan that re-registers a provider through its configuration.
 */
export type ConfigPlan = {
  /**
   Mechanism discriminator.
   */
  readonly kind: 'config';
  /**
   Provider whose configuration pi will merge this over.
   */
  readonly provider: string;
  /**
   Configuration to register: the incumbent's, with its model list replaced.
   */
  readonly config: ProviderConfig;
  /**
   Retirements that made this plan necessary.
   */
  readonly retirements: readonly Retirement[];
};

/**
 Plan that re-registers a wrapped provider object.
 */
export type WrapperPlan = {
  /**
   Mechanism discriminator.
   */
  readonly kind: 'wrapper';
  /**
   Provider whose composed object was wrapped.
   */
  readonly provider: string;
  /**
   Wrapped provider, which lists fewer models and otherwise behaves like the original.
   */
  readonly wrapped: Provider;
  /**
   Retirements that made this plan necessary.
   */
  readonly retirements: readonly Retirement[];
};

/**
 One provider's filtering plan.
 */
export type ProviderFilterPlan = ConfigPlan | WrapperPlan;

/**
 Outcome of planning across a whole catalog.
 */
export type FilterPlanning = {
  /**
   Plans for providers that lose at least one model.
   */
  readonly plans: readonly ProviderFilterPlan[];
  /**
   Every retirement the rule decided, including providers with no plan.
   */
  readonly retirements: readonly Retirement[];
  /**
   Declined retirements grouped by reason, so a log can say what the rule saw and
   refused to order.
   */
  readonly abstentions: AbstentionCounts;
  /**
   Providers the rule retired models for but that cannot be filtered at all, so their
   models stay visible.
   */
  readonly skippedProviders: readonly SkippedProvider[];
};

/**
 One provider this pass could not filter.
 */
export type SkippedProvider = {
  /**
   Provider left untouched.
   */
  readonly provider: string;
  /**
   What stopped the filtering, naming the offending model or the missing surface.
   */
  readonly reason: string;
};

//endregion Types

//region Planning

/**
 Build the configuration plan for one provider, when its models can all be re-declared.

 @param provider - provider to plan for

 @param chatModels - chat models the registry reports, across every provider

 @param read - injected registry reads

 @param base - incumbent configuration to merge over

 @param index - retired identities for this pass

 @returns a configuration plan, or text explaining why the provider cannot be re-declared

 @example
 ```typescript
 planConfig({ provider, chatModels, read, base, index });
 ```
 */
function planConfig(
  {
    provider,
    chatModels,
    read,
    base,
    index,
  }: {
    readonly provider: string;
    readonly chatModels: readonly ForeignBorrowed<Model<Api>>[];
    readonly read: CatalogRead;
    readonly base: ProviderConfig;
    readonly index: ReadonlyMap<string, ReadonlySet<string>>;
  },
): { readonly config: ProviderConfig; } | { readonly gap: string; } {
  /**
   Chat models this provider still serves.
   */
  const keptChat = chatModels.filter(function isKept(model,) {
    return (model.provider === provider)
      && (!isRetired({
        index,
        provider: model.provider,
        api: model.api,
        modelId: model.id,
      },));
  },);
  /**
   Every model the re-registration must carry, of all three types.
   */
  const models: ProviderModelConfig[] = [
    ...keptChat.map(function toChatConfig(model,) {
      return toModelConfig(model,);
    },),
    ...read.readImageModels(provider,)
      .map(function toImageConfig(model,) {
        return toModelConfig(model,);
      },),
    ...read.readClassifierModels(provider,)
      .map(function toClassifierConfig(model,) {
        return toModelConfig(model,);
      },),
  ];
  /**
   First reason this provider cannot be re-declared, when there is one.
   */
  const gap = planGap({
    models,
    base,
  },);
  if (gap !== REGISTERABLE)
    return { gap, };
  return {
    config: {
      ...base,
      models,
    },
  };
}

/**
 Plan every provider filter a catalog needs.

 A provider only earns a plan when the rule retires one of its models, and a family's
 keeper is never retired, so a plan always keeps at least the keeper.

 @param read - injected registry reads

 @returns plans, every retirement, the abstention tally, and the skipped providers

 @example
 ```typescript
 planProviderFilters({ read: catalogRead });
 ```
 */
export function planProviderFilters(
  {
    read,
  }: {
    readonly read: CatalogRead;
  },
): FilterPlanning {
  /**
   Chat models in registry order.
   */
  const chatModels = read.readChatModels();
  /**
   Rule input built from the chat catalog, which is the only type pi versions by id.
   */
  const entries: CatalogEntry[] = chatModels.map(function toEntry(model,) {
    return {
      provider: model.provider,
      api: model.api,
      modelId: model.id,
    };
  },);
  /**
   Rule outcome for the whole chat catalog.
   */
  const decision = decideRetirements({ entries, },);
  /**
   Retired identities grouped by provider.
   */
  const index = buildRetiredIndex({ retirements: decision.retirements, },);
  /**
   Plans in first-seen provider order.
   */
  const plans: ProviderFilterPlan[] = [];
  /**
   Providers that cannot be filtered by either mechanism.
   */
  const skippedProviders: SkippedProvider[] = [];
  for (const provider of index.keys()) {
    /**
     Retirements decided for this provider.
     */
    const retirements = decision.retirements
      .filter(function onProvider(retirement,) {
        return retirement.provider === provider;
      },);
    /**
     Incumbent configuration for this provider, when pi exposes one.
     */
    const incumbent = read.readProviderConfig(provider,);
    /**
     Composed provider object, when the registry exposes one.
     */
    const composed = read.readComposedProvider(provider,);
    if ((incumbent.kind === 'present') && (incumbent.config !== undefined)) {
      /**
       Configuration plan, or the reason the models cannot be re-declared.
       */
      const planned = planConfig({
        provider,
        chatModels,
        read,
        base: incumbent.config,
        index,
      },);
      if ('config' in planned) {
        plans.push({
          kind: 'config',
          provider,
          config: planned.config,
          retirements,
        },);
        continue;
      }
      if ((composed.kind !== 'present') || (composed.provider === undefined)) {
        skippedProviders.push({
          provider,
          reason: planned.gap,
        },);
        continue;
      }
      plans.push({
        kind: 'wrapper',
        provider,
        wrapped: wrapProvider({
          provider: composed.provider,
          index,
        },),
        retirements,
      },);
      continue;
    }
    if ((composed.kind === 'present') && (composed.provider !== undefined)) {
      plans.push({
        kind: 'wrapper',
        provider,
        wrapped: wrapProvider({
          provider: composed.provider,
          index,
        },),
        retirements,
      },);
      continue;
    }
    skippedProviders.push({
      provider,
      reason: 'pi exposes neither a provider configuration nor a composed provider to filter',
    },);
  }
  return {
    plans,
    retirements: decision.retirements,
    abstentions: decision.abstentions,
    skippedProviders,
  };
}

//endregion Planning
