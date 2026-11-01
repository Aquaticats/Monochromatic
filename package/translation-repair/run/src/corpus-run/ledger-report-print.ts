import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { LedgerReading, } from './ledger-directory.ts';
import {
  type CandidateReading,
  summariseLedger,
  workOfModel,
} from './ledger-read.ts';

//region Ledger report print
// What the ledger report says, one function per part: a candidate with the
// reasons given for it, the files that would not read, the whole ledger's
// summary and one seat's work.
//
// SPLIT OUT OF `ledger-report.ts` so the entry holds only what a real process
// supplies, and each printer is read through its own cases. Each prints with
// `console.log`, which is the report.

/**
 Most UTF-16 units of a candidate shown before it is cut, ending on a whole
 character (`wholeOpening`).
 */
const EXCERPT_CHARS = 400;

/**
 Multiplier turning a fraction into a percentage.
 */
const PERCENT = 100;

/**
 Prints one candidate a named seat wrote, with what judges said about it.

 A candidate longer than the excerpt is cut to a whole character and says so,
 with how many of its UTF-16 units are shown.

 @param reading - candidate and the remarks about it

 @param at - position in this seat's output, so a reader can cite one

 @example
 ```ts
 printReading({ reading, at: 0, },);
 ```
 */
export function printReading(
  {
    reading,
    at,
  }: {
    readonly reading: CandidateReading;
    readonly at: number;
  },
): void {
  console.log(
    `\n--- ${String(at + 1,)} --- ${reading.won ? 'CHOSEN' : 'not chosen'} `
      + `--- ${reading.task}`,
  );
  /**
   Exactly what the seat put in front of the judges.
   */
  const { rendered, } = reading;

  /**
   UTF-16 units the whole candidate runs to.
   */
  const total = rendered.length;

  /**
   The opening of the candidate that fits the excerpt, to a whole character.
   */
  const shown = wholeOpening({
    text: rendered,
    units: EXCERPT_CHARS,
  },);
  console.log(shown,);

  // A CUT IS SAID, NOT LEFT TO BE GUESSED. The candidate's text is the
  // evidence this view exists to show, and an opening printed with nothing
  // after it reads as the whole candidate. THE PLURAL IS FIXED because a cut
  // only happens past the excerpt's length, so the count it follows is never
  // one.
  if (shown.length < total)
    console.log(`  (cut: ${String(shown.length,)} of ${String(total,)} UTF-16 units shown)`,);

  /**
   Disinterested judges that named this candidate.
   */
  const { remarks, } = reading;

  if (remarks.length === 0)
    console.log('  (no disinterested judge named this candidate)',);
  for (const remark of remarks) {
    console.log(`  ${remark}`,);
  }
}

/**
 Reports the files that would not read, and what their absence costs.

 NAMED AS A SHORTFALL RATHER THAN LISTED AND DROPPED. Every figure this report
 prints is computed over the files that read, so an unreadable contest silently
 lowers a seat's candidate count and its ballot count together. A reader who
 did not know that would take a partial standing for a whole one.

 @param reading - what the ledger directory yielded

 @example
 ```ts
 printRefusals({ reading, },);
 ```
 */
export function printRefusals(
  { reading, }: { readonly reading: LedgerReading; },
): void {
  /**
   Both halves of the reading, named so no member chain runs two steps deep.
   */
  const {
    refused,
    rounds,
  } = reading;

  for (const refusal of refused) {
    console.log(`  UNREADABLE ${refusal.file}: ${refusal.says}`,);
  }

  if (refused.length === 0)
    return;

  /**
   Every ledger file this report accounts for, read or not.
   */
  const totalLedgerFiles = refused.length + rounds.length;
  console.log(
    `  ${String(refused.length,)} of `
      + `${String(totalLedgerFiles,)} ledger ${
        wordForCount({
          count: totalLedgerFiles,
          one: 'file',
          many: 'files',
        },)
      } could not be read. `
      + 'Every figure here counts only the files that could, so a seat that wrote into an '
      + 'unreadable contest is undercounted, and so is every judge who weighed it. Re-run the '
      + 'pass to rewrite them, or read the standing as a floor.',
  );
}

/**
 Prints what every seat did, over the contests that read.

 @param reading - what the ledger directory yielded

 @example
 ```ts
 printSummary({ reading, },);
 ```
 */
export function printSummary(
  { reading, }: { readonly reading: LedgerReading; },
): void {
  /**
   What every seat did.
   */
  const summary = summariseLedger({ rounds: reading.rounds, },);

  console.log(
    `${String(summary.abstentions,)} ${
      wordForCount({
        count: summary.abstentions,
        one: 'ballot',
        many: 'ballots',
      },)
    } named nothing, `
      + `${String(summary.namedMissing,)} named a candidate the slate did not have`,
  );
  for (const work of summary.models) {
    /**
     Share of disinterested ballots that named this seat's work.
     */
    const share = (work.ballots === 0)
      ? 'UNJUDGED'
      : `${((work.votes / work.ballots) * PERCENT).toFixed(1,)}%`;

    console.log(
      `  ${work.model}: ${String(work.candidates,)} ${
        wordForCount({
          count: work.candidates,
          one: 'candidate',
          many: 'candidates',
        },)
      }, ${String(work.wins,)} chosen, `
        + `${share} of ${String(work.ballots,)} disinterested ${
          wordForCount({
            count: work.ballots,
            one: 'ballot',
            many: 'ballots',
          },)
        }, `
        + `${String(work.selfVotes,)} ${
          wordForCount({
            count: work.selfVotes,
            one: 'self-vote',
            many: 'self-votes',
          },)
        }`,
    );
  }
  console.log(
    '\nPass --model <id> to read one seat\'s text and the reasons judges gave. A low share means '
      + 'rarely picked as best, which is not the same as wrong.',
  );
}

/**
 Prints one seat's candidates and the reasons judges gave for choosing them.

 @param reading - what the ledger directory yielded

 @param wanted - seat to read in full

 @example
 ```ts
 printSeat({ reading, wanted, },);
 ```
 */
export function printSeat(
  {
    reading,
    wanted,
  }: {
    readonly reading: LedgerReading;
    readonly wanted: string;
  },
): void {
  /**
   Everything that seat wrote.
   */
  const written = workOfModel({
    rounds: reading.rounds,
    model: wanted,
  },);

  /**
   Its candidates the panel chose.
   */
  const chosen = written.filter(function won(reading_,): boolean {
    return reading_.won;
  },);

  console.log(
    `${wanted} wrote ${String(written.length,)} ${
      wordForCount({
        count: written.length,
        one: 'candidate',
        many: 'candidates',
      },)
    }, `
      + `${String(chosen.length,)} chosen`,
  );
  written.forEach(function show(
    reading_,
    at,
  ): void {
    printReading({
      reading: reading_,
      at,
    },);
  },);
}

//endregion Ledger report print
