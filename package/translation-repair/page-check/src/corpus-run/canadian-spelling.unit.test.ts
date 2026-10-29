/**
 Guards the Canadian spelling pass against the audit of 2026-09-26 (ledger
 K3, K7, K9, K10, K11, H14): inflections of listed stems and forms the house
 policy names were left ("honoring", "counselor", "summarised", "judgement",
 "programmes", "my mum"), a capitalised word opening a sentence or standing
 in a title-case heading was never respelled (the heading at `Chinatsu_Suzuki/page.en.md:29`
 shipped unconverted), quoted English the original carries was respelled, an
 underscore or slash beside a word stopped it, "id" inside "idée" or "id3"
 became "ID", and a quote mark in a JSX comment or a stray backtick shielded
 the rest of the text.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { canadianizeText, } from '../../dist/final/node/index.mjs';

/**
 Rewrites one text whose original carries no English, and returns only the
 text.

 @param text - text to rewrite

 @returns Rewritten text

 @example
 ```ts
 respelled({ text: 'her favorite color', },); // 'her favourite colour'
 ```
 */
function respelled(
  { text, }: { readonly text: string; },
): string {
  return canadianizeText({
    text,
    source: '她打盹。',
  },).text;
}

/**
 Words whose spelling Canadian English shares, or whose other sense a
 respelling would damage: none may change.
 */
const NEVER_RESPELLED = [
  'humorous',
  'humoral',
  'honorary',
  'laborious',
  'coloration',
  'analyses',
  'paralyses',
  'meter',
  'check',
  'tire',
  'license',
  'practice',
  'fulfill',
  'skillful',
  'exercise',
  'advise',
  'promise',
  'surprise',
  'otherwise',
  'compromise',
  'vigorous',
  'glamorous',
  'humorist',
  'honorific',
  'vaporize',
];

