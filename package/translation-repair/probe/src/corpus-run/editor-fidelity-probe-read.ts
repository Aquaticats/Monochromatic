import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { buildEditorMessages, } from '@monochromatic-dev/translation-repair-repair/ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { sheetFieldsOf, } from '../editor-fidelity-context.ts';
import type { EditorFidelityTrial, } from '../editor-fidelity-trials.ts';
import {
  type AskEnd,
  stoppedWord,
} from './editor-fidelity-asks.ts';
import {
  messageChars,
  pricesOf,
} from './editor-fidelity-ceiling.ts';
import {
  type EditorCardTally,
  editorDefectOnlyLine,
  type EditorFidelityRow,
  editorKindLines,
  type EditorPair,
  editorTallyLine,
  editorTallyOf,
} from './editor-fidelity-plan.ts';
import type { EditorTrialReading, } from './editor-fidelity-trial.ts';
import {
  probeRosterWith,
  readCandidateIds,
  readCandidatesAlone,
} from './probe-candidates.ts';

//region Editor fidelity probe readers
// The small readers the run asks of its command line, its trials and its readings: which cards are seated, how big a
// sheet is, what a pair is called and how a reading is worded. Apart from the run so that file stays the sequence of
// decisions.

/**
 Reads the cards the command line seats, alone.

 THIS PROBE SEATS THE NAMED CARDS AND NOBODY ELSE, so `--candidates-alone` is required rather than defaulted: a
 roster seated beside the cards would be a probe that spends on more than was asked, and the flag a reader of the
 command already knows from the judge probe says it plainly.

 @param line - probe's command line

 @returns Cards, once each, in the order named

 @throws StatedRefusalError when a card is unknown, `--candidates-alone` is not written, no card is named, or a card
 has no price table

 @example
 ```ts
 const cardIds = seatedCardsOf({ line, },);
 ```
 */
export function seatedCardsOf(
  { line, }: { readonly line: CommandLineOf<'editor-fidelity-probe'>; },
): readonly RosterModelId[] {
  /**
   Cards as named, each checked against the roster.
   */
  const candidates = readCandidateIds({ line, },);
  /**
   Whether the operator said the cards run without a roster beside them.
   */
  const alone = readCandidatesAlone({ line, },);
  if (!alone) {
    throw new StatedRefusalError({
      says: 'this probe seats only the cards --candidates names; write --candidates-alone to say so',
    },);
  }
  /**
   The cards, each once.
   */
  const cardIds = probeRosterWith({
    candidates,
    alone,
  },);
  for (const cardId of cardIds)
    pricesOf({ cardId, },);
  return cardIds;
}

/**
 Characters of the sheet a trial sends, which the ceiling sizes the prompt by.

 @param trial - trial the sheet is built for

 @returns Characters of every message, summed

 @example
 ```ts
 const chars = sheetCharsOf({ trial, },);
 ```
 */
export function sheetCharsOf({ trial, }: { readonly trial: EditorFidelityTrial; },): number {
  return messageChars({
    messages: buildEditorMessages({
      sourceText: trial.sourceText,
      targetText: trial.damagedText,
      envelopes: trial.envelopes,
      issues: trial.issues,
      ...sheetFieldsOf({ context: trial.context, },),
    },)
      .messages,
  },);
}

/**
 Names a planned pair on the lines the overlapped map writes about a failure it leaves unthrown.

 @param item - planned pair

 @returns Card and trial by their handles

 @example
 ```ts
 // yields 'claude-sonnet-5-5-effort-low on tabby-reference/insertion'
 pairName({ item: pair, },);
 ```
 */
export function pairName({ item, }: { readonly item: EditorPair; },): string {
  return `${item.cardId} on ${item.trial
    .trialId}`;
}

/**
 Says in a word how a trial read, for the progress line.

 @param reading - how the trial read

 @returns Short phrase with no corpus text in it

 @example
 ```ts
 // says 'clean fix exact, nothing else touched'
 readingWord({ reading, },);
 ```
 */
export function readingWord({ reading, }: { readonly reading: EditorTrialReading; },): string {
  if (reading.kind === 'refused')
    return 'refused, no edit proposed';
  if (reading.kind === 'unusable')
    return `unusable (${reading.cause})`;
  /**
   Whether something outside the defect changed.
   */
  const touched = reading.score
    .collateral
    ? 'collateral'
    : 'nothing else touched';
  if (reading.score
    .fixed
    === 'none')
    return `not fixed, ${touched}`;
  if (reading.score
    .fixed
    === 'defect-only')
    return `defect only, not a clean fix, ${touched}`;
  return `clean fix ${reading.score
    .fixed}, ${touched}`;
}

/**
 Says the pass a row belongs to, counted from one, for a row of a later pass; the first pass says nothing, so a row of one
 pass reads as it always did.

 @param pair - card and trial, whose pass is from zero and absent for the first

 @returns `; pass 2` and so on for a later pass, the empty text otherwise

 @example
 ```ts
 // yields '; pass 2'
 passWord({ pair: { cardId, trial, pass: 1, }, },);
 ```
 */
function passWord({ pair, }: { readonly pair: EditorPair; },): string {
  /**
   The pass from zero.
   */
  const pass = pair.pass ?? 0;
  return (pass === 0) ? '' : `; pass ${String(pass + 1,)}`;
}

/**
 Writes the row line of a finished pair.

 @param position - zero-based place of the pair among the planned pairs

 @param total - planned pairs

 @param pair - card and trial

 @param reading - how the trial read

 @param stops - how each ask of the trial ended

 @returns Line, without a newline

 @example
 ```ts
 log.info(rowLineOf({ position: 0, total: 2, pair, reading, stops: ['ok',], },),);
 ```
 */
export function rowLineOf(
  {
    position,
    total,
    pair,
    reading,
    stops,
  }: {
    readonly position: number;
    readonly total: number;
    readonly pair: EditorPair;
    readonly reading: EditorTrialReading;
    readonly stops: readonly AskEnd[];
  },
): string {
  return `row ${String(position + 1,)} of ${String(total,)}: ${pairName({ item: pair, },)}: ${
    readingWord({ reading, },)
  }; entry ${pair.trial
    .spec
    .entryId}${passWord({ pair, },)}; stopped ${stoppedWord({ ends: stops, },)}`;
}

/**
 Tallies each card's rows and says each tally, its kind lines and its defect-only line.

 @param cardIds - cards seated, in the order named

 @param rows - every completed row

 @param emit - says one line

 @returns Tally of each card, in the order the cards were named

 @example
 ```ts
 const tallies = reportTallies({ cardIds, rows, emit, },);
 ```
 */
export function reportTallies(
  {
    cardIds,
    rows,
    emit,
  }: {
    readonly cardIds: readonly RosterModelId[];
    readonly rows: readonly EditorFidelityRow[];
    readonly emit: (line: string,) => void;
  },
): readonly EditorCardTally[] {
  return cardIds.map(function tallyOf(cardId,): EditorCardTally {
    /**
     This card's rows, tallied.
     */
    const tally = editorTallyOf({
      cardId,
      rows,
    },);
    emit(editorTallyLine({ tally, },),);
    for (const said of editorKindLines({ tally, },))
      emit(said,);
    emit(editorDefectOnlyLine({ tally, },),);
    return tally;
  },);
}

//endregion Editor fidelity probe readers
