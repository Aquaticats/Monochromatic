import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import { citedReferenceBlockText, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { marksSentenceOf, } from './consistency-marks-sentence.ts';
import {
  aroundBlocksOf,
  type CarryChosenPlace,
  type CarryEdgeWords,
  type CarryReceivingPlace,
  chosenEnglishBlocksOf,
  keepingBlockOf,
  originalBlocksOf,
  receivingLabel,
  replaceBlocksOf,
} from './consistency-carry-sheet-parts.ts';
import {
  type ConsistencyRepeat,
  repeatBlockOf,
  repeatedTextOf,
  selectMarks,
} from './consistency-sheet.ts';
import {
  DECLARED_IDENTITY_RULES,
  declaredNamesBlock,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import { QUOTED_WORDS_REMINDER, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  HOUSE_POLICY_BLOCK,
  NAME_FORM_SCOPE_RULE,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { PAGE_APPARATUS_IS_KEPT, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  tenseCandidateGroups,
  tenseCandidateLines,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { TRANSLATE_LINE_STRUCTURE_RULE, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Consistency carry writer sheet
// THE SHEET A WRITER READS TO CARRY THE CHOSEN RENDERING TO THE RECEIVING PLACES
// OF ONE BLOCK. The writers are the translate stage's writer bench, asked for the
// same reply the translate stage asks (`{"translation": "..."}`), but the
// passage they rewrite is one paragraph of a settled page, and what they change
// in it is bounded: only the words that render the repeated wording in each
// receiving place of the paragraph, and only as far as that place's own ORIGINAL
// allows. A model writes the paragraph; the floors and the bench's choice decide
// whether any of it reaches the page.

/**
 The reply-format instruction, kept last in the assembled sheet.
 */
const WRITER_REPLY_RULE =
  'Reply with ONLY a JSON object of shape {"translation": "..."}, where the translation is the whole rewritten paragraph and nothing else. No prose, no code fences, no commentary.';

/**
 Everything a writer's sheet shows.

 @example
 ```ts
 const subject: CarryWriterSubject = { repeat, chosen, receiving: [place,], paragraph, keeping: [], lineStructured: false, };
 ```
 */
export type CarryWriterSubject = {
  /**
   The repeated wording or frame.
   */
  readonly repeat: ConsistencyRepeat;

  /**
   The place every place should follow.
   */
  readonly chosen: CarryChosenPlace;

  /**
   The receiving places whose paragraph the writer rewrites, in page order, at
   least one; places of one paragraph are rewritten together.
   */
  readonly receiving: readonly CarryReceivingPlace[];

  /**
   The English paragraph that holds the receiving places' words.
   */
  readonly paragraph: string;

  /**
   English words of the group's other places inside that paragraph that must
   come out of the rewrite exactly as they went in, such as the chosen place's
   own words when it stands in the same paragraph.
   */
  readonly keeping: readonly string[];

  /**
   Stretches of the words of places of the group that run over an edge of that
   paragraph and must stay as they are (the chosen place, a place kept for its context,
   a place that already agrees, a receiving place that code cannot move the chosen
   words into), each of which must come out of the rewrite at the edge it stood at: the
   first words of the paragraph where the place comes in over its start, the last words
   where it goes out over its end. Absent where no place runs over an edge.
   */
  readonly crossing?: readonly CarryEdgeWords[];

  /**
   The original block before the receiving places' block, where it has one in
   its slice.
   */
  readonly before?: string;

  /**
   The original block after the receiving places' block, where it has one in
   its slice.
   */
  readonly after?: string;

  /**
   Whether the line-structure rule governs the receiving places' slice.
   */
  readonly lineStructured: boolean;

  /**
   Declared identity lines of the page, when it declares any.
   */
  readonly identityContext?: string;

  /**
   Lines of the pages the original cites, when it cites any.
   */
  readonly referenceContext?: string;

  /**
   Whether the original's entry records a death, which adds the tense candidates.
   */
  readonly lifeEnded?: boolean;

  /**
   Names the translation declares for its subject; present only with a recorded death.
   */
  readonly subjectNames?: readonly string[];
};

/**
 The rules of the writer's sheet.

 @param lineStructured - whether the line-structure rule governs the slice

 @param several - whether the paragraph holds the repeated wording in several receiving places

 @param keeping - whether the paragraph holds words of other places that must stay

 @param crossing - whether the paragraph holds words of a place that runs over an edge of it, which must stay at that edge

 @returns Rule text, ending in the house rules and the reply format

 @example
 ```ts
 const rules = writerRules({ lineStructured: false, several: false, keeping: false, crossing: false, },);
 ```
 */
function writerRules(
  {
    lineStructured,
    several,
    keeping,
    crossing,
  }: {
    readonly lineStructured: boolean;
    readonly several: boolean;
    readonly keeping: boolean;
    readonly crossing: boolean;
  },
): string {
  /**
   The places the task is about, in the sheet's words.
   */
  const places = several
    ? 'each of THIS PLACE 1, THIS PLACE 2 and so on, which the same paragraph holds'
    : 'this place';
  /**
   Whose ORIGINAL decides what may be adapted, in the sheet's words.
   */
  const own = several ? 'each of those places\'' : 'THIS place\'s';
  /**
   The words to replace, in the sheet's words.
   */
  const toReplace = several
    ? 'THE WORDS TO REPLACE 1, THE WORDS TO REPLACE 2 and so on, one for each of those places'
    : 'THE WORDS TO REPLACE';
  /**
   The rule about the words of other places inside the paragraph.
   */
  const keepingRule = keeping
    ? '\n- The paragraph also holds words of other places, of this repeat or of another (WORDS OF OTHER PLACES TO KEEP EXACTLY). They stay exactly as they are, character for character, wherever they stand.'
    : '';
  /**
   The rule about the stretch of a place that runs over an edge of the paragraph.
   */
  const crossingRule = crossing
    ? '\n- An entry of WORDS OF OTHER PLACES TO KEEP EXACTLY marked as the FIRST or the LAST words of the paragraph belongs to a place that runs over the edge of the paragraph. Those words stay the first or the last words of your paragraph, exactly as they are, and you change nothing of them.'
    : '';
  return [
    `You are a bilingual Chinese-to-English translator working on a memorial archive.

One Chinese wording is repeated in several places of one page. The English of the page was settled one passage at a time, so the places render it differently, and a reader who meets all of them reads the difference as a mistake. A panel of judges has chosen which place's rendering the whole page should follow: the CHOSEN RENDERING. Your task is to make ${places} follow it.

Rewrite the CURRENT ENGLISH PARAGRAPH and return the whole paragraph. The words that now render the repeated wording \
in ${places} (${toReplace}) must come out rendered as the CHOSEN RENDERING renders it, so that the same Chinese \
wording reads as the same English in every place.

Rules:
- Change only the words that render the repeated wording. Every other word of the CURRENT ENGLISH PARAGRAPH stays exactly as it is: its names, its tense, its punctuation, its line breaks and its Markdown, apart from what the next rules allow.
- Adapt only what ${own} own ORIGINAL requires. The CHOSEN RENDERING may carry a word that renders Chinese standing outside the repeated wording in the chosen place, such as an "also" that renders a word standing before the repeated wording there. Never carry such a word here: a word renders ORIGINAL in a place only where that place's ORIGINAL has the Chinese it renders, so the repeated wording is rendered as the chosen place renders it without that word, and a word of this paragraph that renders ORIGINAL outside the repeated wording stays as this paragraph has it.
- In a frame with a slot, the slot content is each place's own: the SLOT IN THE ORIGINAL of a place is rendered with that place's own words, placed where the chosen place's frame places its slot (CHOSEN PLACE: ENGLISH SLOT, where it is shown), in grammatical English, and the frame words around it follow the CHOSEN RENDERING.
- The surrounding ORIGINAL blocks are context for this place, not part of the paragraph: render nothing of them.${keepingRule}${crossingRule}
- ONE TENSE, IN THIS ORDER: where TENSE CANDIDATES are shown, the entry records a death, and the life of a person who has died is told in the past tense by the house rule; a line of it told in the present is moved to the past. Then the tense the page holds for what is still true as it is read. Never let one sentence start in one tense and finish in another. ${QUOTED_WORDS_REMINDER}
- Do not add content the ORIGINAL does not support and do not drop content it states.
- Names, handles and place names already used by this paragraph and by DECLARED NAMES are authoritative; keep them exactly. ${NAME_FORM_SCOPE_RULE}
- ${PAGE_APPARATUS_IS_KEPT}
- Keep the paragraph's Markdown: a block quote stays a block quote, a list marker stays, a footnote marker stays where it stands, and a link keeps its destination.`,
    lineStructured ? TRANSLATE_LINE_STRUCTURE_RULE : '',
    HOUSE_POLICY_BLOCK,
    WRITER_REPLY_RULE,
  ]
    .filter(function isPresent(part,): boolean {
      return part !== '';
    },)
    .join('\n\n',);
}

/**
 Builds the exchange asking one writer to carry the chosen rendering to the
 receiving places of one block.

 @param subject - the repeated wording, the chosen place and the receiving places

 @returns Messages for one exchange

 @example
 ```ts
 const messages = buildCarryWriterMessages({ subject, },);
 ```
 */
export function buildCarryWriterMessages(
  { subject, }: { readonly subject: CarryWriterSubject; },
): readonly ChatMessage[] {
  /**
   The tense candidates of the two English paragraphs, shown only where the
   entry records a death, in the lines every judging sheet prints them in.
   */
  const tenseLines = tenseCandidateLines({
    groups: tenseCandidateGroups({
      lifeEnded: subject.lifeEnded === true,
      ...((subject.subjectNames === undefined) ? {} : { subjectNames: subject.subjectNames, }),
      texts: [
        {
          label: 'CURRENT ENGLISH PARAGRAPH',
          text: subject.paragraph,
        },
        {
          label: 'CHOSEN PLACE: ENGLISH PARAGRAPH',
          text: subject.chosen
            .paragraph,
        },
      ],
    },),
  },);
  /**
   Every text the sheet shows.
   */
  const texts = [
    repeatedTextOf({ repeat: subject.repeat, },),
    ...tenseLines,
    subject.identityContext ?? '',
    subject.referenceContext ?? '',
    subject.before ?? '',
    subject.after ?? '',
    subject.paragraph,
    ...subject.keeping,
    ...(subject.crossing ?? []).map(function wordsOf({ words, },): string {
      return words;
    },),
    subject.chosen
      .paragraph,
    subject.chosen
      .slot
      ?? '',
    ...[
      subject.chosen,
      ...subject.receiving,
    ].flatMap(function textsOf(place,): readonly string[] {
      return [
        place.view
          .paragraph,
        place.view
          .slotSource
          ?? '',
        place.span,
      ];
    },),
  ];
  /**
   Fence absent from every enclosed value.
   */
  const fence = selectFence({ texts, },);
  /**
   Marks absent from every shown text.
   */
  const marks = selectMarks({ texts, },);
  /**
   Declared identity ahead of the places, none when the page has none.
   */
  const identityBlock = declaredNamesBlock({
    fence,
    ...((subject.identityContext === undefined) ? {} : { identityContext: subject.identityContext, }),
  },);
  /**
   Cited pages and their rule, nothing when the original cites none.
   */
  const referenceBlock = citedReferenceBlockText({
    fence,
    ...((subject.referenceContext === undefined) ? {} : { referenceContext: subject.referenceContext, }),
  },);
  /**
   The tense block, fenced like the other blocks, or nothing when no paragraph has a candidate.
   */
  const tenseBlock = (tenseLines.length === 0)
    ? ''
    : `${fence} TENSE CANDIDATES ${fence}\n${tenseLines.join('\n',)}\n${fence}\n`;
  /**
   The ORIGINAL of each receiving place.
   */
  const receivingBlocks = subject.receiving
    .map(function blocksOf(
      place,
      position,
    ): string {
      return originalBlocksOf({
        view: place.view,
        name: receivingLabel({
          name: 'THIS PLACE',
          position,
          count: subject.receiving
            .length,
        },),
        fence,
        marks,
      },);
    },);
  return [
    {
      role: 'system',
      content: `${writerRules({
        lineStructured: subject.lineStructured,
        several: subject.receiving
          .length
          > 1,
        keeping: (subject.keeping
          .length
          + (subject.crossing
            ?.length
            ?? 0))
          > 0,
        crossing: (subject.crossing
          ?.length
          ?? 0) > 0,
      },)}${(identityBlock === '') ? '' : `\n\n${DECLARED_IDENTITY_RULES}`}`,
    },
    {
      role: 'user',
      content: `${
        marksSentenceOf({
          marks,
          shown: 'ORIGINAL',
        },)
      }\n\n${identityBlock}${referenceBlock}${
        repeatBlockOf({
          repeat: subject.repeat,
          fence,
        },)
      }${receivingBlocks.join('',)}${
        aroundBlocksOf({
          ...((subject.before === undefined) ? {} : { before: subject.before, }),
          ...((subject.after === undefined) ? {} : { after: subject.after, }),
          fence,
        },)
      }${
        originalBlocksOf({
          view: subject.chosen
            .view,
          name: 'CHOSEN PLACE',
          fence,
          marks,
        },)
      }${
        chosenEnglishBlocksOf({
          chosen: subject.chosen,
          fence,
        },)
      }${
        keepingBlockOf({
          keeping: subject.keeping,
          crossing: subject.crossing ?? [],
          fence,
        },)
      }${
        replaceBlocksOf({
          receiving: subject.receiving,
          fence,
        },)
      }${fence} CURRENT ENGLISH PARAGRAPH ${fence}\n${subject.paragraph}\n${tenseBlock}${fence} END ${fence}`,
    },
  ];
}

//endregion Consistency carry writer sheet
