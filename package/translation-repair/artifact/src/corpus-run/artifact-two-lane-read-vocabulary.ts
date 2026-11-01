import {
  type ArtifactDecisionComparison,
  type ArtifactDeliveryRow,
  type ArtifactLaneRelation,
  ArtifactParseError,
  type ArtifactSliceDelivery,
  type ArtifactSliceOutcome,
  requireArray,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  requireExactKeys,
  requireKeyOf,
} from '../artifact-exact-guard.ts';

//region Artifact version 2 union parsing
// Reading the three small unions a version 2 row dispatches on, back off disk.
//
// EVERY MEMBER'S KEYS ARE LISTED, per member rather than per union, because
// that is the check the shape needs: `{ kind: 'not-evaluated', acceptedText }`
// is not a slice this version can describe, and a parser reading the
// discriminator and then taking whatever fields it recognizes would accept it
// and hand a reader an outcome carrying a wording nobody decided.
//
// AN UNKNOWN DISCRIMINATOR IS ALWAYS REFUSED, in both `unknownKeys` modes. It names a
// member this version cannot project into any of its own, so there is no
// tolerant reading of it: taking the row anyway would mean recording a slice
// under a name this reader made up.

/**
 What a reader does about keys the version does not name here.

 @example
 ```ts
 const unknownKeys: UnknownKeyPolicy = 'tolerate';
 ```
 */
export type UnknownKeyPolicy =
  /**
   Refuse them, which is right everywhere version 2 owns the shape.
   */
  | 'refuse'
  /**
   Take the fields this version names and leave the rest, which is right
   inside a raw lane result: those are typed by the live pipeline, they grow
   by addition, and a later field there is not a later version here.
   */
    | 'tolerate';

/**
 Keys each outcome member may carry, including its own discriminator.
 */
const OUTCOME_KEYS: Readonly<Record<ArtifactSliceOutcome['kind'], readonly string[]>> = {
  decided: [
    'kind',
    'acceptedText',
  ],
  'not-evaluated': ['kind',],
  unfilled: ['kind',],
  'incumbent-fallback': ['kind',],
  'not-applicable': ['kind',],
};

/**
 Whether the archive holds any wording at a slice, as version 2 records it.

 KEYED BY THE FROZEN `ArtifactDeliveryRow` field rather than listed, so the
 compiler refuses a member the vocabulary file gains that this reader lacks.
 */
export const INCUMBENT_KINDS: Readonly<Record<ArtifactDeliveryRow['incumbentKind'], true>> = {
  present: true,
  absent: true,
};

/**
 How the two documents relate at one slice, as version 2 records it, keyed by
 the frozen `ArtifactLaneRelation` for the reason `INCUMBENT_KINDS` gives.
 */
export const LANE_RELATIONS: Readonly<Record<ArtifactLaneRelation, true>> = {
  'archive-stands': true,
  'repair-only': true,
  'translate-only': true,
  'both-agree': true,
  'both-differ': true,
  'gap-remains': true,
};

/**
 Members a delivery may name, keyed by the frozen `ArtifactSliceDelivery`
 for the reason `INCUMBENT_KINDS` gives.
 */
const DELIVERY_KINDS: Readonly<Record<ArtifactSliceDelivery['kind'], true>> = {
  'replacement-shipped': true,
  'replacement-withdrawn': true,
  'incumbent-retained': true,
  'gap-remains': true,
};

/**
 Mechanisms that can take a replacement back, keyed by the frozen
 `ArtifactSliceDelivery` for the reason `INCUMBENT_KINDS` gives.
 */
const WITHDRAWAL_REASONS: Readonly<Record<
  Extract<ArtifactSliceDelivery, { readonly kind: 'replacement-withdrawn'; }>['reason'],
  true
  >> = {
    'assembly-integrity': true,
    'blocked-non-translation': true,
  };

/**
 Members a decision comparison may name, keyed by the frozen
 `ArtifactDecisionComparison` for the reason `INCUMBENT_KINDS` gives.
 */
const COMPARISON_KINDS: Readonly<Record<ArtifactDecisionComparison['kind'], true>> = {
  comparable: true,
  'not-comparable': true,
};

/**
 Verdicts a comparable pair may carry, keyed by the frozen
 `ArtifactDecisionComparison` for the reason `INCUMBENT_KINDS` gives.
 */
const COMPARISON_VERDICTS: Readonly<Record<
  Extract<ArtifactDecisionComparison, { readonly kind: 'comparable'; }>['verdict'],
  true
  >> = {
    same: true,
    different: true,
  };

/**
 Lanes an undecided pair may name, keyed by the frozen
 `ArtifactDecisionComparison` for the reason `INCUMBENT_KINDS` gives.
 */
const UNDECIDED_LANE_NAMES: Readonly<Record<
  Extract<ArtifactDecisionComparison, { readonly kind: 'not-comparable'; }>['undecidedLanes'][number],
  true
  >> = {
    repair: true,
    translate: true,
  };

/**
 Fields this version gives a MEANING to on some outcome member.

 Checked even where unknown keys are tolerated, because the two cases are not
 alike: a field version 2 never heard of is a later pipeline adding evidence,
 while `acceptedText` on a member that decided nothing is this version's own
 vocabulary used to say something it cannot mean.
 */
const RESERVED_OUTCOME_KEYS: readonly string[] = ['acceptedText',];

