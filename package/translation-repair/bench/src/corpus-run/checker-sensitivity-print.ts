import type { IssueResolutionTally, } from '@monochromatic-dev/translation-repair-adjudication/ts';

//region Checker sensitivity print
// What the checker sensitivity runner says, as text a case can read whole:
// one line per single-issue case, one per issue of a sheet, and the note that
// closes the run.

/**
 Line one single-issue case leaves.

 @param label - case name

 @param expectation - what a discriminating checker should answer

 @param heard - checkers whose reply arrived and validated

 @param tally - the one issue's tally

 @returns The line, without a newline

 @example
 ```ts
 console.log(singleCheckLine({ label: 'untouched', expectation: 'not-fixed', heard: 3, tally, },),);
 ```
 */
export function singleCheckLine(
  {
    label,
    expectation,
    heard,
    tally,
  }: {
    readonly label: string;
    readonly expectation: string;
    readonly heard: number;
    readonly tally: IssueResolutionTally;
  },
): string {
  return `CHECKER ${label} expected=${expectation} heard=${
    String(heard,)
  } fixed=${String(tally.fixed,)} notFixed=${
    String(tally.notFixed,)
  } worse=${String(tally.worse,)} resolved=${
    String(tally.resolved,)
  } regressed=${String(tally.regressed,)}`;
}

/**
 Line one issue of a sheet leaves.

 @param sheet - sheet the issue belongs to, which opens the label

 @param issueId - issue the line is about

 @param expectation - what a discriminating checker should answer

 @param tally - the issue's tally

 @returns The line, without a newline

 @example
 ```ts
 console.log(sheetCheckLine({ sheet: 'mixed-sheet', issueId: 'adjudicated/tense', expectation: 'fixed', tally, },),);
 ```
 */
export function sheetCheckLine(
  {
    sheet,
    issueId,
    expectation,
    tally,
  }: {
    readonly sheet: string;
    readonly issueId: string;
    readonly expectation: string;
    readonly tally: IssueResolutionTally;
  },
): string {
  return `CHECKER ${sheet}/${issueId} expected=${expectation} fixed=${
    String(tally.fixed,)
  } notFixed=${String(tally.notFixed,)} worse=${
    String(tally.worse,)
  } resolved=${String(tally.resolved,)}`;
}

/**
 Note closing the run, saying which case matters.
 */
export const CHECKER_NOTE: string = 'NOTE the untouched case is the one that matters: a majority calling an '
  + 'unrepaired text fixed would mean the 98.1 percent resolution rate '
  + 'measures the checkers rather than the repairs. The mixed sheet asks '
  + 'the same question under the shape that rate was measured on, since '
  + 'production passes every accepted issue of a chunk in one call.';

//endregion Checker sensitivity print
