import {
  requireExactKeys,
  requireKeyOf,
  requireOneOf,
} from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
  requireStringList,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { UnplacedReason, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type {
  ArtifactConsistencyBallot,
  ArtifactConsistencyChoiceBallot,
  ArtifactConsistencyRepeat,
  ArtifactConsistencyUnplaced,
} from './artifact-two-lane-consistency.ts';

//region Artifact consistency parts
// The shapes inside one recorded group, each read key by key. What the parts
// say about each other is the group reader's question
// (`artifact-two-lane-read-consistency.ts`).

/**
 Every reason a place can stand in no slice, keyed by the union so the compiler
 refuses a member the record lacks.
 */
const UNPLACED_REASONS: Readonly<Record<UnplacedReason, true>> = {
  'in-no-slice': true,
  'crosses-slice-end': true,
};

/**
 Reads the repeated wording or frame of a group.

 @param value - recorded repeat

 @param path - where it sits, for the parse error

 @returns The repeat

 @throws {@link ArtifactParseError} when it is the wrong shape or kind

 @example
 ```ts
 const repeat = parseConsistencyRepeat({ value: group.repeat, path: `${path}.repeat`, },);
 ```
 */
export function parseConsistencyRepeat(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsistencyRepeat {
  /**
   The repeat as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  /**
   Which kind of repeat it is.
   */
  const kind = requireOneOf({
    value: record.kind,
    allowed: [
      'maximal',
      'refrain',
    ],
    path: `${path}.kind`,
  },);
  if (kind === 'maximal') {
    requireExactKeys({
      record,
      allowed: [
        'kind',
        'wording',
      ],
      path,
    },);
    return {
      kind,
      wording: requireString({
        value: record.wording,
        path: `${path}.wording`,
      },),
    };
  }
  requireExactKeys({
    record,
    allowed: [
      'kind',
      'prefix',
      'suffix',
    ],
    path,
  },);
  return {
    kind,
    prefix: requireString({
      value: record.prefix,
      path: `${path}.prefix`,
    },),
    suffix: requireString({
      value: record.suffix,
      path: `${path}.suffix`,
    },),
  };
}

/**
 Refuses a place that does not end after it starts or that stands on a line
 before the first, which no repeat the detector finds does.

 SHARED BY THE TWO READERS OF A PLACE, the one a slice holds whole and the one
 no slice does, so the rule is stated once.

 @param startOffset - offset where the place starts

 @param endOffset - exclusive offset where it ends

 @param line - one-based line it starts on

 @param path - where the place sits, for the parse error

 @throws {@link ArtifactParseError} when the place ends at or before its start, or its line is zero

 @example
 ```ts
 requirePlaceBounds({ startOffset: 6, endOffset: 18, line: 3, path: `${path}.unplaced[0]`, },);
 ```
 */
export function requirePlaceBounds(
  {
    startOffset,
    endOffset,
    line,
    path,
  }: {
    readonly startOffset: number;
    readonly endOffset: number;
    readonly line: number;
    readonly path: string;
  },
): void {
  if ((endOffset <= startOffset) || (line < 1)) {
    throw new ArtifactParseError({
      path,
      reason: 'a place that ends after it starts, on a line counted from one',
    },);
  }
}

/**
 Reads one place no slice holds whole.

 @param value - recorded place

 @param path - where it sits, for the parse error

 @returns The place

 @throws {@link ArtifactParseError} when it is the wrong shape, or ends at or
 before its start, or its line is zero

 @example
 ```ts
 const unplaced = parseConsistencyUnplaced({ value, path: `${path}.unplaced[0]`, },);
 ```
 */
export function parseConsistencyUnplaced(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsistencyUnplaced {
  /**
   The place as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'startOffset',
      'endOffset',
      'line',
      'reason',
    ],
    path,
  },);
  /**
   Where the place starts and ends in the original.
   */
  const startOffset = requireCount({
    value: record.startOffset,
    path: `${path}.startOffset`,
  },);
  /**
   Exclusive end of the place in the original.
   */
  const endOffset = requireCount({
    value: record.endOffset,
    path: `${path}.endOffset`,
  },);
  /**
   One-based line.
   */
  const line = requireCount({
    value: record.line,
    path: `${path}.line`,
  },);
  requirePlaceBounds({
    startOffset,
    endOffset,
    line,
    path,
  },);
  return {
    startOffset,
    endOffset,
    line,
    reason: requireKeyOf({
      value: record.reason,
      record: UNPLACED_REASONS,
      path: `${path}.reason`,
    },),
  };
}

/**
 Reads one heard ballot.

 @param value - recorded ballot

 @param path - where it sits, for the parse error

 @returns The ballot

 @throws {@link ArtifactParseError} when it is the wrong shape

 @example
 ```ts
 const ballot = parseConsistencyBallot({ value, path: `${path}.ballots[0]`, },);
 ```
 */
export function parseConsistencyBallot(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsistencyBallot {
  /**
   The ballot as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'seat',
      'reason',
      'quotes',
      'choice',
      'keep',
    ],
    path,
  },);
  return {
    seat: requireString({
      value: record.seat,
      path: `${path}.seat`,
    },),
    reason: requireString({
      value: record.reason,
      path: `${path}.reason`,
    },),
    quotes: requireArray({
      value: record.quotes,
      path: `${path}.quotes`,
    },)
      .map(function toQuote(
        entry,
        at,
      ): ArtifactConsistencyBallot['quotes'][number] {
        /**
         Where the quote sits, for the parse error.
         */
        const quotePath = `${path}.quotes[${String(at,)}]`;
        /**
         The quote as a record.
         */
        const quote = requireRecord({
          value: entry,
          path: quotePath,
        },);
        requireExactKeys({
          record: quote,
          allowed: [
            'label',
            'quote',
            'slot',
          ],
          path: quotePath,
        },);
        return {
          label: requireString({
            value: quote.label,
            path: `${quotePath}.label`,
          },),
          quote: requireString({
            value: quote.quote,
            path: `${quotePath}.quote`,
          },),
          ...((quote.slot === undefined)
            ? {}
            : {
              slot: requireString({
                value: quote.slot,
                path: `${quotePath}.slot`,
              },),
            }),
        };
      },),
    choice: requireString({
      value: record.choice,
      path: `${path}.choice`,
    },),
    keep: requireStringList({
      value: record.keep,
      path: `${path}.keep`,
    },),
  };
}

/**
 Reads one heard ballot that only chooses: the run-off's and a carry's.

 @param value - recorded ballot

 @param path - where it sits, for the parse error

 @returns The ballot

 @throws {@link ArtifactParseError} when it is the wrong shape

 @example
 ```ts
 const ballot = parseConsistencyChoiceBallot({ value, path: `${path}.ballots[0]`, },);
 ```
 */
export function parseConsistencyChoiceBallot(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsistencyChoiceBallot {
  /**
   The ballot as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'seat',
      'reason',
      'choice',
    ],
    path,
  },);
  return {
    seat: requireString({
      value: record.seat,
      path: `${path}.seat`,
    },),
    reason: requireString({
      value: record.reason,
      path: `${path}.reason`,
    },),
    choice: requireString({
      value: record.choice,
      path: `${path}.choice`,
    },),
  };
}

//endregion Artifact consistency parts
