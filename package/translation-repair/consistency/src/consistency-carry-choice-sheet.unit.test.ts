/**
 Tests for the sheet the bench reads to choose among a block's candidate
 paragraphs: what is shown (everything the writers of the candidates were
 shown), how the candidates are named, and the reply with the reason first.

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
  buildCarryChoiceMessages,
  type CarryChoiceSubject,
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
 The receiving place with the words of its English that the carry replaces.
 */
const RECEIVING_PLACE = {
  view: RECEIVING_VIEW,
  span: 'wanted to accompany more people with her cuteness',
};

/**
 What the sheet shows for a repeated sentence.
 */
const SUBJECT: CarryChoiceSubject = {
  repeat: { kind: 'maximal', wording: '以可爱的样子陪伴在更多人的身边', },
  receiving: [RECEIVING_PLACE,],
  chosen: {
    view: CHOSEN_VIEW,
    span: 'wanted to stay by more people\'s side looking cute',
    paragraph: 'She also wanted to stay by more people\'s side looking cute.',
  },
  keeping: [],
  candidates: [
    { label: 'CANDIDATE 1', current: false, text: 'She wanted to stay by more people\'s side looking cute.', },
    { label: 'CANDIDATE 2', current: true, text: 'She wanted to accompany more people with her cuteness.', },
    { label: 'CANDIDATE 3', current: false, text: 'She also wanted to stay by more people\'s side looking cute.', },
  ],
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
function sheetText({ subject, }: { readonly subject: CarryChoiceSubject; },): string {
  return buildCarryChoiceMessages({ subject, },)
    .map(function contentOf(message,): string {
      return message.content;
    },)
    .join('\n',);
}

/**
 The user message of an exchange.

 @param subject - what the sheet shows

 @returns Text of the places and candidates

 @example
 ```ts
 const content = userText({ subject: SUBJECT, },);
 ```
 */
function userText({ subject, }: { readonly subject: CarryChoiceSubject; },): string {
  return buildCarryChoiceMessages({ subject, },)[1]
    ?.content
    ?? '';
}

/**
 The label the rules give the block of the words a candidate replaces, read from the sentence that explains it.

 @param rules - system message of an exchange

 @returns Label; empty where the rules have no such sentence

 @example
 ```ts
 const label = labelOfWordsToReplace({ rules, },); // 'THE WORDS TO REPLACE'
 ```
 */
function labelOfWordsToReplace({ rules, }: { readonly rules: string; },): string {
  /**
   The words that lead into the label.
   */
  const lead = 'now render the repeated wording in each receiving place are shown as ';
  /**
   The rules after the lead, none when the rules lack the sentence.
   */
  const [, afterLead = '',] = rules.split(
    lead,
    2,
  );
  /**
   The label, which the sentence follows with its numbering.
   */
  const [label = '',] = afterLead.split(
    ' (numbered',
    1,
  );
  return label;
}

/**
 The fenced blocks of a user message whose title is a label or the label and a number, with the words each holds.

 @param content - user message of an exchange

 @param label - label the rules name

 @returns Title and first line of each such block, in the order the sheet shows them; none for an empty label

 @example
 ```ts
 const blocks = blocksLabelled({ content, label: 'THE WORDS TO REPLACE', },);
 ```
 */
function blocksLabelled(
  {
    content,
    label,
  }: {
    readonly content: string;
    readonly label: string;
  },
): readonly { readonly title: string; readonly words: string; }[] {
  if (label === '')
    return [];
  /**
   Every line of the message.
   */
  const lines = content.split('\n',);
  return lines.flatMap(function blockOf(
    line,
    position,
  ): readonly { readonly title: string; readonly words: string; }[] {
    /**
     The parts of a line that opens a block: the fence, the title, then the fence again.
     */
    const parts = line.split(' ',);
    /**
     The title between the two fences.
     */
    const title = parts.slice(
      1,
      -1,
    )
      .join(' ',);
    return (line.startsWith('=',) && ((title === label) || title.startsWith(`${label} `,)))
      ? [{ title, words: lines[position + 1] ?? '', },]
      : [];
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildCarryChoiceMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS WITH A SYSTEM MESSAGE OF RULES AND THE HOUSE RULES, then a user message of the originals, the chosen '
            + 'rendering and the candidates',
          fn: async () => {
            const [system, user,] = buildCarryChoiceMessages({ subject: SUBJECT, },);
            expect({
              roles: [system?.role, user?.role,],
              houseRules: system?.content.includes('House rules this corpus is written under',),
              original: user?.content.includes('她想⟦以可爱的样子陪伴在更多人的身边⟧。',),
              chosen: user?.content.includes('CHOSEN RENDERING',),
              candidates: ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',].every(function shown(label,): boolean {
                return user?.content.includes(label,) === true;
              },),
            },).toEqual({ roles: ['system', 'user',], houseRules: true, original: true, chosen: true, candidates: true, },);
          },
        },),
        it({
          name: 'SHOWS THE JUDGES WHAT THE WRITERS WERE SHOWN: the receiving place\'s ORIGINAL, the chosen place\'s ORIGINAL, '
            + 'the chosen rendering and the chosen place\'s English paragraph, before the candidates',
          fn: async () => {
            const content = userText({ subject: SUBJECT, },);
            const order = [
              'THIS PLACE: ORIGINAL',
              '她想⟦以可爱的样子陪伴在更多人的身边⟧。',
              'CHOSEN PLACE: ORIGINAL',
              '她也想⟦以可爱的样子陪伴在更多人的身边⟧。',
              'CHOSEN RENDERING',
              'CHOSEN PLACE: ENGLISH PARAGRAPH',
              'CANDIDATE 1',
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
          name: 'SHOWS THE ORIGINAL BLOCKS BEFORE AND AFTER THE RECEIVING PLACES\' BLOCK when the slice has them, as the '
            + 'writers\' sheet does, and none otherwise',
          fn: async () => {
            const around = userText({ subject: { ...SUBJECT, before: '傍晚的阳光很暖。', after: '小猫睡着了。', }, },);
            const bare = userText({ subject: SUBJECT, },);
            expect({
              before: around.includes('ORIGINAL BEFORE THIS PLACE',) && around.includes('傍晚的阳光很暖。',),
              after: around.includes('ORIGINAL AFTER THIS PLACE',) && around.includes('小猫睡着了。',),
              bareBefore: bare.includes('ORIGINAL BEFORE THIS PLACE',),
              bareAfter: bare.includes('ORIGINAL AFTER THIS PLACE',),
            },).toEqual({ before: true, after: true, bareBefore: false, bareAfter: false, },);
          },
        },),
        it({
          name: 'SHOWS THE CHOSEN PLACE\'S ENGLISH SLOT when it has one located, as the writers\' sheet does, and none otherwise',
          fn: async () => {
            const bare = userText({ subject: SUBJECT, },);
            const slotted = userText({ subject: { ...SUBJECT, chosen: { ...SUBJECT.chosen, slot: 'looking cute', }, }, },);
            expect({
              bare: bare.includes('CHOSEN PLACE: ENGLISH SLOT',),
              slotted: slotted.includes('CHOSEN PLACE: ENGLISH SLOT',) && slotted.includes('\nlooking cute\n',),
            },).toEqual({ bare: false, slotted: true, },);
          },
        },),
        it({
          name: 'NUMBERS THE RECEIVING PLACES WHEN A BLOCK HOLDS SEVERAL, shows the words of other places to keep, and says '
            + 'nothing of either for a single place',
          fn: async () => {
            const several = userText({
              subject: { ...SUBJECT, receiving: [RECEIVING_PLACE, RECEIVING_PLACE,], keeping: ['She also wanted',], },
            },);
            const single = userText({ subject: SUBJECT, },);
            expect({
              numbered: ['THIS PLACE 1: ORIGINAL', 'THIS PLACE 2: ORIGINAL',].map(function opens(label,): boolean {
                return fenceOpening({ content: several, label, },) !== '';
              },),
              keeping: several.includes('WORDS OF OTHER PLACES TO KEEP EXACTLY',)
                && several.includes('\nShe also wanted\n',),
              singleLabel: fenceOpening({ content: single, label: 'THIS PLACE: ORIGINAL', },) !== '',
              singleKeeping: single.includes('WORDS OF OTHER PLACES TO KEEP EXACTLY',),
            },).toEqual({ numbered: [true, true,], keeping: true, singleLabel: true, singleKeeping: false, },);
          },
        },),
        it({
          name: 'SHOWS THE WORDS EACH RECEIVING PLACE REPLACES, under the label the writers\' sheet gives them, numbered when '
            + 'a block holds several and bare for a single place, between the words to keep and the candidates',
          fn: async () => {
            const single = userText({ subject: { ...SUBJECT, keeping: ['She also wanted',], }, },);
            const several = userText({
              subject: {
                ...SUBJECT,
                receiving: [RECEIVING_PLACE, { ...RECEIVING_PLACE, span: 'wanted to be cute for more people', },],
              },
            },);
            expect({
              single: fenceOpening({ content: single, label: 'THE WORDS TO REPLACE', },) !== '',
              singleWords: single.includes('\nwanted to accompany more people with her cuteness\n',),
              singleOrder: [
                single.indexOf('WORDS OF OTHER PLACES TO KEEP EXACTLY',),
                single.indexOf('THE WORDS TO REPLACE',),
                single.indexOf('CANDIDATE 1',),
              ].every(function ascending(at, position, all,): boolean {
                return (at >= 0) && ((position === 0) || ((all[position - 1] ?? 0) < at));
              },),
              numbered: ['THE WORDS TO REPLACE 1', 'THE WORDS TO REPLACE 2',].map(function opens(label,): boolean {
                return fenceOpening({ content: several, label, },) !== '';
              },),
              secondWords: several.includes('\nwanted to be cute for more people\n',),
              bareWhenSeveral: several.includes(' THE WORDS TO REPLACE =====\n',),
            },).toEqual({
              single: true,
              singleWords: true,
              singleOrder: true,
              numbered: [true, true,],
              secondWords: true,
              bareWhenSeveral: false,
            },);
          },
        },),
        it({
          name: 'EXPLAINS THE WORDS TO REPLACE IN THE RULES AS THE BLOCKS THE SHEET SHOWS: the label the rules name opens one '
            + 'block for each receiving place with that place\'s words, numbered when there are several, and no block '
            + 'opens for no receiving place',
          fn: async () => {
            const [system,] = buildCarryChoiceMessages({ subject: SUBJECT, },);
            const label = labelOfWordsToReplace({ rules: system?.content ?? '', },);
            expect({
              named: label,
              single: blocksLabelled({ content: userText({ subject: SUBJECT, },), label, },),
              several: blocksLabelled({
                content: userText({
                  subject: {
                    ...SUBJECT,
                    receiving: [RECEIVING_PLACE, { ...RECEIVING_PLACE, span: 'wanted to be cute for more people', },],
                  },
                },),
                label,
              },),
              none: blocksLabelled({ content: userText({ subject: { ...SUBJECT, receiving: [], }, },), label, },),
            },).toEqual({
              named: 'THE WORDS TO REPLACE',
              single: [{ title: 'THE WORDS TO REPLACE', words: 'wanted to accompany more people with her cuteness', },],
              several: [
                { title: 'THE WORDS TO REPLACE 1', words: 'wanted to accompany more people with her cuteness', },
                { title: 'THE WORDS TO REPLACE 2', words: 'wanted to be cute for more people', },
              ],
              none: [],
            },);
          },
        },),
        it({
          name: 'NAMES THE CANDIDATE THAT IS THE PARAGRAPH AS THE PAGE HAS IT NOW, and no candidate\'s author',
          fn: async () => {
            const content = userText({ subject: SUBJECT, },);
            expect({
              current: content.includes(' CANDIDATE 2 (THE PARAGRAPH AS THE PAGE HAS IT NOW) ',),
              others: content.split('THE PARAGRAPH AS THE PAGE HAS IT NOW',).length - 1,
              author: ['writer', 'translator', 'splice',].some(function isNamed(word,): boolean {
                return content.toLowerCase()
                  .includes(word,);
              },),
            },).toEqual({ current: true, others: 1, author: false, },);
          },
        },),
        it({
          name: 'TELLS THE JUDGE TO WRITE THE REASON FIRST and to choose after it, in the rules, in a reply shape that names '
            + 'no label of the sheet, and treats an "also" the chosen place carries for Chinese outside the repeat as an addition',
          fn: async () => {
            const [system, user,] = buildCarryChoiceMessages({ subject: SUBJECT, },);
            expect({
              rule: system?.content.includes('Write the assessment first and choose after it',),
              keys: system?.content.includes('keys come in this order: assessment, choice',),
              shape: user?.content.endsWith(
                'Reply with JSON only: {"assessment":"...","choice":"<the label of the candidate you choose>"}',
              ),
              also: system?.content.includes(
                'A word that renders Chinese outside the repeated wording in the chosen place (such as an "also" the chosen place carries for a word before its repeated wording) is an addition here',
              ),
              standard: system?.content.includes('never choose a candidate because it came first, came last, is longest or is the commonest',),
            },).toEqual({ rule: true, keys: true, shape: true, also: true, standard: true, },);
            expect(['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',].map(function inReplyShape(label,): boolean {
              return (user?.content.split('Reply with JSON only:',)[1] ?? '').includes(label,);
            },),).toEqual([false, false, false,],);
          },
        },),
        it({
          name: 'SHOWS A FRAME\'S BOUNDS AND THE RECEIVING PLACE\'S SLOT, and a repeated run\'s neither',
          fn: async () => {
            const frame = sheetText({
              subject: {
                ...SUBJECT,
                repeat: { kind: 'refrain', prefix: '所以她是个', suffix: '的女孩吧', },
                receiving: [{ ...RECEIVING_PLACE, view: { ...RECEIVING_VIEW, slotSource: '温柔', }, },],
              },
            },);
            const run = sheetText({ subject: SUBJECT, },);
            expect({
              frame: frame.includes('FRAME BEGINS WITH',) && frame.includes('SLOT IN THE ORIGINAL',) && frame.includes('温柔',),
              run: run.includes('SLOT IN THE ORIGINAL',) || run.includes('FRAME BEGINS WITH',),
            },).toEqual({ frame: true, run: false, },);
          },
        },),
        it({
          name: 'SHOWS THE TENSE CANDIDATES OF EACH CANDIDATE TEXT, labelled, when the entry records a death, and none otherwise',
          fn: async () => {
            const dead: CarryChoiceSubject = {
              ...SUBJECT,
              candidates: [
                { label: 'CANDIDATE 1', current: true, text: 'Mittens is asleep in the window.', },
                { label: 'CANDIDATE 2', current: false, text: 'Mittens was asleep in the window.', },
              ],
              lifeEnded: true,
              subjectNames: ['Mittens',],
            };
            const [, user,] = buildCarryChoiceMessages({ subject: dead, },);
            const [, living,] = buildCarryChoiceMessages({ subject: { ...dead, lifeEnded: false, }, },);
            expect({
              block: user?.content.includes('TENSE CANDIDATES',),
              line: user?.content.includes(
                'CANDIDATE 1:\n- "Mittens is asleep in the window." (matched "Mittens is", rule copula-or-auxiliary)',
              ),
              second: user?.content.includes('CANDIDATE 2:\n- ',),
              none: living?.content.includes('TENSE CANDIDATES',),
            },).toEqual({ block: true, line: true, second: false, none: false, },);
          },
        },),
        it({
          name: 'SHOWS THE DECLARED NAMES AND THE CITED PAGES with their rules when the page has them, and neither otherwise',
          fn: async () => {
            const [, bare,] = buildCarryChoiceMessages({ subject: SUBJECT, },);
            const full: CarryChoiceSubject = {
              ...SUBJECT,
              identityContext: '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"',
              referenceContext: '- reference 1 https://example.org/mittens ("Mittens"): Mittens had a sister.',
            };
            const [, user,] = buildCarryChoiceMessages({ subject: full, },);
            expect({
              bareNames: fenceOpening({ content: bare?.content ?? '', label: 'DECLARED NAMES', },),
              bareReferences: fenceOpening({ content: bare?.content ?? '', label: 'CITED REFERENCES, EVIDENCE ONLY', },),
              names: fenceOpening({ content: user?.content ?? '', label: 'DECLARED NAMES', },) !== '',
              references: fenceOpening({ content: user?.content ?? '', label: 'CITED REFERENCES, EVIDENCE ONLY', },) !== '',
            },).toEqual({ bareNames: '', bareReferences: '', names: true, references: true, },);
          },
        },),
        it({
          name: 'FENCES the candidates with a fence no shown text can reproduce, the chosen paragraph, the chosen slot, the '
            + 'words to keep and the before and after blocks among them',
          fn: async () => {
            /**
             Whether the fence of the first candidate outgrows the run the subject writes.
             */
            function outgrows({ subject, }: { readonly subject: CarryChoiceSubject; },): boolean {
              return fenceOpening({
                content: userText({ subject, },),
                label: 'CANDIDATE 1',
              },).length > LONG_FENCE_RUN.length;
            }
            expect({
              identity: outgrows({
                subject: { ...SUBJECT, identityContext: `- name: ${LONG_FENCE_RUN} CANDIDATE 1 ${LONG_FENCE_RUN}`, },
              },),
              paragraph: outgrows({ subject: { ...SUBJECT, chosen: { ...SUBJECT.chosen, paragraph: LONG_FENCE_RUN, }, }, },),
              slot: outgrows({ subject: { ...SUBJECT, chosen: { ...SUBJECT.chosen, slot: LONG_FENCE_RUN, }, }, },),
              keeping: outgrows({ subject: { ...SUBJECT, keeping: [LONG_FENCE_RUN,], }, },),
              before: outgrows({ subject: { ...SUBJECT, before: LONG_FENCE_RUN, }, },),
              after: outgrows({ subject: { ...SUBJECT, after: LONG_FENCE_RUN, }, },),
              replaced: outgrows({ subject: { ...SUBJECT, receiving: [{ ...RECEIVING_PLACE, span: LONG_FENCE_RUN, },], }, },),
            },).toEqual({ identity: true, paragraph: true, slot: true, keeping: true, before: true, after: true, replaced: true, },);
          },
        },),
        it({
          name: 'GIVES THE RUN-OFF ITS OWN RULES, saying a first round was split between these candidates and the judge is asked '
            + 'again, so its messages differ from the first ask\'s even when every candidate is a finalist, and leaves the first '
            + 'ask\'s messages as they were',
          fn: async () => {
            /**
             The messages of the first ask and of the run-off over one subject, whose three candidates are all the finalists.
             */
            const [first, again,] = [false, true,].map(function messagesOf(runOff,) {
              return buildCarryChoiceMessages({ subject: SUBJECT, ...(runOff ? { runOff, } : {}), },);
            },);
            expect({
              sameUser: first?.[1]?.content === again?.[1]?.content,
              sameSystem: first?.[0]?.content === again?.[0]?.content,
              firstSaysSplit: first?.[0]?.content.includes('A first round of judges was split',),
              againSaysSplit: again?.[0]?.content.includes('A first round of judges was split between the candidates',),
              againAsksAgain: again?.[0]?.content.includes('you are asked again, between these candidates only',),
              againKeepsRules: again?.[0]?.content.includes('Your task is to choose the candidate that renders the ORIGINAL best',),
            },).toEqual({
              sameUser: true,
              sameSystem: false,
              firstSaysSplit: false,
              againSaysSplit: true,
              againAsksAgain: true,
              againKeepsRules: true,
            },);
            expect(first,).toEqual(buildCarryChoiceMessages({ subject: SUBJECT, runOff: false, },),);
          },
        },),
      ],
    },),
  ],
},);
