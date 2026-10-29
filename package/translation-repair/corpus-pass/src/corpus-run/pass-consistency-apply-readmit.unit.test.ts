/**
 Tests for putting back the code splices the gain check dropped but the final page would carry. The check drops every
 splice whose carried apostrophe is not proven on the page as the round saw it, all at once, and a gain left out because of
 another gain that a floor then refused, or because of a gain that is itself left out, could have stood. After the check and
 the floors agree, the largest set of the dropped gains the page can hold is put back where at most twelve are left out
 (every set is looked at), and otherwise the dropped gains are tried again one at a time in page order on the final page: a
 gain is put back when its mark is proven there with it made, every gain made stays proven and the floors pass its slice,
 and the pass repeats until it puts back none. A gain left out says the reason that holds on the final page.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pageOfSlices,
  provenApostrophesInEnglish,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  applyConsistency,
  type ConsistencyFloorVerdict,
  consistencyFloorVerdict,
  readmittedSlicesOf,
  type SettledGroup,
  type SettledSlice,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  pickFrom,
  randomOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/seeded-draws.test-fixture.ts';
import {
  groupIn,
  withEdit,
} from './consistency-apply-group.test-fixture.ts';
import {
  droppedRound,
  floorsPass,
  type SecondSliceSplice,
} from './consistency-readmit-page.test-fixture.ts';
import { factsOf, } from './consistency-round-page.test-fixture.ts';

//region Gain readmission tests

/**
 The first slice of the page whose gains pair: two chosen sentences, each with a plural possessive.
 */
const OVERDROP_CHOSEN = 'Bowl 1 of the parents’ is full. Bowl 2 of the parents’ is full.';

/**
 The second slice of that page: the receiving words hold a declared name.
 */
const OVERDROP_FIRST = 'Then Bowl 1 of the kennel is empty.';

/**
 The third slice of that page.
 */
const OVERDROP_SECOND = 'Then Bowl 2 of the family is empty.';

/**
 What applying the page of two gains that pair came to, for a case to compare.

 @param declaredNames - declared name forms the floors keep in each slice
 @param leading - whether the second slice's gain is a leading mark, which pairs with the third slice's trailing one
 @returns Outcome of each place of each group, slices that changed with their text, and findings
 @example
 ```ts
 const seen = overdropApplied({ declaredNames: [], leading: true, },);
 ```
 */
function overdropApplied(
  {
    declaredNames,
    leading,
  }: {
    readonly declaredNames: readonly string[];
    readonly leading: boolean;
  },
): {
  readonly outcomes: readonly (readonly string[])[];
  readonly changed: readonly (readonly [number, string])[];
  readonly findings: readonly string[];
  readonly keptBecause: readonly string[];
} {
  /**
   The receiving words of the second and of the third slice.
   */
  const first = 'Bowl 1 of the kennel';
  const second = 'Bowl 2 of the family';
  const firstStart = OVERDROP_FIRST.indexOf(first,);
  const secondStart = OVERDROP_SECOND.indexOf(second,);
  const applied = applyConsistency({
    settled: [
      withEdit({
        group: groupIn({
          places: [
            { slice: 0, english: OVERDROP_CHOSEN, quote: 'Bowl 1 of the parents’', },
            { slice: 1, english: OVERDROP_FIRST, quote: first, },
          ],
        },),
        edit: {
          kind: 'splice',
          placePositions: [1,],
          sliceIndex: 1,
          start: firstStart,
          end: firstStart + first.length,
          text: leading ? '’Bowl 1 of the folks' : 'Bowl 1 of the folks',
        },
      },),
      withEdit({
        group: groupIn({
          places: [
            { slice: 0, english: OVERDROP_CHOSEN, quote: 'Bowl 2 of the parents’', },
            { slice: 2, english: OVERDROP_SECOND, quote: second, },
          ],
        },),
        edit: {
          kind: 'splice',
          placePositions: [1,],
          sliceIndex: 2,
          start: secondStart,
          end: secondStart + second.length,
          text: 'Bowl 2 of the parents’',
        },
      },),
    ],
    slices: factsOf({
      englishes: [
        { sliceIndex: 0, english: OVERDROP_CHOSEN, },
        { sliceIndex: 1, english: OVERDROP_FIRST, },
        { sliceIndex: 2, english: OVERDROP_SECOND, },
      ],
    },),
    declared: [],
    declaredNames,
    l: levelCapturingLogger({ lines: [], },),
  },);
  return {
    outcomes: applied.groups.map(function recordedFor(group,): readonly string[] {
      /**
       What each place of the group was recorded as, in the order of the places.
       */
      const recorded: string[] = [];
      for (const place of group.occurrences)
        recorded.push(place.outcome,);
      return recorded;
    },),
    changed: applied.slices.map(function pairOf({ sliceIndex, text, },): readonly [number, string] {
      return [sliceIndex, text,];
    },),
    findings: applied.findings,
    keptBecause: applied.groups.map(function keptOf({ occurrences, },): string {
      // A place that records no reason is named `none`.
      return occurrences[1]?.keptBecause ?? 'none';
    },),
  };
}

