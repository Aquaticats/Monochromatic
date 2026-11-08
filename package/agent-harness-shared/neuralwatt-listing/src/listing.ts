/**
 Model catalog mapping for the NeuralWatt provider: listing entries the
 service publishes to the model facts each harness projects onto its own
 model profile shape. @module
 */

import {
  NEURALWATT_EFFORT_LEVEL_BY_SPELLING,
  type NeuralWattEffortsDict,
  type NeuralWattThinkingLevel,
} from './thinking-level-map.ts';

//region External JSON narrowing

/**
 One listing entry as the endpoint sends it: every field unknown until this
 mapping narrows it, so a field shape change lands as a dropped entry, never a
 served lie. The listing's `metadata.pricing` block narrows into
 `NeuralWattModelPricing` so a harness that prices models can; this core never
 estimates cost itself.
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
 Narrow one external value to a finite number, the only shape worth serving
 as a price; a null price is the endpoint declining to name one.

  @param value - value read out of external JSON

  @returns `true` when `value` prices a field
  */
function finiteNumber(value: unknown): value is number {
  return ((typeof value) === 'number') && Number.isFinite(value);
}

/**
 Usable members of one effort list: the strings it carries, or nothing at all
 when the field is absent, not a list, or holds a member that is not a
 non-empty string. An unusable list collapses into the fallback chain instead
 of half-serving it.

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

//region Model facts

/**
 Input modality one model earns: text always, image when the listing marks
 the model vision-capable.
 */
export type NeuralWattModelInput = 'text' | 'image';

/**
 Pricing one model discloses, per million tokens in the listing's currency:
 each field absent when the listing leaves it null or names no finite number,
 which the serving harness reads as a zero price. `pricingTbd` carries the
 listing's own "price not decided" flag rather than guessing.
 */
export type NeuralWattModelPricing = {
  readonly inputPerMillion?: number;
  readonly outputPerMillion?: number;
  readonly cachedInputPerMillion?: number;
  readonly cachedOutputPerMillion?: number;
  readonly currency?: string;
  readonly pricingTbd?: boolean;
  readonly serviceTier?: string;
  readonly flexDiscountMultiplier?: number;
};

/* oxlint-disable no-restricted-syntax/no-optional-escape -- mirrors `PiAiModelProfile['reasoningEfforts'] = false | PiAiReasoningEfforts` (dsh-llm-pi-ai `lib/types/catalog.d.ts`), where `false` is not absence but the declaration "non-reasoning model", so a harness projection serves it unchanged */
/**
 Thinking-level declaration one model earns: `false` for a model the listing
 marks non-reasoning or offers no level beyond off, the offered dict
 otherwise. The `NeuralWattModelFacts` field stays absent when the entry lists
 no efforts at all, which is a third state this union never carries.
 */
export type NeuralWattReasoningEfforts = false | NeuralWattEffortsDict;
/* oxlint-enable no-restricted-syntax/no-optional-escape */

/**
 Serviceable model facts one listing entry earns: every field optional except
 identity and input, and every optional field absent rather than guessed when
 the listing discloses nothing.
 */
export type NeuralWattModelFacts = {
  readonly id: string;
  readonly name: string;
  readonly contextWindow?: number;
  readonly maxTokens?: number;
  readonly input: readonly NeuralWattModelInput[];
  readonly reasoningEfforts?: NeuralWattReasoningEfforts;
  readonly pricing?: NeuralWattModelPricing;
};

//endregion Model facts

//region Effort spellings to thinking levels

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
 Thinking-level declaration one entry's reasoning block earns, carried as the
 facts fields it contributes: `false` when the listing marks the model
 non-reasoning or offers no level beyond off, the level dict otherwise, and no
 field at all when the entry lists no efforts. The dict carries the off level
 only for a model the listing says thinks off by default; every other model
 keeps the level out so an unnamed default reaches the service as the
 parameter's absence.

  @param metadata - narrowed entry metadata this mapping reads

  @returns reasoning fields to spread onto the model facts
  */
function reasoningEffortsFields({
  metadata,
}: {
  readonly metadata: Readonly<Record<string, unknown>>;
}): Pick<NeuralWattModelFacts, 'reasoningEfforts'> {
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
   spelling the level vocabulary expresses, collected through a mutable
   scratch record since the offered dict's fields are readonly. The scratch
   holds spellings only, never the dict's `null` instruction.
   */
  const scratch: Record<string, string> = {};
  for (const spelling of spellings) {
    /**
     Thinking level this spelling maps to, or undefined when the level
     vocabulary cannot express the spelling.
     */
    const level = NEURALWATT_EFFORT_LEVEL_BY_SPELLING[spelling];
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
    scratch[level] = spelling;
  }
  return Object.keys(scratch)
    .some(selectsThinkingLevel)
    ? { reasoningEfforts: scratch }
    : { reasoningEfforts: false };
}