/**
 Reads what a lane did about one slice.

 @param value - outcome JSON

 @param unknownKeys - what to do about keys this version does not name, which
 differs between the ledger, whose shape version 2 owns, and a raw lane
 result, which the live pipeline owns

 @param path - dotted path for error message

 @param benchFloorRecorded - whether the generation records the mark of an unfilled outcome the select bench never
 bought a candidate for

 @returns Outcome as version 2 describes it

 @throws {@link ArtifactParseError} when the discriminator names no member of
 this version, when a member carries a field belonging to another, when the mark is anything but true, or when a
 decision carries no wording

 @example
 ```ts
 const outcome = parseSliceOutcome({ value, unknownKeys: 'refuse', path, },);
 ```
 */
export function parseSliceOutcome(
  {
    value,
    unknownKeys,
    path,
    benchFloorRecorded = false,
  }: {
    readonly value: unknown;
    readonly unknownKeys: UnknownKeyPolicy;
    readonly path: string;
    readonly benchFloorRecorded?: boolean;
  },
): ArtifactSliceOutcome {
  /**
   Outcome as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);

  /**
   Member it names.
   */
  const kind = requireKeyOf({
    value: record.kind,
    record: OUTCOME_KEYS,
    path: `${path}.kind`,
  },);

  /**
   Keys this member may carry.
   */
  const allowed = ((kind === 'unfilled') && benchFloorRecorded)
    ? [
      ...OUTCOME_KEYS[kind],
      'benchBelowFloor',
    ]
    : OUTCOME_KEYS[kind];
  if (unknownKeys === 'refuse') {
    requireExactKeys({
      record,
      allowed,
      path,
    },);
  } else {
    /**
     Reserved field this member has no meaning for, or nothing.
     */
    const misplaced = RESERVED_OUTCOME_KEYS.find(function isMisplaced(key,): boolean {
      return (key in record) && (!allowed.includes(key,));
    },);
    if (misplaced !== undefined) {
      throw new ArtifactParseError({
        path: `${path}.${misplaced}`,
        reason: `no ${misplaced} on an outcome of kind ${kind}, which decided nothing`,
      },);
    }
  }
  if (kind === 'decided') {
    return {
      kind,
      acceptedText: requireString({
        value: record.acceptedText,
        path: `${path}.acceptedText`,
      },),
    };
  }
  if (kind === 'unfilled') {
    if (record.benchBelowFloor === undefined)
      return { kind, };
    if (benchFloorRecorded && (record.benchBelowFloor === true)) {
      return {
        kind,
        benchBelowFloor: true,
      };
    }
    if (benchFloorRecorded) {
      throw new ArtifactParseError({
        path: `${path}.benchBelowFloor`,
        reason: 'true or absent',
      },);
    }
  }
  return { kind, };
}

/**
 Reads how one lane's document came to carry what it carries.

 @param value - delivery JSON

 @param path - dotted path for error message

 @returns Delivery as version 2 describes it

 @throws {@link ArtifactParseError} when the discriminator names no member of
 this version, when a member carries a key belonging to another, or when a
 withdrawal names no mechanism

 @example
 ```ts
 const delivery = parseSliceDelivery({ value, path, },);
 ```
 */
export function parseSliceDelivery(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactSliceDelivery {
  /**
   Delivery as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);

  /**
   Member it names.
   */
  const kind = requireKeyOf({
    value: record.kind,
    record: DELIVERY_KINDS,
    path: `${path}.kind`,
  },);
  if (kind === 'replacement-withdrawn') {
    requireExactKeys({
      record,
      allowed: [
        'kind',
        'reason',
      ],
      path,
    },);
    return {
      kind,
      reason: requireKeyOf({
        value: record.reason,
        record: WITHDRAWAL_REASONS,
        path: `${path}.reason`,
      },),
    };
  }
  requireExactKeys({
    record,
    allowed: ['kind',],
    path,
  },);
  return { kind, };
}

/**
 Reads whether the two lanes' own decisions were comparable.

 @param value - decision comparison JSON

 @param path - dotted path for error message

 @returns Decision comparison as version 2 describes it

 @throws {@link ArtifactParseError} when the discriminator names no member of
 this version, when a member carries a key belonging to another, or when an
 undecided lane is named something other than a lane

 @example
 ```ts
 const decisions = parseDecisionComparison({ value, path, },);
 ```
 */
export function parseDecisionComparison(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactDecisionComparison {
  /**
   Comparison as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);

  /**
   Member it names.
   */
  const kind = requireKeyOf({
    value: record.kind,
    record: COMPARISON_KINDS,
    path: `${path}.kind`,
  },);
  if (kind === 'comparable') {
    requireExactKeys({
      record,
      allowed: [
        'kind',
        'verdict',
      ],
      path,
    },);
    return {
      kind,
      verdict: requireKeyOf({
        value: record.verdict,
        record: COMPARISON_VERDICTS,
        path: `${path}.verdict`,
      },),
    };
  }
  requireExactKeys({
    record,
    allowed: [
      'kind',
      'undecidedLanes',
    ],
    path,
  },);
  return {
    kind,
    undecidedLanes: requireArray({
      value: record.undecidedLanes,
      path: `${path}.undecidedLanes`,
    },)
      .map(function readLane(
        lane,
        position,
      ) {
        return requireKeyOf({
          value: lane,
          record: UNDECIDED_LANE_NAMES,
          path: `${path}.undecidedLanes[${String(position,)}]`,
        },);
      },),
  };
}

//endregion Artifact version 2 union parsing
