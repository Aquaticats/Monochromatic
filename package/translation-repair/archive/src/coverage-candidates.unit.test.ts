/**
 Tests for listing every passage a pairing covers with nothing.

 WHAT THESE PIN is that both scales answer one question. A source section the
 matcher paired with nothing and a source block inside a paired section that
 the aligner paired with nothing are the same finding at two sizes, and the
 coverage stage asks about both. A lister that reported only one would leave
 the other class of passage unasked about, which reads as a translation that
 covers everything.

 THE POSITIVE CONTROL COMES FIRST, because every case here is a list and an
 empty list satisfies the ones that expect nothing. The first case pins that
 this lister does report something, so the empty answers mean what they say.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { listCoverageCandidates, } from '../dist/final/node/index.mjs';

/**
 Original with two sections, each carrying two paragraphs.
 */
const SOURCE_TEXT = [
  '## Sill',
  '',
  'The cat sleeps on the sill.',
  '',
  'Sun reaches it by noon.',
  '',
  '## Bowl',
  '',
  'The bowl is by the door.',
  '',
  'It is filled twice a day.',
].join('\n',);

/**
 Translation carrying the same shape, so nothing is unpaired.
 */
const MATCHING_TEXT = SOURCE_TEXT;

/**
 Translation missing the second SECTION entirely.
 */
const SECTION_MISSING_TEXT = [
  '## Sill',
  '',
  'The cat sleeps on the sill.',
  '',
  'Sun reaches it by noon.',
].join('\n',);

/**
 Translation carrying both sections, with one BLOCK gone from the first.
 */
const BLOCK_MISSING_TEXT = [
  '## Sill',
  '',
  'The cat sleeps on the sill.',
  '',
  '## Bowl',
  '',
  'The bowl is by the door.',
  '',
  'It is filled twice a day.',
].join('\n',);

/**
 Lists candidates for one pair of texts, so each case states only its shapes.

 @param sourceText - original document text

 @param targetText - translation text

 @returns Candidates the lister reported

 @example
 ```ts
 const candidates = candidatesFor({ sourceText: SOURCE_TEXT, targetText: MATCHING_TEXT, },);
 ```
 */
function candidatesFor(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): ReturnType<typeof listCoverageCandidates> {
  return listCoverageCandidates({
    source: parseDocument({ text: sourceText, },),
    target: parseDocument({ text: targetText, },),
  },);
}

await describe({
  name: listCoverageCandidates.name,
  children: [
    it({
      name:
        'POSITIVE CONTROL: a translation missing a whole section produces a candidate, so a lister that '
          + 'always returned nothing would fail here rather than pass every case that expects nothing',
      fn: async () => {
        expect(candidatesFor({
          sourceText: SOURCE_TEXT,
          targetText: SECTION_MISSING_TEXT,
        },).length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name:
        'reports NOTHING when both sides carry the same sections and the same blocks, which is the path '
          + 'most of the corpus takes: a pairing that covers everything leaves the coverage stage nothing '
          + 'to ask about',
      fn: async () => {
        expect(candidatesFor({
          sourceText: SOURCE_TEXT,
          targetText: MATCHING_TEXT,
        },),).toEqual([],);
      },
    },),

    it({
      name:
        'names the unpaired SECTION by its index and carries its whole text, heading included, since '
          + 'what the coverage stage asks about is the passage rather than its position',
      fn: async () => {
        /**
         Section-scale candidates only, which is what this case is about.
         */
        const sections = candidatesFor({
          sourceText: SOURCE_TEXT,
          targetText: SECTION_MISSING_TEXT,
        },).filter(function isSection(candidate,): boolean {
          return candidate.scale === 'section';
        },);

        expect(sections.length,).toBe(1,);
        expect(sections[0]?.sourceText.includes('## Bowl',),).toBe(true,);
        expect(sections[0]?.sourceText.includes('The bowl is by the door.',),).toBe(true,);
      },
    },),

    it({
      name:
        'names an unpaired BLOCK inside a section that DID pair, with the pair it sits in, which is the '
          + 'scale a section-only lister cannot see: both sides carry the section, and one paragraph of it '
          + 'is missing',
      fn: async () => {
        /**
         Block-scale candidates only.
         */
        const blocks = candidatesFor({
          sourceText: SOURCE_TEXT,
          targetText: BLOCK_MISSING_TEXT,
        },).filter(function isBlock(candidate,): boolean {
          return candidate.scale === 'block';
        },);

        expect(blocks.length,).toBe(1,);
        expect(blocks[0]?.sourceText,).toBe('Sun reaches it by noon.',);
      },
    },),

    it({
      name:
        'reports NOTHING for a translation with no sections at all, rather than every section of the '
          + 'original: an empty side is a document the pairing cannot speak about, and answering with the '
          + 'whole original would report a coverage question about every passage at once',
      fn: async () => {
        expect(candidatesFor({
          sourceText: SOURCE_TEXT,
          targetText: '',
        },),).toEqual([],);
      },
    },),
  ],
},);
