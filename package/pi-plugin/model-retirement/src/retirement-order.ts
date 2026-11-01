/**
 Recency ordering between two model-id parses.

 Ordering is the only evidence a retirement rests on, so every branch here either
 decides from id text or declares the pair unordered. Absence is modelled with the
 {@link UNORDERED} sentinel rather than a nullish union, matching the convention
 `package/agent-harness-shared/model-selection/src/version.ts` uses for `NO_MAJOR_VERSION`.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  isDateShapedRaw,
  type ModelIdParse,
} from './id-tokens.ts';

//region Constants

/**
 Digit-length gap between two version components that marks them as belonging to
 different numbering schemes.

 A month-day pair such as `02-15` meets an eight-digit snapshot such as `20260420` in
 `qwen3.5-plus`, and comparing those numerically would order a date against a month.
 */
const INCOMMENSURABLE_DIGIT_GAP = 2;

/**
 Sentinel for a pair whose id text carries no ordering evidence.
 */
export const UNORDERED: unique symbol = Symbol('model ids carry no ordering evidence',);

//endregion Constants

//region Types

/**
 Recency comparison outcome between two parses.
 */
export type Recency = 'left' | 'right' | typeof UNORDERED;

//endregion Types

//region Component comparison

/**
 Compare two version components for scheme compatibility.

 @param leftRaw - raw text of the left component, leading zeros intact

 @param rightRaw - raw text of the right component

 @returns whether the components use numbering schemes close enough to order

 @example
 ```typescript
 isCommensurable({ leftRaw: '2', rightRaw: '20260420' }); // false
 ```
 */
function isCommensurable(
  {
    leftRaw,
    rightRaw,
  }: {
    readonly leftRaw: string;
    readonly rightRaw: string;
  },
): boolean {
  return Math.abs(leftRaw.length - rightRaw.length) < INCOMMENSURABLE_DIGIT_GAP;
}

/**
 Compare two aligned version components.

 @param left - parse of the left entry

 @param right - parse of the right entry

 @param index - component position both parses cover

 @returns which side is newer at this position, or {@link UNORDERED} when the
 components are equal or belong to different numbering schemes

 @example
 ```typescript
 compareComponentAt({ left, right, index: 1 });
 ```
 */
function compareComponentAt(
  {
    left,
    right,
    index,
  }: {
    readonly left: ModelIdParse;
    readonly right: ModelIdParse;
    readonly index: number;
  },
): Recency {
  /**
   Left component at this position.
   */
  const leftPart = nonNullishOrThrow(left.versionParts[index],);
  /**
   Right component at this position.
   */
  const rightPart = nonNullishOrThrow(right.versionParts[index],);
  if (leftPart === rightPart)
    return UNORDERED;
  /**
   Raw text of both components, which decides whether they are comparable at all.
   */
  const commensurable = isCommensurable({
    leftRaw: nonNullishOrThrow(left.versionRaws[index],),
    rightRaw: nonNullishOrThrow(right.versionRaws[index],),
  },);
  if (!commensurable)
    return UNORDERED;
  return leftPart > rightPart ? 'left' : 'right';
}

/**
 Outcome of comparing the positions two version vectors share.
 */
type SharedOutcome = {
  /**
   `decided` when a differing pair ordered the entries, `unordered` when a differing
   pair belongs to two numbering schemes, and `tied` when every shared position matched.
   */
  readonly kind: 'decided' | 'unordered' | 'tied';
  /**
   Which side is newer, present only when the kind is `decided`.
   */
  readonly recency?: 'left' | 'right';
};

/**
 Find the first position where two version vectors differ.

 @param left - parse of the left entry

 @param right - parse of the right entry

 @returns whether a shared position decided the order, refused to, or matched

 @example
 ```typescript
 compareSharedComponents({ left, right }); // { kind: 'tied' }
 ```
 */
function compareSharedComponents(
  {
    left,
    right,
  }: {
    readonly left: ModelIdParse;
    readonly right: ModelIdParse;
  },
): SharedOutcome {
  /**
   Count of positions both vectors cover.
   */
  const shared = Math.min(
    left.versionParts
      .length,
    right.versionParts
      .length,
  );
  for (let index = 0; index < shared; index += 1) {
    /**
     Left component at this position.
     */
    const leftPart = nonNullishOrThrow(left.versionParts[index],);
    /**
     Right component at this position.
     */
    const rightPart = nonNullishOrThrow(right.versionParts[index],);
    if (leftPart === rightPart)
      continue;
    /**
     Comparison at the first differing position.
     */
    const outcome = compareComponentAt({
      left,
      right,
      index,
    },);
    if ((outcome === 'left') || (outcome === 'right'))
      return {
        kind: 'decided',
        recency: outcome,
      };
    return { kind: 'unordered', };
  }
  return { kind: 'tied', };
}

//endregion Component comparison

//region Date and prefix comparison

/**
 Compare two date sequences of equal length.

 @param leftDates - raw date components of the left parse

 @param rightDates - raw date components of the right parse

 @returns which side is more recent, or {@link UNORDERED} when every date matches

 @example
 ```typescript
 compareDateSequences({ leftDates: ['2407'], rightDates: ['2512'] }); // 'right'
 ```
 */
