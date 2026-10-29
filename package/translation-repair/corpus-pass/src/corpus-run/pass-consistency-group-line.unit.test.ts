/**
 Tests for the line the consistency pass logs for each group (`pass-consistency-group-line.ts`).

 WHAT THIS FILE EXISTS TO STOP. The settled TianqiChen66624 run ended with
 `group 1 of 2: refrain verdict=asked placed=4 unplaced=0 replaced=0 consistent=0 kept-context=0 kept-floor=0 overlapped=0
 unlocated=0`: the bench was asked, nothing changed, and the line named no reason, because three of the nine outcomes
 (`chosen`, `kept-current`, `untouched`) had no column and the reasons the record holds were not printed. Every path by
 which a place is neither replaced nor counted by another column is driven here with a hand-built record, and each line is
 read for the reason the code's own decision gives.

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ConsistencyInvariantError, } from '@monochromatic-dev/translation-repair-consistency';
import { groupLine, } from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 The outcomes a place can end in, as the record names them.
 */
type Outcome =
  | 'chosen'
    | 'replaced'
    | 'consistent'
    | 'kept-context'
    | 'kept-floor'
    | 'kept-current'
    | 'overlapped'
    | 'unlocated'
    | 'untouched';

/**
 One recorded place of a hand-built group.
 */
type Place = {
  readonly label: string;
  readonly outcome: Outcome;
  readonly keptBecause?: string;
};

/**
 The group the function under test reads, which the hand-built records of this file stand in for.
 */
type RecordedGroup = Parameters<typeof groupLine>[0]['group'];

/**
 A recorded place with the outcome it ended in.

 @param position - zero-based position, which names the label
 @param outcome - outcome the place ended in
 @param keptBecause - reason the pass left the place as it was, where the record has one
 @returns Place
 */
function placeOf(
  { position, outcome, keptBecause, }: { readonly position: number; readonly outcome: Outcome; readonly keptBecause?: string; },
): Place {
  return {
    label: `OCCURRENCE ${String(position + 1,)}`,
    outcome,
    ...((keptBecause === undefined) ? {} : { keptBecause, }),
  };
}

/**
 A recorded group.

 @param outcomes - outcome of each place, in order
 @param verdict - what was asked of the bench and what it answered
 @param carries - how the chosen rendering was carried to the receiving places
 @param keptBecause - reason recorded for the place at a position
 @param kind - repeat kind
 @returns Group as the record holds it
 */
function groupOf(
  { outcomes, verdict, carries = [], keptBecause = {}, kind = 'refrain', }: {
    readonly outcomes: readonly Outcome[];
    readonly verdict: unknown;
    readonly carries?: readonly unknown[];
    readonly keptBecause?: Readonly<Record<number, string>>;
    readonly kind?: 'maximal' | 'refrain';
  },
): RecordedGroup {
  return {
    repeat: (kind === 'refrain') ? { kind, prefix: 'Meow ', suffix: '.', } : { kind, wording: 'meow', },
    occurrences: outcomes.map(function recorded(outcome, position,): Place {
      return placeOf({ position, outcome, ...((keptBecause[position] === undefined) ? {} : { keptBecause: keptBecause[position], }), },);
    },),
    unplaced: [],
    verdict,
    carries,
  } as unknown as RecordedGroup;
}

/**
 A verdict of an asked bench with the winner given.
 */
function asked({ winner, }: { readonly winner: unknown; },): unknown {
  return { kind: 'asked', ballots: [], usable: 3, quorum: 3, winner, };
}

/**
 The record of a carry to one place.
 */
function carryOf(
  { position, decision, kinds = ['current', 'splice',], }: {
    readonly position: number;
    readonly decision: unknown;
    readonly kinds?: readonly string[];
  },
): unknown {
  return {
    label: `OCCURRENCE ${String(position + 1,)}`,
    candidates: kinds.map(function candidate(kind, index,): unknown {
      return { label: `CANDIDATE ${String(index + 1,)}`, kind, writers: [], text: 'The cat naps.', };
    },),
    ballots: [],
    usable: 3,
    quorum: 3,
    decision,
  };
}

/**
 The text after `because=` on a line, empty where there is none.
 */