/**
 What a read of a round came to, for a case to compare.

 @param settled - settled slices that came back
 @returns For each slice, its index, the names of the replacements it makes and of the gains left out with the reason of each, and its text
 @example
 ```ts
 const seen = seenSettled({ settled, },);
 ```
 */
function seenSettled(
  { settled, }: { readonly settled: readonly SettledSlice[]; },
): readonly unknown[] {
  return settled.map(function seenOf({ plan, made, gainReasons, },): unknown {
    return {
      slice: plan.facts.sliceIndex,
      toMake: plan.toMake.map(function nameOf({ id, },): string {
        return id;
      },),
      leftOut: plan.unprovenGains.map(function nameOf({ id, },): readonly [string, unknown] {
        return [id, gainReasons.get(id,),];
      },),
      made: made?.text,
    };
  },);
}

/**
 What applying replacements of one slice comes to, from the chosen words of a first slice, for a case to compare.

 @param second - English of the second slice
 @param edits - replacements of the second slice, each a group of its own in the order given, which is their priority
 @returns For each group the outcome and the reason of its place in the second slice, and the text of each slice that changed
 @example
 ```ts
 const { placed, changed, } = oneSliceApplied({ second: 'Fill the family bowl.', edits, },);
 ```
 */
function oneSliceApplied(
  {
    second,
    edits,
  }: {
    readonly second: string;
    readonly edits: readonly SecondSliceSplice[];
  },
): {
  readonly placed: readonly (readonly [string, string])[];
  readonly changed: readonly string[];
} {
  /**
   The chosen words of the first slice, which the second slice's gains copy.
   */
  const chosenSlice = 'Fill the parents’ dish and the pups’ bed.';
  const applied = applyConsistency({
    settled: edits.map(function groupOf({
      quote,
      chosen,
      start,
      end,
      text,
    },): SettledGroup {
      return withEdit({
        group: groupIn({
          places: [
            { slice: 0, english: chosenSlice, quote: chosen, },
            { slice: 1, english: second, quote, },
          ],
        },),
        edit: {
          kind: 'splice',
          placePositions: [1,],
          sliceIndex: 1,
          start,
          end,
          text,
        },
      },);
    },),
    slices: factsOf({
      englishes: [
        { sliceIndex: 0, english: chosenSlice, },
        { sliceIndex: 1, english: second, },
      ],
    },),
    declared: [],
    declaredNames: [],
    l: levelCapturingLogger({ lines: [], },),
  },);
  return {
    placed: applied.groups.map(function placeOf({ occurrences, },): readonly [string, string] {
      return [occurrences[1]?.outcome ?? 'none', occurrences[1]?.keptBecause ?? 'none',];
    },),
    changed: applied.slices.map(function textOf({ text, },): string {
      return text;
    },),
  };
}

/**
 What putting back a round of gains comes to where a leading gain blocks two trailing gains, the names of the gains put back and
 of those left out in the one slice that has them.

 @param neutrals - how many gains with no mark beside the ones that pair are dropped with them, each valid whatever else is made
 @param trailing - how many trailing gains the leading one pairs with, at most three
 @returns Names of the replacements the slice makes, and of the gains it leaves out with the reason of each
 @example
 ```ts
 const { made, leftOut, } = wideRoundPutBack({ neutrals: 9, },);
 ```
 */