function compareDateSequences(
  {
    leftDates,
    rightDates,
  }: {
    readonly leftDates: readonly string[];
    readonly rightDates: readonly string[];
  },
): Recency {
  for (let index = 0; index < leftDates.length; index += 1) {
    /**
     Left date at this position.
     */
    const left = Math.trunc(
      Number(nonNullishOrThrow(leftDates[index],),),
    );
    /**
     Right date at this position.
     */
    const right = Math.trunc(
      Number(nonNullishOrThrow(rightDates[index],),),
    );
    if (left === right)
      continue;
    return left > right ? 'left' : 'right';
  }
  return UNORDERED;
}

/**
 Decide between two parses whose version vectors are identical.

 A parse carrying no date is a rolling alias, which stays available longer than any
 pinned snapshot of the same version.

 @param left - parse of the left entry

 @param right - parse of the right entry

 @returns which side is more recent, or {@link UNORDERED} when both carry the same
 dates or date shapes that cannot be aligned

 @example
 ```typescript
 compareEqualLengthDates({ left, right }); // 'left' when the left parse is the alias
 ```
 */
function compareEqualLengthDates(
  {
    left,
    right,
  }: {
    readonly left: ModelIdParse;
    readonly right: ModelIdParse;
  },
): Recency {
  /**
   Whether both parses carry exactly the same date components.
   */
  const sameDates = (left.dateRaws
    .length
    === right.dateRaws
      .length)
    && left.dateRaws
      .every(function matchesSameDate(
        raw,
        index
      ) {
        return raw === right.dateRaws[index];
      },);
  if (sameDates)
    return UNORDERED;
  if (left.dateRaws
    .length
    === 0)
    return 'left';
  if (right.dateRaws
    .length
    === 0)
    return 'right';
  if (left.dateRaws
    .length
    !== right.dateRaws
      .length)
    return UNORDERED;
  return compareDateSequences({
    leftDates: left.dateRaws,
    rightDates: right.dateRaws,
  },);
}

/**
 Decide a prefix case, where one version vector continues the other.

 Extra components that are all dates mark a pinned snapshot, which loses to the
 rolling alias. Extra components that are all versions mark a point release, which
 beats its own base version. A mix of the two leaves the pair unordered.

 @param shorter - parse carrying the prefix

 @param longer - parse continuing that prefix

 @returns which parse is more recent, or {@link UNORDERED} when the extras mix schemes

 @example
 ```typescript
 comparePrefix({ shorter: parseModelId('claude-opus-5'), longer: parseModelId('claude-opus-5-5') }); // 'longer'
 ```
 */
function comparePrefix(
  {
    shorter,
    longer,
  }: {
    readonly shorter: ModelIdParse;
    readonly longer: ModelIdParse;
  },
): 'shorter' | 'longer' | typeof UNORDERED {
  /**
   Raw components the longer parse carries past the shared prefix.
   */
  const extras = longer.versionRaws
    .slice(shorter.versionRaws
      .length,);
  /**
   Whether every extra component reads as a calendar date.
   */
  const allDates = extras.every(function extraIsDate(raw,) {
    return isDateShapedRaw(raw,);
  },);
  if (allDates)
    return 'shorter';
  /**
   Whether no extra component reads as a calendar date.
   */
  const noDates = extras.every(function extraIsNotDate(raw,) {
    return !isDateShapedRaw(raw,);
  },);
  if (noDates)
    return 'longer';
  return UNORDERED;
}

//endregion Date and prefix comparison

//region Entry point

/**
 Order two parses by recency.

 @param left - parse of the left entry

 @param right - parse of the right entry

 @returns `'left'` when the left entry is newer, `'right'` when the right entry is
 newer, and {@link UNORDERED} when the id text cannot order them

 @example
 ```typescript
 compareRecency({ left: parseModelId('glm-5.2'), right: parseModelId('glm-5.3') }); // 'right'
 ```
 */
export function compareRecency(
  {
    left,
    right,
  }: {
    readonly left: ModelIdParse;
    readonly right: ModelIdParse;
  },
): Recency {
  if (left.nameShape !== right.nameShape)
    return UNORDERED;
  if ((left.versionParts
    .length
    === 0) || (right.versionParts
      .length
      === 0))
    return UNORDERED;
  /**
   Ordering decided by the first differing shared component, when there is one.
   */
  const shared = compareSharedComponents({
    left,
    right,
  },);
  if (shared.kind === 'decided')
    return nonNullishOrThrow(shared.recency,);
  if (shared.kind === 'unordered')
    return UNORDERED;
  if (left.versionParts
    .length
    === right.versionParts
      .length)
    return compareEqualLengthDates({
      left,
      right,
    },);
  /**
   Whether the left parse carries the shorter version vector.
   */
  const shorterIsLeft = left.versionParts
    .length
    < right.versionParts
      .length;
  /**
   Prefix comparison expressed in shorter and longer terms.
   */
  const prefixOutcome = shorterIsLeft
    ? comparePrefix({
      shorter: left,
      longer: right,
    },)
    : comparePrefix({
      shorter: right,
      longer: left,
    },);
  if (prefixOutcome === UNORDERED)
    return UNORDERED;
  if (prefixOutcome === 'shorter')
    return shorterIsLeft ? 'left' : 'right';
  return shorterIsLeft ? 'right' : 'left';
}

//endregion Entry point
