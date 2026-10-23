/**
 Two invented subjects for the guard that holds every stage's system prompt to the same text on every call (R4 of the
 second review of the Claude route). The Claude body marks each system prompt for the five-minute cache, so a system
 text that embeds a subject's own words is written to the cache at the higher price and never read back.

 The subjects share a shape and differ in every word: a cat on a windowsill and a dog on a porch, with their own declared
 names, cited references, notes and findings. A stage whose system text is the same for both embeds nothing of either.

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import type { RepairRegion, } from '@monochromatic-dev/translation-repair-adjudication';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type { systemTextOf, } from '@monochromatic-dev/translation-repair-provider';
import type {
  AdjudicatedIssue,
  EditableEnvelope,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type { JudgeReference, } from '../dist/final/node/index.mjs';
import { TRANSLATORS_NOTE, } from './rendered-sheets-texts.test-fixture.ts';

//region System prompt subjects

/**
 Messages one sheet builder hands a provider, in the shape the Claude body reads them in.
 */
export type PromptMessages = Parameters<typeof systemTextOf>[0]['messages'];

/**
 One stage's sheet builder, called with a subject.
 */
export type PromptStage = {
  /**
   Stage the sheet is written for, with the variant the case renders when the stage has several.
   */
  readonly name: string;

  /**
   Builds the stage's messages for a subject.
   */
  readonly render: (args: { readonly subject: PromptSubject; },) => PromptMessages;
};

/**
 What a subject is built from: the words that differ between the two subjects.
 */
type SubjectWords = {
  /**
   The animal in the original.
   */
  readonly originalAnimal: string;

  /**
   The place in the original.
   */
  readonly originalPlace: string;

  /**
   The animal in English.
   */
  readonly animal: string;

  /**
   The place in English.
   */
  readonly place: string;

  /**
   The animal's name in the original.
   */
  readonly originalName: string;

  /**
   The animal's name in English.
   */
  readonly name: string;

  /**
   The last part of the cited reference's address.
   */
  readonly address: string;

  /**
   A row of equals signs every shown text carries on a line of its own (a setext heading underline), which lengthens
   the fence a sheet encloses its blocks in; empty for none.
   */
  readonly underline: string;

  /**
   A stray run of the marks the consistency sheets enclose repeated wording in, set between the sentences of every
   shown text; empty for none.
   */
  readonly stray: string;

  /**
   The explicit line break the verse writes, so the two subjects count different numbers of breaks.
   */
  readonly lineBreak: string;
};

/**
 One invented subject: the texts and the structured records a sheet builder is handed.
 */
export type PromptSubject = {
  /**
   Name of the subject, for a failing case.
   */
  readonly label: string;

  /**
   The original passage.
   */
  readonly source: string;

  /**
   The archive rendering, told in the present.
   */
  readonly archive: string;

  /**
   The repair, told in the past.
   */
  readonly repair: string;

  /**
   The original of the passage next door.
   */
  readonly neighbourSource: string;

  /**
   The archive rendering of the passage next door.
   */
  readonly neighbourArchive: string;

  /**
   Declared names.
   */
  readonly identity: string;

  /**
   Cited references.
   */
  readonly references: string;

  /**
   The animal's name, which a recorded death names.
   */
  readonly name: string;

  /**
   The words two places of the original share, as a repeated sentence.
   */
  readonly wording: string;

  /**
   The frame two places of the original share, as a refrain.
   */
  readonly prefix: string;

  /**
   A title of a work the original writes.
   */
  readonly title: string;

  /**
   An original with explicit line breaks and no archive rendering.
   */
  readonly verseSource: string;

  /**
   A rendering of that original with its lines run together.
   */
  readonly verseFlat: string;

  /**
   The explicit line break the verse writes, as the subject spells it.
   */
  readonly lineBreak: string;

  /**
   A rendering of that original that keeps its line break.
   */
  readonly verseBroken: string;

  /**
   The archive block the original does not claim.
   */
  readonly aside: string;

  /**
   A reviewer's revision of that block.
   */
  readonly asideRevised: string;

  /**
   The opening sentence of the archive rendering, which the claim quotes.
   */
  readonly archiveQuote: string;

  /**
   The opening sentence of the repair.
   */
  readonly repairQuote: string;

  /**
   A finding a retry is told.
   */
  readonly finding: string;

  /**
   A line a picture says.
   */
  readonly pictureContext: string;

  /**
   An objection a gate raised.
   */
  readonly objection: string;

  /**
   A claim the repair panel accepted.
   */
  readonly claim: string;

  /**
   The claims of that issue.
   */
  readonly claims: AdjudicatedIssue['claims'];

  /**
   The accepted issue quoting both documents.
   */
  readonly issue: AdjudicatedIssue;

  /**
   The envelope over the quoted sentence.
   */
  readonly envelope: EditableEnvelope;

  /**
   The envelope over the whole archive rendering.
   */
  readonly paragraphEnvelope: EditableEnvelope;

  /**
   The region a repair replaced.
   */
  readonly region: RepairRegion;

  /**
   The sentence a restoration judge is asked to find again.
   */
  readonly judgeReference: JudgeReference;
};