//endregion Effort spellings to thinking levels

//region Pricing disclosure

/**
 Pricing one entry's metadata block earns, carried as the facts field it
 contributes: the field only when the entry discloses a pricing record, and
 each price absent where the listing leaves it null or names no finite number.

  @param metadata - narrowed entry metadata this mapping reads

  @returns pricing fields to spread onto the model facts
  */
function pricingFieldsOf({
  metadata,
}: {
  readonly metadata: Readonly<Record<string, unknown>>;
}): Pick<NeuralWattModelFacts, 'pricing'> {
  /**
   Pricing block read before the record guard narrows it.
   */
  const pricingField = metadata.pricing;
  if (!isRecord(pricingField)) {
    return {};
  }
  /**
   Input price per million tokens, absent when the listing names none.
   */
  const inputPerMillion = finiteNumber(pricingField.input_per_million)
    ? pricingField.input_per_million
    : undefined;
  /**
   Output price per million tokens, absent when the listing names none.
   */
  const outputPerMillion = finiteNumber(pricingField.output_per_million)
    ? pricingField.output_per_million
    : undefined;
  /**
   Cached-input price per million tokens, absent when the listing names none.
   */
  const cachedInputPerMillion = finiteNumber(
    pricingField.cached_input_per_million
  )
    ? pricingField.cached_input_per_million
    : undefined;
  /**
   Cached-output price per million tokens, absent when the listing names none.
   */
  const cachedOutputPerMillion = finiteNumber(
    pricingField.cached_output_per_million
  )
    ? pricingField.cached_output_per_million
    : undefined;
  /**
   Currency the prices are denominated in, absent when the listing names none.
   */
  const currency = nonEmptyString(pricingField.currency)
    ? pricingField.currency
    : undefined;
  /**
   The listing's own "price not decided" flag, absent when the listing names
   none.
   */
  const pricingTbd = (typeof pricingField.pricing_tbd) === 'boolean'
    ? pricingField.pricing_tbd
    : undefined;
  /**
   Service tier the prices belong to, absent when the listing names none.
   */
  const serviceTier = nonEmptyString(pricingField.service_tier)
    ? pricingField.service_tier
    : undefined;
  /**
   Flex discount multiplier, absent when the listing names none.
   */
  const flexDiscountMultiplier = finiteNumber(
    pricingField.flex_discount_multiplier
  )
    ? pricingField.flex_discount_multiplier
    : undefined;
  return {
    pricing: {
      ...(inputPerMillion === undefined ? {} : { inputPerMillion }),
      ...(outputPerMillion === undefined ? {} : { outputPerMillion }),
      ...(cachedInputPerMillion === undefined ? {} : { cachedInputPerMillion }),
      ...(cachedOutputPerMillion === undefined ? {} : { cachedOutputPerMillion }),
      ...(currency === undefined ? {} : { currency }),
      ...(pricingTbd === undefined ? {} : { pricingTbd }),
      ...(serviceTier === undefined ? {} : { serviceTier }),
      ...(flexDiscountMultiplier === undefined
        ? {}
        : { flexDiscountMultiplier }),
    },
  };
}

//endregion Pricing disclosure

//region Listing entries to model facts

/**
 Serviceable model facts in listing order: entries the listing marks
 deprecated or gives no id drop out, and every remaining entry maps only the
 facts it discloses. Named and self-contained so the collecting mutation never
 leaks a binding across the mapping.

  @param listing - listing entries read from the service endpoint

  @returns model facts ready for a harness projection

  @example
  ```ts
  const facts = mapNeuralWattListing({ listing: neuralWattListingSnapshot });
  ```
  */
export function mapNeuralWattListing({
  listing,
}: {
  readonly listing: readonly NeuralWattListingEntry[];
}): NeuralWattModelFacts[] {
  /**
   Model facts collected in listing order.
   */
  const models: NeuralWattModelFacts[] = [];
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
     Context window the facts earn, or undefined when nothing sizes one.
     */
    const contextWindow = positiveTokenCount(contextCandidate)
      ? contextCandidate
      : undefined;
    /**
     Output limit the facts earn, or undefined when the listing sizes none.
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
      ...pricingFieldsOf({ metadata }),
    });
  }
  return models;
}

//endregion Listing entries to model facts
