import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type AskEnd,
  EDITOR_DAMAGE_KINDS,
  editorCeilings,
  type EditorDamageKind,
  editorFidelityTrials,
  type EditorFidelityTrial,
  type EditorTrialReading,
  openingLines,
  rowLineOf,
} from '../dist/final/node/index.mjs';
import { inventedReference, } from './editor-fidelity.test-fixture.ts';
import { reviewedFixture, } from './fidelity-reference.test-fixture.ts';

//region Invented editor fidelity arm log lines
// The lines of an arm log that the probe's own writers produce, called here so that a writer that changes its wording fails
// the cases of the read step before it fails an operator: the opening lines (manifest, pool, ceiling, guard, rule) and the
// row line. The pool is a stand-in of invented trials, two to an entry, built from the one invented reference; no corpus
// content. The one line written by hand elsewhere is the SPEND line, whose writer also moves the run's meter.

/**
 Trials the invented pool holds, which a log's pool line and planned line say.
 */
export const POOL_TRIALS = 88;

/**
 The specification the manifest line of an invented arm counts.
 */
const { spec: SPEC, } = reviewedFixture();

/**
 Characters of the sheet each ask of the invented arms is priced at.
 */
const SHEET_CHARS = 27_000;

/**
 How the stop word of a row is read: the ends an ask can have, in the words a row line prints.
 */
const ASK_ENDS: readonly AskEnd[] = [
  'ok',
  'refused',
  'output-cap',
  'unparseable',
  'rejected',
  'threw',
];

/**
 One row of an invented arm: the trial it walked, the entry the trial comes from, whether it was a clean fix and how its
 asks ended.
 */
export type RowSpec = {
  /**
   The trial's handle, `<entry>-s0/<kind>`.
   */
  readonly trial: string;

  /**
   The entry the trial comes from.
   */
  readonly entry: string;

  /**
   Whether the row reads as a clean fix.
   */
  readonly fixed: boolean;

  /**
   The stop words the row states, empty for a row that states none.
   */
  readonly stops: string;

  /**
   The pass over the pool the row belongs to, counted from one; absent for the first pass, whose row lines name none.
   */
  readonly pass?: number;

  /**
   Whether the row's reply came from the prompt reuse layer and not from a call, which the run client logs as a
   `PROMPT-REUSE` line just before the row's own; absent for a row that made a call.
   */
  readonly reused?: boolean;
};

/**
 The first trial of the invented reference, whose other members the stand-in trials override.
 */
const BASE_TRIAL: EditorFidelityTrial = nonNullishOrThrow(editorFidelityTrials({
  references: [inventedReference(),],
  damageKinds: EDITOR_DAMAGE_KINDS,
},)[0],);

/**
 A stand-in trial: the invented reference's first trial under another handle, entry and defect family, which is all the
 pool line and the row line read.

 @param trialId - handle, `<entry>-s0/<kind>`

 @param entryId - entry the trial comes from

 @param damageKind - defect family

 @returns Trial

 @example
 ```ts
 const trial = stubTrial({ trialId: 'cat-0-s0/insertion', entryId: 'cat-0', damageKind: 'insertion', },);
 ```
 */
export function stubTrial(
  {
    trialId,
    entryId,
    damageKind,
  }: {
    readonly trialId: string;
    readonly entryId: string;
    readonly damageKind: EditorDamageKind;
  },
): EditorFidelityTrial {
  return {
    ...BASE_TRIAL,
    trialId,
    damageKind,
    spec: {
      ...BASE_TRIAL.spec,
      entryId,
    },
  };
}

/**
 The stand-in pool: two trials to an entry, the insertion then the alteration, entries named `cat-0`, `cat-1` and so on.

 @param count - how many trials, the invented pool's when omitted

 @returns Trials

 @example
 ```ts
 const pool = poolStubs({ count: 40, },);
 ```
 */
