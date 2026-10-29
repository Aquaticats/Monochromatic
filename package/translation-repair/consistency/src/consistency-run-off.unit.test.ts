/**
 Tests for the run-off among the finalists of a split first round: the sheet that
 shows only the finalists, and the winner the run-off's choices give.

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
  buildConsistencyRunOffMessages,
  type ConsistencyChoiceVoice,
  type ConsistencyOccurrenceView,
  type ConsistencyRendering,
  type ConsistencySubject,
  runOffWinnerOf,
} from '../dist/final/node/index.mjs';

/**
 Four places of a repeated sentence, each in a slice of its own.
 */
const PLACES: readonly ConsistencyOccurrenceView[] = [
  {
    sliceIndex: 0,
    paragraph: '窗台上的阳光让小猫睡着了。',
    markStart: 0,
    markEnd: 12,
    english: { kind: 'present', text: 'The sunny window put the kitten to sleep.', },
  },
  {
    sliceIndex: 1,
    paragraph: '窗台上的阳光让小猫睡着了。',
    markStart: 0,
    markEnd: 12,
    english: { kind: 'present', text: 'The sunlight on the sill lulled the kitten to sleep.', },
  },
  {
    sliceIndex: 2,
    paragraph: '窗台上的阳光让小猫睡着了。',
    markStart: 0,
    markEnd: 12,
    english: { kind: 'present', text: 'Mittens is asleep in the window.', },
  },
  {
    sliceIndex: 3,
    paragraph: '窗台上的阳光让小猫睡着了。',
    markStart: 0,
    markEnd: 12,
    english: { kind: 'present', text: 'The warm sill sent the kitten to sleep.', },
  },
];

/**
 The sheet's subject.
 */
const SUBJECT: ConsistencySubject = {
  repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
  occurrences: PLACES,
};

/**
 Renderings the first round located for the four places.
 */
const RENDERINGS: readonly ConsistencyRendering[] = [
  { kind: 'located', quote: 'The sunny window put the kitten to sleep', core: 'The sunny window put the kitten to sleep', start: 0, },
  { kind: 'unlocated', reason: 'no-agreement', },
  { kind: 'located', quote: 'Mittens is asleep in the window', core: 'Mittens is asleep in the window', start: 0, },
  { kind: 'located', quote: 'The warm sill sent the kitten to sleep', core: 'The warm sill sent the kitten to sleep', start: 0, },
];

/**
 Labels of the four places.
 */
const LABELS = ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 4',] as const;

/**
 Builds one heard choice.

 @param seat - seat that answered

 @param choice - label chosen

 @returns Choice with its seat

 @example
 ```ts
 const voice = voiceOf({ seat: 'a', choice: 'OCCURRENCE 3', },);
 ```
 */
function voiceOf({ seat, choice, }: { readonly seat: string; readonly choice: string; },): ConsistencyChoiceVoice {
  return {
    seat,
    reason: 'It keeps the memorial past tense.',
    choice,
  };
}

/**
 Every message of an exchange, joined.

 @param finalists - labels of the places that tied

 @param subject - what the sheet shows

 @returns Both messages' text, system first

 @example
 ```ts
 const text = sheetText({ finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',], },);
 ```
 */