/**
 Builds a subject from its words.

 @param words - what differs between the subjects

 @returns Subject

 @example
 ```ts
 const subject = subjectOf({ words: { animal: 'kitten', ...rest, }, },);
 ```
 */
function subjectOf(
  { words, }: { readonly words: SubjectWords; },
): PromptSubject {
  /**
   Opening sentence of the original.
   */
  const sourceQuote = `那晚${words.originalAnimal}在${words.originalPlace}上睡着了。`;
  /**
   Opening sentence of the archive rendering.
   */
  const archiveQuote = `That night the ${words.animal} falls asleep on the ${words.place}.`;
  /**
   Opening sentence of the repair.
   */
  const repairQuote = `That night the ${words.animal} fell asleep on the ${words.place}.`;
  /**
   What joins the sentences of an original: nothing, the underline on a line of its own, or the stray marks.
   */
  const joinOriginal = (words.underline === '') ? words.stray : `\n${words.underline}\n`;
  /**
   What joins the sentences of an English text: a space, or the underline on a line of its own.
   */
  const joinEnglish = (words.underline === '')
    ? ((words.stray === '') ? ' ' : ` ${words.stray} `)
    : `\n${words.underline}\n`;
  /**
   The original passage.
   */
  const source = `${sourceQuote}${joinOriginal}她说：「我明天还来。」`;
  /**
   The archive rendering.
   */
  const archive = `${archiveQuote}${joinEnglish}She says, "I will come again tomorrow." ${TRANSLATORS_NOTE}`;
  /**
   The claims of the accepted issue, each quoting both documents.
   */
  const claims: AdjudicatedIssue['claims'] = [
    {
      claimId: 'claim/tense',
      claim: {
        category: 'fluency/grammar',
        severity: 'minor',
        summary: `The ${words.animal}'s narration is told in the present.`,
        spans: [
          {
            side: 'source',
            nodeId: 'block/1',
            nodeHash: hashContent({ content: source, },),
            startOffset: 0,
            endOffset: sourceQuote.length,
            quotedText: sourceQuote,
          },
          {
            side: 'target',
            nodeId: 'block/1',
            nodeHash: hashContent({ content: archive, },),
            startOffset: 0,
            endOffset: archiveQuote.length,
            quotedText: archiveQuote,
          },
        ],
      },
    },
  ];
  return {
    label: words.name,
    source,
    archive,
    repair: `${repairQuote}${joinEnglish}She said, "I will come again tomorrow." ${TRANSLATORS_NOTE}`,
    neighbourSource: `隔壁的${words.originalAnimal}也睡着了。`,
    neighbourArchive: `The ${words.animal} next door fell asleep too.${joinEnglish}${TRANSLATORS_NOTE}`,
    identity: `name: ${words.originalName} = ${words.name}`,
    references: `- reference 1 https://example.org/${words.address} ("${words.name}"): ${words.name} `
      + 'had an older sister who was also a tabby.',
    name: words.name,
    wording: `那晚${words.originalAnimal}在${words.originalPlace}上睡着了`,
    prefix: `所以${words.originalAnimal}`,
    title: `${words.originalAnimal}之歌`,
    verseSource: `${words.originalAnimal}睡着了。${words.lineBreak}它梦见了鱼。`,
    lineBreak: words.lineBreak,
    verseFlat: `The ${words.animal} fell asleep. It dreamed of fish.`,
    verseBroken: `The ${words.animal} fell asleep.${words.lineBreak}It dreamed of fish.`,
    aside: `The ${words.animal} was the favourite of the whole street.${joinEnglish}${TRANSLATORS_NOTE}`,
    asideRevised: `The ${words.animal} was a favourite of the neighbours.${joinEnglish}${TRANSLATORS_NOTE}`,
    archiveQuote,
    repairQuote,
    finding: `The translation drops what the ${words.animal} promised.`,
    pictureContext: `- picture 1 (chat.webp): [left] ${words.originalName}：我明天还来。`,
    objection: `The tense shifts mid-paragraph for the ${words.animal}.`,
    claim: `fluency/grammar minor: the ${words.animal}'s narration is told in the present`,
    claims,
    issue: {
      issueId: 'issue/tense',
      status: 'accepted',
      severity: 'minor',
      claims,
      tallies: {},
    },
    envelope: {
      envelopeId: 'envelope/tense',
      startOffset: 0,
      endOffset: archiveQuote.length,
      baseText: archiveQuote,
      baseHash: hashContent({ content: archiveQuote, },),
      issueIds: ['issue/tense',],
    },
    paragraphEnvelope: {
      envelopeId: 'envelope/paragraph',
      startOffset: 0,
      endOffset: archive.length,
      baseText: archive,
      baseHash: hashContent({ content: archive, },),
      issueIds: [],
    },
    region: {
      envelopeId: 'envelope/tense',
      issueIds: ['issue/tense',],
      before: archiveQuote,
      editorAfter: repairQuote,
    },
    judgeReference: {
      seedId: 'seed/promise',
      deletedText: `${words.name} said, "I will come again tomorrow."`,
    },
  };
}

