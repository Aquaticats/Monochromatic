import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { MalformedArtifact, } from './attribution-read.ts';

//region Score attribution warnings
// The warnings `score-attribution` prints beside its counts: artifacts that
// could not be read, and accepted issues whose join to attribution failed.
// Each prints nothing when its count is zero.

/**
 Names the artifacts that could not be read.

 @param malformed - artifacts the gather could not parse

 @example
 ```ts
 printMalformedArtifacts({ malformed, },);
 ```
 */
export function printMalformedArtifacts({ malformed, }: { readonly malformed: readonly MalformedArtifact[]; },): void {
  if (malformed.length === 0)
    return;

  console.log(
    `WARNING ${String(malformed.length,)} ${
      wordForCount({
        count: malformed.length,
        one: 'artifact',
        many: 'artifacts',
      },)
    } could not be read and ${
      wordForCount({
        count: malformed.length,
        one: 'is',
        many: 'are',
      },)
    } `
      + 'in NEITHER population this report counts, so every count is over the rest. Named '
      + 'rather than summarized, because a truncated artifact is a different '
      + 'problem from a malformed one:',
  );
  for (const failure of malformed)
    console.log(`  ${failure.name}: ${failure.reason}`,);
}

/**
 Warns about accepted issues that joined only some of their claims to
 attribution.

 @param count - accepted issues with a partial join

 @example
 ```ts
 printPartialJoinWarning({ count: report.partialJoinAccepted, },);
 ```
 */
export function printPartialJoinWarning({ count, }: { readonly count: number; },): void {
  if (count <= 0)
    return;

  console.log(
    `WARNING ${String(count,)} accepted ${
      wordForCount({
        count,
        one: 'issue',
        many: 'issues',
      },)
    } joined only SOME of ${
      wordForCount({
        count,
        one: 'its',
        many: 'their',
      },)
    } claims to attribution. ${
      wordForCount({
        count,
        one: 'It is',
        many: 'Those are',
      },)
    } held out of every `
      + 'other count in this report rather than counted as support, because the unattributed '
      + 'member may have come from a critic that got no credit. A nonzero '
      + 'number here is a defect in the join, not a fact about critics.',
  );
}

/**
 Warns about accepted issues on eligible entries that carry no attribution.

 @param count - accepted issues with no attribution

 @example
 ```ts
 printUnattributedWarning({ count: report.unattributedAccepted, },);
 ```
 */
export function printUnattributedWarning({ count, }: { readonly count: number; },): void {
  if (count <= 0)
    return;

  console.log(
    `WARNING ${String(count,)} accepted ${
      wordForCount({
        count,
        one: 'issue',
        many: 'issues',
      },)
    } on ELIGIBLE entries ${
      wordForCount({
        count,
        one: 'carries',
        many: 'carry',
      },)
    } no attribution, meaning a claim id the index `
      + 'does not hold. That is a defect in the join, not a quiet critic.',
  );
}

//endregion Score attribution warnings
