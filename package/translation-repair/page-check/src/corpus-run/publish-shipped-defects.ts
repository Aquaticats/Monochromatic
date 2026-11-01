//region The defects one published page ships with

import { placeholderHeadingDefects, } from './publish-placeholder-heading.ts';
import {
  archiveKeptDefects,
  pageWithoutBodyDefects,
  type PublishDefect,
  tenseSwapDefects,
} from './publish-defects.ts';
import type { WouldShipSource, } from './would-ship-text.ts';

/**
 Every defect one page ships with, in the order the log names them: the content checks that failed, the slices the
 archive kept, the tense swaps, a page with no body, a heading of placeholder marks, the defects of the composed page and
 the defects of the grammar's reading.

 The archive is kept at a slice because no wording passed the rule (owner, 2026-09-27, "Keep archive, ship").

 @param entryId - entry the page belongs to

 @param artifact - settled entry, read for what the consolidation decided

 @param sourceText - whole original page

 @param pageText - page that would be written

 @param failedChecks - content checks that failed, which the publisher ran

 @param composedDefects - defects the composition of the page named

 @param parsingDefects - defects the grammar's reading of the page named

 @returns Defects, failed checks first and the grammar's last

 @example
 ```ts
 const defects = shippedDefects({ entryId, artifact, sourceText, pageText, failedChecks, composedDefects, parsingDefects, },);
 ```
 */
export function shippedDefects(
  {
    entryId,
    artifact,
    sourceText,
    pageText,
    failedChecks,
    composedDefects,
    parsingDefects,
  }: {
    readonly entryId: string;
    readonly artifact: Pick<WouldShipSource, 'consolidation'>;
    readonly sourceText: string;
    readonly pageText: string;
    readonly failedChecks: readonly PublishDefect[];
    readonly composedDefects: readonly PublishDefect[];
    readonly parsingDefects: readonly PublishDefect[];
  },
): readonly PublishDefect[] {
  return [
    ...failedChecks,
    ...archiveKeptDefects({
      entryId,
      consolidation: artifact.consolidation,
    },),
    // WHERE THE TENSE DETECTOR DECIDED WHAT SHIPS (owner, 2026-10-29, "Swap only where flagged"): named, so a run
    // shows each swap.
    ...tenseSwapDefects({
      entryId,
      consolidation: artifact.consolidation,
    },),
    // A PAGE WITH NO BODY, whatever the archive held: the one outcome the commitment to ship the original under
    // a note rules out, and a 0-byte page said nothing before this (B1046).
    ...pageWithoutBodyDefects({
      entryId,
      sourceText,
      pageText,
    },),
    // A HEADING OF PLACEHOLDER MARKS ONLY where the original has words, reported and left as it stands (owner,
    // 2026-10-29, "Report it as a defect"; B1052).
    ...placeholderHeadingDefects({
      entryId,
      sourceText,
      pageText,
    },),
    ...composedDefects,
    // A PAGE THE GRAMMAR REFUSED is a defect every slice floor passed, since the would-ship reading runs after them; the
    // yulianNyanner page of 2026-09-06 carried a curled JSX string literal that way (page-grammar.ts).
    ...parsingDefects,
  ];
}

//endregion The defects one published page ships with
