/**
 Tests for reading the consistency section of a settled artifact: which
 generations carry it, every key exact, and every group recounted from its own
 ballots so a file that contradicts them is refused.

 EVERY CASE BREAKS ONE VALID SECTION IN EXACTLY ONE WAY. The valid section is
 the one the pass writes over the cat page (a repeated sentence and a refrain of
 four lines), so each field parses on its own in each case and what fails is a
 relation: a count, a winner, an outcome, a slice row.

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

import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  type ArtifactConsistency,
  NO_CONSISTENCY,
  parseConsistency,
  parseSettledTwoLaneArtifact,
} from '@monochromatic-dev/translation-repair-artifact';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  archiveEnglishOf,
  ballotText,
  firstPlaceWins,
  otherSheetsSplittingRunOff,
  preparedCatPage,
  refrainAnsweredInOrder,
  refrainChoosing,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import {
  REFRAIN_QUOTES,
  SENTENCE_QUOTES,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-quotes.test-fixture.ts';
import { isRefrainSheet, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';
import {
  fourthAlreadyFollows,
  outcomesByGroup,
  sentenceQuoteSpansTheRefrain,
  slotWithTheName,
  withBlankedBlocks,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-edge.test-fixture.ts';
import { LIVE_CONSISTENCY_FIFTEEN, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-live-fifteen.test-fixture.ts';
import { runCatPass, } from './consistency-run.test-fixture.ts';
import {
  appendAt,
  dropLastAt,
  editedBy,
  type JsonEdit,
  removeAt,
  setAt,
  valueAt,
} from '@monochromatic-dev/translation-repair-document-structure/ts/corpus-run/json-edit.test-fixture.ts';
import { settledArtifactOver, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-artifact.test-fixture.ts';

/**
 Slices the cat page's comparison names.
 */
const SLICES = new Set([0, 1, 2, 3,],);

/**
 Plain JSON of the section the pass writes over the cat page.

 @param behave - what each seat does when asked, the first place winning every group by default

 @returns Section as a file holds it

 @example
 ```ts
 const section = await sectionOf({},);
 ```
 */
async function sectionOf(
  {
    behave = firstPlaceWins,
    others,
  }: {
    readonly behave?: SeatScript;
    readonly others?: SeatScript;
  },
): Promise<unknown> {
  await using dir = await scratchDir({ prefix: 'consistency-read-', },);
  const run = await runCatPass({
    behave,
    ...((others === undefined) ? {} : { others, }),
    entryCacheDir: dir.path,
  },);
  return structuredClone(run.consistency,);
}

/**
 Plain JSON of the section the pass writes over a page a case states.

 @param behave - what each seat does when asked

 @param prepared - preparation of the page, the cat page by default

 @param english - what each slice ships, the archive by default

 @returns Section as a file holds it

 @example
 ```ts
 const section = await sectionOfRun({ behave: firstPlaceWins, },);
 ```
 */
async function sectionOfRun(
  {
    behave,
    prepared,
    english,
  }: {
    readonly behave: SeatScript;
    readonly prepared?: PreparedDocumentPair;
    readonly english?: readonly string[];
  },
): Promise<ArtifactConsistency> {
  await using dir = await scratchDir({ prefix: 'consistency-read-', },);
  const run = await runCatPass({
    behave,
    entryCacheDir: dir.path,
    ...((prepared === undefined) ? {} : { prepared, }),
    ...((english === undefined) ? {} : { english, }),
  },);
  return structuredClone(run.consistency,);
}

/**
 Reads a value as the section of a generation that writes it.

 @param value - what the file holds

 @returns Section

 @example
 ```ts
 const section = readWritten({ value, },);
 ```
 */
function readWritten({ value, }: { readonly value: unknown; },): ReturnType<typeof parseConsistency> {
  return parseConsistency({
    value,
    path: 'cat.consistency',
    generation: 17,
    sliceIndexes: SLICES,
  },);
}

/**
 What reading a value refuses with, as the path and reason it names.

 @param value - what the file holds

 @returns Refusal's message, or a note that nothing was refused

 @example
 ```ts
 const refusal = refusalOf({ value, },);
 ```
 */
function refusalOf({ value, }: { readonly value: unknown; },): string {
  try {
    readWritten({ value, },);
  }
  catch (error) {
    if (error instanceof ArtifactParseError)
      return error.message;
    throw error;
  }
  return 'not refused';
}

/**
 Refusal for a section that has been altered.

 @param section - valid section to alter

 @param edits - alterations, in order

 @returns Refusal's message

 @example
 ```ts
 const refusal = refusalAfter({ section, edits: [setAt({ path: ['findings',], value: 3, },),], },);
 ```
 */
function refusalAfter(
  {
    section,
    edits,
  }: {
    readonly section: unknown;
    readonly edits: readonly JsonEdit[];
  },
): string {
  return refusalOf({
    value: editedBy({
      root: section,
      edits,
    },),
  },);
}

/**
 Path to one place of one group.

 @param group - position of the group on the page

 @param place - position of the place in the group

 @param field - what to reach inside the place

 @returns Path

 @example
 ```ts
 const path = placePath({ group: 1, place: 1, field: ['outcome',], },);
 ```
 */
function placePath(
  {
    group,
    place,
    field,
  }: {
    readonly group: number;
    readonly place: number;
    readonly field: readonly string[];
  },
): readonly (string | number)[] {
  return [
    'groups',
    group,
    'occurrences',
    place,
    ...field,
  ];
}

/**
 Five equal refrain ballots, as the seats send them.

 @param quotes - quotes each ballot gives

 @param choice - place each ballot chooses

 @param keep - places each ballot keeps

 @returns Refrain behaviour of five seats

 @example
 ```ts
 const behave = fiveAlike({ quotes: REFRAIN_QUOTES, choice: 'OCCURRENCE 1', keep: [], },);
 ```
 */