await describe({
  name: 'canadianizeText spelling (ledger K3, K7, K9, K10, K11, H14, B68)',
  children: [
    it({
      name: 'RESPELLS INFLECTIONS of the listed stems and the forms the house policy names (ledger K3)',
      fn: async () => {
        expect([
          respelled({ text: 'The cat was honoring her neighbors, coloring the neighborhoods and favoring gray.', },),
          respelled({ text: 'Her counselor was counseling cats; she summarised, recognising and analysing.', },),
          respelled({ text: 'The cat watched TV programmes about enrollment, then organisation.', },),
          respelled({ text: 'The cat trusted her judgement at the harbor, with the splendor and ardor of a hunter.', },),
          respelled({ text: 'The kittens quarreled, then fueled up; signaling travelers shared a meager meal.', },),
          respelled({ text: 'The stray was marginalised, then hospitalised; the vet was symbolising hope.', },),
          respelled({ text: 'I protected my mum from the cat.', },),
        ],).toEqual([
          'The cat was honouring her neighbours, colouring the neighbourhoods and favouring grey.',
          'Her counsellor was counselling cats; she summarized, recognizing and analyzing.',
          'The cat watched TV programs about enrolment, then organization.',
          'The cat trusted her judgment at the harbour, with the splendour and ardour of a hunter.',
          'The kittens quarrelled, then fuelled up; signalling travellers shared a meagre meal.',
          'The stray was marginalized, then hospitalized; the vet was symbolizing hope.',
          'I protected my mom from the cat.',
        ],);
      },
    },),
    it({
      name: 'NEVER RESPELLS a word Canadian English spells the same way, or one with another sense (ledger K3)',
      fn: async () => {
        expect(NEVER_RESPELLED.map(function rewrite(word,): string {
          return respelled({ text: `The cat saw ${word} there.`, },);
        },),).toEqual(NEVER_RESPELLED.map(function kept(word,): string {
          return `The cat saw ${word} there.`;
        },),);
      },
    },),
    it({
      name: 'RESPELLS A CAPITALISED WORD opening a sentence, a line or a list item, and in a title-case heading, '
        + 'keeping its capital (ledger K11)',
      fn: async () => {
        expect([
          respelled({ text: 'The cat napped. Behavior was calm.', },),
          respelled({ text: 'Colors faded in the sun.', },),
          respelled({ text: '- Favorite toy: the red ball', },),
          respelled({ text: '## The Cat’s Favor', },),
          respelled({ text: '## The Cat’s Favorite Color', },),
          respelled({ text: 'Mum fed the cat.', },),
        ],).toEqual([
          'The cat napped. Behaviour was calm.',
          'Colours faded in the sun.',
          '- Favourite toy: the red ball',
          '## The Cat’s Favour',
          '## The Cat’s Favourite Colour',
          'Mom fed the cat.',
        ],);
      },
    },),
    it({
      name: 'LEAVES A CAPITALISED NAME OR TITLE: mid-sentence, after a title abbreviation, before another '
        + 'capital, in a sentence-case heading, in emphasis, and in capitals (ledger K11)',
      fn: async () => {
        /**
         Texts whose capitalised listed words are names or titles.
         */
        const kept = [
          'The cat visited the Lincoln Center and Mr. Gray.',
          'Gray said the cat napped.',
          'Center Parcs hosted the cat.',
          'The cat met Dr. Favor today.',
          'The cat met J. Favor today.',
          '## A visit to the Lincoln Center',
          '## Reading *The Color of Cats*',
          'COLOR ME CAT',
          'The cat read *THE SHINY COLORS* twice.',
        ];
        expect(kept.map(function rewrite(text,): string {
          return respelled({ text, },);
        },),).toEqual(kept,);
      },
    },),
    it({
      name: 'KEEPS ENGLISH THE ORIGINAL WRITES in English, and respells the same words where it does not (ledger K9)',
      fn: async () => {
        /**
         Slice quoting English the original carries.
         */
        const text = 'She wrote “my favorite color is gray” on the box.';
        expect([
          canadianizeText({
            text,
            source: '她在盒子上写了“my favorite color is gray”。',
          },).text,
          canadianizeText({
            text,
            source: '她在盒子上写了一句话。',
          },).text,
        ],).toEqual([
          text,
          'She wrote “my favourite colour is grey” on the box.',
        ],);
      },
    },),
    it({
      name: 'READS A WORD BESIDE EMPHASIS UNDERSCORES OR A SLASH as prose, and one inside an identifier or a path '
        + 'as no prose (ledger K10)',
      fn: async () => {
        expect([
          respelled({ text: '_her favorite color_ and *her favorite color*', },),
          respelled({ text: 'The cat’s humor/behavior was gray/white.', },),
          respelled({ text: 'The cat set my_color_var and opened assets/color/cat.png and /color today.', },),
        ],).toEqual([
          '_her favourite colour_ and *her favourite colour*',
          'The cat’s humour/behaviour was grey/white.',
          'The cat set my_color_var and opened assets/color/cat.png and /color today.',
        ],);
      },
    },),
    it({
      name: 'LEAVES "id" INSIDE A LONGER WORD, beside a digit, in a footnote label and as Freud\'s, and writes a '
        + 'bare one as ID (ledger H14)',
      fn: async () => {
        expect([
          respelled({ text: 'The cat wrote idée, ide\u0301e and idō to Sa\u0301id.', },),
          respelled({ text: 'The cat used id3 tags and 3id.', },),
          respelled({ text: 'See [^id] for the cat.', },),
          respelled({ text: 'See [id] for the cat.', },),
          respelled({ text: 'Freud split the cat\'s mind into the id, the ego and the superego.', },),
        ],).toEqual([
          'The cat wrote idée, ide\u0301e and idō to Sa\u0301id.',
          'The cat used id3 tags and 3id.',
          'See [^id] for the cat.',
          'See [ID] for the cat.',
          'Freud split the cat\'s mind into the id, the ego and the superego.',
        ],);
      },
    },),
    it({
      name: 'READS PROSE PAST a quote mark in a JSX comment or expression, a stray backtick, and a code span of two backticks '
        + '(ledger K7)',
      fn: async () => {
        expect([
          respelled({ text: '{/* cat\'s note */} Her favorite color was gray.', },),
          respelled({ text: 'The cat`s favorite toy was gray.', },),
          respelled({ text: '``code `color` here`` and favorite', },),
          respelled({ text: 'A `color\n\nand her favorite` toy.', },),
          respelled({ text: '<Cat name={cat\'s} /> Her favorite color was gray.', },),
          respelled({ text: '{/* the cat\'s note */} Her \'favorite\' color was gray.', },),
          respelled({ text: '<Paw alt="the cat\n> favorite paw" /> Her color.', },),
        ],).toEqual([
          '{/* cat\'s note */} Her favourite colour was grey.',
          'The cat`s favourite toy was grey.',
          '``code `color` here`` and favourite',
          'A `colour\n\nand her favourite` toy.',
          '<Cat name={cat\'s} /> Her favourite colour was grey.',
          '{/* the cat\'s note */} Her \'favourite\' colour was grey.',
          '<Paw alt="the cat\n> favorite paw" /> Her colour.',
        ],);
      },
    },),
    it({
      name: 'RESPELLS A CAPITALISED WORD OPENING A PARAGRAPH on the line after a heading, since a '
        + 'heading closes the paragraph it follows with no blank line between (ledger B68)',
      fn: async () => {
        expect(respelled({ text: 'The cat ate.\n# Loud fact\nColor stays on the tabby.', },),)
          .toBe('The cat ate.\n# Loud fact\nColour stays on the tabby.',);
      },
    },),
    it({
      name: 'READS A STRAY BACKTICK AS LITERAL where its only partner stands past a heading, and keeps '
        + 'the code span written there, since the heading ends the paragraph (ledger B68)',
      fn: async () => {
        expect(respelled({ text: 'A cat says `color\n# Loud heading\nand `not color` again.', },),)
          .toBe('A cat says `colour\n# Loud heading\nand `not color` again.',);
      },
    },),
    it({
      name: 'LEAVES "mum" AS WRITTEN when no word stands before it across a space, a quote mark included, '
        + 'rather than reading a possessive where none is there (`previousWord`)',
      fn: async () => {
        expect(respelled({ text: 'Keep "mum" about the cat.', },),)
          .toBe('Keep "mum" about the cat.',);
      },
    },),

    it({
      name: 'KEEPS a sentence-opening capitalized word inside underscore emphasis and RESPELLS the same '
        + 'word outside it, the underscore count deciding what stands in emphasis',
      fn: async () => {
        expect(respelled({ text: '_Coloring_ neighborhoods was fun.', },),)
          .toBe('_Coloring_ neighbourhoods was fun.',);
        expect(respelled({ text: 'Coloring neighborhoods was fun.', },),)
          .toBe('Colouring neighbourhoods was fun.',);
      },
    },),
  ],
},);
