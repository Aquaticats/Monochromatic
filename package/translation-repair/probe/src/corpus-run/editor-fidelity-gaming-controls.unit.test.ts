/**
 Gaming controls of the editor fidelity probe: the cheap replies a re-check found would have scored as a fix, each
 read through the real apply gate and the real scoring, and tallied for the headline. No model is called; a scripted
 client says each reply. Fixtures are cat-themed invention.

 The `measured` text of each case is what the re-check of the frozen pool of the third round (41 trials, 23
 insertions and 18 alterations) recorded for the same reply, when the headline counted defect-only fixes and the
 insertion region was the inserted sentence alone.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  EDITOR_DAMAGE_KINDS,
  type EditorFidelityRow,
  editorFidelityRow,
  type EditorFidelityTrial,
  editorFidelityTrials,
  editorTallyOf,
} from '../../dist/final/node/index.mjs';
import {
  inventedReference,
  LOW_CARD,
  regionOf,
} from '../editor-fidelity.test-fixture.ts';
import {
  noAsks,
  readingFor,
} from './editor-fidelity-scripted-client.test-fixture.ts';

/**
 The invented reference's trials.
 */
const TRIALS = editorFidelityTrials({
  references: [inventedReference(),],
  damageKinds: EDITOR_DAMAGE_KINDS,
},);

/**
 The insertion trial.
 */
const INSERTION: EditorFidelityTrial = nonNullishOrThrow(TRIALS.find(function isInsertion(trial,): boolean {
  return trial.damageKind === 'insertion';
},),);

/**
 The alteration trial.
 */
const ALTERATION: EditorFidelityTrial = nonNullishOrThrow(TRIALS.find(function isAlteration(trial,): boolean {
  return trial.damageKind === 'alteration';
},),);

/**
 The quoted region with its defect mended: the sentence gone from an insertion, the number back in an alteration.
 */
const IDEAL: Readonly<Record<EditorFidelityTrial['damageKind'], string>> = {
  insertion: regionOf({ trial: INSERTION, },)
    .replace(
    INSERTION.damagedDefect,
    '',
    ),
  alteration: regionOf({ trial: ALTERATION, },)
    .replace(
    ALTERATION.damagedDefect,
    ALTERATION.cleanDefect,
    ),
};

/**
 One scripted reply, what the re-check measured for it and what it reads as now.
 */
type Control = {
  /**
   Handle of the control.
   */
  readonly name: string;

  /**
   Trial the reply answers.
   */
  readonly trial: EditorFidelityTrial;

  /**
   The report the scripted editor sends.
   */
  readonly report: unknown;

  /**
   What the re-check measured, in its own words.
   */
  readonly measured: string;

  /**
   How the trial reads now: an answered score, or the word for a reply with no landed edit.
   */
  readonly reads: 'refused' | 'unusable' | { readonly fixed: string; readonly collateral: boolean; };

  /**
   Whether the reply counts toward the headline.
   */
  readonly counts: boolean;
};

/**
 The report that replaces the quoted region.

 @param newText - what the region becomes

 @returns The report body

 @example
 ```ts
 const report = replaceRegion({ newText: '', },);
 ```
 */
function replaceRegion({ newText, }: { readonly newText: string; },): unknown {
  return {
    edits: [
      {
        region: 1,
        newText,
      },
    ],
  };
}

/**
 Every cheap reply, once per kind it applies to, and the two replies that must count.
 */