function fiveAlike(
  {
    quotes,
    choice,
    keep,
  }: {
    readonly quotes: readonly Record<string, string>[];
    readonly choice: string;
    readonly keep: readonly string[];
  },
): SeatScript {
  return refrainAnsweredInOrder({
    answers: Array.from(
      { length: 5, },
      function same(): { readonly kind: 'answers'; readonly text: string; } {
        return {
          kind: 'answers',
          text: ballotText({
            quotes,
            choice,
            keep,
          },),
        };
      },
    ),
  },);
}

/**
 The refrain's quotes with the quote of the second place replaced.

 @param over - fields of the second place's quote this case replaces

 @returns Four quotes

 @example
 ```ts
 const quotes = refrainQuotesWithSecond({ over: { slot: 'bored', }, },);
 ```
 */
function refrainQuotesWithSecond(
  { over, }: { readonly over: Readonly<Record<string, string>>; },
): readonly Record<string, string>[] {
  return REFRAIN_QUOTES.map(function replaced(
    quote,
    at,
  ): Record<string, string> {
    return (at === 1) ? { ...quote, ...over, } : quote;
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseConsistency.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS AN ABSENT SECTION ON AN OLDER GENERATION AS A PASS THAT CHANGED NOTHING',
          fn: async () => {
            expect(parseConsistency({
              value: undefined,
              path: 'cat.consistency',
              generation: 14,
              sliceIndexes: SLICES,
            },),).toEqual(NO_CONSISTENCY,);
          },
        },),
        it({
          name: 'READS THE LIVE SECTION OF GENERATION FIFTEEN AS WRITTEN, the straight-against-curly quotes counted as the '
            + 'strings they were, and refuses it as generation sixteen, which gives no slot reason',
          fn: async () => {
            const slices = new Set(Array.from({ length: 16, }, function indexOf(_unused, at,): number {
              return at;
            },),);
            expect(parseConsistency({
              value: structuredClone(LIVE_CONSISTENCY_FIFTEEN,),
              path: 'live.consistency',
              generation: 15,
              sliceIndexes: slices,
            },),).toEqual(LIVE_CONSISTENCY_FIFTEEN,);
            expect(
              String(caught(function readAsSixteen(): unknown {
                return parseConsistency({
                  value: structuredClone(LIVE_CONSISTENCY_FIFTEEN,),
                  path: 'live.consistency',
                  generation: 16,
                  sliceIndexes: slices,
                },);
              },),),
            ).toBe(
              'ArtifactParseError: artifact parse failed at live.consistency.groups[0].occurrences[0].rendering.reason: '
                + 'expected a reason generation sixteen gives, which locates a place on its quote and not its slot.',
            );
          },
        },),
        it({
          name: 'REFUSES AN ABSENT SECTION ON A GENERATION THAT WRITES IT, and a present one on a generation below it',
          fn: async () => {
            expect(refusalOf({ value: undefined, },),).toContain('recorded consistency from generation fifteen',);
            expect(caught(function readBelow(): unknown {
              return parseConsistency({
                value: NO_CONSISTENCY,
                path: 'cat.consistency',
                generation: 14,
                sliceIndexes: SLICES,
              },);
            },) instanceof ArtifactParseError,).toBe(true,);
          },
        },),
        it({
          name: 'READS THE SECTION THE PASS WRITES BACK AS WRITTEN, groups, ballots, changed slices and findings',
          fn: async () => {
            const section = await sectionOf({},);
            expect(readWritten({ value: section, },),).toEqual(section,);
            expect(valueAt({ root: section, path: ['slices',], },),).toEqual(readWritten({ value: section, },).slices,);
            expect(readWritten({ value: section, },).slices.length,).toBe(3,);
          },
        },),
        it({
          name: 'READS AN EMPTY SECTION',
          fn: async () => {
            expect(readWritten({ value: { groups: [], slices: [], findings: [], }, },),).toEqual(NO_CONSISTENCY,);
          },
        },),
        it({
          name: 'REFUSES A KEY THE SECTION, A GROUP, A PLACE, A VERDICT, A BALLOT OR A CHANGED SLICE DOES NOT HAVE',
          fn: async () => {
            const section = await sectionOf({},);
            const messages = [
              ['extra',],
              ['groups', 0, 'extra',],
              ['groups', 0, 'occurrences', 0, 'extra',],
              ['groups', 0, 'verdict', 'extra',],
              ['groups', 0, 'verdict', 'ballots', 0, 'extra',],
              ['slices', 0, 'extra',],
              ['groups', 0, 'repeat', 'extra',],
            ].map(function refusedWith(path,): string {
              return refusalAfter({ section, edits: [setAt({ path, value: 1, },),], },);
            },);
            expect(messages.every(function refused(message,): boolean {
              return message !== 'not refused';
            },),).toBe(true,);
            expect(messages[0],).toContain('cat.consistency',);
            expect(messages[4],).toContain('cat.consistency.groups[0].verdict.ballots[0]',);
          },
        },),
        it({
          name: 'REFUSES A USABLE COUNT THAT IS NOT THE BALLOTS THAT CAN BE COUNTED',
          fn: async () => {
            expect(refusalAfter({
              section: await sectionOf({},),
              edits: [setAt({ path: ['groups', 0, 'verdict', 'usable',], value: 3, },),],
            },),).toContain('cat.consistency.groups[0].verdict.usable',);
          },
        },),
        it({
          name: 'REFUSES A WINNER THE BALLOTS DO NOT GIVE, whether another place, none, or a reason that is not the one',
          fn: async () => {
            const section = await sectionOf({},);
            const winnerPath = ['groups', 0, 'verdict', 'winner',];
            expect([
              { kind: 'chosen', label: 'OCCURRENCE 2', },
              { kind: 'none', reason: 'no-majority', },
            ].map(function refusedWith(winner,): boolean {
              return refusalAfter({ section, edits: [setAt({ path: winnerPath, value: winner, },),], },)
                .includes('cat.consistency.groups[0].verdict.winner',);
            },),).toEqual([true, true,],);
          },
        },),
        it({
          name: 'REFUSES A QUOTE OR A SLOT A STRICT MAJORITY OF THE BALLOTS DID NOT GIVE',
          fn: async () => {
            const section = await sectionOf({},);
            expect(refusalAfter({
              section,
              edits: [setAt({
                path: placePath({ group: 0, place: 1, field: ['rendering', 'quote',], },),
                value: 'the lamp lulled the cat',
              },),],
            },),).toContain('cat.consistency.groups[0].occurrences[1].rendering',);
            expect(refusalAfter({
              section,
              edits: [setAt({
                path: placePath({ group: 1, place: 1, field: ['rendering', 'slot',], },),
                value: 'bored',
              },),],
            },),).toContain('cat.consistency.groups[1].occurrences[1].rendering',);
          },
        },),
        it({
          name: 'REFUSES AN UNLOCATED REASON THE BALLOTS CONTRADICT, since they agreed on a quote there',
          fn: async () => {
            expect(refusalAfter({
              section: await sectionOf({},),
              edits: [setAt({
                path: placePath({ group: 0, place: 1, field: ['rendering',], },),
                value: { kind: 'unlocated', reason: 'no-agreement', },
              },),],
            },),).toContain('cat.consistency.groups[0].occurrences[1].rendering.reason',);
          },
        },),
        it({
          name: 'REFUSES AN OUTCOME THE BALLOTS AND RENDERINGS DO NOT GIVE: the winner not chosen, a replaced place '
            + 'recorded consistent, a place kept without a majority, an untouched place beside a winner',
          fn: async () => {
            const section = await sectionOf({},);
            expect([
              { place: 0, outcome: 'replaced', },
              { place: 1, outcome: 'consistent', },
              { place: 1, outcome: 'kept-context', },
              { place: 1, outcome: 'untouched', },
            ].map(function refusedWith({ place, outcome, },): boolean {
              return refusalAfter({
                section,
                edits: [setAt({ path: placePath({ group: 0, place, field: ['outcome',], },), value: outcome, },),],
              },).includes('.outcome',);
            },),).toEqual([true, true, true, true,]);
          },
        },),
        it({
          name: 'REFUSES AN OUTCOME OF AN UNLOCATED PLACE THAT IS NOT UNLOCATED, and a ballot-less group beside a winner',
          fn: async () => {
            const section = await sectionOf({},);
            expect(refusalAfter({
              section,
              edits: [setAt({
                path: placePath({ group: 1, place: 1, field: ['rendering',], },),
                value: { kind: 'unlocated', reason: 'not-found', },
              },),],
            },),).not.toBe('not refused',);
            expect(refusalAfter({
              section,
              edits: [
                setAt({ path: ['groups', 0, 'verdict', 'ballots',], value: [], },),
                setAt({ path: ['groups', 0, 'verdict', 'usable',], value: 0, },),
                setAt({ path: ['groups', 0, 'verdict', 'winner',], value: { kind: 'none', reason: 'no-quorum', }, },),
              ],
            },),).not.toBe('not refused',);
          },
        },),
        it({
          name: 'REFUSES A PLACE LABELLED OUT OF ORDER, ON A SLICE THE COMPARISON DOES NOT NAME, OR CARRYING A SLOT WITHOUT A '
            + 'FRAME',
          fn: async () => {
            const section = await sectionOf({},);
            const messages = [
              [setAt({ path: placePath({ group: 0, place: 0, field: ['label',], },), value: 'OCCURRENCE 2', },),],
              [setAt({ path: placePath({ group: 0, place: 0, field: ['sliceIndex',], },), value: 9, },),],
              [
                setAt({ path: placePath({ group: 0, place: 0, field: ['slotStartOffset',], },), value: 1, },),
                setAt({ path: placePath({ group: 0, place: 0, field: ['slotEndOffset',], },), value: 2, },),
              ],
              [
                removeAt({ path: placePath({ group: 1, place: 0, field: ['slotStartOffset',], },), },),
                removeAt({ path: placePath({ group: 1, place: 0, field: ['slotEndOffset',], },), },),
              ],
              [removeAt({ path: placePath({ group: 1, place: 0, field: ['rendering', 'slot',], },), },),],
            ].map(function refusedWith(edits,): string {
              return refusalAfter({ section, edits, },);
            },);
            expect(messages.every(function refused(message,): boolean {
              return message !== 'not refused';
            },),).toBe(true,);
            expect(messages[0],).toContain('cat.consistency.groups[0].occurrences[0].label',);
            expect(messages[1],).toContain('cat.consistency.groups[0].occurrences[0].sliceIndex',);
          },
        },),
        it({
          name: 'REFUSES A CHANGED SLICE WITH NO REPLACED PLACE, A REPLACED PLACE WITH NO CHANGED SLICE, AND A SLICE LISTED TWICE',
          fn: async () => {
            const section = await sectionOf({},);
            const firstRow = valueAt({ root: section, path: ['slices', 0,], },);
            const messages = [
              refusalAfter({ section, edits: [appendAt({ path: ['slices',], value: { sliceIndex: 0, text: 'x', }, },),], },),
              refusalAfter({ section, edits: [dropLastAt({ path: ['slices',], },),], },),
              refusalAfter({ section, edits: [appendAt({ path: ['slices',], value: firstRow, },),], },),
              refusalAfter({ section, edits: [setAt({ path: ['slices', 0, 'sliceIndex',], value: 17, },),], },),
            ];
            expect([
              messages[0]?.includes('a replaced place in slice 0, which a changed slice needs',),
              messages[1]?.includes('a changed slice row for slice',),
              messages[2]?.includes('one row per slice; slice 1 appears more than once',),
              messages[3]?.includes('cat.consistency.slices[0].sliceIndex',),
            ],).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES A GROUP THE BENCH WAS NOT ASKED ABOUT THAT SAYS MORE THAN SO: an outcome other than untouched, '
            + 'or fewer than two places missing from a skip for too few',
          fn: async () => {
            const place = {
              label: 'OCCURRENCE 1',
              sliceIndex: 0,
              startOffset: 6,
              endOffset: 18,
              line: 3,
              rendering: { kind: 'unlocated', reason: 'not-asked', },
              outcome: 'untouched',
            };
            const notAsked = {
              repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
              occurrences: [
                place,
                { ...place, label: 'OCCURRENCE 2', sliceIndex: 2, startOffset: 85, endOffset: 97, line: 11, },
              ],
              unplaced: [],
              carries: [],
              verdict: { kind: 'not-asked', reason: 'no-english', },
            };
            /**
             Refusal for a section of this one group.
             */
            function refusalOfGroup({ group, }: { readonly group: unknown; },): string {
              return refusalOf({ value: { groups: [group,], slices: [], findings: [], }, },);
            }
            expect([
              refusalOfGroup({ group: notAsked, },),
              refusalOfGroup({ group: { ...notAsked, verdict: { kind: 'not-asked', reason: 'too-few-placed', }, }, },),
              refusalOfGroup({ group: { ...notAsked, verdict: { kind: 'not-asked', reason: 'consistent', }, }, },),
              refusalOfGroup({
                group: {
                  ...notAsked,
                  occurrences: notAsked.occurrences.map(function replaced(each,) {
                    return { ...each, outcome: 'replaced', };
                  },),
                },
              },),
            ].map(function refused(message,): boolean {
              return message !== 'not refused';
            },),).toEqual([false, true, true, true,],);
          },
        },),
        it({
          name: 'READS A GROUP WHOSE HANDLING FAILED AND A GROUP WITH A PLACE IN NO SLICE, as written',
          fn: async () => {
            const place = {
              label: 'OCCURRENCE 1',
              sliceIndex: 0,
              startOffset: 6,
              endOffset: 18,
              line: 3,
              rendering: { kind: 'unlocated', reason: 'not-asked', },
              outcome: 'untouched',
            };
            const group = {
              repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
              occurrences: [
                place,
                { ...place, label: 'OCCURRENCE 2', sliceIndex: 2, startOffset: 85, endOffset: 97, line: 11, },
              ],
              unplaced: [{ startOffset: 120, endOffset: 132, line: 14, reason: 'crosses-slice-end', },],
              carries: [],
              verdict: {
                kind: 'failed',
                detail: 'unreachable: slice 2 of the preparation has no comparison row, and the comparison is derived '
                  + 'from rows of that preparation',
              },
            };
            expect(readWritten({ value: { groups: [group,], slices: [], findings: ['one finding',], }, },),)
              .toEqual({ groups: [group,], slices: [], findings: ['one finding',], },);
            expect(refusalOf({
              value: {
                groups: [{ ...group, unplaced: [{ startOffset: 120, endOffset: 132, line: 14, reason: 'lost', },], },],
                slices: [],
                findings: [],
              },
            },),).toContain('cat.consistency.groups[0].unplaced[0].reason',);
          },
        },),
        it({
          name: 'REFUSES A REASON, A KIND OR AN OUTCOME OUTSIDE THE VOCABULARY, each naming the path',
          fn: async () => {
            const section = await sectionOf({},);
            const messages = [
              setAt({ path: ['groups', 0, 'repeat', 'kind',], value: 'echo', },),
              setAt({ path: placePath({ group: 0, place: 1, field: ['outcome',], },), value: 'liked', },),
              setAt({
                path: placePath({ group: 0, place: 1, field: ['rendering',], },),
                value: { kind: 'unlocated', reason: 'shy', },
              },),
              setAt({ path: ['groups', 0, 'verdict', 'kind',], value: 'asleep', },),
              setAt({ path: ['groups', 0, 'verdict', 'winner',], value: { kind: 'none', reason: 'shy', }, },),
              setAt({ path: ['groups', 0, 'verdict', 'ballots', 0, 'choice',], value: 4, },),
            ].map(function refusedWith(edit,): string {
              return refusalAfter({ section, edits: [edit,], },);
            },);
            expect(messages.every(function refused(message,): boolean {
              return message !== 'not refused';
            },),).toBe(true,);
            expect(messages[1],).toContain('cat.consistency.groups[0].occurrences[1].outcome',);
          },
        },),
        it({
          name: 'READS A GROUP NO PLACE WON AS WRITTEN, and refuses an outcome other than untouched beside it',
          fn: async () => {
            const tie = await sectionOf({
              behave: refrainAnsweredInOrder({
                answers: ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 2',]
                  .map(function answerFor(choice,): SeatBehaviour {
                    return refrainChoosing({ choice, },);
                  },),
              },),
              others: otherSheetsSplittingRunOff,
            },);
            expect(readWritten({ value: tie, },),).toEqual(tie,);
            expect(refusalAfter({
              section: tie,
              edits: [setAt({ path: placePath({ group: 1, place: 2, field: ['outcome',], },), value: 'replaced', },),],
            },),).toContain('cat.consistency.groups[1].occurrences[2].outcome',);
          },
        },),
        it({
          name: 'READS A PLACE THE QUOTES COULD NOT LOCATE AS WRITTEN, and refuses an outcome other than unlocated for it',
          fn: async () => {
            const missing = await sectionOf({
              behave: fiveAlike({
                quotes: refrainQuotesWithSecond({ over: { quote: 'So this cat is surely a restless little treasure.', }, },),
                choice: 'OCCURRENCE 1',
                keep: [],
              },),
            },);
            expect(readWritten({ value: missing, },),).toEqual(missing,);
            expect(valueAt({ root: missing, path: placePath({ group: 1, place: 1, field: ['rendering',], },), },),)
              .toEqual({ kind: 'unlocated', reason: 'not-found', },);
            expect(refusalAfter({
              section: missing,
              edits: [setAt({ path: placePath({ group: 1, place: 1, field: ['outcome',], },), value: 'replaced', },),],
            },),).toContain('cat.consistency.groups[1].occurrences[1].outcome',);
          },
        },),
        it({
          name: 'READS A PLACE A MAJORITY KEEPS FOR ITS SURROUNDINGS AS WRITTEN, and refuses an outcome other than kept-context',
          fn: async () => {
            const kept = await sectionOf({
              behave: fiveAlike({ quotes: REFRAIN_QUOTES, choice: 'OCCURRENCE 1', keep: ['OCCURRENCE 2',], },),
            },);
            expect(readWritten({ value: kept, },),).toEqual(kept,);
            expect(valueAt({ root: kept, path: placePath({ group: 1, place: 1, field: ['outcome',], },), },),)
              .toBe('kept-context',);
            expect(refusalAfter({
              section: kept,
              edits: [setAt({ path: placePath({ group: 1, place: 1, field: ['outcome',], },), value: 'replaced', },),],
            },),).toContain('cat.consistency.groups[1].occurrences[1].outcome',);
          },
        },),
        it({
          name: 'READS THE REASON A QUOTE WAS NOT LOCATED FOR AS WRITTEN, and refuses it where the ballots agree on what the '
            + 'reason denies',
          fn: async () => {
            const reasonPath = placePath({ group: 1, place: 1, field: ['rendering', 'reason',], },);
            const section = await sectionOf({
              behave: fiveAlike({
                quotes: refrainQuotesWithSecond({ over: { quote: '...', }, },),
                choice: 'OCCURRENCE 1',
                keep: [],
              },),
            },);
            expect(valueAt({ root: section, path: reasonPath, },),).toBe('empty',);
            expect(refusalOf({ value: section, },),).toBe('not refused',);
            expect(refusalAfter({ section, edits: [setAt({ path: reasonPath, value: 'no-agreement', },),], },),)
              .toContain('cat.consistency.groups[1].occurrences[1].rendering.reason',);
          },
        },),
        it({
          name: 'LOCATES A PLACE WITHOUT A SLOT WHERE THE BALLOTS GAVE AN EMPTY SLOT OR NO SLOT A MAJORITY AGREED, reads that '
            + 'as written, and refuses a slot reason of generation fifteen, which generation sixteen never writes',
          fn: async () => {
            const reasonPath = placePath({ group: 1, place: 1, field: ['rendering', 'reason',], },);
            const emptySlot = await sectionOf({
              behave: fiveAlike({
                quotes: refrainQuotesWithSecond({ over: { slot: '', }, },),
                choice: 'OCCURRENCE 1',
                keep: [],
              },),
            },);
            const disagreeing = await sectionOf({
              behave: refrainAnsweredInOrder({
                answers: ['lazy', 'lazy', 'idle', 'idle', 'cosy',]
                  .map(function answerWith(slot,): { readonly kind: 'answers'; readonly text: string; } {
                    return {
                      kind: 'answers',
                      text: ballotText({
                        quotes: refrainQuotesWithSecond({ over: { slot, }, },),
                        choice: 'OCCURRENCE 1',
                        keep: [],
                      },),
                    };
                  },),
              },),
            },);
            expect([emptySlot, disagreeing,].map(function recorded(section,): unknown {
              return valueAt({ root: section, path: placePath({ group: 1, place: 1, field: ['rendering',], },), },);
            },),).toEqual([
              { kind: 'located', quote: 'So this cat is surely a lazy little treasure', start: 58, },
              { kind: 'located', quote: 'So this cat is surely a lazy little treasure', start: 58, },
            ],);
            expect([emptySlot, disagreeing,].map(function accepted(section,): string {
              return refusalOf({ value: section, },);
            },),).toEqual(['not refused', 'not refused',],);
            expect(['slot-empty', 'slot-no-agreement', 'slot-not-found', 'slot-ambiguous', 'slot-whole',]
              .map(function tamperedWith(reason,): string {
                return refusalAfter({
                  section: emptySlot,
                  edits: [
                    setAt({
                      path: placePath({ group: 1, place: 1, field: ['rendering',], },),
                      value: { kind: 'unlocated', reason, },
                    },),
                    setAt({ path: placePath({ group: 1, place: 1, field: ['outcome',], },), value: 'unlocated', },),
                  ],
                },);
              },)
              .every(function namesTheReason(message,): boolean {
                return message.includes('cat.consistency.groups[1].occurrences[1].rendering.reason',)
                  && message.includes('a reason generation sixteen gives, which locates a place on its quote and not its slot',);
              },),).toBe(true,);
            expect(reasonPath.length,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'REFUSES A LOCATED SLOT THAT DOES NOT STAND EXACTLY ONCE IN THE WORDS OF ITS PLACE, though the ballots agree on it',
          fn: async () => {
            const section = await sectionOf({
              behave: fiveAlike({ quotes: REFRAIN_QUOTES, choice: 'OCCURRENCE 1', keep: [], },),
            },);
            /**
             Edits that make every ballot and the recorded rendering name one slot.
             */
            function slotEdits({ slot, }: { readonly slot: string; },): readonly JsonEdit[] {
              return [
                setAt({ path: placePath({ group: 1, place: 1, field: ['rendering', 'slot',], },), value: slot, },),
                ...[0, 1, 2, 3, 4,].map(function ballotSlot(ballot,): JsonEdit {
                  return setAt({
                    path: ['groups', 1, 'verdict', 'ballots', ballot, 'quotes', 1, 'slot',],
                    value: slot,
                  },);
                },),
              ];
            }
            expect(['bored', 'e',].map(function refusedFor(slot,): boolean {
              return refusalAfter({ section, edits: slotEdits({ slot, },), },)
                .includes('the slot a strict majority of the usable ballots gave inside those words, and none where they gave none',);
            },),).toEqual([true, true,],);
          },
        },),
        it({
          name: 'REFUSES A PLACE THAT ENDS WHERE IT STARTS OR ON A LINE BEFORE THE FIRST',
          fn: async () => {
            const section = await sectionOf({},);
            const messages = [
              setAt({ path: placePath({ group: 0, place: 0, field: ['endOffset',], },), value: 6, },),
              setAt({ path: placePath({ group: 0, place: 0, field: ['line',], },), value: 0, },),
            ].map(function refusedWith(edit,): string {
              return refusalAfter({ section, edits: [edit,], },);
            },);
            expect(messages.every(function refused(message,): boolean {
              return message.includes('cat.consistency.groups[0].occurrences[0]',)
                && message.includes('a place that ends after it starts',);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'READS BACK EACH OUTCOME AND EACH REASON THE PASS WRITES, a splice a floor withheld, an overlap, a place that '
            + 'already follows, a place with no English, a quote standing twice, a group whose handling failed',
          fn: async () => {
            const prepared = preparedCatPage();
            const archive = archiveEnglishOf({ prepared, },);
            const sections: Record<string, ArtifactConsistency> = {
              withheldSplice: await sectionOfRun({
                behave: slotWithTheName,
                prepared: { ...prepared, declaredNames: ['Mittens',], },
                english: [
                  archive[0] as string,
                  '## Two\n\nThe smell of fish in the kitchen woke the kitten. So this cat is surely Mittens, a lazy little treasure.',
                  archive[2] as string,
                  archive[3] as string,
                ],
              },),
              overlapped: await sectionOfRun({
                behave: sentenceQuoteSpansTheRefrain,
              },),
              following: await sectionOfRun({
                behave: fourthAlreadyFollows,
                english: [
                  archive[0] as string,
                  archive[1] as string,
                  archive[2] as string,
                  '## Four\n\nThe rain in the yard sent the kitten running home. So this cat must be a warm little treasure.',
                ],
              },),
              alike: await sectionOfRun({
                behave: firstPlaceWins,
                english: [archive[0] as string, archive[1] as string, archive[0] as string, archive[3] as string,],
              },),
              noEnglish: await sectionOfRun({
                behave: firstPlaceWins,
                english: [archive[0] as string, '', archive[2] as string, archive[3] as string,],
              },),
              twice: await sectionOfRun({
                behave: function quotesTheSecond({ sheet, },): SeatBehaviour {
                  return isRefrainSheet({ sheet, },)
                    ? firstPlaceWins({ sheet, },)
                    : {
                      kind: 'answers',
                      text: ballotText({
                        quotes: [
                          SENTENCE_QUOTES[0] as Record<string, string>,
                          { label: 'OCCURRENCE 2', quote: 'the kitten to sleep', },
                        ],
                        choice: 'OCCURRENCE 1',
                        keep: [],
                      },),
                    };
                },
                english: [
                  archive[0] as string,
                  archive[1] as string,
                  '## Three\n\nIn the evening, the sunlight lulled the kitten to sleep, and the kitten to sleep again. '
                    + 'So this cat has to be a sweet little darling.',
                  archive[3] as string,
                ],
              },),
              failed: await sectionOfRun({
                behave: firstPlaceWins,
                prepared: withBlankedBlocks({ prepared, },),
              },),
            };
            const outcome = Object.fromEntries(Object.entries(sections,).map(function read(
              [name, section,],
            ): readonly [string, string] {
              return [name, refusalOf({ value: section, },),];
            },),);
            expect(outcome,).toEqual({
              withheldSplice: 'not refused',
              overlapped: 'not refused',
              following: 'not refused',
              alike: 'not refused',
              noEnglish: 'not refused',
              twice: 'not refused',
              failed: 'not refused',
            },);
            expect(Object.entries(sections,).map(function outcomesIn([name, section,],): readonly [string, readonly string[]] {
              return [name, outcomesByGroup({ consistency: section, },).flat(),];
            },),).toEqual([
              ['withheldSplice', ['chosen', 'replaced', 'chosen', 'kept-current', 'replaced', 'replaced',],],
              ['overlapped', ['chosen', 'replaced', 'chosen', 'replaced', 'overlapped', 'replaced',],],
              ['following', ['chosen', 'replaced', 'chosen', 'replaced', 'replaced', 'consistent',],],
              ['alike', ['consistent', 'consistent', 'chosen', 'replaced', 'unlocated', 'replaced',],],
              ['noEnglish', ['chosen', 'replaced', 'chosen', 'unlocated', 'replaced', 'replaced',],],
              ['twice', ['chosen', 'unlocated', 'chosen', 'replaced', 'replaced', 'replaced',],],
              ['failed', ['untouched', 'untouched', 'untouched', 'untouched', 'untouched', 'untouched',],],
            ],);
            expect(Object.entries(sections,).map(function gaveWayIn([name, section,],): readonly [string, readonly unknown[]] {
              return [
                name,
                section.groups.flatMap(function gaveWayOf({ occurrences, },): readonly unknown[] {
                  return occurrences.flatMap(function named({ gaveWayTo, },): readonly unknown[] {
                    return (gaveWayTo === undefined) ? [] : [gaveWayTo,];
                  },);
                },),
              ];
            },),).toEqual([
              ['withheldSplice', [],],
              ['overlapped', [{ group: 0, label: 'OCCURRENCE 2', },],],
              ['following', [],],
              ['alike', [],],
              ['noEnglish', [],],
              ['twice', [],],
              ['failed', [],],
            ],);
          },
        },),
        it({
          name: 'READS A SECTION OF GENERATION SIXTEEN, WHICH NAMES NO PLACE AN OVERLAPPED PLACE GAVE WAY TO, as it was written, '
            + 'and refuses it as generation seventeen, and refuses the name generation seventeen writes as generation sixteen',
          fn: async () => {
            const section = await sectionOfRun({ behave: sentenceQuoteSpansTheRefrain, },);
            /**
             The section as a file of generation sixteen held it: the overlapped place of the refrain with no name.
             */
            const older = editedBy({
              root: section,
              edits: [removeAt({ path: placePath({ group: 1, place: 2, field: ['gaveWayTo',], }), },),],
            },);
            expect(parseConsistency({
              value: structuredClone(older,),
              path: 'cat.consistency',
              generation: 16,
              sliceIndexes: SLICES,
            },),).toEqual(older,);
            expect(refusalOf({ value: older, },),).toBe(
              'artifact parse failed at cat.consistency.groups[1].occurrences[2].gaveWayTo: expected the place an overlapped '
                + 'place gave way to, which generation seventeen records.',
            );
            /**
             What reading the section generation seventeen writes as generation sixteen refuses with.
             */
            const refusedAsSixteen = caught(function readsSixteen(): unknown {
              return parseConsistency({
                value: structuredClone(section,),
                path: 'cat.consistency',
                generation: 16,
                sliceIndexes: SLICES,
              },);
            },);
            expect(String(refusedAsSixteen,),).toBe(
              'ArtifactParseError: artifact parse failed at cat.consistency.groups[1].occurrences[2].gaveWayTo: expected no key '
                + 'here beyond label, sliceIndex, startOffset, endOffset, line, slotStartOffset, slotEndOffset, rendering, outcome.',
            );
          },
        },),
        it({
          name: 'REFUSES SLOT OFFSETS A REPEATED RUN\'S PLACE CARRIES ALONE, A SLOT THAT LIES OUTSIDE ITS PLACE, AND A SLOT '
            + 'THAT ENDS BEFORE IT STARTS',
          fn: async () => {
            const section = await sectionOf({},);
            const messages = [
              [setAt({ path: placePath({ group: 0, place: 0, field: ['slotStartOffset',], },), value: 7, },),],
              [setAt({ path: placePath({ group: 0, place: 0, field: ['slotEndOffset',], },), value: 9, },),],
              [
                setAt({ path: placePath({ group: 1, place: 0, field: ['slotStartOffset',], },), value: 12, },),
                setAt({ path: placePath({ group: 1, place: 0, field: ['slotEndOffset',], },), value: 9, },),
              ],
              [
                setAt({ path: placePath({ group: 1, place: 0, field: ['slotStartOffset',], },), value: 2, },),
                setAt({ path: placePath({ group: 1, place: 0, field: ['slotEndOffset',], },), value: 4, },),
              ],
            ].map(function refusedWith(edits,): string {
              return refusalAfter({ section, edits, },);
            },);
            expect(messages.map(function refused(message,): boolean {
              return message !== 'not refused';
            },),).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES A GROUP OF FEWER THAN TWO PLACES THAT IS SAID TO HAVE BEEN CONSIDERED, since only a skip for too '
            + 'few places can have one: no English, already consistent, or failed',
          fn: async () => {
            const place = {
              label: 'OCCURRENCE 1',
              sliceIndex: 0,
              startOffset: 6,
              endOffset: 18,
              line: 3,
              rendering: { kind: 'unlocated', reason: 'not-asked', },
              outcome: 'untouched',
            };
            const base = {
              repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
              occurrences: [place,],
              unplaced: [],
              carries: [],
            };
            expect([
              { ...base, verdict: { kind: 'not-asked', reason: 'no-english', }, },
              { ...base, verdict: { kind: 'not-asked', reason: 'consistent', },
                occurrences: [{ ...place, outcome: 'consistent', },], },
              { ...base, verdict: { kind: 'failed', detail: 'unreachable: a group lists no place', }, },
              { ...base, verdict: { kind: 'not-asked', reason: 'too-few-placed', }, },
            ].map(function refusalOfOne(group,): string {
              return refusalOf({ value: { groups: [group,], slices: [], findings: [], }, },);
            },).map(function refused(message,): boolean {
              return message !== 'not refused';
            },),).toEqual([true, true, true, false,],);
          },
        },),
        it({
          name: 'REFUSES A GROUP WHOSE PLACES SHARE A SLICE THAT IS SAID TO READ ALIKE, since the pass asks the bench when two '
            + 'places stand in one slice',
          fn: async () => {
            const place = {
              label: 'OCCURRENCE 1',
              sliceIndex: 0,
              startOffset: 6,
              endOffset: 18,
              line: 3,
              rendering: { kind: 'unlocated', reason: 'not-asked', },
              outcome: 'consistent',
            };
            expect(refusalOf({
              value: {
                groups: [{
                  repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
                  occurrences: [place, { ...place, label: 'OCCURRENCE 2', startOffset: 30, endOffset: 42, line: 5, },],
                  unplaced: [],
                  carries: [],
                  verdict: { kind: 'not-asked', reason: 'consistent', },
                },],
                slices: [],
                findings: [],
              },
            },),).toContain('cat.consistency.groups[0].occurrences[1].sliceIndex',);
          },
        },),
        it({
          name: 'REFUSES A SEAT THAT IS HEARD TWICE in one group\'s ballots, since a gather hears each seat once',
          fn: async () => {
            const section = await sectionOf({},);
            const first = valueAt({ root: section, path: ['groups', 0, 'verdict', 'ballots', 0,], },);
            const usable = valueAt({ root: section, path: ['groups', 0, 'verdict', 'usable',], },) as number;
            expect(refusalAfter({
              section,
              edits: [
                appendAt({ path: ['groups', 0, 'verdict', 'ballots',], value: first, },),
                setAt({ path: ['groups', 0, 'verdict', 'usable',], value: usable + 1, },),
              ],
            },),).toContain('cat.consistency.groups[0].verdict.ballots[',);
          },
        },),
        it({
          name: 'REFUSES A REASON A PLACE WAS NOT LOCATED FOR THAT NO BALLOT OR GROUP COULD HAVE GIVEN: a missing or '
            + 'ambiguous quote where no quote won, a slot reason beside no quote or in a repeated run',
          fn: async () => {
            const split = await sectionOf({
              behave: fiveAlike({ quotes: REFRAIN_QUOTES, choice: 'OCCURRENCE 1', keep: [], },),
            },);
            const noQuote = await sectionOf({
              behave: refrainAnsweredInOrder({
                answers: ['a', 'b', 'c', 'd', 'e',].map(function answerWith(word,): SeatBehaviour {
                  return {
                    kind: 'answers',
                    text: ballotText({
                      quotes: refrainQuotesWithSecond({ over: { quote: `So this cat is surely a ${word} little treasure.`, }, },),
                      choice: 'OCCURRENCE 1',
                      keep: [],
                    },),
                  };
                },),
              },),
            },);
            const rendering = (reason: string,): JsonEdit[] => [
              setAt({ path: placePath({ group: 1, place: 1, field: ['rendering',], },), value: { kind: 'unlocated', reason, }, },),
              setAt({ path: placePath({ group: 1, place: 1, field: ['outcome',], },), value: 'unlocated', },),
            ];
            const sentence = (reason: string,): JsonEdit[] => [
              setAt({ path: placePath({ group: 0, place: 1, field: ['rendering',], },), value: { kind: 'unlocated', reason, }, },),
              setAt({ path: placePath({ group: 0, place: 1, field: ['outcome',], },), value: 'unlocated', },),
            ];
            expect(valueAt({ root: noQuote, path: placePath({ group: 1, place: 1, field: ['rendering', 'reason',], },), },),)
              .toBe('no-agreement',);
            expect([
              refusalAfter({ section: noQuote, edits: rendering('not-found',), },),
              refusalAfter({ section: noQuote, edits: rendering('ambiguous',), },),
              refusalAfter({ section: noQuote, edits: rendering('slot-not-found',), },),
              refusalAfter({ section: noQuote, edits: rendering('slot-ambiguous',), },),
              refusalAfter({ section: noQuote, edits: rendering('slot-empty',), },),
              refusalAfter({ section: split, edits: sentence('slot-no-agreement',), },),
              refusalAfter({ section: split, edits: sentence('slot-not-found',), },),
            ].map(function refused(message,): boolean {
              return message !== 'not refused';
            },),).toEqual([true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES A LOCATED RENDERING THAT SHOWS A READER NOTHING, since the pass reads such a quote as empty',
          fn: async () => {
            const dots = await sectionOf({
              behave: fiveAlike({
                quotes: refrainQuotesWithSecond({ over: { quote: '...', }, },),
                choice: 'OCCURRENCE 1',
                keep: [],
              },),
            },);
            expect(refusalAfter({
              section: dots,
              edits: [
                setAt({
                  path: placePath({ group: 1, place: 1, field: ['rendering',], },),
                  value: { kind: 'located', quote: '...', slot: 'lazy', },
                },),
                setAt({ path: placePath({ group: 1, place: 1, field: ['outcome',], },), value: 'replaced', },),
              ],
            },),).not.toBe('not refused',);
          },
        },),
        it({
          name: 'REFUSES A SLICE THAT HOLDS BOTH A REPLACED PLACE AND A PLACE THE FLOORS REFUSED, since the floors answer for '
            + 'a slice once, and an overlap that no kept replacement in its slice explains',
          fn: async () => {
            const section = await sectionOf({},);
            const mixed = refusalAfter({
              section,
              edits: [setAt({ path: placePath({ group: 1, place: 2, field: ['outcome',], },), value: 'kept-floor', },),],
            },);
            const slices = valueAt({ root: section, path: ['slices',], },) as readonly { readonly sliceIndex: number; }[];
            const lone = refusalAfter({
              section,
              edits: [
                setAt({ path: placePath({ group: 1, place: 1, field: ['outcome',], },), value: 'overlapped', },),
                setAt({
                  path: ['slices',],
                  value: slices.filter(function notFirst(row,): boolean {
                    return row.sliceIndex !== 1;
                  },),
                },),
              ],
            },);
            expect([mixed !== 'not refused', lone !== 'not refused',],).toEqual([true, true,],);
          },
        },),
        it({
          name: 'READS THE SECTION INSIDE A WHOLE SETTLED ARTIFACT AND REFUSES A BROKEN ONE THERE',
          fn: async () => {
            const section = await sectionOf({},);
            const artifact = settledArtifactOver({
              prepared: preparedCatPage(),
              entryId: 'CatEntry1',
              laneSelection: { kind: 'pending-human-decision', },
              consolidation: { kind: 'not-run', },
            },);
            const whole = structuredClone({ ...artifact, consistency: section, },);
            expect(parseSettledTwoLaneArtifact({ value: whole, },).consistency,).toEqual(section,);
            const broken = editedBy({
              root: whole,
              edits: [setAt({ path: ['consistency', 'groups', 0, 'verdict', 'usable',], value: 3, },),],
            },);
            expect(caught(function readBroken(): unknown {
              return parseSettledTwoLaneArtifact({ value: broken, },);
            },) instanceof ArtifactParseError,).toBe(true,);
            const absent = editedBy({
              root: whole,
              edits: [removeAt({ path: ['consistency',], },),],
            },);
            expect(caught(function readAbsent(): unknown {
              return parseSettledTwoLaneArtifact({ value: absent, },);
            },) instanceof ArtifactParseError,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
