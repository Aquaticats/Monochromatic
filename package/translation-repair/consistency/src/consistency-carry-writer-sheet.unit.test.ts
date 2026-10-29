/**
 Tests for the sheet a writer reads to carry the chosen rendering to the
 receiving places of one block: what the writer is shown, what it may change,
 and the reply it gives.

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
  buildCarryWriterMessages,
  type CarryWriterSubject,
  type ConsistencyOccurrenceView,
} from '../dist/final/node/index.mjs';
import {
  fenceOpening,
  LONG_FENCE_RUN,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/sheet-fence.test-fixture.ts';

/**
 The chosen place as the consistency sheet shows it.
 */
const CHOSEN_VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 8,
  paragraph: '她也想以可爱的样子陪伴在更多人的身边。',
  markStart: 3,
  markEnd: 18,
  english: { kind: 'present', text: 'She also wanted to stay by more people\'s side looking cute.', },
};

/**
 The receiving place as the consistency sheet shows it.
 */
const RECEIVING_VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 15,
  paragraph: '她想以可爱的样子陪伴在更多人的身边。',
  markStart: 2,
  markEnd: 17,
  english: { kind: 'present', text: 'She wanted to accompany more people with her cuteness.', },
};

/**
 The receiving place as the writer's sheet shows it.
 */
const RECEIVING_PLACE = {
  view: RECEIVING_VIEW,
  span: 'wanted to accompany more people with her cuteness',
};

/**
 What the sheet shows for a repeated sentence.
 */
const SUBJECT: CarryWriterSubject = {
  repeat: { kind: 'maximal', wording: '以可爱的样子陪伴在更多人的身边', },
  chosen: {
    view: CHOSEN_VIEW,
    span: 'wanted to stay by more people\'s side looking cute',
    paragraph: 'She also wanted to stay by more people\'s side looking cute.',
  },
  receiving: [RECEIVING_PLACE,],
  paragraph: 'She wanted to accompany more people with her cuteness.',
  keeping: [],
  lineStructured: false,
};

/**
 Both messages of an exchange, joined.

 @param subject - what the sheet shows

 @returns Both messages' text, system first

 @example
 ```ts
 const text = sheetText({ subject: SUBJECT, },);
 ```
 */
function sheetText({ subject, }: { readonly subject: CarryWriterSubject; },): string {
  return buildCarryWriterMessages({ subject, },)
    .map(function contentOf(message,): string {
      return message.content;
    },)
    .join('\n',);
}

/**
 The user message of an exchange.

 @param subject - what the sheet shows

 @returns Text of the places

 @example
 ```ts
 const content = userText({ subject: SUBJECT, },);
 ```
 */
function userText({ subject, }: { readonly subject: CarryWriterSubject; },): string {
  return buildCarryWriterMessages({ subject, },)[1]
    ?.content
    ?? '';
}

/**
 The subject with a second receiving place in the same paragraph.

 @returns Subject

 @example
 ```ts
 const two = withTwoPlaces();
 ```
 */
