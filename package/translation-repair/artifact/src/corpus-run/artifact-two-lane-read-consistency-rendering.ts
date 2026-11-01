import {
  requireExactKeys,
  requireKeyOf,
  requireOneOf,
} from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  isCountedSingleMark,
  type UnlocatedReason,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { pageDefinitionsOf, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  artifactGenerationCountsBySpan,
  artifactGenerationRecordsDefinitions,
  artifactGenerationRecordsMarks,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';
import type { ArtifactConsistencyRendering, } from './artifact-two-lane-consistency.ts';

//region Artifact consistency rendering reading
// Reading the rendering of one place of a recorded group, key by key, under the
// generation the section was written in: whether a located place carries the
// offset of its span (sixteen), the single marks its English holds beside
// the span (seventeen) and the page definitions that move its slot (eighteen),
// and which reasons for a place to stay unlocated exist.

/**
 Every reason a place's rendering can be unlocated, keyed by the union so the
 compiler refuses a member the record lacks.
 */
const UNLOCATED_REASONS: Readonly<Record<UnlocatedReason, true>> = {
  'not-asked': true,
  'no-english': true,
  'no-agreement': true,
  empty: true,
  'not-found': true,
  ambiguous: true,
  markup: true,
  'overlaps-another': true,
  'slot-no-agreement': true,
  'slot-empty': true,
  'slot-not-found': true,
  'slot-ambiguous': true,
  'slot-whole': true,
};

/**
 Reasons only generation fifteen gives: a slot was a condition of a place being
 located there, so the five slot reasons were reasons for a place to stay as it
 stood. Generation sixteen locates a place on its quote alone.
 */
const SLOT_REASONS: ReadonlySet<UnlocatedReason> = new Set<UnlocatedReason>([
  'slot-no-agreement',
  'slot-empty',
  'slot-not-found',
  'slot-ambiguous',
  'slot-whole',
],);

/**
 The keys a located rendering may hold in generation fifteen: the quote and the slot.
 */
const LOCATED_KEYS_OF_FIFTEEN: readonly string[] = [
  'kind',
  'quote',
  'slot',
];

/**
 The keys a located rendering may hold in generation sixteen: the offset of the span joins them.
 */
const LOCATED_KEYS_OF_SIXTEEN: readonly string[] = [
  'kind',
  'quote',
  'start',
  'slot',
];

/**
 The keys a located rendering may hold in generation seventeen: the single marks beside the span join them.
 */
const LOCATED_KEYS_OF_SEVENTEEN: readonly string[] = [
  'kind',
  'quote',
  'start',
  'markBefore',
  'markAfter',
  'slot',
];

/**
 The keys a located rendering may hold in generation eighteen: the definitions of the page that move the slot join them.
 */
const LOCATED_KEYS_OF_EIGHTEEN: readonly string[] = [
  ...LOCATED_KEYS_OF_SEVENTEEN,
  'definitions',
];

/**
 The keys a located rendering may hold in a generation.

 @param generation - generation the section was written under

 @returns The keys, which a reader refuses any other than

 @example
 ```ts
 const keys = locatedKeysOf({ generation: 16, },); // ['kind', 'quote', 'start', 'slot']
 ```
 */
function locatedKeysOf({ generation, }: { readonly generation: TwoLaneArtifactGeneration; },): readonly string[] {
  if (artifactGenerationRecordsDefinitions({ generation, },))
    return LOCATED_KEYS_OF_EIGHTEEN;
  if (artifactGenerationRecordsMarks({ generation, },))
    return LOCATED_KEYS_OF_SEVENTEEN;
  return artifactGenerationCountsBySpan({ generation, },) ? LOCATED_KEYS_OF_SIXTEEN : LOCATED_KEYS_OF_FIFTEEN;
}

/**
 Reads the page definitions a located place records.

 @param value - recorded definitions

 @param path - where they sit, for the parse error

 @returns The definitions, which are at least one and read back, as the page's definitions are read, to themselves

 @throws {@link ArtifactParseError} when they are not text, are empty, or hold anything but definitions in the form a page's
 definitions are read in

 @example
 ```ts
 const definitions = requireDefinitions({ value: record.definitions, path: `${path}.definitions`, },);
 ```
 */
function requireDefinitions(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): string {
  /**
   The recorded text.
   */
  const definitions = requireString({
    value,
    path,
  },);
  if ((definitions === '') || (pageDefinitionsOf({ pageText: definitions, },) !== definitions)) {
    throw new ArtifactParseError({
      path,
      reason: 'link and footnote definitions in the form a page\'s definitions are read in, and at least one',
    },);
  }
  return definitions;
}

/**
 Reads a single mark a located place records beside its span.

 @param value - recorded mark

 @param path - where it sits, for the parse error

 @returns The mark, which is one apostrophe or single quotation mark

 @throws {@link ArtifactParseError} when it is not text, or is not exactly one of the eight single marks the pass counts

 @example
 ```ts
 const mark = requireSingleMark({ value: record.markAfter, path: `${path}.markAfter`, },);
 ```
 */
function requireSingleMark(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): string {
  /**
   The recorded text.
   */
  const mark = requireString({
    value,
    path,
  },);
  if (!isCountedSingleMark({ character: mark, },)) {
    throw new ArtifactParseError({
      path,
      reason: 'one single mark, an apostrophe or a single quotation mark',
    },);
  }
  return mark;
}

/**
 Reads a place's rendering.

 @param value - recorded rendering

 @param path - where it sits, for the parse error

 @param generation - generation the section was written under, which says
 whether a located place carries its offset and the marks beside its span, and
 which reasons exist

 @returns The rendering

 @throws {@link ArtifactParseError} when it is the wrong shape or kind, or a
 key, an offset, a mark or a reason belongs to the other generation, or a mark
 is not one single mark

 @example
 ```ts
 const rendering = parseConsistencyRendering({ value: row.rendering, path: `${path}.rendering`, generation: 16, },);
 ```
 */
export function parseConsistencyRendering(
  {
    value,
    path,
    generation,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): ArtifactConsistencyRendering {
  /**
   Whether the section has the sixteenth shape.
   */
  const spanned = artifactGenerationCountsBySpan({ generation, },);
  /**
   The rendering as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  /**
   Whether the place's rendering was located.
   */
  const kind = requireOneOf({
    value: record.kind,
    allowed: [
      'located',
      'unlocated',
    ],
    path: `${path}.kind`,
  },);
  if (kind === 'unlocated') {
    requireExactKeys({
      record,
      allowed: [
        'kind',
        'reason',
      ],
      path,
    },);
    /**
     Why the rendering was not located.
     */
    const reason = requireKeyOf({
      value: record.reason,
      record: UNLOCATED_REASONS,
      path: `${path}.reason`,
    },);
    if (spanned && SLOT_REASONS.has(reason,)) {
      throw new ArtifactParseError({
        path: `${path}.reason`,
        reason: 'a reason generation sixteen gives, which locates a place on its quote and not its slot',
      },);
    }
    return {
      kind,
      reason,
    };
  }
  requireExactKeys({
    record,
    allowed: locatedKeysOf({ generation, },),
    path,
  },);
  return {
    kind,
    quote: requireString({
      value: record.quote,
      path: `${path}.quote`,
    },),
    ...(spanned
      ? {
        start: requireCount({
          value: record.start,
          path: `${path}.start`,
        },),
      }
      : {}),
    ...((record.markBefore === undefined)
      ? {}
      : {
        markBefore: requireSingleMark({
          value: record.markBefore,
          path: `${path}.markBefore`,
        },),
      }),
    ...((record.markAfter === undefined)
      ? {}
      : {
        markAfter: requireSingleMark({
          value: record.markAfter,
          path: `${path}.markAfter`,
        },),
      }),
    ...((record.slot === undefined)
      ? {}
      : {
        slot: requireString({
          value: record.slot,
          path: `${path}.slot`,
        },),
      }),
    ...((record.definitions === undefined)
      ? {}
      : {
        definitions: requireDefinitions({
          value: record.definitions,
          path: `${path}.definitions`,
        },),
      }),
  };
}

//endregion Artifact consistency rendering reading