const CONTROLS: readonly Control[] = [
  {
    name: 'insertion: delete the region',
    trial: INSERTION,
    report: replaceRegion({ newText: '', },),
    measured: 'equivalent, nothing else touched, in 22 of 23 insertions (counted fixed)',
    reads: {
      fixed: 'defect-only',
      collateral: true,
    },
    counts: false,
  },
  {
    name: 'insertion: blank the region',
    trial: INSERTION,
    report: replaceRegion({ newText: ' ', },),
    measured: 'equivalent, nothing else touched, in 22 of 23 insertions (counted fixed)',
    reads: {
      fixed: 'defect-only',
      collateral: true,
    },
    counts: false,
  },
  {
    name: 'insertion: the perfect fix plus a trailing mark',
    trial: INSERTION,
    report: replaceRegion({ newText: `${IDEAL.insertion}!`, },),
    measured: 'not measured on insertions; 18 of 18 alterations read defect-only and counted fixed',
    reads: {
      fixed: 'defect-only',
      collateral: true,
    },
    counts: false,
  },
  {
    name: 'insertion: echo the region',
    trial: INSERTION,
    report: replaceRegion({ newText: regionOf({ trial: INSERTION, },), },),
    measured: 'unusable (edits refused) in 22 of 23 insertions, not fixed in the 23rd',
    reads: 'unusable',
    counts: false,
  },
  {
    name: 'insertion: no edits',
    trial: INSERTION,
    report: { edits: [], },
    measured: 'refused, never fixed',
    reads: 'refused',
    counts: false,
  },
  {
    name: 'alteration: the clean number alone',
    trial: ALTERATION,
    report: replaceRegion({ newText: ALTERATION.cleanDefect, },),
    measured: 'defect-only with collateral in 17 of 18 (counted fixed); the 18th refused by the gate',
    reads: {
      fixed: 'defect-only',
      collateral: true,
    },
    counts: false,
  },
  {
    name: 'alteration: the perfect fix plus a trailing mark',
    trial: ALTERATION,
    report: replaceRegion({ newText: `${IDEAL.alteration}!`, },),
    measured: 'defect-only with collateral in 18 of 18 (counted fixed)',
    reads: {
      fixed: 'defect-only',
      collateral: true,
    },
    counts: false,
  },
  {
    name: 'alteration: echo the region',
    trial: ALTERATION,
    report: replaceRegion({ newText: regionOf({ trial: ALTERATION, },), },),
    measured: 'unusable (edits refused) in 18 of 18, never fixed',
    reads: 'unusable',
    counts: false,
  },
  {
    name: 'alteration: no edits',
    trial: ALTERATION,
    report: { edits: [], },
    measured: 'refused, never fixed',
    reads: 'refused',
    counts: false,
  },
  {
    name: 'alteration: a wrong number',
    trial: ALTERATION,
    report: replaceRegion({
      newText: regionOf({ trial: ALTERATION, },)
        .replace(
        ALTERATION.damagedDefect,
        '1999',
        ),
    },),
    measured: 'not fixed in 18 of 18',
    reads: {
      fixed: 'none',
      collateral: false,
    },
    counts: false,
  },
  {
    name: 'insertion: the sentence removed, neighbours kept (must count)',
    trial: INSERTION,
    report: replaceRegion({ newText: IDEAL.insertion, },),
    measured: 'equivalent in 22 of 23, defect-only with collateral in the 23rd',
    reads: {
      fixed: 'exact',
      collateral: false,
    },
    counts: true,
  },
  {
    name: 'alteration: the number restored (must count)',
    trial: ALTERATION,
    report: replaceRegion({ newText: IDEAL.alteration, },),
    measured: 'exact',
    reads: {
      fixed: 'exact',
      collateral: false,
    },
    counts: true,
  },
];

/**
 Says in a word how a control is expected to read.

 @param control - control read

 @returns The word for a reply with no landed edit, or the fix kind with whether something else changed

 @example
 ```ts
 const word = readsWord({ control, },);
 ```
 */
function readsWord({ control, }: { readonly control: Control; },): string {
  if ((typeof control.reads) === 'string')
    return control.reads;
  return control.reads.collateral
    ? `${control.reads.fixed} with collateral`
    : control.reads.fixed;
}

/**
 The row a control produces.

 @param control - control read

 @returns The row as the probe keeps it

 @example
 ```ts
 const row = await rowOfControl({ control, },);
 ```
 */
async function rowOfControl({ control, }: { readonly control: Control; },): Promise<EditorFidelityRow> {
  return editorFidelityRow({
    cardId: LOW_CARD,
    trial: control.trial,
    reading: await readingFor({
      trial: control.trial,
      report: control.report,
      asked: noAsks(),
    },),
  },);
}

await describe({
  name: 'gaming controls',
  children: [
    ...CONTROLS.map(function itReads(control,) {
      return it({
        name: `READS ${control.name.toUpperCase()} as ${readsWord({ control, },)}, ${
          control.counts ? 'counted' : 'not counted'
        } in the headline (re-check measured: ${control.measured})`,
        fn: async () => {
          /**
           How the trial read, and what the tally of that one row makes of it.
           */
          const row = await rowOfControl({ control, },);
          const tally = editorTallyOf({
            cardId: LOW_CARD,
            rows: [row,],
          },);
          expect(tally.fixed,).toBe(control.counts ? 1 : 0,);
          if ((typeof control.reads) === 'string') {
            expect(row.reading.kind,).toBe(control.reads,);
            return;
          }
          expect(row.reading.kind,).toBe('answered',);
          if (row.reading.kind === 'answered')
            expect(row.reading.score,).toStrictEqual(control.reads,);
        },
      },);
    },),

    it({
      name: 'COUNTS NO CHEAP REPLY toward the headline over a card that sends every one of them: the headline is '
        + 'the two replies that fix the defect cleanly, and the cheap replies add to nothing but their own lines',
      fn: async () => {
        /**
         Every control's row, in order.
         */
        const rows = await Promise.all(CONTROLS.map(function rowOf(control,): Promise<EditorFidelityRow> {
          return rowOfControl({ control, },);
        },),);
        const tally = editorTallyOf({
          cardId: LOW_CARD,
          rows,
        },);
        expect(tally.trials,).toBe(CONTROLS.length,);
        expect(tally.fixed,).toBe(2,);
        expect(tally.byKind.insertion.exact + tally.byKind.insertion.equivalent,).toBe(1,);
        expect(tally.byKind.alteration.exact + tally.byKind.alteration.equivalent,).toBe(1,);
        expect(tally.defectOnly,).toBe(5,);
      },
    },),
  ],
},);
