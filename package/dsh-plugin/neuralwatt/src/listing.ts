/**
 Model catalog mapping for the NeuralWatt provider: listing entries the
 service publishes to provider profiles the shipped pi-ai adapter serves.
 @module
 */

import type {
  PiAiModelProfile,
  PiAiProviderProfile,
  PiAiReasoningEfforts,
} from '@deepseek-ai/dsh-llm-pi-ai';
import {
  NEURALWATT_API,
  NEURALWATT_API_KEY_ENV,
  NEURALWATT_BASE_URL,
  NEURALWATT_ROUTE,
} from './constants.ts';

//region External JSON narrowing

/**
 One listing entry as the endpoint sends it: every field unknown until this
 mapping narrows it, so a field shape change lands as a dropped entry, never
 a served lie. The listing's `metadata.pricing` block exists but is
 deliberately absent here: this provider serves models, it does not estimate
 cost.
 */
export type NeuralWattListingEntry = {
  readonly id?: unknown;
  readonly max_model_len?: unknown;
  readonly deprecated?: unknown;
  readonly metadata?: unknown;
};

/**
 Narrow one external value to a plain record, the shape every metadata read
 starts from; arrays and null describe no object of named fields.

 @param value - value read out of external JSON

 @returns `true` when `value` carries named fields

 @example
 ```ts
 const metadata = isRecord(entry.metadata) ? entry.metadata : {};
 ```
 */
export function isRecord(value: unknown): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value));
}

/**
 Narrow one external value to a non-empty string, the only spelling worth
 serving as a name.

 @param value - value read out of external JSON

 @returns `true` when `value` is usable as a name
 */
function nonEmptyString(value: unknown): value is string {
  return ((typeof value) === 'string') && (value.length > 0);
}

/**
 Narrow one external value to a usable token count: a positive integer is a
 capacity, every other value is the endpoint declining to size the field.

 @param value - value read out of external JSON

 @returns `true` when `value` sizes a capacity
 */
function positiveTokenCount(value: unknown): value is number {
  return ((typeof value) === 'number') && Number.isInteger(value)
    && (value > 0);
}

/**
 Usable members of one effort list: the strings it carries, or nothing at
 all when the field is absent, not a list, or holds a member that is not a
 non-empty string. An unusable list collapses into the fallback chain
 instead of half-serving it.

 @param value - value read out of external JSON

 @returns effort spellings the field discloses
 */
function stringListOrEmpty(value: unknown): readonly string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  /**
   List members collected as the strings the field discloses.
   */
  const members: string[] = [];
  for (const member of value) {
    if (!nonEmptyString(member)) {
      return [];
    }
    members.push(member);
  }
  return members;
}

//endregion External JSON narrowing

//region Effort spellings to thinking levels

/**
 Effort spellings to thinking levels, the exact vocabulary this mapping
 expresses: each level dispatches the same spelling the listing gave, so the
 wire format never invents one. A read yields a level only for spellings the
 table names, so a spelling this vocabulary cannot express skips instead of
 inventing a level.
 */
const EFFORT_LEVEL_BY_SPELLING:
  Readonly<Record<string, keyof PiAiReasoningEfforts>> = {
    none: 'off',
    minimal: 'minimal',
    low: 'low',
    medium: 'medium',
    high: 'high',
    xhigh: 'xhigh',
    max: 'max',
  };

/**
 Keep the effort keys that select thinking, so a dict whose only key is the
 off level never declares a reasoning model.

 @param key - thinking level key one spelling mapped to

 @returns `true` when the key selects thinking
 */
function selectsThinkingLevel(key: string): boolean {
  return key !== 'off';
}

/**
 Whether the listing says the model thinks when the caller names no effort.
 Only an explicit `default_enabled: false` marks a model whose thinking starts
 disabled, so a listing that stays silent keeps this mapping from claiming a
 default. The dispatch seam cannot tell the default level apart from the off
 level, so a model that thinks by default gives up the off level rather than
 have every untouched default dispatch the off wire.

 @param reasoning - narrowed reasoning block this mapping reads

 @returns `false` only when the listing says thinking starts disabled
 */
function thinksByDefault({
  reasoning,
}: {
  readonly reasoning: Readonly<Record<string, unknown>>;
}): boolean {
  return reasoning.default_enabled !== false;
}

/**
 Thinking-level declaration one entry's reasoning block earns, carried as
 the profile fields it contributes: `false` when the listing marks the model
 non-reasoning or offers no level beyond off, the level dict otherwise, and
 no field at all when the entry lists no efforts. The dict carries the off
 level only for a model the listing says thinks off by default; every other
 model keeps the level out so an unnamed default reaches the service as the
 parameter's absence.

 @param metadata - narrowed entry metadata this mapping reads

 @returns reasoning fields to spread onto the model profile
 */
