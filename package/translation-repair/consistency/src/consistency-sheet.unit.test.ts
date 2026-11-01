/**
 Tests for the sheet a judge reads about one repeated source wording: the rules,
 the places with the wording marked, the English of each place's slice, the
 declared names and the cited pages, and the reply shape with the reason first.

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
  buildConsistencyMessages,
  consistencyLabels,
  type ConsistencyOccurrenceView,
  consistencyResponseFormat,
  type ConsistencySubject,
} from '../dist/final/node/index.mjs';
import { fenceOpening, LONG_FENCE_RUN, } from '@monochromatic-dev/translation-repair-test-fixture/ts/sheet-fence.test-fixture.ts';

/**
 Original paragraph of the first place, holding the repeated sentence once.
 */
const FIRST_PARAGRAPH = '窗台上的阳光让小猫睡着了。所以这只猫一定是胖胖的小宝贝。';

/**
 English of the first place's slice.
 */
const FIRST_ENGLISH = 'The sunny window put the kitten to sleep. So this cat must be a chubby little treasure.';

/**
 English of the second place's slice.
 */
const SECOND_ENGLISH = 'In the evening, the sunlight on the sill lulled the kitten to sleep.';

/**
 Two places of a repeated sentence, in two slices.
 */
const TWO_PLACES: readonly ConsistencyOccurrenceView[] = [
  {
    sliceIndex: 0,
    paragraph: FIRST_PARAGRAPH,
    markStart: 0,
    markEnd: 12,
    english: { kind: 'present', text: FIRST_ENGLISH, },
  },
  {
    sliceIndex: 2,
    paragraph: '傍晚的时候，窗台上的阳光让小猫睡着了，小鱼干也笑了。',
    markStart: 6,
    markEnd: 18,
    english: { kind: 'present', text: SECOND_ENGLISH, },
  },
];

/**
 The sheet's subject for a repeated sentence.
 */
const SENTENCE: ConsistencySubject = {
  repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
  occurrences: TWO_PLACES,
};

/**
 The sheet's subject when the entry records a death and the first place's
 English narrates its subject in the present tense.
 */
const TENSE_SUBJECT: ConsistencySubject = {
  ...SENTENCE,
  occurrences: [
    {
      ...(TWO_PLACES[0] as ConsistencyOccurrenceView),
      english: { kind: 'present', text: 'Mittens is asleep in the sunny window.', },
    },
    TWO_PLACES[1] as ConsistencyOccurrenceView,
  ],
  lifeEnded: true,
  subjectNames: ['Mittens',],
};

/**
 Every message of an exchange, joined.

 @param subject - what the sheet shows

 @returns Both messages' text, system first

 @example
 ```ts
 const text = sheetText({ subject: SENTENCE, },);
 ```
 */
function sheetText({ subject, }: { readonly subject: ConsistencySubject; },): string {
  return buildConsistencyMessages({ subject, },)
    .map(function contentOf(message,): string {
      return message.content;
    },)
    .join('\n',);
}

/**
 The reply example the sheet ends on: it names a placeholder for each label and no real label, since the rules forbid
 choosing by position and a real label in the example would be one to copy.
 */