function sheetText(
  {
    finalists,
    subject = SUBJECT,
  }: {
    readonly finalists: readonly string[];
    readonly subject?: ConsistencySubject;
  },
): string {
  return buildConsistencyRunOffMessages({
    subject,
    finalists,
    renderings: RENDERINGS,
  },)
    .map(function contentOf(message,): string {
      return message.content;
    },)
    .join('\n',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildConsistencyRunOffMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS ONLY THE TIED FINALISTS, each with its original, its English and the words that render the repeated '
            + 'wording, and no place that did not tie',
          fn: async () => {
            const text = sheetText({ finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',], },);
            expect({
              three: text.includes('OCCURRENCE 3 ORIGINAL',) && text.includes('OCCURRENCE 3 TRANSLATION',),
              four: text.includes('OCCURRENCE 4 ORIGINAL',) && text.includes('OCCURRENCE 4 TRANSLATION',),
              threeWords: text.includes('OCCURRENCE 3 WORDS THAT RENDER THE REPEATED WORDING',)
                && text.includes('\nMittens is asleep in the window\n',),
              fourWords: text.includes('\nThe warm sill sent the kitten to sleep\n',),
              one: text.includes('OCCURRENCE 1',),
              two: text.includes('OCCURRENCE 2',),
              oneEnglish: text.includes('The sunny window put the kitten to sleep',),
            },).toEqual({
              three: true,
              four: true,
              threeWords: true,
              fourWords: true,
              one: false,
              two: false,
              oneEnglish: false,
            },);
          },
        },),
        it({
          name: 'SAYS SO for a finalist whose rendering the first round did not locate',
          fn: async () => {
            const text = sheetText({ finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',], },);
            expect(text.includes(
              'OCCURRENCE 2 WORDS THAT RENDER THE REPEATED WORDING',
            ) && text.includes('(not located in the TRANSLATION)',),).toBe(true,);
          },
        },),
        it({
          name: 'ASKS FOR THE REASON FIRST, naming any word of the chosen rendering that renders source outside the marked '
            + 'wording, and for one choice among the tied places',
          fn: async () => {
            const [system, user,] = buildConsistencyRunOffMessages({
              subject: SUBJECT,
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              renderings: RENDERINGS,
            },);
            expect({
              roles: [system?.role, user?.role,],
              reasonFirst: system?.content.includes('keys come in this order: assessment, choice',),
              outside: system?.content.includes('name any word of the rendering you choose that renders ORIGINAL outside the marked wording',),
              houseRules: system?.content.includes('House rules this corpus is written under',),
              shape: user?.content.endsWith(
                'Reply with JSON only: {"assessment":"...","choice":"<the label of the place you choose>"}',
              ),
              quotesAsked: (system?.content ?? '').includes('"candidateQuotes"',),
            },).toEqual({
              roles: ['system', 'user',],
              reasonFirst: true,
              outside: true,
              houseRules: true,
              shape: true,
              quotesAsked: false,
            },);
          },
        },),
        it({
          name: 'LISTS THE TENSE CANDIDATES OF THE FINALISTS ONLY, labelled as the first round labelled them, when the entry '
            + 'records a death',
          fn: async () => {
            const text = sheetText({
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              subject: { ...SUBJECT, lifeEnded: true, subjectNames: ['Mittens',], },
            },);
            expect({
              block: text.includes('TENSE CANDIDATES',),
              line: text.includes('OCCURRENCE 3 TRANSLATION:\n- "Mittens is asleep in the window." (matched "Mittens is", rule copula-or-auxiliary)',),
            },).toEqual({ block: true, line: true, },);
            expect(sheetText({ finalists: ['OCCURRENCE 1', 'OCCURRENCE 4',], subject: { ...SUBJECT, lifeEnded: true, subjectNames: ['Mittens',], }, },)
              .includes('TENSE CANDIDATES',),).toBe(false,);
          },
        },),
        it({
          name: 'SHOWS A FRAME\'S SLOT IN THE ORIGINAL for a finalist, and points a second place of one slice at the first',
          fn: async () => {
            const text = sheetText({
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              subject: {
                repeat: { kind: 'refrain', prefix: '所以这只猫一定是', suffix: '的小宝贝', },
                occurrences: [
                  PLACES[0] as ConsistencyOccurrenceView,
                  PLACES[1] as ConsistencyOccurrenceView,
                  { ...PLACES[2] as ConsistencyOccurrenceView, slotSource: '胖胖', },
                  { ...PLACES[2] as ConsistencyOccurrenceView, markStart: 6, markEnd: 12, slotSource: '乖仔', },
                ],
              },
            },);
            expect({
              slotThree: text.includes('OCCURRENCE 3 SLOT IN THE ORIGINAL',) && text.includes('胖胖',),
              slotFour: text.includes('OCCURRENCE 4 SLOT IN THE ORIGINAL',) && text.includes('乖仔',),
              sameSlice: text.includes('the same slice as OCCURRENCE 3',),
            },).toEqual({ slotThree: true, slotFour: true, sameSlice: true, },);
          },
        },),
        it({
          name: 'SAYS SO when a finalist\'s slice carries no English, and shows the declared names and the cited pages when the '
            + 'page has them',
          fn: async () => {
            const text = sheetText({
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              subject: {
                ...SUBJECT,
                occurrences: [
                  PLACES[0] as ConsistencyOccurrenceView,
                  PLACES[1] as ConsistencyOccurrenceView,
                  { ...PLACES[2] as ConsistencyOccurrenceView, english: { kind: 'absent', }, },
                  PLACES[3] as ConsistencyOccurrenceView,
                ],
                identityContext: '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"',
                referenceContext: '- reference 1 https://example.org/mittens ("Mittens"): Mittens had a sister.',
              },
            },);
            expect({
              none: text.includes('(this slice carries no English)',),
              names: text.includes('TRANSLATION declares "Mittens"',) && text.includes('Declared identity, when a DECLARED NAMES block',),
              references: text.includes('Mittens had a sister.',),
            },).toEqual({ none: true, names: true, references: true, },);
          },
        },),
      ],
    },),
    describe({
      name: runOffWinnerOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE FINALIST A STRICT MAJORITY OF THE USABLE CHOICES NAMED when the run-off reaches its quorum',
          fn: async () => {
            const decided = runOffWinnerOf({
              voices: [
                voiceOf({ seat: 'a', choice: 'OCCURRENCE 4', },),
                voiceOf({ seat: 'b', choice: 'OCCURRENCE 4', },),
                voiceOf({ seat: 'c', choice: 'OCCURRENCE 3', },),
              ],
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              quorum: 2,
              labels: LABELS,
              renderings: RENDERINGS,
            },);
            expect(decided,).toEqual({ usable: 3, winner: { kind: 'chosen', label: 'OCCURRENCE 4', }, },);
          },
        },),
        it({
          name: 'LEAVES THE PAGE AS IT STANDS ON A SECOND TIE, recorded as a run-off tie',
          fn: async () => {
            const decided = runOffWinnerOf({
              voices: [
                voiceOf({ seat: 'a', choice: 'OCCURRENCE 4', },),
                voiceOf({ seat: 'b', choice: 'OCCURRENCE 3', },),
              ],
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              quorum: 2,
              labels: LABELS,
              renderings: RENDERINGS,
            },);
            expect(decided,).toEqual({ usable: 2, winner: { kind: 'none', reason: 'run-off-tied', }, },);
          },
        },),
        it({
          name: 'LEAVES THE PAGE AS IT STANDS BELOW THE QUORUM, recorded as a run-off without quorum, and when nobody was heard',
          fn: async () => {
            const few = runOffWinnerOf({
              voices: [voiceOf({ seat: 'a', choice: 'OCCURRENCE 4', },),],
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              quorum: 2,
              labels: LABELS,
              renderings: RENDERINGS,
            },);
            const nobody = runOffWinnerOf({
              voices: [],
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              quorum: 2,
              labels: LABELS,
              renderings: RENDERINGS,
            },);
            expect([few, nobody,],).toEqual([
              { usable: 1, winner: { kind: 'none', reason: 'run-off-no-quorum', }, },
              { usable: 0, winner: { kind: 'none', reason: 'run-off-no-quorum', }, },
            ],);
          },
        },),
        it({
          name: 'LEAVES THE PAGE AS IT STANDS ON ONE USABLE CHOICE though the run-off closed on a quorum of one, since one '
            + 'judge alone never decides (the lead\'s ruling of 2026-10-25); two usable choices decide, the positive control',
          fn: async () => {
            const one = runOffWinnerOf({
              voices: [voiceOf({ seat: 'a', choice: 'OCCURRENCE 4', },),],
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              quorum: 1,
              labels: LABELS,
              renderings: RENDERINGS,
            },);
            const two = runOffWinnerOf({
              voices: [voiceOf({ seat: 'a', choice: 'OCCURRENCE 4', },), voiceOf({ seat: 'b', choice: 'OCCURRENCE 4', },),],
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              quorum: 1,
              labels: LABELS,
              renderings: RENDERINGS,
            },);
            expect([one, two,],).toEqual([
              { usable: 1, winner: { kind: 'none', reason: 'run-off-no-quorum', }, },
              { usable: 2, winner: { kind: 'chosen', label: 'OCCURRENCE 4', }, },
            ],);
          },
        },),
        it({
          name: 'DOES NOT COUNT A CHOICE OF A PLACE THAT DID NOT TIE or a blank reason, and keeps the first round\'s reason '
            + 'when the winner\'s own rendering was not located',
          fn: async () => {
            const outsider = runOffWinnerOf({
              voices: [
                voiceOf({ seat: 'a', choice: 'OCCURRENCE 1', },),
                voiceOf({ seat: 'b', choice: 'OCCURRENCE 1', },),
                { seat: 'c', reason: ' ', choice: 'OCCURRENCE 4', },
                voiceOf({ seat: 'd', choice: 'OCCURRENCE 3', },),
                voiceOf({ seat: 'e', choice: 'OCCURRENCE 3', },),
              ],
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              quorum: 1,
              labels: LABELS,
              renderings: RENDERINGS,
            },);
            const unlocated = runOffWinnerOf({
              voices: [voiceOf({ seat: 'a', choice: 'OCCURRENCE 2', },), voiceOf({ seat: 'b', choice: 'OCCURRENCE 2', },),],
              finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',],
              quorum: 2,
              labels: LABELS,
              renderings: RENDERINGS,
            },);
            expect([outsider, unlocated,],).toEqual([
              { usable: 2, winner: { kind: 'chosen', label: 'OCCURRENCE 3', }, },
              { usable: 2, winner: { kind: 'none', reason: 'chosen-unlocated', }, },
            ],);
          },
        },),
      ],
    },),
  ],
},);
