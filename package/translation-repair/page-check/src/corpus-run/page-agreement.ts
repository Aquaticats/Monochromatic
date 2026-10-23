import type { ParsedArchiveText, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { withoutSourceRecordAt, } from './source-ships.ts';
import {
  droppedRunSets,
  type GrammarReader,
  slicesRefusedAsLiteral,
} from './source-ships-runs.ts';
import type { WouldShipSource, } from './would-ship-text.ts';
import {
  type PageLengthCheck,
  pageCarriesEveryWording,
  pageWeighsWhatItShould,
  pageWeightRefutes,
  type PageWordingCheck,
} from './published-page-check.ts';

//region Page agreement
// WHETHER A PAGE ON DISK CARRIES WHAT ITS ARTIFACT SAYS SHIPS, decided once for
// both readers (ledger A16c). `verify-published` reports it, and a pass starting
// in a runs directory rewrites from its artifact a page that does not; two
// definitions would have the pass rewrite pages the verifier accepts, or leave
// pages the verifier reports. Moved out of `verify-published.ts`, where it was
// the body of the entry report.

/**
 What one entry's page came to against its artifact.

 THREE ANSWERS, NOT TWO. A page that carries every wording but whose length
 could not be checked is not the evidence a weighed page is.

 @example
 ```ts
 const agreement: EntryAgreement = 'agreed-unweighed';
 ```
 */
export type EntryAgreement = 'agreed-weighed' | 'agreed-unweighed' | 'disagreed';

/**
 What the judgement reads of an artifact: what each slice ships, and the
 archive text it stored, which a parsed settled artifact carries.

 @example
 ```ts
 const source: AgreementSource = parseSettledTwoLaneArtifact({ value, },);
 ```
 */
export type AgreementSource = WouldShipSource & {
  /**
   The slicing's record, read for the archive it stored.
   */
  readonly preparation: {
    /**
     Whole archive English, or that the file predates storing it.
     */
    readonly archiveText: ParsedArchiveText;
  };
};

/**
 The verdict and the two checks behind it, for a reader that reports them.

 @example
 ```ts
 const { agreement, wording, weight, wrongLength, } = pageAgreement({ artifact, pageText, },);
 ```
 */
export type PageAgreement = {
  /**
   The verdict.
   */
  readonly agreement: EntryAgreement;

  /**
   Which wordings the page carries.
   */
  readonly wording: PageWordingCheck;

  /**
   What the page weighs against what it should.
   */
  readonly weight: PageLengthCheck;

  /**
   Whether the length says text no slice decided on was lost or added.
   */
  readonly wrongLength: boolean;

  /**
   Slices whose original the page was shipped without because no writing of it parses (`page-source-unparseable`), none
   for a page that carries what its artifact says or lost something else.
   */
  readonly sourceDropped: readonly number[];
};

/**
 Judges one page against one reading of its artifact.

 @param artifact - settled artifact, read for what each slice ships and the archive text it stored

 @param pageText - page as it sits on disk

 @returns The verdict and what decided it, with no slice recorded as left off

 @example
 ```ts
 const judged = judgeAgainst({ artifact, pageText, },);
 ```
 */
function judgeAgainst(
  {
    artifact,
    pageText,
  }: {
    readonly artifact: AgreementSource;
    readonly pageText: string;
  },
): PageAgreement {
  /**
   What the page turned out to carry.
   */
  const wording = pageCarriesEveryWording({
    artifact,
    pageText,
  },);

  /**
   What the page should weigh against what it does, or that the artifact
   predates the stored archive text and nothing can be weighed.
   */
  const weight = pageWeighsWhatItShould({
    artifact,
    archive: artifact
      .preparation
      .archiveText,
    pageText,
  },);

  /**
   Whether the length says the page lost or gained text nobody decided on.
   */
  const wrongLength = pageWeightRefutes({ weight, },);

  /**
   Wordings the page does not carry.
   */
  const { missing, } = wording;

  /**
   The verdict both checks come to.
   */
  const agreement: EntryAgreement = (wrongLength || (missing.length > 0))
    ? 'disagreed'
    : ((weight.kind === 'unweighable') ? 'agreed-unweighed' : 'agreed-weighed');

  return {
    agreement,
    wording,
    weight,
    wrongLength,
    sourceDropped: [],
  };
}

/**
 Judges one page against its artifact.

 A PAGE SHIPPED WITHOUT A SOURCE RUN IS JUDGED AGAINST THE ARTIFACT WITHOUT THAT RUN'S ORIGINAL. The composition ships
 such a page (`source-ships-compose.ts`) when no writing of the run makes the grammar accept it, and it names the sets
 it leaves off through `droppedRunSets`, the function asked here. The sets are tried only for a page that disagrees as
 recorded, so a page agreeing with every original costs no grammar read, and a page that lost text elsewhere still
 disagrees under each of them.

 @param artifact - settled artifact, read for what each slice ships and the
 archive text it stored

 @param pageText - page as it sits on disk

 @param readGrammar - what the grammar says of a text, the strict MDX grammar the site compiles with by default

 @returns The verdict and what decided it

 @example
 ```ts
 if (pageAgreement({ artifact, pageText, },).agreement === 'disagreed') republish();
 ```
 */
export function pageAgreement(
  {
    artifact,
    pageText,
    readGrammar,
  }: {
    readonly artifact: AgreementSource;
    readonly pageText: string;
    readonly readGrammar?: GrammarReader;
  },
): PageAgreement {
  /**
   The page against every original the artifact records.
   */
  const asRecorded = judgeAgainst({
    artifact,
    pageText,
  },);
  if (asRecorded.agreement !== 'disagreed')
    return asRecorded;
  /**
   What the grammar says of each run, handed on only where a case stands in for it.
   */
  const grammar = (readGrammar === undefined) ? {} : { readGrammar, };
  // ONLY AN ARTIFACT WITH A RUN NO WRITING MAKES PARSE can have been shipped without one, so a page lacking runs the
  // grammar accepts is a page that lost them.
  /**
   Slices of the runs no writing makes parse.
   */
  const unwritable = slicesRefusedAsLiteral({
    artifact,
    ...grammar,
  },);
  if (unwritable.length === 0)
    return asRecorded;
  for (const dropped of droppedRunSets({
    artifact,
    ...grammar,
  },)) {
    /**
     The page against the artifact without the originals of these slices.
     */
    const judged = judgeAgainst({
      artifact: withoutSourceRecordAt({
        artifact,
        slices: dropped,
      },),
      pageText,
    },);
    if (judged.agreement !== 'disagreed') {
      return {
        ...judged,
        sourceDropped: dropped,
      };
    }
  }
  return asRecorded;
}

//endregion Page agreement