const REPLY_SHAPE = 'Reply with JSON only: {"assessment":"...","candidateQuotes":[{"label":"<the label of the OCCURRENCE>","quote":"..."}],'
  + '"choice":"<the label of the OCCURRENCE you choose>","keep":[]}';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildConsistencyMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS WITH A SYSTEM MESSAGE OF RULES AND THE HOUSE RULES, then a user message of the places',
          fn: async () => {
            const [system, user,] = buildConsistencyMessages({ subject: SENTENCE, },);
            expect({
              roles: [system?.role, user?.role,],
              houseRules: system?.content.includes('House rules this corpus is written under',),
              task: system?.content.includes('the places render it alike',),
              places: (user?.content.includes('OCCURRENCE 1 ORIGINAL',) === true)
                && (user?.content.includes('OCCURRENCE 2 ORIGINAL',)),
            },).toEqual({
              roles: ['system', 'user',],
              houseRules: true,
              task: true,
              places: true,
            },);
          },
        },),
        it({
          name: 'TELLS THE JUDGE TO WRITE THE REASON FIRST, in the rules, in the reply shape and in the order of '
            + 'the keys the schema declares (the owner\'s ruling of 2026-10-11)',
          fn: async () => {
            const [system, user,] = buildConsistencyMessages({ subject: SENTENCE, },);
            const userText = user?.content ?? '';
            const text = sheetText({ subject: SENTENCE, },);
            expect({
              rule: text.includes('Write the assessment first and choose after it',),
              keysInRules: text.includes('keys come in this order: assessment, candidateQuotes, choice, keep',),
              shape: userText.indexOf('{"assessment"',) < userText.indexOf('"choice"',),
              shapeShown: user?.content.endsWith(REPLY_SHAPE,),
              systemFirst: system?.role,
              schema: Object.keys(
                consistencyResponseFormat({ labels: ['OCCURRENCE 1', 'OCCURRENCE 2',], },).json_schema.schema
                  .properties as Record<string, unknown>,
              ),
              required: consistencyResponseFormat({ labels: ['OCCURRENCE 1', 'OCCURRENCE 2',], },).json_schema.schema.required,
            },).toEqual({
              rule: true,
              keysInRules: true,
              shape: true,
              shapeShown: true,
              systemFirst: 'system',
              schema: ['assessment', 'candidateQuotes', 'choice', 'keep',],
              required: ['assessment', 'candidateQuotes', 'choice', 'keep',],
            },);
          },
        },),
        it({
          name: 'ASKS THE JUDGES TO QUOTE ONLY THE WORDS THAT RENDER THE REPEATED WORDING, and to name in the reason any word '
            + 'of the chosen rendering that renders source outside the repeat',
          fn: async () => {
            const text = sheetText({ subject: SENTENCE, },);
            expect({
              reason: text.includes(
                'name any word of the rendering you choose that renders ORIGINAL outside the marked wording (the other places must not be given such a word)',
              ),
              quote: text.includes('that render the marked wording and no other words',),
              leaveOut: text.includes(
                'Leave out a word before or after them that renders ORIGINAL outside the marked wording, such as an "also" that renders a word standing before the marked wording.',
              ),
            },).toEqual({
              reason: true,
              quote: true,
              leaveOut: true,
            },);
          },
        },),
        it({
          name: 'MARKS THE REPEATED WORDING in each paragraph and shows the whole English of each slice',
          fn: async () => {
            const text = sheetText({ subject: SENTENCE, },);
            expect({
              firstMarked: text.includes('⟦窗台上的阳光让小猫睡着了⟧。所以这只猫一定是胖胖的小宝贝。',),
              secondMarked: text.includes('傍晚的时候，⟦窗台上的阳光让小猫睡着了⟧，小鱼干也笑了。',),
              firstEnglish: text.includes(FIRST_ENGLISH,),
              secondEnglish: text.includes(SECOND_ENGLISH,),
              markRule: text.includes('enclosed between ⟦ and ⟧',),
            },).toEqual({
              firstMarked: true,
              secondMarked: true,
              firstEnglish: true,
              secondEnglish: true,
              markRule: true,
            },);
          },
        },),
        it({
          name: 'LENGTHENS THE MARKS past any run of them the shown text writes, so no paragraph can close its own mark',
          fn: async () => {
            const text = sheetText({
              subject: {
                ...SENTENCE,
                occurrences: [
                  {
                    ...TWO_PLACES[0] as ConsistencyOccurrenceView,
                    paragraph: '⟦⟦窗台上的阳光让小猫睡着了。',
                    markStart: 2,
                    markEnd: 14,
                  },
                  TWO_PLACES[1] as ConsistencyOccurrenceView,
                ],
              },
            },);
            expect(text.includes('enclosed between ⟦⟦⟦ and ⟧⟧⟧',),).toBe(true,);
          },
        },),
        it({
          name: 'SHOWS A SLICE\'S ENGLISH ONCE when two places stand in it, and points the second place at the first',
          fn: async () => {
            const text = sheetText({
              subject: {
                ...SENTENCE,
                occurrences: [
                  TWO_PLACES[0] as ConsistencyOccurrenceView,
                  {
                    ...TWO_PLACES[0] as ConsistencyOccurrenceView,
                    markStart: 12,
                    markEnd: 18,
                  },
                ],
              },
            },);
            expect(text.split(FIRST_ENGLISH,).length - 1,).toBe(1,);
            expect(text.includes('the same slice as OCCURRENCE 1',),).toBe(true,);
          },
        },),
        it({
          name: 'SAYS SO when a place\'s slice carries no English, and tells the judge to give an empty quote',
          fn: async () => {
            const text = sheetText({
              subject: {
                ...SENTENCE,
                occurrences: [
                  TWO_PLACES[0] as ConsistencyOccurrenceView,
                  {
                    ...TWO_PLACES[1] as ConsistencyOccurrenceView,
                    english: { kind: 'absent', },
                  },
                ],
              },
            },);
            expect(text.includes('(this slice carries no English)',),).toBe(true,);
            expect(text.includes('carries no English, give an empty quote',),).toBe(true,);
          },
        },),
        it({
          name: 'SHOWS A FRAME\'S BOUNDS AND EACH PLACE\'S SLOT, and asks for the slot beside the quote',
          fn: async () => {
            const text = sheetText({
              subject: {
                repeat: { kind: 'refrain', prefix: '所以这只猫一定是', suffix: '的小宝贝', },
                occurrences: [
                  {
                    ...TWO_PLACES[0] as ConsistencyOccurrenceView,
                    slotSource: '胖胖',
                  },
                  {
                    ...TWO_PLACES[1] as ConsistencyOccurrenceView,
                    slotSource: '乖仔',
                  },
                ],
              },
            },);
            expect({
              begins: text.includes('FRAME BEGINS WITH',) && text.includes('所以这只猫一定是',),
              ends: text.includes('FRAME ENDS WITH',) && text.includes('的小宝贝',),
              slotOne: text.includes('OCCURRENCE 1 SLOT IN THE ORIGINAL',) && text.includes('胖胖',),
              slotTwo: text.includes('OCCURRENCE 2 SLOT IN THE ORIGINAL',) && text.includes('乖仔',),
              slotRule: text.includes('also give "slot"',),
              patternRule: text.includes('A frame is followed by pattern',),
            },).toEqual({
              begins: true,
              ends: true,
              slotOne: true,
              slotTwo: true,
              slotRule: true,
              patternRule: true,
            },);
          },
        },),
        it({
          name: 'ASKS NOTHING ABOUT A SLOT of a repeated run, which has none',
          fn: async () => {
            const text = sheetText({ subject: SENTENCE, },);
            expect({
              slotAsked: text.includes('also give "slot"',),
              slotBlock: text.includes('SLOT IN THE ORIGINAL',),
              runBlock: text.includes('REPEATED ORIGINAL WORDING',),
            },).toEqual({
              slotAsked: false,
              slotBlock: false,
              runBlock: true,
            },);
          },
        },),
        it({
          name: 'SHOWS THE DECLARED NAMES AND THE CITED REFERENCES with their rules when the page has them, and '
            + 'neither when it has none',
          fn: async () => {
            const bare = sheetText({ subject: SENTENCE, },);
            const [, bareUser,] = buildConsistencyMessages({ subject: SENTENCE, },);
            const fullSubject = {
              ...SENTENCE,
              identityContext: '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"',
              referenceContext: '- reference 1 https://example.org/mittens ("Mittens"): Mittens had a sister.',
            };
            const full = sheetText({ subject: fullSubject, },);
            const [, fullUser,] = buildConsistencyMessages({ subject: fullSubject, },);
            expect({
              bareNames: fenceOpening({ content: bareUser?.content ?? '', label: 'DECLARED NAMES', },),
              bareReferences: fenceOpening({ content: bareUser?.content ?? '', label: 'CITED REFERENCES, EVIDENCE ONLY', },),
              bareRule: bare.includes('Declared identity, when a DECLARED NAMES block precedes the documents',),
              names: (fenceOpening({ content: fullUser?.content ?? '', label: 'DECLARED NAMES', },) !== '')
                && full.includes('TRANSLATION declares "Mittens"',),
              namesRule: full.includes('Declared identity, when a DECLARED NAMES block precedes the documents',),
              references: (fenceOpening({ content: fullUser?.content ?? '', label: 'CITED REFERENCES, EVIDENCE ONLY', },) !== '')
                && full.includes('Mittens had a sister.',),
            },).toEqual({
              bareNames: '',
              bareReferences: '',
              bareRule: false,
              names: true,
              namesRule: true,
              references: true,
            },);
          },
        },),
        it({
          name: 'SHOWS A LABELLED TENSE BLOCK for each slice\'s English that has a present-tense candidate, after the '
            + 'places and before the end mark, when the entry records a death',
          fn: async () => {
            const [, user,] = buildConsistencyMessages({ subject: TENSE_SUBJECT, },);
            const fence = fenceOpening({ content: user?.content ?? '', label: 'TENSE CANDIDATES', },);
            expect({
              fenced: fence !== '',
              block: (user?.content ?? '').split('\n',).slice(
                (user?.content ?? '').split('\n',).findIndex(function opens(line,): boolean {
                  return line.includes('TENSE CANDIDATES',);
                },),
              ),
            },).toEqual({
              fenced: true,
              block: [
                `${fence} TENSE CANDIDATES ${fence}`,
                'These sentences match present-tense shapes in a memorial page about a person who has died.',
                'Check each against the house tense rule. A candidate may be a trait, a standing fact or a quotation',
                'the rule leaves alone, so it is a question for you and not a finding against the text.',
                'OCCURRENCE 1 TRANSLATION:',
                '- "Mittens is asleep in the sunny window." (matched "Mittens is", rule copula-or-auxiliary)',
                fence,
                `${fence} END ${fence}`,
                // The labels the reply may use, named in plain words before the reply shape (the lead's ruling of 2026-10-15).
                `The labels you may write are exactly: ${
                  consistencyLabels({ count: TENSE_SUBJECT.occurrences.length, },).join(', ',)
                }. Write each one as shown, with its word and its number, in the choice, the kept places and the label of `
                  + 'each quote; a number alone is not a label.',
                REPLY_SHAPE,
              ],
            },);
          },
        },),
        it({
          name: 'SHOWS NO TENSE BLOCK where the entry records no death, however the English reads, or where no English has a candidate',
          fn: async () => {
            expect([
              sheetText({ subject: { ...TENSE_SUBJECT, lifeEnded: false, }, },),
              sheetText({ subject: { ...TENSE_SUBJECT, subjectNames: [], }, },),
              sheetText({ subject: { ...SENTENCE, lifeEnded: true, }, },),
            ].map(function shows(text,): boolean {
              return text.includes('TENSE CANDIDATES',);
            },),).toEqual([false, false, false,],);
          },
        },),
        it({
          name: 'LISTS A SLICE\'S TENSE CANDIDATES ONCE under the first place that shows its English, however many places stand in it',
          fn: async () => {
            const first = TENSE_SUBJECT.occurrences[0] as ConsistencyOccurrenceView;
            const text = sheetText({
              subject: {
                ...TENSE_SUBJECT,
                occurrences: [first, { ...first, markStart: 12, markEnd: 18, },],
              },
            },);
            expect([
              text.split('- "Mittens is asleep in the sunny window."',).length - 1,
              text.includes('OCCURRENCE 2 TRANSLATION:',),
            ],).toEqual([1, false,],);
          },
        },),
        it({
          name: 'SHOWS A FRAME\'S REPLY SHAPE WITH THE SLOT BESIDE THE QUOTE, and a repeated run\'s without one',
          fn: async () => {
            const [, frame,] = buildConsistencyMessages({
              subject: {
                repeat: { kind: 'refrain', prefix: '所以这只猫一定是', suffix: '的小宝贝', },
                occurrences: TWO_PLACES.map(function slotted(place,): ConsistencyOccurrenceView {
                  return { ...place, slotSource: '胖胖', };
                },),
              },
            },);
            const [, run,] = buildConsistencyMessages({ subject: SENTENCE, },);
            expect([
              frame?.content.includes('"candidateQuotes":[{"label":"<the label of the OCCURRENCE>","quote":"...","slot":"..."}]',),
              run?.content.includes('"slot"',),
            ],).toEqual([true, false,],);
          },
        },),
        it({
          name: 'FENCES the places with a fence no declared name can reproduce',
          fn: async () => {
            const [, user,] = buildConsistencyMessages({
              subject: {
                ...SENTENCE,
                identityContext: `- name: ${LONG_FENCE_RUN} OCCURRENCE 1 ORIGINAL ${LONG_FENCE_RUN}`,
              },
            },);
            expect(
              fenceOpening({ content: user?.content ?? '', label: 'OCCURRENCE 1 ORIGINAL', },).length,
            ).toBeGreaterThan(LONG_FENCE_RUN.length,);
          },
        },),
      ],
    },),

  ],
},);