function becauseOf({ line, }: { readonly line: string; },): string {
  if (!line.includes('because=',))
    return '';
  return line.slice(line.indexOf('because=',) + 'because='.length,);
}

//endregion Fixtures

await describe({
  name: groupLine.name,
  concurrency: 1,
  children: [
    describe({
      name: 'columns',
      children: [
        it({
          name: 'IS EXPORTED by the package',
          fn: async () => {
            expect(typeof groupLine,).toBe('function',);
          },
        },),

        it({
          name: 'COUNTS every outcome, so the columns add up to the places placed',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'replaced', 'consistent', 'kept-context', 'kept-floor', 'kept-current', 'overlapped', 'unlocated', 'untouched',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                carries: [carryOf({ position: 5, decision: { kind: 'none', reason: 'no-majority', }, },),],
              },),
            },);
            for (const column of [
              'replaced=1',
              'consistent=1',
              'kept-context=1',
              'kept-floor=1',
              'overlapped=1',
              'unlocated=1',
              'chosen=1',
              'kept-current=1',
              'untouched=1',
            ]) {
              expect(line,).toContain(` ${column}`,);
            }
            expect(line,).toContain('placed=9',);
          },
        },),

        it({
          name: 'NAMES NO REASON for a group whose places a column counts (replaced, floored, consistent)',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'replaced', 'replaced', 'kept-floor', 'consistent',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
              },),
            },);
            expect(line,).not.toContain('because=',);
          },
        },),
      ],
    },),

    describe({
      name: 'a group whose bench was not asked',
      children: [
        ...(['too-few-placed', 'no-english',] as const).map(function notAsked(reason,) {
          return it({
            name: `NAMES the reason ${reason}`,
            fn: async () => {
              const line = groupLine({
                groupNumber: 1,
                group: groupOf({ outcomes: ['untouched', 'untouched',], verdict: { kind: 'not-asked', reason, }, },),
              },);
              expect(becauseOf({ line, },),).toBe(`not-asked:${reason}`,);
              expect(line,).toContain(' untouched=2',);
            },
          },);
        },),

        it({
          name: 'NAMES NO REASON where the group was already consistent, whose places the consistent column counts',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({ outcomes: ['consistent', 'consistent',], verdict: { kind: 'not-asked', reason: 'consistent', }, },),
            },);
            expect(line,).not.toContain('because=',);
            expect(line,).toContain(' consistent=2',);
          },
        },),

        it({
          name: 'NAMES the failure of a group that trouble stopped',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({ outcomes: ['untouched', 'untouched', 'untouched',], verdict: { kind: 'failed', detail: 'TimeoutError', }, },),
            },);
            expect(becauseOf({ line, },),).toBe('failed:TimeoutError',);
          },
        },),
      ],
    },),

    describe({
      name: 'an asked group with no winner',
      children: (['no-quorum', 'no-majority', 'chosen-unlocated', 'run-off-no-quorum', 'run-off-tied',] as const).map(function noWinner(reason,) {
        return it({
          name: `NAMES the winner reason ${reason}, the four places untouched`,
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['untouched', 'untouched', 'untouched', 'untouched',],
                verdict: asked({ winner: { kind: 'none', reason, }, },),
              },),
            },);
            expect(becauseOf({ line, },),).toBe(`no-winner:${reason}`,);
            expect(line,).toContain(' untouched=4',);
            expect(line,).toContain(' replaced=0',);
          },
        },);
      },),
    },),

    describe({
      name: 'an asked group with a winner whose other places kept their English',
      children: [
        ...(['no-alternative', 'no-quorum', 'no-majority',] as const).map(function noDecision(reason,) {
          return it({
            name: `NAMES the carry reason ${reason}, three places kept-current beside the chosen one (the shape of TianqiChen66624 group 1)`,
            fn: async () => {
              const line = groupLine({
                groupNumber: 1,
                group: groupOf({
                  outcomes: ['chosen', 'kept-current', 'kept-current', 'kept-current',],
                  verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                  carries: [1, 2, 3,].map(function carry(position,): unknown {
                    return carryOf({ position, decision: { kind: 'none', reason, }, },);
                  },),
                },),
              },);
              expect(becauseOf({ line, },),).toBe(`carry:${reason}`,);
              expect(line,).toContain(' chosen=1',);
              expect(line,).toContain(' kept-current=3',);
              expect(line,).toContain(' replaced=0',);
            },
          },);
        },),

        it({
          name: 'NAMES a carry that failed',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'kept-current',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                carries: [carryOf({ position: 1, decision: { kind: 'failed', detail: 'Error', }, },),],
              },),
            },);
            expect(becauseOf({ line, },),).toBe('carry:failed',);
          },
        },),

        it({
          name: 'NAMES a choice that kept the English as it stands',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'kept-current',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                carries: [carryOf({ position: 1, decision: { kind: 'chosen', label: 'CANDIDATE 1', }, },),],
              },),
            },);
            expect(becauseOf({ line, },),).toBe('carry:chose-current',);
          },
        },),

        it({
          name: 'NAMES a splice that cannot reach the place, because the place has no words code can move',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'kept-current',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                carries: [carryOf({ position: 1, decision: { kind: 'chosen', label: 'CANDIDATE 2', }, },),],
              },),
            },);
            expect(becauseOf({ line, },),).toBe('carry:splice-cannot-reach-place',);
          },
        },),

        ...([
          'gain-pairs',
          'gain-unproves',
          'gain-floors',
          'gain-overlaps',
          'gain-refused',
          'edge-second-mark',
          'edge-reading-changes',
          'edge-reference-mark',
        ] as const).map(function keptReason(reason,) {
          return it({
            name: `NAMES the recorded reason ${reason} ahead of the carry's decision`,
            fn: async () => {
              const line = groupLine({
                groupNumber: 1,
                group: groupOf({
                  outcomes: ['chosen', 'kept-current',],
                  verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                  carries: [carryOf({ position: 1, decision: { kind: 'chosen', label: 'CANDIDATE 2', }, },),],
                  keptBecause: { 1: reason, },
                },),
              },);
              expect(becauseOf({ line, },),).toBe(`kept:${reason}`,);
            },
          },);
        },),

        it({
          name: 'NAMES a place with no carry record, rather than guessing',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'kept-current',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
              },),
            },);
            expect(becauseOf({ line, },),).toBe('carry:unrecorded',);
          },
        },),

        it({
          name: 'LISTS each distinct reason once, in the order the places stand, when places differ',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'replaced', 'kept-current', 'kept-current', 'kept-current',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                carries: [
                  carryOf({ position: 2, decision: { kind: 'none', reason: 'no-majority', }, },),
                  carryOf({ position: 3, decision: { kind: 'none', reason: 'no-majority', }, },),
                  carryOf({ position: 4, decision: { kind: 'failed', detail: 'Error', }, },),
                ],
              },),
            },);
            expect(becauseOf({ line, },),).toBe('carry:no-majority,carry:failed',);
            expect(line,).toContain(' replaced=1',);
            expect(line,).toContain(' kept-current=3',);
          },
        },),
      ],
    },),

    describe({
      name: 'a kept-current place whose carry offered no splice (n166, Z1)',
      children: [
        it({
          name: 'NAMES the carry decision and not the recorded edge reason, where the carry offered the writers and the current text '
            + 'and the bench kept the current text',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'kept-current',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                carries: [carryOf({ position: 1, decision: { kind: 'chosen', label: 'CANDIDATE 1', }, kinds: ['current', 'writer',], },),],
                keptBecause: { 1: 'edge-second-mark', },
              },),
            },);
            expect(becauseOf({ line, },),).toBe('carry:chose-current',);
            expect(line,).not.toContain('kept:',);
          },
        },),

        ...([
          'gain-pairs',
          'gain-unproves',
          'gain-floors',
          'gain-overlaps',
          'gain-refused',
          'edge-second-mark',
          'edge-reading-changes',
          'edge-reference-mark',
        ] as const).map(function notOffered(reason,) {
          return it({
            name: `NAMES the carry decision, not the recorded reason ${reason}, where no candidate of the carry is the splice`,
            fn: async () => {
              const line = groupLine({
                groupNumber: 1,
                group: groupOf({
                  outcomes: ['chosen', 'kept-current',],
                  verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                  carries: [carryOf({ position: 1, decision: { kind: 'none', reason: 'no-majority', }, kinds: ['current', 'writer',], },),],
                  keptBecause: { 1: reason, },
                },),
              },);
              expect(becauseOf({ line, },),).toBe('carry:no-majority',);
            },
          },);
        },),

        it({
          name: 'NAMES the carry record as missing, not the recorded reason, where the record holds no carry for a place that has a reason',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'kept-current',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                keptBecause: { 1: 'edge-second-mark', },
              },),
            },);
            expect(becauseOf({ line, },),).toBe('carry:unrecorded',);
          },
        },),

        it({
          name: 'NAMES the recorded reason for a place whose carry offered the splice and the carry decision for one whose carry did not, '
            + 'in the order the places stand',
          fn: async () => {
            const line = groupLine({
              groupNumber: 1,
              group: groupOf({
                outcomes: ['chosen', 'kept-current', 'kept-current',],
                verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                carries: [
                  carryOf({ position: 1, decision: { kind: 'chosen', label: 'CANDIDATE 2', }, },),
                  carryOf({ position: 2, decision: { kind: 'none', reason: 'no-majority', }, kinds: ['current', 'writer',], },),
                ],
                keptBecause: { 1: 'edge-second-mark', 2: 'edge-second-mark', },
              },),
            },);
            expect(becauseOf({ line, },),).toBe('kept:edge-second-mark,carry:no-majority',);
          },
        },),
      ],
    },),

    describe({
      name: 'a record the pass cannot produce (n166, Z2)',
      children: [
        it({
          name: 'THROWS an invariant error naming the group and the place when the carry chose a label no candidate of the record holds',
          fn: async () => {
            const refusal = caught(function unexplainedLabel(): unknown {
              return groupLine({
                groupNumber: 2,
                group: groupOf({
                  outcomes: ['chosen', 'kept-current',],
                  verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                  carries: [carryOf({ position: 1, decision: { kind: 'chosen', label: 'CANDIDATE 9', }, },),],
                },),
              },);
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: group 2 OCCURRENCE 2 ended kept-current and its carry chose CANDIDATE 9, '
                + 'which is not among the candidates the carry recorded (2 recorded)',
            );
          },
        },),

        it({
          name: 'THROWS an invariant error naming the group and the place when a writer text was chosen for a place that ended kept-current',
          fn: async () => {
            const refusal = caught(function writerOnKeptPlace(): unknown {
              return groupLine({
                groupNumber: 2,
                group: groupOf({
                  outcomes: ['chosen', 'kept-current',],
                  verdict: asked({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
                  carries: [carryOf({ position: 1, decision: { kind: 'chosen', label: 'CANDIDATE 3', }, kinds: ['current', 'splice', 'writer',], },),],
                },),
              },);
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: group 2 OCCURRENCE 2 ended kept-current and its carry chose CANDIDATE 3, a '
                + 'writer text, and a writer text that is chosen replaces every receiving place of its block',
            );
          },
        },),

        it({
          name: 'THROWS an invariant error naming the group and the place when a verdict that named no winner stands beside a replaced place',
          fn: async () => {
            const refusal = caught(function replacedWithoutWinner(): unknown {
              return groupLine({
                groupNumber: 2,
                group: groupOf({
                  outcomes: ['untouched', 'replaced',],
                  verdict: asked({ winner: { kind: 'none', reason: 'no-quorum', }, },),
                },),
              },);
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: group 2 holds a verdict that named no winner (no-quorum) and OCCURRENCE 2 ended '
                + 'replaced, and a group whose bench named no winner replaces no place',
            );
          },
        },),

        it({
          name: 'DOES NOT THROW for a no-winner verdict whose places all stood, the case every group of that verdict is',
          fn: async () => {
            const line = groupLine({
              groupNumber: 2,
              group: groupOf({
                outcomes: ['untouched', 'untouched',],
                verdict: asked({ winner: { kind: 'none', reason: 'no-quorum', }, },),
              },),
            },);
            expect(becauseOf({ line, },),).toBe('no-winner:no-quorum',);
          },
        },),
      ],
    },),
  ],
},);