/**
 The cat on the windowsill.
 */
export const CAT_SUBJECT: PromptSubject = subjectOf({
  words: {
    originalAnimal: '小猫咪',
    originalPlace: '窗台',
    animal: 'kitten',
    place: 'windowsill',
    originalName: '咪咪',
    name: 'Mittens',
    address: 'mittens',
    underline: '',
    stray: '',
    lineBreak: '<br/>',
  },
},);

/**
 The dog on the porch.
 */
export const DOG_SUBJECT: PromptSubject = subjectOf({
  words: {
    originalAnimal: '小狗狗',
    originalPlace: '门廊',
    animal: 'puppy',
    place: 'porch',
    originalName: '旺旺',
    name: 'Biscuit',
    address: 'biscuit',
    underline: '',
    stray: '',
    lineBreak: '<br/><br/>',
  },
},);

/**
 A third subject whose every text holds a line of eleven equals signs, so a sheet that fences its blocks chooses a
 fence of twelve where the cat and the dog get five.
 */
export const FENCED_SUBJECT: PromptSubject = subjectOf({
  words: {
    originalAnimal: '小狐狸',
    originalPlace: '树桩',
    animal: 'fox cub',
    place: 'tree stump',
    originalName: '阿福',
    name: 'Biscotti',
    address: 'biscotti',
    underline: '===========',
    stray: '',
    lineBreak: '<br/>',
  },
},);

/**
 A fourth subject whose texts hold a stray run of two of the marks the consistency sheets enclose repeated wording in,
 so a sheet that chooses its marks against the shown texts chooses three where the cat gets one.
 */
export const MARKED_SUBJECT: PromptSubject = subjectOf({
  words: {
    originalAnimal: '小兔子',
    originalPlace: '草地',
    animal: 'bunny',
    place: 'meadow',
    originalName: '团团',
    name: 'Clover',
    address: 'clover',
    underline: '',
    stray: '⟦⟦',
    lineBreak: '<br/>',
  },
},);

//endregion System prompt subjects
