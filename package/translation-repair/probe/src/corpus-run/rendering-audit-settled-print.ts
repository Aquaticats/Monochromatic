import type { AuditRepeatBand, } from './rendering-audit-settled-band.ts';
import type {
  AudienceSplit,
  AuditRelocationPair,
  VoiceRate,
} from './rendering-audit-settled-read.ts';
import type { PageRelationTally, } from './rendering-audit-settled-relation.ts';
import { distinctSlicePairs, } from './rendering-audit-settled-relocation.ts';

//region Settled audit printing
// How a persisted run reads on a terminal.
//
// SPLIT OUT OF THE REPORT so the report is orchestration and this is wording.
// Both were in one file until the repeat readings arrived and pushed it past
// its line budget; splitting was the remedy rather than raising the cap.
//
// EVERY PRINTER NAMES ITS DENOMINATOR. A count with no denominator is the
// single most quotable wrong number a telemetry probe can emit, and these
// numbers exist to be quoted into a doc.

/**
 Width model ids are padded to, so a column of rates reads down the page.
 */
const MODEL_COLUMN = 48;

/**
 Width relation labels are padded to, so a column of counts reads down the
 page. Set by the longest label the reader can produce,
 `silent:contest-declined-and-archive-silent`.
 */
const RELATION_COLUMN = 42;

/**
 Prints one half of the population.

 @param split - that half, summed

 @example
 ```ts
 printSplit({ split, },);
 ```
 */
export function printSplit({ split, }: { readonly split: AudienceSplit; },): void {
  /**
   Everything this half amounts to.
   */
  const {
    audits,
    subjects,
    claimed,
    subjectsWithClaims,
    corroborated,
    agreed,
    near,
    degraded,
  } = split;

  console.log(
    `  ${(audits === 'archive') ? 'ARCHIVE text' : 'FRESH   text'}  subjects=${
      String(subjects,)
    }  drew a claim=${String(subjectsWithClaims,)}  claims=${String(claimed,)}  corroborated=${
      String(corroborated,)
    }  agreed=${String(agreed,)}  near=${String(near,)}  degraded=${String(degraded,)}`,
  );
}

/**
 Prints how much of the audit describes wording a later stage overruled.

 @param tallies - one per relation present

 @example
 ```ts
 printRelations({ tallies, },);
 ```
 */
export function printRelations(
  { tallies, }: { readonly tallies: readonly PageRelationTally[]; },
): void {
  console.log('\nWHAT A DOCUMENT WOULD CARRY AT THE SAME SLICES',);
  for (const tally of tallies) {
    /**
     Relation name, padded so the counts beside it read down the page.
     */
    const named = tally.label
      .padEnd(RELATION_COLUMN,);

    console.log(
      `  ${named}  subjects=${String(tally.subjects,)}  claims=${String(tally.claimed,)}`,
    );
  }
  console.log(
    '  A displaced subject was audited on wording no reader of a document would meet.'
      + ' An undecided one is waiting on a decision, not overruled.',
  );
}

/**
 Prints what each auditor thought was worth a claim.

 @param rates - one rate per auditor that answered

 @example
 ```ts
 printVoices({ rates, },);
 ```
 */
export function printVoices({ rates, }: { readonly rates: readonly VoiceRate[]; },): void {
  console.log('\nWHAT EACH AUDITOR THOUGHT WAS WORTH A CLAIM',);
  rates.forEach(function printRate(rate,): void {
    /**
     This auditor's tally.
     */
    const {
      modelId,
      asked,
      answered,
      spoke,
      claims,
      dropped,
    } = rate;

    /**
     Subjects the roster lost this auditor on, which is the number a
     voice-loss question reads and the one line used to hide.
     */
    const lost = asked - answered;

    console.log(
      `  ${modelId.padEnd(
        MODEL_COLUMN,
        ' ',
      )} asked=${String(asked,)} answered=${String(answered,)} lost=${String(lost,)} spoke on=${
        String(spoke,)
      } claims=${String(claims,)} dropped=${String(dropped,)}`,
    );
  },);
}

/**
 Prints the omission and addition pairs the relocation rule says are one
 relocation.

 TWO COUNTS IN THE HEADING, because the pairs are per claim: three voices
 each filing one omission beside two each filing one addition are six claim
 pairs over one pair of slices, and a heading saying `6` would be quoted as
 six relocations. The slice-pair count is the one a reader means.

 @param pairs - candidate relocations

 @example
 ```ts
 printRelocations({ pairs, },);
 ```
 */
export function printRelocations(
  { pairs, }: { readonly pairs: readonly AuditRelocationPair[]; },
): void {
  console.log(
    `\nRELOCATION CANDIDATES: claim pairs=${String(pairs.length,)} slice pairs=${
      String(distinctSlicePairs({ pairs, },),)
    }`,
  );
  pairs.forEach(function printPair(pair,): void {
    console.log(
      `  ${pair.runSet}/${pair.entryId}  omission at ${String(pair.omissionAt,)} <-> addition at ${
        String(pair.additionAt,)
      }`,
    );
  },);
}

/**
 Prints the spread two audits of one text landed in.

 SAYS WHEN THERE IS NOTHING TO REPORT rather than printing zeroes. A band of
 zero over zero pairs and a band of zero over forty pairs are opposite
 findings, and a row of zeroes reads as the second.

 @param band - spread over the pairs found

 @param over - what was paired, named for the reader

 @example
 ```ts
 printBand({ band, over: 'texts audited twice inside this run', },);
 ```
 */
export function printBand(
  {
    band,
    over,
  }: {
    readonly band: AuditRepeatBand;
    readonly over: string;
  },
): void {
  console.log(`\nINSTRUMENT BAND over ${over}`,);
  if (band.pairs === 0) {
    console.log(
      '  NOTHING PAIRED. Either no text was audited twice, or the rows predate'
        + ' the recorded text identity that pairing needs. No band is quotable'
        + ' from this run, so no comparison in it resolves anything.',
    );
    return;
  }

  console.log(
    `  pairs=${String(band.pairs,)}  same claim count=${String(band.agreedExactly,)}  widest gap=${
      String(band.widest,)
    }  total gap=${String(band.totalGap,)}  silent on one side only=${
      String(band.silentOnOneSide,)
    }`,
  );
  console.log(
    `  claims ${String(band.leftClaimed,)} against ${String(band.rightClaimed,)}`
      + `, corroborated ${String(band.leftCorroborated,)} against ${
        String(band.rightCorroborated,)
      }`,
  );
}

//endregion Settled audit printing
