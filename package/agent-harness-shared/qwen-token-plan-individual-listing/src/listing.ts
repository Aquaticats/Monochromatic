/**
 Model catalog mapping for the Qwen Token Plan individual route: listing
 entries to the resolved facts each harness projects onto its own model
 profile shape. Membership comes from the listing, facts from the curated
 table, and ids the table does not know inherit their nearest older siblings'
 facts as an understating floor before pi's schema defaults
 (`doc/planning/qwen-token-plan-individual-provider.md` sections "Owner
 answers" and "Model facts table"). @module
 */

import {
  QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW,
  QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS,
  QWEN_TOKEN_PLAN_MEDIA_NAME_MARKERS,
  type QwenTokenPlanModelFacts,
  type QwenTokenPlanResolvedModelFacts,
} from './model-facts.ts';
import {
  compareQwenTokenPlanModelAgeKeys,
  qwenTokenPlanModelAgeKey,
  qwenTokenPlanModelFamilyKey,
  type QwenTokenPlanModelAgeKey,
} from './model-family.ts';
import { qwenTokenPlanIndividualFacts, } from './model-facts.data.ts';

//region Listing entries

/**
 One listing entry as this mapping reads it: the bare listing carries `id`,
 `created`, `owned_by`, and `object`, and only the first two feed facts, so the
 fetch narrows to exactly these fields. `created` decides sibling ordering when
 present.
 */
export type QwenTokenPlanListingEntry = {
  readonly id: string;
  readonly created?: number;
};

/**
 Narrow one external value to a plain record, the shape every external read
 starts from; arrays and null describe no object of named fields.

  @param value - value read out of external JSON

  @returns `true` when `value` carries named fields

  @example
  ```ts
  const body = isRecord(decoded) ? decoded : {};
  ```
  */
export function isRecord(value: unknown): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value));
}

//endregion Listing entries

//region Media name guard

/**
 Whether one lowercased id carries a media marker word: the guard that keeps a
 media id arriving between refreshes from being served as chat. Substring match
 over the whole id, so `qwen-image-3.0-pro` and `wan2.7-image-pro` both trip it;
 a facts row always overrides the guard.

  @param id - listing id no facts row names

  @returns `true` when the name marks the id media
  */
function isMediaNamed({
  id,
}: {
  readonly id: string;
}): boolean {
  /**
   Lowercased id the marker words match against.
   */
  const lowered = id.toLowerCase();
  for (const marker of QWEN_TOKEN_PLAN_MEDIA_NAME_MARKERS) {
    if (lowered.includes(marker)) {
      return true;
    }
  }
  return false;
}

//endregion Media name guard

//region Sibling fallback walk

/**
 Age key one id earns from the listing stamps this mapping collected: the
 stamp the listing carried for the id, else its parsed version.

  @param id - model id to date

  @param createdById - listing `created` stamps by id

  @returns the age key sibling ordering compares
  */
function ageKeyOf({
  id,
  createdById,
}: {
  readonly id: string;
  readonly createdById: ReadonlyMap<string, number>;
}): QwenTokenPlanModelAgeKey {
  /**
   Listing stamp for this id, which the entry carried when the listing gave
   one.
   */
  const created = createdById.get(id);
  return qwenTokenPlanModelAgeKey(created === undefined
    ? { id, }
    : {
      id,
      created,
    });
}

/**
 Chat facts rows older than the unknown id, nearest older first: the walk the
 owner chose ("check its older siblings, newer to older"), so each empty field
 fills from the first sibling that has it as an understating floor. Rows the
 table only uses to skip media ids never join the walk, since their placeholder
 facts describe no chat model.

  @param id - listing id the facts table does not know

  @param createdById - listing `created` stamps by id

  @param facts - facts rows the walk draws from

  @returns older chat rows, nearest older first
  */
function olderSiblingsOf({
  id,
  createdById,
  facts,
}: {
  readonly id: string;
  readonly createdById: ReadonlyMap<string, number>;
  readonly facts: readonly QwenTokenPlanModelFacts[];
}): QwenTokenPlanModelFacts[] {
  /**
   Family key the unknown id and its siblings share.
   */
  const familyKey = qwenTokenPlanModelFamilyKey(id);
  /**
   Age key of the unknown id itself, which every sibling must fall behind.
   */
  const ownAgeKey = ageKeyOf({
    id,
    createdById,
  });
  /**
   Older chat siblings collected before the nearest-first ordering.
   */
  const older: QwenTokenPlanModelFacts[] = [];
  for (const row of facts) {
    if ((row.kind !== 'chat') || (row.id === id)
      || (qwenTokenPlanModelFamilyKey(row.id) !== familyKey)) {
      continue;
    }
    if (compareQwenTokenPlanModelAgeKeys({
      left: ageKeyOf({
        id: row.id,
        createdById,
      }),
      right: ownAgeKey,
    }) < 0) {
      older.push(row);
    }
  }
  return older.toSorted(function newestFirst(
    left,
    right
  ): number {
    /**
     Age order of the two siblings with `sort`'s sign convention flipped, so
     the nearest older sibling lands first.
     */
    return compareQwenTokenPlanModelAgeKeys({
      left: ageKeyOf({
        id: right.id,
        createdById,
      }),
      right: ageKeyOf({
        id: left.id,
        createdById,
      }),
    });
  });
}

//endregion Sibling fallback walk

//region Listing entries to resolved facts

/**
 Resolved facts for one id the facts table knows: the row's own words win, the
 missing numbers fall to pi's schema defaults, and a non-reasoning row never
 serves efforts. The compat flag re-derives from the served efforts so the
 documented invariant holds whatever the row declares.

  @param id - listing id the row names

  @param row - facts row the table carries

  @returns resolved facts ready for a harness projection
  */