function withTwoPlaces(): CarryWriterSubject {
  return {
    ...SUBJECT,
    receiving: [
      ...SUBJECT.receiving,
      {
        view: { ...RECEIVING_VIEW, paragraph: '她想以可爱的样子陪伴在更多人的身边，她想以可爱的样子陪伴在更多人的身边。', markStart: 20, markEnd: 35, },
        span: 'wanted to be cute for more people',
      },
    ],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildCarryWriterMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS WITH A SYSTEM MESSAGE OF RULES, THE HOUSE RULES AND THE REPLY FORMAT, then a user message of the places',
          fn: async () => {
            const [system, user,] = buildCarryWriterMessages({ subject: SUBJECT, },);
            expect({
              roles: [system?.role, user?.role,],
              houseRules: system?.content.includes('House rules this corpus is written under',),
              replyLast: system?.content.endsWith(
                'Reply with ONLY a JSON object of shape {"translation": "..."}, where the translation is the whole rewritten paragraph and nothing else. No prose, no code fences, no commentary.',
              ),
              userEnds: user?.content.endsWith(' END ',) === false,
            },).toEqual({
              roles: ['system', 'user',],
              houseRules: true,
              replyLast: true,
              userEnds: true,
            },);
          },
        },),
        it({
          name: 'SHOWS THE RECEIVING PLACE\'S ORIGINAL WITH THE REPEAT MARKED, the chosen place\'s, the chosen rendering, the '
            + 'chosen place\'s English paragraph, the words to replace and the current English paragraph, in that order',
          fn: async () => {
            const content = userText({ subject: SUBJECT, },);
            const order = [
              'THIS PLACE: ORIGINAL',
              '她想⟦以可爱的样子陪伴在更多人的身边⟧。',
              'CHOSEN PLACE: ORIGINAL',
              '她也想⟦以可爱的样子陪伴在更多人的身边⟧。',
              'CHOSEN RENDERING',
              'wanted to stay by more people\'s side looking cute',
              'CHOSEN PLACE: ENGLISH PARAGRAPH',
              'THE WORDS TO REPLACE',
              'wanted to accompany more people with her cuteness',
              'CURRENT ENGLISH PARAGRAPH',
            ].map(function positionOf(needle,): number {
              return content.indexOf(needle,);
            },);
            expect(order.every(function found(at,): boolean {
              return at >= 0;
            },),).toBe(true,);
            expect(order,).toEqual(order.toSorted(function ascending(left, right,): number {
              return left - right;
            },),);
          },
        },),
        it({
          name: 'TELLS THE WRITER TO CHANGE ONLY THE WORDS THAT RENDER THE REPEAT, and never to carry a word of the chosen '
            + 'place that renders Chinese standing outside the repeat there, such as an "also"',
          fn: async () => {
            const text = sheetText({ subject: SUBJECT, },);
            expect({
              onlyRepeat: text.includes('Change only the words that render the repeated wording.',),
              own: text.includes('Adapt only what THIS place\'s own ORIGINAL requires.',),
              also: text.includes(
                'such as an "also" that renders a word standing before the repeated wording there. Never carry such a word here',
              ),
              marks: text.includes('enclosed between ⟦ and ⟧',),
            },).toEqual({ onlyRepeat: true, own: true, also: true, marks: true, },);
          },
        },),
        it({
          name: 'SHOWS THE BLOCKS AROUND THE RECEIVING PLACE\'S PARAGRAPH as context when its slice has them, and tells the writer '
            + 'to render nothing of them',
          fn: async () => {
            const [, bare,] = buildCarryWriterMessages({ subject: SUBJECT, },);
            const [, around,] = buildCarryWriterMessages({
              subject: { ...SUBJECT, before: '傍晚的阳光很暖。', after: '小猫睡着了。', },
            },);
            expect({
              bareBefore: bare?.content.includes('ORIGINAL BEFORE THIS PLACE',),
              bareAfter: bare?.content.includes('ORIGINAL AFTER THIS PLACE',),
              before: (around?.content.includes('ORIGINAL BEFORE THIS PLACE',) === true)
                && around?.content.includes('傍晚的阳光很暖。',),
              after: (around?.content.includes('ORIGINAL AFTER THIS PLACE',) === true)
                && around?.content.includes('小猫睡着了。',),
              rule: sheetText({ subject: SUBJECT, },).includes('render nothing of them',),
            },).toEqual({ bareBefore: false, bareAfter: false, before: true, after: true, rule: true, },);
          },
        },),
        it({
          name: 'SHOWS A FRAME\'S SLOT IN THE ORIGINAL for each place, and tells the writer the slot is each place\'s own '
            + 'while the frame words follow the chosen rendering',
          fn: async () => {
            const text = sheetText({
              subject: {
                ...SUBJECT,
                repeat: { kind: 'refrain', prefix: '所以她是个', suffix: '的女孩吧', },
                chosen: { ...SUBJECT.chosen, view: { ...CHOSEN_VIEW, slotSource: '可靠', }, },
                receiving: [{ ...RECEIVING_PLACE, view: { ...RECEIVING_VIEW, slotSource: '温柔', }, },],
              },
            },);
            expect({
              receivingSlot: text.includes('THIS PLACE: SLOT IN THE ORIGINAL',) && text.includes('温柔',),
              chosenSlot: text.includes('CHOSEN PLACE: SLOT IN THE ORIGINAL',) && text.includes('可靠',),
              rule: text.includes('the slot content is each place\'s own',),
              begins: text.includes('FRAME BEGINS WITH',),
            },).toEqual({ receivingSlot: true, chosenSlot: true, rule: true, begins: true, },);
          },
        },),
        it({
          name: 'SHOWS THE CHOSEN PLACE\'S ENGLISH SLOT when the chosen place has one located, and none otherwise',
          fn: async () => {
            const bare = userText({ subject: SUBJECT, },);
            const slotted = userText({ subject: { ...SUBJECT, chosen: { ...SUBJECT.chosen, slot: 'looking cute', }, }, },);
            expect({
              bare: fenceOpening({ content: bare, label: 'CHOSEN PLACE: ENGLISH SLOT', },),
              slotted: fenceOpening({ content: slotted, label: 'CHOSEN PLACE: ENGLISH SLOT', },) !== '',
              words: slotted.includes('CHOSEN PLACE: ENGLISH SLOT',) && slotted.includes('\nlooking cute\n',),
            },).toEqual({ bare: '', slotted: true, words: true, },);
          },
        },),
        it({
          name: 'NUMBERS THE RECEIVING PLACES AND THEIR WORDS TO REPLACE WHEN A PARAGRAPH HOLDS SEVERAL, and names a single one '
            + 'bare',
          fn: async () => {
            const single = userText({ subject: SUBJECT, },);
            const several = userText({ subject: withTwoPlaces(), },);
            const rules = sheetText({ subject: withTwoPlaces(), },);
            expect({
              singleLabels: ['THIS PLACE: ORIGINAL', 'THE WORDS TO REPLACE',].map(function opens(label,): boolean {
                return fenceOpening({ content: single, label, },) !== '';
              },),
              severalLabels: [
                'THIS PLACE 1: ORIGINAL',
                'THIS PLACE 2: ORIGINAL',
                'THE WORDS TO REPLACE 1',
                'THE WORDS TO REPLACE 2',
              ].map(function opens(label,): boolean {
                return fenceOpening({ content: several, label, },) !== '';
              },),
              bareLabelInSeveral: several.includes(
                `${fenceOpening({ content: several, label: 'THE WORDS TO REPLACE 1', },)} THE WORDS TO REPLACE ${
                  fenceOpening({ content: several, label: 'THE WORDS TO REPLACE 1', },)
                }`,
              ),
              rule: rules.includes('THE WORDS TO REPLACE 1, THE WORDS TO REPLACE 2 and so on',),
            },).toEqual({
              singleLabels: [true, true,],
              severalLabels: [true, true, true, true,],
              bareLabelInSeveral: false,
              rule: true,
            },);
          },
        },),
        it({
          name: 'NAMES THE WORDS OF OTHER PLACES THAT MUST STAY and tells the writer to keep them exactly, and says nothing '
            + 'of them when there are none',
          fn: async () => {
            const kept = { ...SUBJECT, keeping: ['She also wanted to stay by more people\'s side',], };
            expect({
              block: userText({ subject: kept, },).includes(
                'WORDS OF OTHER PLACES TO KEEP EXACTLY',
              ),
              words: userText({ subject: kept, },).includes('\nShe also wanted to stay by more people\'s side\n',),
              rule: sheetText({ subject: kept, },).includes('They stay exactly as they are, character for character',),
              noneBlock: userText({ subject: SUBJECT, },).includes('WORDS OF OTHER PLACES TO KEEP EXACTLY',),
              noneRule: sheetText({ subject: SUBJECT, },).includes('character for character',),
            },).toEqual({ block: true, words: true, rule: true, noneBlock: false, noneRule: false, },);
          },
        },),
        it({
          name: 'LISTS A STRETCH OF A PLACE THAT RUNS OVER AN EDGE OF THE PARAGRAPH among the words to keep and tells the '
            + 'writer it stays the first or the last words, and says nothing of it when no place runs over',
          fn: async () => {
            const crossing = { ...SUBJECT, crossing: [{ edge: 'start', words: 'to sleep', },] as const, };
            const alone = { ...SUBJECT, keeping: [], crossing: [{ edge: 'end', words: 'put the kitten', },] as const, };
            expect({
              block: userText({ subject: crossing, },).includes('WORDS OF OTHER PLACES TO KEEP EXACTLY',),
              first: userText({ subject: crossing, },).includes('\nto sleep\n(these are the FIRST words of the paragraph',),
              last: userText({ subject: alone, },).includes('\nput the kitten\n(these are the LAST words of the paragraph',),
              rule: sheetText({ subject: crossing, },).includes('marked as the FIRST or the LAST words of the paragraph',),
              keepingRule: sheetText({ subject: alone, },).includes('They stay exactly as they are, character for character',),
              noneRule: sheetText({ subject: SUBJECT, },).includes('FIRST or the LAST words',),
              noneBlock: userText({ subject: SUBJECT, },).includes('(these are the',),
            },).toEqual({
              block: true,
              first: true,
              last: true,
              rule: true,
              keepingRule: true,
              noneRule: false,
              noneBlock: false,
            },);
          },
        },),
        it({
          name: 'ADDS THE LINE-STRUCTURE RULE only where the receiving place\'s slice is line-structured',
          fn: async () => {
            const plain = sheetText({ subject: SUBJECT, },);
            const verse = sheetText({ subject: { ...SUBJECT, lineStructured: true, }, },);
            expect(verse.length > plain.length,).toBe(true,);
            expect(
              verse.startsWith(plain.slice(0, 200,),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'SHOWS THE TENSE CANDIDATES of both English paragraphs, labelled, when the entry records a death, and none '
            + 'otherwise',
          fn: async () => {
            const dead = {
              ...SUBJECT,
              chosen: { ...SUBJECT.chosen, paragraph: 'Mittens is asleep in the window.', },
              lifeEnded: true,
              subjectNames: ['Mittens',],
            };
            const [, user,] = buildCarryWriterMessages({ subject: dead, },);
            expect({
              block: user?.content.includes('TENSE CANDIDATES',),
              line: user?.content.includes(
                'CHOSEN PLACE: ENGLISH PARAGRAPH:\n- "Mittens is asleep in the window." (matched "Mittens is", rule copula-or-auxiliary)',
              ),
              none: buildCarryWriterMessages({ subject: { ...dead, lifeEnded: false, }, },)[1]?.content.includes('TENSE CANDIDATES',),
            },).toEqual({ block: true, line: true, none: false, },);
          },
        },),
        it({
          name: 'SHOWS THE DECLARED NAMES AND THE CITED PAGES with their rules when the page has them, and neither otherwise',
          fn: async () => {
            const [, bare,] = buildCarryWriterMessages({ subject: SUBJECT, },);
            const full = {
              ...SUBJECT,
              identityContext: '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"',
              referenceContext: '- reference 1 https://example.org/mittens ("Mittens"): Mittens had a sister.',
            };
            const [, user,] = buildCarryWriterMessages({ subject: full, },);
            expect({
              bareNames: fenceOpening({ content: bare?.content ?? '', label: 'DECLARED NAMES', },),
              bareReferences: fenceOpening({ content: bare?.content ?? '', label: 'CITED REFERENCES, EVIDENCE ONLY', },),
              names: (fenceOpening({ content: user?.content ?? '', label: 'DECLARED NAMES', },) !== '')
                && sheetText({ subject: full, },).includes('Declared identity, when a DECLARED NAMES block precedes the documents',),
              references: fenceOpening({ content: user?.content ?? '', label: 'CITED REFERENCES, EVIDENCE ONLY', },) !== '',
            },).toEqual({ bareNames: '', bareReferences: '', names: true, references: true, },);
          },
        },),
        it({
          name: 'FENCES the blocks with a fence no shown text can reproduce, the words to keep, the before and after blocks '
            + 'and the chosen slot among them, and lengthens the marks past any the paragraphs write',
          fn: async () => {
            const receiving = [
              {
                ...RECEIVING_PLACE,
                view: { ...RECEIVING_VIEW, paragraph: '⟦⟦她想以可爱的样子陪伴在更多人的身边。', markStart: 3, markEnd: 20, },
              },
            ];
            /**
             Whether the fence of the paragraph block is longer than the run the subject writes.
             */
            function outgrows({ subject, }: { readonly subject: CarryWriterSubject; },): boolean {
              return fenceOpening({
                content: userText({ subject, },),
                label: 'CURRENT ENGLISH PARAGRAPH',
              },).length > LONG_FENCE_RUN.length;
            }
            expect({
              identity: outgrows({
                subject: {
                  ...SUBJECT,
                  identityContext: `- name: ${LONG_FENCE_RUN} CURRENT ENGLISH PARAGRAPH ${LONG_FENCE_RUN}`,
                  receiving,
                },
              },),
              keeping: outgrows({ subject: { ...SUBJECT, keeping: [LONG_FENCE_RUN,], }, },),
              before: outgrows({ subject: { ...SUBJECT, before: LONG_FENCE_RUN, }, },),
              after: outgrows({ subject: { ...SUBJECT, after: LONG_FENCE_RUN, }, },),
              slot: outgrows({ subject: { ...SUBJECT, chosen: { ...SUBJECT.chosen, slot: LONG_FENCE_RUN, }, }, },),
              marks: sheetText({ subject: { ...SUBJECT, receiving, }, },).includes('enclosed between ⟦⟦⟦ and ⟧⟧⟧',),
            },).toEqual({ identity: true, keeping: true, before: true, after: true, slot: true, marks: true, },);
          },
        },),
      ],
    },),
  ],
},);
