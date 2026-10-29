import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type {
  ForcedAlignStep,
  InsertionAnchor,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { headingAffinity, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { SectionPair, } from './pair-sections-wire.ts';

//region Section pairing steps
// TRANSLATES A ROSTER'S SECTION PAIRING INTO THE VOCABULARY `alignDocumentSections`
// ALREADY READS, so nothing downstream of alignment learns that a model was
// involved. `pair-blocks-steps.ts` does the same job one scale down.
//
// THE PAIRING IS WHAT PROVES AN INSERTION BOUNDARY. Section-scale insertion can
// place an untranslated section only where every optimal alignment agrees it belongs,
// and measured over the pinned corpus the deterministic aligner never agrees
// with itself about anything: all 11 unpaired source sections come back
// `ambiguous`, so `may-pair` refuses every one and the insertion path emits
// nothing. Two paired sections either side of an unpaired one pin it to the
// span between them, and when that span is one boundary wide the anchor is
// PROVEN in exactly the sense the deterministic path meant by it.
//
// AFFINITY IS REPORTED, NOT INVENTED. A `paired` step carries the affinity of
// its pairing, and a roster pairing has whatever affinity the two headings
// actually share, which on these entries is 0.00. Recording the measurement is
// the point: it is why a model was asked.

/**
 Reason a section carries when the roster was asked about it and left it out.

 Not `forced-gap`, which means no optimal alignment pairs it, and not
 `ambiguous`, which means several do. Those describe a scorer's table. This
 describes a reading, and the two disagree often enough that collapsing them
 would hide which mechanism produced an unpaired section.
 */
const ROSTER_UNPAIRED = 'roster-unpaired';

/**
 Target index standing for "no paired section on that side of this one".

 Before the first target rather than a nullish union, so the boundary
 arithmetic reads the same whether or not a paired neighbour exists.
 */
const BEFORE_FIRST_TARGET = -1;

/**
 Where an unpaired original section could sit, given its paired neighbours.

 @param previousTarget - translation section the nearest EARLIER paired
 original renders as, or {@link BEFORE_FIRST_TARGET} when none precedes it

 @param nextTarget - translation section the nearest LATER paired original
 renders as, or the translation section count when none follows it

 @returns Proven boundary when the neighbours pin exactly one, the whole span
 otherwise

 @example
 ```ts
 const anchor = anchorBetween({ previousTarget: 2, nextTarget: 3, },);
 ```
 */
function anchorBetween(
  {
    previousTarget,
    nextTarget,
  }: {
    readonly previousTarget: number;
    readonly nextTarget: number;
  },
): InsertionAnchor {
  /**
   Every boundary the section could be written at, in document order.

   It sits after whatever the earlier neighbour renders as and before whatever
   the later one does, so each unclaimed translation section between them is
   one more place it could go.
   */
  const boundaries: number[] = [];
  for (let at = previousTarget + 1; at <= nextTarget; at += 1)
    boundaries.push(at,);

  /**
   The single place, when the neighbours agree on one.
   */
  const [only,] = boundaries;
  if ((boundaries.length === 1) && (only !== undefined))
    return {
      kind: 'proven',
      beforeTargetIndex: only,
    };

  return {
    kind: 'several-boundaries',
    boundaries,
  };
}

/**
 Translation section each original renders as, at every original index.

 @param pairs - correspondences the roster agreed on

 @param sourceCount - original sections

 @returns Target index per original, {@link BEFORE_FIRST_TARGET} where unpaired

 @example
 ```ts
 const targets = targetsBySource({ pairs, sourceCount: 8, },);
 ```
 */
function targetsBySource(
  {
    pairs,
    sourceCount,
  }: {
    readonly pairs: readonly SectionPair[];
    readonly sourceCount: number;
  },
): readonly number[] {
  /**
   Pairs keyed by their original index, which the reader has already proved
   unique.
   */
  const bySource = new Map(pairs.map(function keyed(pair,): readonly [
    number,
    number,
  ] {
    return [
      pair.source,
      pair.target,
    ];
  },),);

  /**
   One entry per original section, whether or not it was paired.
   */
  const targets: number[] = [];
  for (let source = 0; source < sourceCount; source += 1)
    targets.push(bySource.get(source,) ?? BEFORE_FIRST_TARGET,);
  return targets;
}

/**
 Nearest paired target passed before each section, in scan order.

 ONE SCAN in whichever direction the caller hands it, because "the paired
 neighbour before this section" and "the paired neighbour after it" are the
 same walk read from opposite ends.

 @param targets - target per original, in the order to scan

 @param start - answer for the first section scanned, which is where a section
 with no paired neighbour on that side belongs

 @returns Nearest paired target per section, in the order scanned

 @example
 ```ts
 const nearest = scanNearest({ targets, start: BEFORE_FIRST_TARGET, },);
 ```
 */
function scanNearest(
  {
    targets,
    start,
  }: {
    readonly targets: readonly number[];
    readonly start: number;
  },
): readonly number[] {
  // ONE PASS, appending: a fold that copies the answers so far at every section
  // does quadratic work for a linear walk.
  /**
   Answer per section reached so far, in scan order.
   */
  const found: number[] = [];
  /**
   Paired target most recently passed.
   */
  let nearest = start;
  for (const target of targets) {
    found.push(nearest,);
    if (target !== BEFORE_FIRST_TARGET)
      nearest = target;
  }
  return found;
}

/**
 Converts a roster's section pairing into the aligner's step vocabulary.

 @param pairs - correspondences the roster agreed on, strictly increasing on
 both sides

 @param sourceHeadings - original section labels in document order, which also
 count the sections

 @param targetHeadings - translation section labels in document order

 @returns One step per original section, then every unclaimed translation
 section, matching what `alignHeadingsForced` emits

 @example
 ```ts
 const steps = sectionPairingToSteps({ pairs, sourceHeadings, targetHeadings, },);
 ```
 */
export function sectionPairingToSteps(
  {
    pairs,
    sourceHeadings,
    targetHeadings,
  }: {
    readonly pairs: readonly SectionPair[];
    readonly sourceHeadings: readonly string[];
    readonly targetHeadings: readonly string[];
  },
): readonly ForcedAlignStep[] {
  /**
   Translation section each original renders as, or its absence.
   */
  const targets = targetsBySource({
    pairs,
    sourceCount: sourceHeadings.length,
  },);

  /**
   Paired neighbour BEFORE every original section.
   */
  const previous = scanNearest({
    targets,
    start: BEFORE_FIRST_TARGET,
  },);

  /**
   Paired neighbour AFTER every original section, scanned from the end and
   turned around so it reads in document order beside `previous`.
   */
  const next = scanNearest({
    targets: targets.toReversed(),
    start: targetHeadings.length,
  },)
    .toReversed();

  /**
   Translation sections some original claims.
   */
  const claimed = new Set(pairs.map(function toTarget(pair,): number {
    return pair.target;
  },),);

  // ONE ENTRY PER ORIGINAL SECTION in `targets`, `previous` and `next`, which
  // `sourceHeadings` counts, and a paired target names a translation section:
  // the stage read each pair against these sides' counts (`readSectionPairing`).
  /**
   Decisions about the original's sections, in document order.
   */
  const sourceSteps = targets.map(function toStep(
    target,
    source,
  ): ForcedAlignStep {
    if (target !== BEFORE_FIRST_TARGET)
      return {
        kind: 'paired',
        sourceIndex: source,
        targetIndex: target,
        affinity: headingAffinity({
          source: nonNullishOrThrow(sourceHeadings[source],),
          target: nonNullishOrThrow(targetHeadings[target],),
        },),
      };

    return {
      kind: 'source-only',
      sourceIndex: source,
      reason: ROSTER_UNPAIRED,
      anchor: anchorBetween({
        previousTarget: nonNullishOrThrow(previous[source],),
        nextTarget: nonNullishOrThrow(next[source],),
      },),
    };
  },);

  /**
   Translation sections no original claims, emitted after every original
   decision exactly as the deterministic aligner emits them.
   */
  const targetSteps = targetHeadings
    .map(function toIndex(
      _heading,
      target,
    ): number {
      return target;
    },)
    .filter(function unclaimed(target,): boolean {
      return !claimed.has(target,);
    },)
    .map(function toStep(target,): ForcedAlignStep {
      return {
        kind: 'target-only',
        targetIndex: target,
        reason: ROSTER_UNPAIRED,
      };
    },);

  return [
    ...sourceSteps,
    ...targetSteps,
  ];
}

//endregion Section pairing steps
