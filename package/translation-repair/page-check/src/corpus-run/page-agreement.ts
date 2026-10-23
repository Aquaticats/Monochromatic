import type { ParsedArchiveText, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { withoutSourceRecordAt, } from './source-ships.ts';
import type { GrammarReader, } from './source-ships-runs.ts';
import { composeOverTiling, } from './source-ships-tiling.ts';
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

 A PAGE SHIPPED WITHOUT SOURCE RUNS IS JUDGED AGAINST THE ARTIFACT WITHOUT THEIR ORIGINALS, when the composition says
 it would be. The composition ships such a page (`source-ships-compose.ts`) when no writing of the originals makes the
 grammar accept it. This function runs that composition over what the artifact records (`source-ships-tiling.ts`), and
 only a page that disagrees as recorded is judged again, so a page agreeing with every original costs no grammar read.
 A page that never carried originals, as the pages shipped before they were carried, still disagrees wherever the
 composition keeps them. THE RESIDUAL: the composition runs over the archive the slices tile and not over the archive
 itself, so a page whose archive text between slices matters to the grammar can be judged by a composition that
 differs from the one that wrote it; and a page that lost every original it should carry, where the composition would
 also ship without them, reads as shipped without them (the report names the slices) and not as lost.

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
   The composition of the page the artifact says would ship.
   */
  const composed = composeOverTiling({
    artifact,
    ...((readGrammar === undefined) ? {} : { readGrammar, }),
  },);
  if (composed.step !== 'dropped')
    return asRecorded;
  /**
   The page against the artifact without the originals the composition leaves off.
   */
  const judged = judgeAgainst({
    artifact: withoutSourceRecordAt({
      artifact,
      slices: composed.slices,
    },),
    pageText,
  },);
  return (judged.agreement === 'disagreed')
    ? asRecorded
    : {
      ...judged,
      sourceDropped: composed.slices,
    };
}

//endregion Page agreement
