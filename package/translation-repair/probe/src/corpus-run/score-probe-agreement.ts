import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ProbeAgreement,
  ProbeAgreementItem,
  SampleManifest,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { GradedRepairItem, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { GatheredProbe, } from './probe-telemetry-report.ts';

//region Score probe agreement
// Joins the human repair grades to the probe's readings by sheet position and
// prints how often the two agree.

/**
 Pairs each graded issue with the probe reading of the same issue.

 @param manifest - draw manifest, the only record of which issue sat at which position

 @param graded - human verdicts in sheet order

 @param byIssueId - probe readings keyed by the issue whose record carried them

 @returns One item per manifest position, carrying its reading when the issue was probed

 @throws {@link StatedRefusalError} when the sheet and the manifest differ in
 length, since joining them by position would mislabel every verdict after the
 first divergence

 @example
 ```ts
 const items = joinProbeGrades({ manifest, graded, byIssueId: gathered.byIssueId, },);
 ```
 */
export function joinProbeGrades(
  {
    manifest,
    graded,
    byIssueId,
  }: {
    readonly manifest: SampleManifest;
    readonly graded: readonly GradedRepairItem[];
    readonly byIssueId: GatheredProbe['byIssueId'];
  },
): readonly ProbeAgreementItem[] {
  /**
   Items the draw names, in sheet order.
   */
  const { items: drawn, } = manifest;
  if (graded.length !== drawn.length) {
    throw new StatedRefusalError({
      says: `sheet and manifest disagree about the draw: sheet has ${
        String(graded.length,)
      } ${
        wordForCount({
          count: graded.length,
          one: 'item',
          many: 'items',
        },)
      }, manifest has ${String(drawn.length,)}. Joining them by position would mislabel every verdict after `
        + 'the first divergence.',
    },);
  }

  return drawn.map(function toItem(
    entry,
    index,
  ): ProbeAgreementItem {
    /**
     Human grade at this position.
     */
    const grade = graded[index];
    if (grade === undefined) {
      throw new Error(
        'unreachable: a manifest position has no grade, though the sheet and the manifest were checked to be of one length',
      );
    }

    /**
     Probe reading covering this issue, absent when unprobed.
     */
    const reading = byIssueId.get(entry.issueId,);
    return {
      verdict: grade.verdict,
      ...(reading === undefined ? {} : { reading, }),
    };
  },);
}

/**
 Counts the joined positions whose slice the naturalness lane rewrote after
 probing.

 Reported rather than silently folded in. The probe runs inside the accuracy
 stage and the lane runs after it, so on these positions the probe judged
 one text while the repair sheet asked the human to grade another. Every
 cell of the agreement table treats the two as being about the same wording,
 which is true everywhere except here.

 @param manifest - draw manifest

 @param refinedIssueIds - issues the lane rewrote after the probe ran

 @returns Positions of the manifest whose issue was rewritten

 @example
 ```ts
 const refinedJoined = countRefinedJoined({ manifest, refinedIssueIds: gathered.refinedIssueIds, },);
 ```
 */
export function countRefinedJoined(
  {
    manifest,
    refinedIssueIds,
  }: {
    readonly manifest: SampleManifest;
    readonly refinedIssueIds: ReadonlySet<string>;
  },
): number {
  return manifest.items
    .filter(function wasRefined(entry,) {
      return refinedIssueIds
        .has(entry.issueId,);
    },)
    .length;
}

/**
 Prints the joint counts of the probe and the human grades, and the notes that
 keep them from being over-read.

 @param agreement - joint counts across both instruments

 @param refinedJoined - joined positions the naturalness lane rewrote after probing

 @example
 ```ts
 printProbeAgreement({ agreement, refinedJoined, },);
 ```
 */
export function printProbeAgreement(
  {
    agreement,
    refinedJoined,
  }: {
    readonly agreement: ProbeAgreement;
    readonly refinedJoined: number;
  },
): void {
  console.log(
    `AGREEMENT joined=${String(agreement.joined,)} probeFlagged=${
      String(agreement.probeFlagged,)
    } refutedByHuman=${String(agreement.refutedByHuman,)} sharedWithHuman=${
      String(agreement.sharedWithHuman,)
    } flaggedUnscored=${String(agreement.flaggedUnscored,)} unflaggedFailures=${
      String(agreement.unflaggedFailures,)
    } refinedJoined=${String(refinedJoined,)}`,
  );
  if (refinedJoined > 0)
    console.log(
      `NOTE refinedJoined counts positions where the naturalness lane rewrote `
        + `the slice AFTER the probe ran. There the probe judged the accuracy `
        + `stage's wording while the repair sheet asked the human to grade the `
        + `RETURNED wording, so those rows compare two different texts and `
        + `belong in neither column as evidence about the probe. Read the other `
        + `counts over the remaining ${String(agreement.joined - refinedJoined,)}.`,
    );
  console.log(
    'NOTE refutedByHuman is the clean number: the human read the same wording '
      + 'and said it breaks nothing nearby, so each one is a correct repair a '
      + 'gate would have discarded. sharedWithHuman is NOT confirmation, since '
      + 'the sheet\'s N fires both for a repair that did not fix its target and '
      + 'for one that broke something.',
  );
}

//endregion Score probe agreement