export function poolStubs({ count = POOL_TRIALS, }: { readonly count?: number; } = {},): readonly EditorFidelityTrial[] {
  return Array.from(
    { length: count, },
    function trialAt(
      _unused,
      index,
    ): EditorFidelityTrial {
      /**
       The entry of this trial, two trials to an entry.
       */
      const entryId = `cat-${String(Math.floor(index / 2,),)}`;
      /**
       The defect family of this trial.
       */
      const damageKind: EditorDamageKind = ((index % 2) === 0) ? 'insertion' : 'alteration';
      return stubTrial({
        trialId: `${entryId}-s0/${damageKind}`,
        entryId,
        damageKind,
      },);
    },
  );
}

/**
 Reads the ends of a row's asks from the stop word its spec states.

 @param stops - stop word, ends joined by a plus

 @returns Ends

 @throws {@link Error} when a word is not an end an ask can have

 @example
 ```ts
 // yields ['refused', 'ok']
 const ends = askEndsOf({ stops: 'refused+ok', },);
 ```
 */
function askEndsOf({ stops, }: { readonly stops: string; },): readonly AskEnd[] {
  return stops.split('+',)
    .map(function endOf(word,): AskEnd {
      /**
       The end the word names.
       */
      const found = ASK_ENDS.find(function isWord(end,): boolean {
        return end === word;
      },);
      if (found === undefined)
        throw new Error(`the stop word ${word} is not an end an ask can have`,);
      return found;
    },);
}

/**
 The row line of an invented row, written by the probe's writer; a row that states no stop reason has the clause cut off,
 which is how a log from before the clause was written reads.

 @param cardId - card

 @param row - row

 @param position - zero-based place among the planned asks

 @param total - asks planned

 @returns Line, without the logger's prefix

 @example
 ```ts
 const line = rowLineFor({ cardId, row, position: 0, total: 88, },);
 ```
 */
export function rowLineFor(
  {
    cardId,
    row,
    position,
    total,
  }: {
    readonly cardId: RosterModelId;
    readonly row: RowSpec;
    readonly position: number;
    readonly total: number;
  },
): string {
  /**
   How the trial read.
   */
  const reading: EditorTrialReading = {
    kind: 'answered',
    score: {
      fixed: row.fixed ? 'exact' : 'none',
      collateral: false,
    },
    applied: 1,
    rejected: 0,
    findings: [],
  };
  /**
   The line with its stop clause.
   */
  const line = rowLineOf({
    position,
    total,
    pair: {
      cardId,
      trial: stubTrial({
        trialId: row.trial,
        entryId: row.entry,
        damageKind: row.trial
          .endsWith('/alteration',)
          ? 'alteration'
          : 'insertion',
      },),
      ...(((row.pass ?? 1) > 1) ? { pass: (row.pass ?? 1) - 1, } : {}),
    },
    reading,
    stops: (row.stops === '') ? [] : askEndsOf({ stops: row.stops, },),
  },);
  return (row.stops === '') ? line.slice(
    0,
    line.indexOf('; stopped ',),
  ) : line;
}

/**
 The lines an arm writes before it asks anything, by the opening writer: the manifest, the pool, the ceiling of the card,
 the spend guard and the rule.

 @param cardId - card

 @param plannedAsks - asks the ceiling is priced over

 @param maxUsd - amount from `--max-usd`

 @param benchSpentUsd - amount from `--bench-spent-usd`

 @param pool - trials the pool line counts, invented pool when omitted

 @returns Lines, without the logger's prefix

 @example
 ```ts
 const lines = openingOf({ cardId, plannedAsks: 88, maxUsd: 3, benchSpentUsd: 0, },);
 ```
 */
export function openingOf(
  {
    cardId,
    plannedAsks,
    maxUsd,
    benchSpentUsd,
    pool = poolStubs(),
  }: {
    readonly cardId: RosterModelId;
    readonly plannedAsks: number;
    readonly maxUsd: number;
    readonly benchSpentUsd: number;
    readonly pool?: readonly EditorFidelityTrial[];
  },
): readonly string[] {
  return openingLines({
    specs: [SPEC,],
    trials: pool,
    ceilings: editorCeilings({
      cardIds: [cardId,],
      sheetChars: Array.from(
        { length: plannedAsks, },
        function sheet(): number {
          return SHEET_CHARS;
        },
      ),
    },),
    maxUsd,
    benchSpentUsd,
  },);
}

//endregion Invented editor fidelity arm log lines
