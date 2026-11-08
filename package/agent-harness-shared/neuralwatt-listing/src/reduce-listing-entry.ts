/**
 Snapshot reduction for the NeuralWatt listing: one live listing entry to the
 checked-in snapshot shape, keeping only the fields the mapping reads. The
 `refresh-snapshot` task feeds its output to the chunk writer, so a future
 snapshot carries exactly what the mapping needs and nothing it guesses from.
 @module
 */

import {
  isRecord,
  type NeuralWattListingEntry,
} from './listing.ts';

//region Field sets

/**
 Entry-level fields the mapping reads outside metadata. Deprecation is
 handled separately: the observable chunk shape carries it only when the entry
 asserts it.
 */
const ENTRY_FIELDS = [
  'id',
  'max_model_len',
] as const;

/**
 Metadata fields the mapping reads directly.
 */
const METADATA_FIELDS = [
  'display_name',
] as const;

/**
 Capability fields the mapping reads: reasoning drives the efforts dict and
 vision drives the input list.
 */
const CAPABILITY_FIELDS = [
  'reasoning',
  'vision',
] as const;

/**
 Reasoning fields the mapping reads: `default_enabled` decides whether the
 off level may be offered, and the effort lists decide which spellings are.
 */
const REASONING_FIELDS = [
  'default_enabled',
  'supported_efforts',
  'accepted_efforts',
  'effort_aliases',
] as const;

/**
 Limit fields the mapping reads as capacities.
 */
const LIMIT_FIELDS = [
  'max_context_length',
  'max_output_tokens',
] as const;

/**
 Pricing fields a harness that prices models projects from.
 */
const PRICING_FIELDS = [
  'input_per_million',
  'output_per_million',
  'cached_input_per_million',
  'cached_output_per_million',
  'currency',
  'pricing_tbd',
  'service_tier',
  'flex_discount_multiplier',
] as const;

//endregion Field sets

//region Narrowing helpers

/**
 Keep the named fields one external record discloses, dropping every field
 that is absent rather than storing an explicit hole.

  @param source - narrowed external record to read

  @param fields - field names the mapping reads

  @returns picked fields, in the order the names list them

  */
function pickFields({
  source,
  fields,
}: {
  readonly source: Readonly<Record<string, unknown>>;
  readonly fields: readonly string[];
}): Record<string, unknown> {
  /**
   Fields kept, in the order the names list them.
   */
  const picked: Record<string, unknown> = {};
  for (const field of fields) {
    /**
     Value this field discloses, or undefined when it declines to.
     */
    const value = source[field];
    if (value !== undefined) {
      picked[field] = value;
    }
  }
  return picked;
}

/**
 Keep one named metadata sub-block when the block discloses at least one
 mapped field, so an empty block never reaches the snapshot as a bare object.

  @param name - metadata key the sub-block hangs under

  @param source - value read under that key, before narrowing

  @param fields - field names the mapping reads inside the block

  @returns single-key record when the block speaks, empty record otherwise

  */
function namedSubBlock({
  name,
  source,
  fields,
}: {
  readonly name: string;
  readonly source: unknown;
  readonly fields: readonly string[];
}): Record<string, unknown> {
  if (!isRecord(source)) {
    return {};
  }
  /**
   Mapped fields this block discloses.
   */
  const picked = pickFields({
    source,
    fields,
  });
  for (const field of fields) {
    if (Object.hasOwn(
      picked,
      field
    )) {
      return { [name]: picked };
    }
  }
  return {};
}

//endregion Narrowing helpers

//region Public API

/**
 Reduce one live listing entry to the checked-in snapshot shape: the entry
 fields and metadata fields the mapping reads, including `reasoning
 .default_enabled` so a thinking-off-by-default model keeps offering the off
 level from the snapshot path, and the pricing block a harness that prices
 models projects from. Deprecation travels only when the entry asserts it.

  @param entry - listing entry read from the live endpoint

  @returns snapshot-shaped entry carrying only the mapped fields

  @example
  ```ts
  const reduced = reduceListingEntry({ entry: liveEntry });
  ```
  */
export function reduceListingEntry({
  entry,
}: {
  readonly entry: NeuralWattListingEntry;
}): NeuralWattListingEntry {
  /**
   Entry metadata read before the record guard narrows it.
   */
  const metadataField = entry.metadata;
  /**
   Narrowed metadata, empty stand-in when the entry discloses none.
   */
  const rawMetadata = isRecord(metadataField) ? metadataField : {};
  return {
    ...pickFields({
      source: entry,
      fields: ENTRY_FIELDS,
    }),
    metadata: {
      ...pickFields({
        source: rawMetadata,
        fields: METADATA_FIELDS,
      }),
      ...namedSubBlock({
        name: 'capabilities',
        source: rawMetadata.capabilities,
        fields: CAPABILITY_FIELDS,
      }),
      ...namedSubBlock({
        name: 'reasoning',
        source: rawMetadata.reasoning,
        fields: REASONING_FIELDS,
      }),
      ...namedSubBlock({
        name: 'limits',
        source: rawMetadata.limits,
        fields: LIMIT_FIELDS,
      }),
      ...namedSubBlock({
        name: 'pricing',
        source: rawMetadata.pricing,
        fields: PRICING_FIELDS,
      }),
    },
    ...(entry.deprecated === true ? { deprecated: true } : {}),
  };
}

//endregion Public API
