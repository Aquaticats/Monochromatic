/**
 Who helped write the text a checker is asked to judge.

 KEPT APART FROM THE CODE THAT BUILDS IT. `tally-resolution.ts` needs the shape
 and the predicate to weigh a verdict, and the builders reach through
 `chunk-measure.ts`, which reads the tally back. Splitting the contract out
 keeps that from closing a cycle.

 @module
 */

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Authorship of the text under check

/**
 Models that helped write the text a checker is judging.

 TWO FIELDS RATHER THAN ONE FLAT MAP. A round that decided the whole chunk
 authors issues that were never named to it, so flattening it would need the
 full issue list at build time and would attribute nothing for any issue
 missing from that list. Naming chunk-wide authors separately keeps the fact
 whole however a later caller asks about it.

 @example
 ```ts
 const authorship: IssueAuthorship = { perIssue: {}, everyIssue: ['hf:zai-org/GLM-5.3-Flash',], };
 ```
 */
export type IssueAuthorship = {
  /**
   Models that wrote text serving one issue, keyed by issue id.
   */
  readonly perIssue: Readonly<Record<string, readonly RosterModelId[]>>;

  /**
   Models that wrote this chunk whole, so they author every issue in it.
   */
  readonly everyIssue: readonly RosterModelId[];
};

/**
 Authorship of text no judged round produced, so every verdict on it counts whole.

 NOT A DEFAULT ARGUMENT. Call sites state it, because "no round wrote this" and
 "nobody asked who wrote this" weigh identically but mean different things, and
 a silent default would let the second pass unnoticed for the first.

 @example
 ```ts
 const authorship = UNATTRIBUTED_TEXT;
 ```
 */
export const UNATTRIBUTED_TEXT: IssueAuthorship = {
  perIssue: {},
  everyIssue: [],
};

/**
 Whether one checker helped write the text answering for one issue.

 @param authorship - who wrote the text under check

 @param issueId - issue whose text this verdict judges

 @param modelId - checker casting that verdict

 @returns Whether this checker has a hand in this issue's text

 @example
 ```ts
 const discounted = wroteTextForIssue({ authorship, issueId, modelId, },);
 ```
 */
export function wroteTextForIssue(
  {
    authorship,
    issueId,
    modelId,
  }: {
    readonly authorship: IssueAuthorship;
    readonly issueId: string;
    readonly modelId: string;
  },
): boolean {
  return [
    ...authorship.everyIssue,
    ...(authorship.perIssue[issueId] ?? []),
  ]
    .some(function isThisChecker(author,) {
      return author === modelId;
    },);
}

//endregion
