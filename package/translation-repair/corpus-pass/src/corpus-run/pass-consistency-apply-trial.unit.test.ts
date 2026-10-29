/**
 Tests for the trial of one splice put back on the page as it stands: it is made, or it stays out with the first reason
 that holds with it made, in this order: its words overlap a replacement that stands, the floors refuse the slice, its own
 apostrophe is not proven, or it would unprove the apostrophe of a splice that stands; and for the two helpers it is built
 from, the slice made with the splice and what the floors say of it.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ConsistencyEdit, } from '@monochromatic-dev/translation-repair-consistency';
import {
  type ConsistencyFloorVerdict,
  floorsWithSplice,
  madeWithSplice,
  readmitLedgerOf,
  trialOf,
} from '../../dist/final/node/index.mjs';
import {
  droppedRound,
  floorsPass,
  type SecondSliceSplice,
} from './consistency-readmit-page.test-fixture.ts';

/**
 The text the published page carries around its two slices when an opener stands between them.
 */
const OPENED = ['', '\n\nShe said ‘fill it.\n\n', '',];

/**
 The splice that puts a trailing apostrophe on the family of the second slice.
 */
const FAMILY: SecondSliceSplice = { quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', };

/**
 What a trial of the first splice the round leaves out came to.

 @param second - English of the second slice
 @param edits - replacements of the second slice, group of their own each
 @param dropped - names of the replacements the round left out
 @param between - text the page carries around its slices, blank line between them where none is given
 @param floorsOn - runs the floors, which pass every slice by default
 @param asked - what the floors said of each set of replacements, empty by default
 @returns Trial
 @example
 ```ts
 const trial = trialOfFirst({ second, edits: [FAMILY,], dropped: ['group 1 OCCURRENCE 2',], },);
 ```
 */
function trialOfFirst(
  {
    second,
    edits,
    dropped,
    between,
    floorsOn = floorsPass,
    asked = new Map<string, ConsistencyFloorVerdict>(),
  }: {
    readonly second: string;
    readonly edits: readonly SecondSliceSplice[];
    readonly dropped: readonly string[];
    readonly between?: readonly string[];
    readonly floorsOn?: () => ConsistencyFloorVerdict;
    readonly asked?: Map<string, ConsistencyFloorVerdict>;
  },
): ReturnType<typeof trialOf> {
  const { page, round, } = droppedRound({ second, edits, dropped, },);
  const ledger = readmitLedgerOf({
    page: (between === undefined) ? page : { ...page, between, },
    round,
    refused: new Set(),
  },);
  const [edit,] = ledger.dropped[1] ?? [];
  if (edit === undefined)
    throw new Error('the round left no splice out',);
  return trialOf({ ledger, position: 1, edit, asked, floorsOn, },);
}

/**
 What the step knows of a page whose second slice leaves a splice out, and that splice.

 @param second - English of the second slice
 @param edits - replacements of the second slice, group of their own each
 @param dropped - names of the replacements the round left out
 @returns Ledger and first splice the second slice leaves out
 @example
 ```ts
 const { ledger, edit, } = ledgerWithSplice({ second, edits: [FAMILY,], dropped: ['group 1 OCCURRENCE 2',], },);
 ```
 */
function ledgerWithSplice(
  {
    second,
    edits,
    dropped,
  }: {
    readonly second: string;
    readonly edits: readonly SecondSliceSplice[];
    readonly dropped: readonly string[];
  },
): { readonly ledger: ReturnType<typeof readmitLedgerOf>; readonly edit: ConsistencyEdit; } {
  const { page, round, } = droppedRound({ second, edits, dropped, },);
  const ledger = readmitLedgerOf({ page, round, refused: new Set(), },);
  const [edit,] = ledger.dropped[1] ?? [];
  if (edit === undefined)
    throw new Error('the round left no splice out',);
  return { ledger, edit, };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: trialOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PUTS A SPLICE BACK WHEN ITS MARK IS PROVEN, NO SPLICE THAT STANDS IS UNPROVEN AND THE FLOORS PASS, with the '
            + 'replacements the slice then makes, the slice made and the verdict of the floors',
          fn: async () => {
            const trial = trialOfFirst({ second: 'Fill the family bowl.', edits: [FAMILY,], dropped: ['group 1 OCCURRENCE 2',], },);
            expect(trial.kind === 'admit' ? [trial.edits.length, trial.made.text, trial.verdict,] : trial,).toEqual([
              1,
              'Fill the parents’ bowl.',
              { kind: 'passes', },
            ],);
          },
        },),
        it({
          name: 'LEAVES A SPLICE OUT WHEN A MARK OF THE PAGE COULD PAIR WITH ITS APOSTROPHE AND THE FLOORS PASS THE SLICE, after '
            + 'the floors are asked',
          fn: async () => {
            /**
             How many times the floors were asked.
             */
            const asked: number[] = [];
            const trial = trialOfFirst({
              second: 'Fill the family bowl.',
              edits: [FAMILY,],
              dropped: ['group 1 OCCURRENCE 2',],
              between: OPENED,
              floorsOn(): ConsistencyFloorVerdict {
                asked.push(1,);
                return { kind: 'passes', };
              },
            },);
            expect([trial, asked,],).toEqual([{ kind: 'out', reason: { kind: 'pairs', }, }, [1,],],);
          },
        },),
        it({
          name: 'STATES THE REFUSAL OF THE FLOORS BEFORE A MARK THAT COULD PAIR WITH THE APOSTROPHE, since the refusal holds '
            + 'whatever the other gains do',
          fn: async () => {
            expect(trialOfFirst({
              second: 'Fill the family bowl.',
              edits: [FAMILY,],
              dropped: ['group 1 OCCURRENCE 2',],
              between: OPENED,
              floorsOn(): ConsistencyFloorVerdict {
                return { kind: 'refused', findings: ['the cat left',], };
              },
            },),).toEqual({ kind: 'out', reason: { kind: 'floors', findings: ['the cat left',], }, },);
          },
        },),
        it({
          name: 'STATES THE REFUSAL OF THE FLOORS BEFORE THE UNPROVING OF A MARK THAT STANDS, and asks them nothing when the '
            + 'splice\'s words overlap a replacement that stands, since no slice is made with it',
          fn: async () => {
            /**
             The words that stand and take the splice's.
             */
            const standing: SecondSliceSplice = { quote: 'the family bowl', chosen: 'the dish', start: 5, end: 20, text: 'the dish', };
            /**
             The splices that touch, the second joining the word the first's mark ends.
             */
            const touching: SecondSliceSplice = { quote: 'bowl', chosen: 'the pups’', start: 15, end: 20, text: 'bowls’', };
            /**
             How many times the floors were asked about the overlapping case.
             */
            const asked: number[] = [];
            expect([
              trialOfFirst({
                second: 'Fill the family bowl.',
                edits: [FAMILY, touching,],
                dropped: ['group 2 OCCURRENCE 2',],
                floorsOn(): ConsistencyFloorVerdict {
                  return { kind: 'refused', findings: ['the cat left',], };
                },
              },),
              trialOfFirst({
                second: 'Fill the family bowl.',
                edits: [standing, FAMILY,],
                dropped: ['group 2 OCCURRENCE 2',],
                floorsOn(): ConsistencyFloorVerdict {
                  asked.push(1,);
                  return { kind: 'refused', findings: ['the cat left',], };
                },
              },),
              asked,
            ],).toEqual([
              { kind: 'out', reason: { kind: 'floors', findings: ['the cat left',], }, },
              { kind: 'out', reason: { kind: 'overlaps', }, },
              [],
            ],);
          },
        },),
        it({
          name: 'LEAVES A SPLICE OUT WHEN ITS WORDS OVERLAP A REPLACEMENT THAT STANDS',
          fn: async () => {
            const standing: SecondSliceSplice = { quote: 'the family bowl', chosen: 'the dish', start: 5, end: 20, text: 'the dish', };
            expect(trialOfFirst({
              second: 'Fill the family bowl.',
              edits: [standing, FAMILY,],
              dropped: ['group 2 OCCURRENCE 2',],
            },),).toEqual({ kind: 'out', reason: { kind: 'overlaps', }, },);
          },
        },),
        it({
          name: 'LEAVES A SPLICE OUT WHEN IT WOULD UNPROVE THE APOSTROPHE A SPLICE THAT STANDS CARRIES IN: two splices that touch, '
            + 'the mark of the first joining the word the second begins',
          fn: async () => {
            const touching: SecondSliceSplice = { quote: 'bowl', chosen: 'the pups’', start: 15, end: 20, text: 'bowls’', };
            expect(trialOfFirst({
              second: 'Fill the family bowl.',
              edits: [FAMILY, touching,],
              dropped: ['group 2 OCCURRENCE 2',],
            },),).toEqual({ kind: 'out', reason: { kind: 'unproves', }, },);
          },
        },),
        it({
          name: 'LEAVES A SPLICE OUT WHEN THE FLOORS REFUSE THE SLICE WITH IT, with what they found, and asks the floors once '
            + 'about the same replacements',
          fn: async () => {
            /**
             How many times the floors were asked.
             */
            const asking: number[] = [];
            const asked = new Map<string, ConsistencyFloorVerdict>();
            const refuse = {
              second: 'Fill the family bowl.',
              edits: [FAMILY,],
              dropped: ['group 1 OCCURRENCE 2',],
              asked,
              floorsOn(): ConsistencyFloorVerdict {
                asking.push(1,);
                return { kind: 'refused', findings: ['the cat left',], };
              },
            };
            const first = trialOfFirst(refuse,);
            const second = trialOfFirst(refuse,);
            expect([first, second, asking.length, [...asked.keys(),],],).toEqual([
              { kind: 'out', reason: { kind: 'floors', findings: ['the cat left',], }, },
              { kind: 'out', reason: { kind: 'floors', findings: ['the cat left',], }, },
              1,
              ['1 group 1 OCCURRENCE 2',],
            ],);
          },
        },),
        it({
          name: 'REFUSES A SLICE WITH NO PLAN, naming its place, since only a slice with a plan leaves a splice out',
          fn: async () => {
            const { page, round, } = droppedRound({ second: 'Fill the family bowl.', edits: [FAMILY,], dropped: ['group 1 OCCURRENCE 2',], },);
            const ledger = readmitLedgerOf({ page, round, refused: new Set(), },);
            const [edit,] = ledger.dropped[1] ?? [];
            if (edit === undefined)
              throw new Error('the round left no splice out',);
            const refusal = caught(function triesInFirstSlice(): unknown {
              return trialOf({ ledger, position: 0, edit, asked: new Map(), floorsOn: floorsPass, },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: slice 0 of the page left out a splice and has no plan, and only a slice '
                + 'with a plan leaves one out',
            );
          },
        },),
      ],
    },),
    describe({
      name: madeWithSplice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES THE SLICE WITH THE SPLICE ADDED, the replacements it then makes and the slice made, and says the splice '
            + 'overlaps nothing that stands',
          fn: async () => {
            const { ledger, edit, } = ledgerWithSplice({
              second: 'Fill the family bowl.',
              edits: [FAMILY,],
              dropped: ['group 1 OCCURRENCE 2',],
            },);
            const withSplice = madeWithSplice({ ledger, position: 1, edit, },);
            expect([withSplice.edits.length, withSplice.made.text, withSplice.overlaps,],).toEqual([
              1,
              'Fill the parents’ bowl.',
              false,
            ],);
          },
        },),
        it({
          name: 'SAYS THE SPLICE OVERLAPS WHEN ITS WORDS TOUCH A REPLACEMENT THAT STANDS, since the slice then cannot make both',
          fn: async () => {
            const standing: SecondSliceSplice = { quote: 'the family bowl', chosen: 'the dish', start: 5, end: 20, text: 'the dish', };
            const withSplice = ledgerWithSplice({
              second: 'Fill the family bowl.',
              edits: [standing, FAMILY,],
              dropped: ['group 2 OCCURRENCE 2',],
            },);
            expect(madeWithSplice({ ledger: withSplice.ledger, position: 1, edit: withSplice.edit, },).overlaps,).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES A SLICE WITH NO PLAN, naming its place',
          fn: async () => {
            const { ledger, edit, } = ledgerWithSplice({
              second: 'Fill the family bowl.',
              edits: [FAMILY,],
              dropped: ['group 1 OCCURRENCE 2',],
            },);
            const refusal = caught(function triesInFirstSlice(): unknown {
              return madeWithSplice({ ledger, position: 0, edit, },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: slice 0 of the page left out a splice and has no plan, and only a slice '
                + 'with a plan leaves one out',
            );
          },
        },),
      ],
    },),
    describe({
      name: floorsWithSplice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS THE FLOORS ABOUT THE SLICE WITH THE SPLICE MADE ONCE, and gives their verdict each time it is asked',
          fn: async () => {
            const { ledger, edit, } = ledgerWithSplice({
              second: 'Fill the family bowl.',
              edits: [FAMILY,],
              dropped: ['group 1 OCCURRENCE 2',],
            },);
            const asked = new Map<string, ConsistencyFloorVerdict>();
            /**
             The texts the floors were given.
             */
            const given: string[] = [];
            /**
             Refuses every slice, noting the text it was given.

             @param candidateText - slice with its replacements made
             @returns Refusal
             */
            function floorsOn({ candidateText, }: { readonly candidateText: string; },): ConsistencyFloorVerdict {
              given.push(candidateText,);
              return { kind: 'refused', findings: ['the cat left',], };
            }
            const first = floorsWithSplice({ ledger, position: 1, edit, asked, floorsOn, },);
            const second = floorsWithSplice({ ledger, position: 1, edit, asked, floorsOn, },);
            expect([first, second, given, [...asked.keys(),],],).toEqual([
              [{ kind: 'refused', findings: ['the cat left',], },],
              [{ kind: 'refused', findings: ['the cat left',], },],
              ['Fill the parents’ bowl.',],
              ['1 group 1 OCCURRENCE 2',],
            ],);
          },
        },),
        it({
          name: 'GIVES NO VERDICT AND ASKS NOTHING WHEN THE SPLICE OVERLAPS A REPLACEMENT THAT STANDS, since no slice is made with it',
          fn: async () => {
            const standing: SecondSliceSplice = { quote: 'the family bowl', chosen: 'the dish', start: 5, end: 20, text: 'the dish', };
            const withSplice = ledgerWithSplice({
              second: 'Fill the family bowl.',
              edits: [standing, FAMILY,],
              dropped: ['group 2 OCCURRENCE 2',],
            },);
            /**
             How many times the floors were asked.
             */
            const asking: number[] = [];
            const verdicts = floorsWithSplice({
              ledger: withSplice.ledger,
              position: 1,
              edit: withSplice.edit,
              asked: new Map(),
              floorsOn(): ConsistencyFloorVerdict {
                asking.push(1,);
                return { kind: 'passes', };
              },
            },);
            expect([verdicts, asking,],).toEqual([[], [],],);
          },
        },),
      ],
    },),
  ],
},);