function wideRoundPutBack(
  {
    neutrals,
    trailing = 2,
  }: {
    readonly neutrals: number;
    readonly trailing?: number;
  },
): { readonly made: readonly string[]; readonly leftOut: readonly (readonly [string, unknown])[]; } {
  /**
   The second slice: a leading gain's words, the words of up to three trailing gains after them, and one sentence for each neutral
   gain.
   */
  const second = `Fill the family bowl. Fill the pack bowl. Then the dogs nap. Then the cats nap. ${
    Array.from(
      { length: neutrals, },
      function sentenceAt(
        _slot,
        at,
      ): string {
        return `Cup ${String(at + 1,)} is here.`;
      },
    )
      .join(' ',)
  }`;
  /**
   The gains that pair: the leading one first in the slice, then the trailing ones.
   */
  const pairing = [
    { quote: 'the family', chosen: 'the parents’', text: '’the folks', },
    { quote: 'the pack', chosen: 'the parents’', text: 'the pups’', },
    { quote: 'the dogs', chosen: 'the pups’', text: 'the pups’', },
    { quote: 'the cats', chosen: 'the pups’', text: 'the kits’', },
  ].slice(
    0,
    1 + trailing,
  );
  /**
   The neutral gains.
   */
  const marked = Array.from(
    { length: neutrals, },
    function neutralAt(
      _slot,
      at,
    ): { readonly quote: string; readonly chosen: string; readonly text: string; } {
      return { quote: `Cup ${String(at + 1,)}`, chosen: 'the parents’', text: `Mug ${String(at + 1,)}`, };
    },
  );
  /**
   The replacements of the slice, with where their words are.
   */
  const edits = [...pairing, ...marked,].map(function locatedAt({
    quote,
    chosen,
    text,
  },): SecondSliceSplice {
    /**
     Where the words start in the slice.
     */
    const start = second.indexOf(quote,);
    return {
      quote,
      chosen,
      start,
      end: start + quote.length,
      text,
    };
  },);
  const { page, round, } = droppedRound({
    second,
    edits,
    dropped: edits.map(function nameOf(
      _edit,
      at,
    ): string {
      return `group ${String(at + 1,)} OCCURRENCE 2`;
    },),
  },);
  const settled = readmittedSlicesOf({
    page,
    round,
    refused: new Set(),
    asked: new Map(),
    floorsOn: floorsPass,
  },);
  const [only,] = settled;
  return {
    made: (only?.plan.toMake ?? []).map(function nameOf({ id, },): string {
      return id;
    },),
    leftOut: (only?.plan.unprovenGains ?? []).map(function nameOf({ id, },): readonly [string, unknown] {
      return [id, only?.gainReasons.get(id,),];
    },),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: applyConsistency.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES THE FIRST IN PAGE ORDER OF TWO GAINS THAT PAIR, and leaves the other out with the mark that could pair '
            + 'with it: a leading mark in the second slice and a trailing one in the third are each proven alone, neither '
            + 'together, and the check dropped both at once',
          fn: async () => {
            expect(overdropApplied({ declaredNames: [], leading: true, },),).toEqual({
              outcomes: [['chosen', 'replaced',], ['chosen', 'kept-current',],],
              changed: [[1, 'Then ’Bowl 1 of the folks is empty.',],],
              findings: [
                'slice 2: the consistency replacement of group 2 OCCURRENCE 2 was left out because the apostrophe it carries in '
                  + 'would read as a quotation mark there, since another quotation mark of the page could pair with it; the place '
                  + 'keeps its words',
              ],
              keptBecause: ['none', 'gain-pairs',],
            },);
          },
        },),
        it({
          name: 'MAKES A GAIN THAT WAS LEFT OUT ONLY FOR A GAIN WHOSE SLICE THE FLOORS WOULD REFUSE, and says of the one the '
            + 'floors refuse that the floors refuse its slice with it, since that holds whatever the other gains do and the '
            + 'mark that could pair with it is not the cause',
          fn: async () => {
            /**
             What the pass made of the page whose second slice loses a declared name to its gain.
             */
            const seen = overdropApplied({ declaredNames: ['kennel',], leading: true, },);
            expect([seen.outcomes, seen.changed, seen.findings.length, seen.keptBecause,],).toEqual([
              [['chosen', 'kept-current',], ['chosen', 'replaced',],],
              [[2, 'Then Bowl 2 of the parents’ is empty.',],],
              1,
              ['gain-floors', 'none',],
            ],);
            /**
             The finding of the floors' refusal.
             */
            const finding = seen.findings[0] ?? '';
            expect(finding,).toBe(
              'slice 1: the consistency replacement of group 1 OCCURRENCE 2 was left out because the floors refuse the slice '
                + 'with it: translate-refused-declared-name (slice 1: archive text carries "kennel" and the replacement does '
                + 'not; keeping the archive text); the place keeps its words',
            );
          },
        },),
        it({
          name: 'MAKES BOTH GAINS WHEN NEITHER IS A LEADING MARK, the control of the two cases where the gains pair',
          fn: async () => {
            expect(overdropApplied({ declaredNames: [], leading: false, },),).toEqual({
              outcomes: [['chosen', 'replaced',], ['chosen', 'replaced',],],
              changed: [
                [1, 'Then Bowl 1 of the folks is empty.',],
                [2, 'Then Bowl 2 of the parents’ is empty.',],
              ],
              findings: [],
              keptBecause: ['none', 'none',],
            },);
          },
        },),
        it({
          name: 'LEAVES NO DROPPED GAIN THAT COULD STAND ON THE FINAL PAGE, over random small pages: each gain left out is put '
            + 'on the final page by itself, and either its mark is not proven there, a gain made is not, or the floors refuse '
            + 'its slice; and every gain made is proven on the final page',
          fn: async () => {
            /**
             The pieces a slice is made of: some hold an opener, a closer or neither.
             */
            const fragments = [
              '',
              '',
              'He said ‘hi there. ',
              'The friends’ bowl. ',
              'She sang “la” loudly. ',
              'Rock ’n’ roll. ',
              'Then it napped. ',
              'It was ’tis fine. ',
            ];
            const violations: string[] = [];
            /**
             The generator of the page being made.
             */
            const current = { rand: randomOf({ seed: 0, },), };
            /**
             Picks one of the options.
             */
            function pick<Option,>(options: readonly Option[],): Option {
              return pickFrom({ options, rand: current.rand, },);
            }
            for (let seed = 1; seed <= 400; seed += 1) {
              current.rand = randomOf({ seed, },);
              const { rand, } = current;
              const count = 3 + Math.floor(rand() * 3,);
              const chosen = Array.from({ length: count - 1, }, function sentenceOf(_, at,): string {
                return `Bowl ${String(at + 1,)} of the parents’ is full.`;
              },);
              const slice0 = `${pick(fragments,)}${chosen.join(' ',)}${pick(fragments,)}`;
              const englishes = [{ sliceIndex: 0, english: slice0, },];
              const settled: SettledGroup[] = [];
              const gains: { readonly k: number; readonly text: string; readonly start: number; readonly end: number; readonly words: string; }[] = [];
              for (let k = 1; k < count; k += 1) {
                const noun = pick(['family', 'family', 'kennel',],);
                const words = `Bowl ${String(k,)} of the ${noun}`;
                const english = `${pick(fragments,)}Then ${words} is empty. ${pick(fragments,)}${pick(fragments,)}`;
                englishes.push({ sliceIndex: k, english, },);
                if (rand() < 0.15)
                  continue;
                const text = pick([
                  `Bowl ${String(k,)} of the parents’`,
                  `’Bowl ${String(k,)} of the folks`,
                  `Bowl ${String(k,)} of the dogs`,
                  `’Bowl ${String(k,)} of the parents’`,
                  `Bowl ${String(k,)} of the parents’`,
                ],);
                const start = english.indexOf(words,);
                settled.push(withEdit({
                  group: groupIn({
                    places: [
                      { slice: 0, english: slice0, quote: `Bowl ${String(k,)} of the parents’`, },
                      { slice: k, english, quote: words, },
                    ],
                  },),
                  edit: { kind: 'splice', placePositions: [1,], sliceIndex: k, start, end: start + words.length, text, },
                },),);
                gains.push({ k, text, start, end: start + words.length, words, },);
              }
              const applied = applyConsistency({
                settled,
                slices: factsOf({ englishes, },),
                declared: [],
                declaredNames: ['kennel',],
                l: levelCapturingLogger({ lines: [], },),
              },);
              const finalTexts = englishes.map(function finalOf({ sliceIndex, english, },): string {
                return applied.slices.find(function isSlice(slice,): boolean {
                  return slice.sliceIndex === sliceIndex;
                },)?.text ?? english;
              },);
              const page = pageOfSlices({
                slices: finalTexts.map(function sliceOf(english, sliceIndex,) {
                  return { sliceIndex, english, };
                },),
              },);
              const proven = provenApostrophesInEnglish({ english: page.text, },);
              gains.forEach(function checkGain(gain, at,): void {
                const outcome = applied.groups[at]?.occurrences[1]?.outcome;
                const text = finalTexts[gain.k] ?? '';
                const offset = page.offsets.get(gain.k,) ?? 0;
                const where = text.indexOf(gain.text,);
                if (outcome === 'replaced') {
                  if (gain.text.startsWith('’',) && (!proven.leading.has(offset + where,)))
                    violations.push(`seed ${String(seed,)} gain ${String(gain.k,)}: made and its leading mark is not proven`,);
                  if (gain.text.endsWith('’',) && (!proven.trailing.has(offset + where + (gain.text.length - 1),)))
                    violations.push(`seed ${String(seed,)} gain ${String(gain.k,)}: made and its trailing mark is not proven`,);
                  return;
                }
                if ((outcome !== 'kept-current') || (!(gain.text.startsWith('’',) || gain.text.endsWith('’',))))
                  return;
                // Could the gain stand? Put it on the final page by itself.
                const original = englishes[gain.k]?.english ?? '';
                const trial = original.slice(0, gain.start,) + gain.text + original.slice(gain.end,);
                const trialPage = pageOfSlices({
                  slices: finalTexts.map(function sliceOf(english, sliceIndex,) {
                    return { sliceIndex, english: (sliceIndex === gain.k) ? trial : english, };
                  },),
                },);
                const trialProven = provenApostrophesInEnglish({ english: trialPage.text, },);
                const trialOffset = trialPage.offsets.get(gain.k,) ?? 0;
                const ownProven = ((!gain.text.startsWith('’',)) || trialProven.leading.has(trialOffset + gain.start,))
                  && ((!gain.text.endsWith('’',)) || trialProven.trailing.has(trialOffset + gain.start + (gain.text.length - 1),));
                const othersProven = gains.every(function stillProven(other, otherAt,): boolean {
                  if ((other.k === gain.k) || (applied.groups[otherAt]?.occurrences[1]?.outcome !== 'replaced'))
                    return true;
                  const otherText = finalTexts[other.k] ?? '';
                  const otherWhere = otherText.indexOf(other.text,);
                  const otherOffset = trialPage.offsets.get(other.k,) ?? 0;
                  return ((!other.text.startsWith('’',)) || trialProven.leading.has(otherOffset + otherWhere,))
                    && ((!other.text.endsWith('’',)) || trialProven.trailing.has(otherOffset + otherWhere + (other.text.length - 1),));
                },);
                const floors = consistencyFloorVerdict({
                  sliceIndex: gain.k,
                  sourceText: '小猫打盹。',
                  currentText: original,
                  candidateText: trial,
                  lineStructured: false,
                  declared: [],
                  declaredNames: ['kennel',],
                  disputedWordings: [],
                },);
                if (ownProven && othersProven && (floors.kind === 'passes'))
                  violations.push(`seed ${String(seed,)} gain ${String(gain.k,)}: left out and could stand on the final page`,);
              },);
            }
            expect(violations,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: readmittedSlicesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS THE GAIN WOULD UNPROVE A GAIN THAT STANDS when its own mark is proven and a gain made is not: two splices '
            + 'that touch, the second taking the space after the first one\'s mark, are each proven alone, and the first one '
            + 'joins the next word once the second is made',
          fn: async () => {
            const second = 'Fill the family bowl.';
            const { page, round, } = droppedRound({
              second,
              edits: [
                { quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', },
                { quote: 'bowl', chosen: 'the pups’', start: 15, end: 20, text: 'bowls’', },
              ],
              dropped: ['group 1 OCCURRENCE 2', 'group 2 OCCURRENCE 2',],
            },);
            const settled = readmittedSlicesOf({
              page,
              round,
              refused: new Set(),
              asked: new Map(),
              floorsOn: floorsPass,
            },);
            expect(seenSettled({ settled, },),).toEqual([
              {
                slice: 1,
                toMake: ['group 1 OCCURRENCE 2',],
                leftOut: [['group 2 OCCURRENCE 2', { kind: 'unproves', },],],
                made: 'Fill the parents’ bowl.',
              },
            ],);
          },
        },),
        it({
          name: 'PUTS A GAIN BACK AFTER A FLOOR REFUSED IT ONCE, in a later pass: the floors refuse the slice with the first gain '
            + 'alone and pass it with the second, so the second is put back first and the first after it, and the floors are '
            + 'asked about each text once',
          fn: async () => {
            const second = 'Fill the family bowl. Then the dogs nap.';
            const dogs = second.indexOf('the dogs',);
            const { page, round, } = droppedRound({
              second,
              edits: [
                { quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', },
                { quote: 'the dogs', chosen: 'the pups’', start: dogs, end: dogs + 8, text: 'the pups’', },
              ],
              dropped: ['group 1 OCCURRENCE 2', 'group 2 OCCURRENCE 2',],
            },);
            /**
             The text of every slice the floors were asked about, in order.
             */
            const asked: string[] = [];
            const settled = readmittedSlicesOf({
              page,
              round,
              refused: new Set(),
              asked: new Map(),
              floorsOn: function scripted({ candidateText, },): ConsistencyFloorVerdict {
                asked.push(candidateText,);
                return (candidateText.includes('parents’',) && (!candidateText.includes('pups’',)))
                  ? { kind: 'refused', findings: ['scripted refusal',], }
                  : { kind: 'passes', };
              },
            },);
            expect(asked,).toEqual([
              'Fill the parents’ bowl. Then the dogs nap.',
              'Fill the family bowl. Then the pups’ nap.',
              'Fill the parents’ bowl. Then the pups’ nap.',
            ],);
            expect(seenSettled({ settled, },),).toEqual([
              {
                slice: 1,
                toMake: ['group 1 OCCURRENCE 2', 'group 2 OCCURRENCE 2',],
                leftOut: [],
                made: 'Fill the parents’ bowl. Then the pups’ nap.',
              },
            ],);
          },
        },),
        it({
          name: 'SAYS THE FLOORS REFUSE THE SLICE WITH A GAIN THEY REFUSE, with what they said, and leaves the gain out',
          fn: async () => {
            const { page, round, } = droppedRound({
              second: 'Fill the family bowl.',
              edits: [{ quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', },],
              dropped: ['group 1 OCCURRENCE 2',],
            },);
            const settled = readmittedSlicesOf({
              page,
              round,
              refused: new Set(),
              asked: new Map(),
              floorsOn: function refusing(): ConsistencyFloorVerdict {
                return { kind: 'refused', findings: ['the cat left the room', 'the dog came in',], };
              },
            },);
            expect(seenSettled({ settled, },),).toEqual([
              {
                slice: 1,
                toMake: [],
                leftOut: [
                  [
                    'group 1 OCCURRENCE 2',
                    { kind: 'floors', findings: ['the cat left the room', 'the dog came in',], },
                  ],
                ],
                made: undefined,
              },
            ],);
          },
        },),
        it({
          name: 'SAYS A MARK OF THE PAGE COULD PAIR WITH THE GAIN WHEN THE FLOORS PASS THE SLICE WITH IT, asking the floors once '
            + 'first, since their refusal is the reason that holds whatever the other gains do',
          fn: async () => {
            const { page, round, } = droppedRound({
              second: 'Fill the family bowl.',
              edits: [{ quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', },],
              dropped: ['group 1 OCCURRENCE 2',],
            },);
            /**
             The same round on a page whose first slice holds an opener after the chosen words.
             */
            const opened = droppedRound({
              second: 'Fill the family bowl.',
              edits: [{ quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', },],
              dropped: ['group 1 OCCURRENCE 2',],
            },);
            /**
             How many times the floors were asked.
             */
            const asked: number[] = [];
            const withOpener = readmittedSlicesOf({
              page: {
                ...opened.page,
                between: ['', '\n\nShe said ‘fill it.\n\n', '',],
              },
              round: opened.round,
              refused: new Set(),
              asked: new Map(),
              floorsOn: function counting(): ConsistencyFloorVerdict {
                asked.push(1,);
                return { kind: 'passes', };
              },
            },);
            const withoutOpener = readmittedSlicesOf({
              page,
              round,
              refused: new Set(),
              asked: new Map(),
              floorsOn: floorsPass,
            },);
            expect(seenSettled({ settled: withOpener, },),).toEqual([
              {
                slice: 1,
                toMake: [],
                leftOut: [['group 1 OCCURRENCE 2', { kind: 'pairs', },],],
                made: undefined,
              },
            ],);
            expect(asked,).toEqual([1,],);
            expect(seenSettled({ settled: withoutOpener, },),).toEqual([
              {
                slice: 1,
                toMake: ['group 1 OCCURRENCE 2',],
                leftOut: [],
                made: 'Fill the parents’ bowl.',
              },
            ],);
          },
        },),
        it({
          name: 'SAYS THE FLOORS REFUSE THE SLICE WITH A GAIN WHOSE MARK A MARK OF THE PAGE COULD PAIR WITH, since the refusal '
            + 'holds whatever the other gains do: the reason is the floors\', for a gain the check of the marks screens out as '
            + 'well as for one it tries, and the floors are asked once about the same replacements',
          fn: async () => {
            /**
             The same round, on a page whose first slice holds an opener after the chosen words.
             */
            const opened = droppedRound({
              second: 'Fill the family bowl.',
              edits: [{ quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', },],
              dropped: ['group 1 OCCURRENCE 2',],
            },);
            /**
             How many times the floors were asked.
             */
            const asked: number[] = [];
            const settled = readmittedSlicesOf({
              page: {
                ...opened.page,
                between: ['', '\n\nShe said ‘fill it.\n\n', '',],
              },
              round: opened.round,
              refused: new Set(),
              asked: new Map(),
              floorsOn: function refusing(): ConsistencyFloorVerdict {
                asked.push(1,);
                return { kind: 'refused', findings: ['the cat left the room',], };
              },
            },);
            expect(asked,).toEqual([1,],);
            expect(seenSettled({ settled, },),).toEqual([
              {
                slice: 1,
                toMake: [],
                leftOut: [['group 1 OCCURRENCE 2', { kind: 'floors', findings: ['the cat left the room',], },],],
                made: undefined,
              },
            ],);
          },
        },),
        it({
          name: 'LEAVES A GAIN OUT BECAUSE ITS WORDS OVERLAP A REPLACEMENT THAT STANDS, through the whole pass: the first group '
            + 'takes the words, its trailing mark would pair with an opener of the slice and is left out, and the second '
            + 'group\'s replacement, made once it is out, holds the words the first would take back',
          fn: async () => {
            const second = 'She said ‘fill the family bowl.';
            const bowl = second.indexOf('family bowl',);
            const family = second.indexOf('the family',);
            const { placed, changed, } = oneSliceApplied({
              second,
              edits: [
                { quote: 'family bowl', chosen: 'the parents’', start: bowl, end: bowl + 11, text: 'family bowls’', },
                { quote: 'the family', chosen: 'the pups’', start: family, end: family + 10, text: 'the folks', },
              ],
            },);
            expect(placed,).toEqual([['kept-current', 'gain-overlaps',], ['replaced', 'none',],],);
            expect(changed,).toEqual(['She said ‘fill the folks bowl.',],);
          },
        },),
        it({
          name: 'PUTS BACK THE LARGEST SET WHERE AT MOST TWELVE GAINS ARE LEFT OUT: a leading gain first in the slice would pair '
            + 'with the two trailing gains after it, so the two trailing gains and the nine gains with no mark are put back and '
            + 'the leading one is left out, not the leading one with the nine alone',
          fn: async () => {
            const { made, leftOut, } = wideRoundPutBack({ neutrals: 9, },);
            expect(made,).toEqual([
              'group 2 OCCURRENCE 2',
              'group 3 OCCURRENCE 2',
              ...Array.from({ length: 9, }, function nameAt(_slot, at,): string {
                return `group ${String(at + 4,)} OCCURRENCE 2`;
              },),
            ],);
            expect(leftOut.map(function nameOf([id,],): string {
              return id;
            },),).toEqual(['group 1 OCCURRENCE 2',],);
          },
        },),
        it({
          name: 'PUTS BACK THE LARGEST OF SEVERAL SETS LARGER THAN THE ORDER OF THE PAGE MAKES: a leading gain pairs with three '
            + 'trailing gains, any of which can stand without it, so the set of all three is put back and not the first two',
          fn: async () => {
            const { made, leftOut, } = wideRoundPutBack({ neutrals: 0, trailing: 3, },);
            expect(made,).toEqual(['group 2 OCCURRENCE 2', 'group 3 OCCURRENCE 2', 'group 4 OCCURRENCE 2',],);
            expect(leftOut.map(function nameOf([id,],): string {
              return id;
            },),).toEqual(['group 1 OCCURRENCE 2',],);
          },
        },),
        it({
          name: 'PUTS BACK IN THE ORDER OF THE PAGE WHERE MORE THAN TWELVE GAINS ARE LEFT OUT: the same slice with one gain with no '
            + 'mark more, thirteen left out, puts back the leading gain and the ten gains with no mark, and leaves the two trailing '
            + 'gains out',
          fn: async () => {
            const { made, leftOut, } = wideRoundPutBack({ neutrals: 10, },);
            expect(made,).toEqual([
              'group 1 OCCURRENCE 2',
              ...Array.from({ length: 10, }, function nameAt(_slot, at,): string {
                return `group ${String(at + 4,)} OCCURRENCE 2`;
              },),
            ],);
            expect(leftOut.map(function nameOf([id,],): string {
              return id;
            },),).toEqual(['group 2 OCCURRENCE 2', 'group 3 OCCURRENCE 2',],);
          },
        },),
        it({
          name: 'SAYS THE WORDS OVERLAP A REPLACEMENT THAT IS MADE when the gain would take the words of one that stands, or '
            + 'when one that stands would take the words of the gain, whichever group comes first',
          fn: async () => {
            const second = 'Fill the family bowl.';
            /**
             The replacement that stands, over the family bowl, and the gain over the family alone.
             */
            const standing = { quote: 'the family bowl', chosen: 'the dish', start: 5, end: 20, text: 'the dish', };
            const gain = { quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', };
            const standingFirst = droppedRound({
              second,
              edits: [standing, gain,],
              dropped: ['group 2 OCCURRENCE 2',],
            },);
            const gainFirst = droppedRound({
              second,
              edits: [gain, standing,],
              dropped: ['group 1 OCCURRENCE 2',],
            },);
            const seen = [standingFirst, gainFirst,].map(function readRound({ page, round, },): unknown {
              return seenSettled({
                settled: readmittedSlicesOf({
                  page,
                  round,
                  refused: new Set(),
                  asked: new Map(),
                  floorsOn: floorsPass,
                },),
              },);
            },);
            expect(seen,).toEqual([
              [
                {
                  slice: 1,
                  toMake: ['group 1 OCCURRENCE 2',],
                  leftOut: [['group 2 OCCURRENCE 2', { kind: 'overlaps', },],],
                  made: undefined,
                },
              ],
              [
                {
                  slice: 1,
                  toMake: ['group 2 OCCURRENCE 2',],
                  leftOut: [['group 1 OCCURRENCE 2', { kind: 'overlaps', },],],
                  made: undefined,
                },
              ],
            ],);
          },
        },),
        it({
          name: 'LEAVES THE GAINS OF A SLICE THE FLOORS REFUSED OUT, with the reason that the floors refused the slice: it keeps its '
            + 'wording whole, so a gain of it is not put back beside the replacements the floors refused',
          fn: async () => {
            const { page, round, } = droppedRound({
              second: 'Fill the family bowl.',
              edits: [{ quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', },],
              dropped: ['group 1 OCCURRENCE 2',],
            },);
            const settled = readmittedSlicesOf({
              page,
              round,
              refused: new Set([1,]),
              asked: new Map(),
              floorsOn: floorsPass,
            },);
            expect(seenSettled({ settled, },),).toEqual([
              {
                slice: 1,
                toMake: [],
                leftOut: [['group 1 OCCURRENCE 2', { kind: 'refused', },],],
                made: undefined,
              },
            ],);
          },
        },),
        it({
          name: 'HANDS BACK A ROUND THAT DROPPED NOTHING AS IT WAS, and asks the floors nothing',
          fn: async () => {
            const { page, round, } = droppedRound({
              second: 'Fill the family bowl.',
              edits: [{ quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', },],
              dropped: [],
            },);
            /**
             Whether the floors were asked.
             */
            const asked: number[] = [];
            const settled = readmittedSlicesOf({
              page,
              round,
              refused: new Set(),
              asked: new Map(),
              floorsOn: function counting(): ConsistencyFloorVerdict {
                asked.push(1,);
                return { kind: 'passes', };
              },
            },);
            expect(settled,).toEqual(round,);
            expect(asked,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Gain readmission tests
