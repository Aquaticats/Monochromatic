import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { MalformedArtifact, } from './attribution-read.ts';

//region Score crosscheck warnings
// The warnings `score-crosscheck` prints beside its counts: artifacts that
// could not be read, claims whose join to attribution failed, and claims the
// whole roster proposed. Each prints nothing when its count is zero.

/**
 Names the artifacts that could not be read.

 @param malformed - artifacts the gather could not parse

 @example
 ```ts
 printCrosscheckMalformed({ malformed, },);
 ```
 */
export function printCrosscheckMalformed(
  { malformed, }: { readonly malformed: readonly MalformedArtifact[]; },
): void {
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
      + 'in NEITHER population this report counts, so every count is over the rest:',
  );
  for (const failure of malformed)
    console.log(`  ${failure.name}: ${failure.reason}`,);
}

/**
 Warns about claims on attributed entries that no attribution record holds.

 @param count - claims with no proposer recorded on an entry that carries attribution

 @example
 ```ts
 printJoinFailureWarning({ count: census.unattributedJoinFailures, },);
 ```
 */
export function printJoinFailureWarning({ count, }: { readonly count: number; },): void {
  if (count <= 0)
    return;

  console.log(
    `WARNING ${String(count,)} ${
      wordForCount({
        count,
        one: 'claim',
        many: 'claims',
      },)
    } ${
      wordForCount({
        count,
        one: 'sits',
        many: 'sit',
      },)
    } on `
      + 'entries that DO carry attribution yet have no proposer recorded. '
      + 'That is the two records disagreeing about claim identity, not a '
      + 'quiet critic, and it is reported apart from the legacy count so it '
      + 'cannot hide inside an expected number.',
  );
}

/**
 Warns about claims every roster model proposed.

 @param count - claims proposed by the whole roster, leaving no seat to judge

 @example
 ```ts
 printUnjudgeableWarning({ count: unjudgeable.length, },);
 ```
 */
export function printUnjudgeableWarning({ count, }: { readonly count: number; },): void {
  if (count <= 0)
    return;

  console.log(
    `WARNING ${String(count,)} ${
      wordForCount({
        count,
        one: 'claim',
        many: 'claims',
      },)
    } ${
      wordForCount({
        count,
        one: 'was',
        many: 'were',
      },)
    } proposed by `
      + 'the WHOLE roster, leaving no seat to judge. Such claims are reported here '
      + 'rather than dropped: they are the most corroborated claims in the '
      + 'run, and removing them would lift every rate by hiding exactly the '
      + 'strongest agreement in the population.',
  );
}

//endregion Score crosscheck warnings