function resolvedFromRow({
  id,
  row,
}: {
  readonly id: string;
  readonly row: QwenTokenPlanModelFacts;
}): QwenTokenPlanResolvedModelFacts {
  /**
   Efforts this model serves: the row's dict only while the row says the model
   reasons, since the non-reasoning state serves none.
   */
  const efforts = (row.reasoning && (row.efforts !== undefined))
    ? row.efforts
    : undefined;
  return {
    id,
    name: row.name,
    kind: row.kind,
    contextWindow: row.contextWindow ?? QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW,
    maxTokens: row.maxTokens ?? QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS,
    input: row.input,
    reasoning: row.reasoning,
    ...(efforts === undefined ? {} : { efforts, }),
    compat: {
      ...row.compat,
      supportsReasoningEffort: efforts !== undefined,
    },
  };
}

/**
 Resolved facts for one id the facts table does not know: the sibling walk
 fills each field from the first older sibling that has it, the remaining
 numbers fall to pi's schema defaults, and the name stays the id itself, since
 a display name is identity and never inherited from a different model. The
 walk covers only ids with no row; a row's absent fields are deliberate
 (`qwen3.7-max` offers no efforts) and stay absent.

  @param id - listing id no row names

  @param createdById - listing `created` stamps by id

  @param facts - facts rows the walk draws from

  @returns resolved facts ready for a harness projection
  */
function resolvedFromSiblings({
  id,
  createdById,
  facts,
}: {
  readonly id: string;
  readonly createdById: ReadonlyMap<string, number>;
  readonly facts: readonly QwenTokenPlanModelFacts[];
}): QwenTokenPlanResolvedModelFacts {
  /**
   Older siblings in the walk's nearest-first order.
   */
  const siblings = olderSiblingsOf({
    id,
    createdById,
    facts,
  });
  /**
   Context window the walk decided, else pi's schema default.
   */
  const contextWindow = siblings.find(
    function namesContextWindow(row): boolean {
      return row.contextWindow !== undefined;
    },
  )
    ?.contextWindow
    ?? QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW;
  /**
   Max output the walk decided, else pi's schema default.
   */
  const maxTokens = siblings.find(function namesMaxTokens(row): boolean {
    return row.maxTokens !== undefined;
  })
    ?.maxTokens
    ?? QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS;
  /**
   Input modalities the nearest older sibling discloses, else text alone.
   */
  const input = siblings[0]
    ?.input
    ?? [
      'text',
    ];
  /**
   Reasoning state the nearest older sibling discloses, else reasoning without
   selectable efforts, matching what the planning doc serves for `auto`.
   */
  const reasoning = siblings[0]
    ?.reasoning
    ?? true;
  /**
   Efforts the first sibling with efforts discloses, else none.
   */
  const efforts = siblings.find(function namesEfforts(row): boolean {
    return row.efforts !== undefined;
  })
    ?.efforts;
  return {
    id,
    name: id,
    kind: 'chat',
    contextWindow,
    maxTokens,
    input,
    reasoning,
    ...(efforts === undefined ? {} : { efforts, }),
    compat: {
      ...QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
      supportsReasoningEffort: efforts !== undefined,
    },
  };
}

/**
 Serviceable resolved facts in listing order: entries the mapping drops are
 media kinds, media-named ids the table does not know, and ids the listing
 gives no usable id for. Everything else serves, with facts resolved row first,
 then the sibling walk, then pi's schema defaults. Named and self-contained so
 the collecting mutation never leaks a binding across the mapping.

  @param listing - listing entries read from the service endpoint or the snapshot

  @param facts - facts rows to resolve against, defaulting to the curated table

  @returns resolved facts ready for a harness projection

  @example
  ```ts
  const models = mapQwenTokenPlanIndividualModels({
    listing: qwenTokenPlanIndividualListingSnapshot.entries,
  });
  ```
  */
export function mapQwenTokenPlanIndividualModels({
  listing,
  facts = qwenTokenPlanIndividualFacts,
}: {
  readonly listing: readonly QwenTokenPlanListingEntry[];
  readonly facts?: readonly QwenTokenPlanModelFacts[];
}): readonly QwenTokenPlanResolvedModelFacts[] {
  /**
   Resolved facts collected in listing order.
   */
  const models: QwenTokenPlanResolvedModelFacts[] = [];
  /**
   Facts rows by id, built once per mapping so each entry resolves against one
   lookup.
   */
  const factsById = new Map<string, QwenTokenPlanModelFacts>(
    facts.map(function toEntry(row): [
      string,
      QwenTokenPlanModelFacts
    ] {
      return [
        row.id,
        row,
      ];
    }),
  );
  /**
   Listing `created` stamps by id, which the sibling walk dates rows with.
   */
  const createdById = new Map<string, number>();
  for (const entry of listing) {
    if (entry.created !== undefined) {
      createdById.set(
        entry.id,
        entry.created
      );
    }
  }
  for (const entry of listing) {
    /**
     Entry id read before the serviceability guards.
     */
    const { id, } = entry;
    if (id.length === 0) {
      continue;
    }
    /**
     Facts row this id earns, or undefined when the table does not know it.
     */
    const row = factsById.get(id);
    if (row !== undefined) {
      if (row.kind !== 'chat') {
        continue;
      }
      models.push(resolvedFromRow({
        id,
        row,
      }));
      continue;
    }
    if (isMediaNamed({ id, })) {
      continue;
    }
    models.push(resolvedFromSiblings({
      id,
      createdById,
      facts,
    }));
  }
  return models;
}

//endregion Listing entries to resolved facts