function reasoningEffortsFields({
  metadata,
}: {
  readonly metadata: Readonly<Record<string, unknown>>;
}): Pick<PiAiModelProfile, 'reasoningEfforts'> {
  /**
   Capability block read before the record guard narrows it.
   */
  const capabilitiesField = metadata.capabilities;
  /**
   Narrowed capabilities, empty stand-in when the entry discloses no
   capability block.
   */
  const capabilities = isRecord(capabilitiesField) ? capabilitiesField : {};
  /**
   Reasoning block read before the record guard narrows it.
   */
  const reasoningField = metadata.reasoning;
  /**
   Narrowed reasoning block, empty stand-in when the entry discloses none.
   */
  const reasoning = isRecord(reasoningField) ? reasoningField : {};
  if (capabilities.reasoning === false) {
    return { reasoningEfforts: false };
  }
  /**
   Alias table read before the record guard narrows it.
   */
  const aliasesField = reasoning.effort_aliases;
  /**
   Narrowed alias table; alias spellings never serve as dispatch values.
   */
  const aliases = isRecord(aliasesField) ? aliasesField : {};
  /**
   Supported spellings the listing names, empty when the list is unusable.
   */
  const supported = stringListOrEmpty(reasoning.supported_efforts);
  /**
   Accepted spellings the listing names, empty when the list is unusable.
   */
  const accepted = stringListOrEmpty(reasoning.accepted_efforts);
  /**
   Effort spellings to offer: the supported list when it speaks, else the
   accepted efforts that name no alias.
   */
  const spellings = supported.length > 0
    ? supported
    : accepted.filter(function namesNoAlias(effort): boolean {
      return !Object.hasOwn(
        aliases,
        effort
      );
    });
  if (spellings.length === 0) {
    return {};
  }
  /**
   Thinking levels keyed the way this service dispatches, one entry per
   spelling the level vocabulary expresses.
   */
  const efforts: PiAiReasoningEfforts = {};
  for (const spelling of spellings) {
    /**
     Thinking level this spelling maps to, or undefined when the level
     vocabulary cannot express the spelling.
     */
    const level = EFFORT_LEVEL_BY_SPELLING[spelling];
    if (level === undefined) {
      continue;
    }
    /**
     The off spelling maps only when thinking starts disabled: naming an
     effort and naming none are the same request at the dispatch seam, so an
     off level on a thinking-by-default model would silence its default.
     */
    if ((level === 'off') && thinksByDefault({ reasoning })) {
      continue;
    }
    efforts[level] = spelling;
  }
  return Object.keys(efforts)
    .some(selectsThinkingLevel)
    ? { reasoningEfforts: efforts }
    : { reasoningEfforts: false };
}

//endregion Effort spellings to thinking levels

//region Listing entries to model profiles

/**
 Serviceable model profiles in listing order: entries the listing marks
 deprecated or gives no id drop out, and every remaining entry maps only the
 facts it discloses. Named and self-contained so the collecting mutation
 never leaks a binding across the mapping.

 @param listing - listing entries read from the service endpoint

 @returns model profiles ready for the provider dict
 */
function mappedServiceableModels({
  listing,
}: {
  readonly listing: readonly NeuralWattListingEntry[];
}): PiAiModelProfile[] {
  /**
   Model profiles collected in listing order.
   */
  const models: PiAiModelProfile[] = [];
  for (const entry of listing) {
    /**
     Entry id read before the serviceability guard.
     */
    const {id} = entry;
    if ((!nonEmptyString(id)) || (entry.deprecated === true)) {
      continue;
    }
    /**
     Entry metadata read before the record guard narrows it.
     */
    const metadataField = entry.metadata;
    /**
     Narrowed metadata, empty stand-in when the entry discloses none.
     */
    const metadata = isRecord(metadataField) ? metadataField : {};
    /**
     Display name read before the id fallback decides the served name.
     */
    const nameField = metadata.display_name;
    /**
     Limits block read before the record guard narrows it.
     */
    const limitsField = metadata.limits;
    /**
     Narrowed limits, empty stand-in when the entry discloses none.
     */
    const limits = isRecord(limitsField) ? limitsField : {};
    /**
     Capability block read before narrowing; vision drives the input list.
     */
    const capabilitiesField = metadata.capabilities;
    /**
     Narrowed capabilities, empty stand-in when the entry discloses none.
     */
    const capabilities = isRecord(capabilitiesField) ? capabilitiesField : {};
    /**
     Context capacity: the limits disclosure wins over the endpoint's model
     length, and both absent leaves the field unset instead of guessing.
     */
    const contextCandidate = positiveTokenCount(limits.max_context_length)
      ? limits.max_context_length
      : entry.max_model_len;
    /**
     Context window the profile earns, or undefined when nothing sizes one.
     */
    const contextWindow = positiveTokenCount(contextCandidate)
      ? contextCandidate
      : undefined;
    /**
     Output limit the profile earns, or undefined when the listing sizes none.
     */
    const maxTokens = positiveTokenCount(limits.max_output_tokens)
      ? limits.max_output_tokens
      : undefined;
    models.push({
      id,
      name: nonEmptyString(nameField) ? nameField : id,
      ...(contextWindow === undefined ? {} : { contextWindow }),
      ...(maxTokens === undefined ? {} : { maxTokens }),
      input: capabilities.vision === true
        ? [
          'text',
          'image',
        ]
        : [
          'text',
        ],
      ...reasoningEffortsFields({ metadata }),
    });
  }
  return models;
}

//endregion Listing entries to model profiles

//region Public API

/**
 Provider dict serving every serviceable listing entry on one route: the
 neuralwatt route, pinning credential, protocol, endpoint, and effort
 dispatch so nothing else has to.

 @param listing - listing entries read from the service endpoint

 @returns provider dict keyed by the route constant

 @example
 ```ts
 const providers = mapNeuralWattProviders({
   listing: neuralWattListingSnapshot,
 });
 ```
 */
export function mapNeuralWattProviders({
  listing,
}: {
  readonly listing: readonly NeuralWattListingEntry[];
}): Record<string, PiAiProviderProfile> {
  return {
    [NEURALWATT_ROUTE]: {
      apiKeyEnv: NEURALWATT_API_KEY_ENV,
      api: NEURALWATT_API,
      baseURL: NEURALWATT_BASE_URL,
      // NeuralWatt rejects the developer role (unsupported_message_role on
      // mimo-v2.6-pro), and pi-ai URL auto-detection enables it for reasoning
      // models, so pin the classic system role for every served model.
      compat: {
        supportsReasoningEffort: true,
        supportsDeveloperRole: false
      },
      models: mappedServiceableModels({ listing }),
    },
  };
}

//endregion Public API
