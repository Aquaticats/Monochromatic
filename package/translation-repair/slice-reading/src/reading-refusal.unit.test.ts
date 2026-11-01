/**
 Tests for telling a model declining to read a picture from a model reading
 one.

 WHY THIS IS ITS OWN SCREEN. Two refusals corroborate each other. Real traffic
 on 2026-08-19 returned `There is no text visible in this image.` from one
 reader and `No legible text is visible.` from the other, and the pair stage
 marked them corroborated at 0.565 trigram overlap, because two ways of saying
 "there is nothing here" share their vocabulary exactly as two transcriptions
 of one passage do. Both had slipped the phrase list by a single word.

 SO THESE PIN THE SHAPE, not a wording. A refusal negates, names the picture
 or its text, and is a sentence rather than a passage. All three are required,
 and one case here removes each in turn to show that each is load-bearing.

 Fixtures are cat-themed invention, except the two refusals, which are the
 exact replies that caused this and carry no corpus content.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  latinWords,
  negatesSomething,
  readsAsRefusal,
  refusalReportsAbsence,
} from '@monochromatic-dev/translation-repair-markup';
import { readingMakesSense, } from '../dist/final/node/index.mjs';

/**
 What one reader answered when it would not read a picture.
 */
const QWEN_REFUSAL = '\n\nThere is no text visible in this image.';

/**
 What the other reader answered about the same picture, worded differently and
 meaning the same.
 */
const KIMI_REFUSAL = 'No legible text is visible.';

/**
 A transcription long enough that no word list is consulted, which happens to
 discuss both a picture and an absence.
 */
const LONG_TRANSCRIPTION = 'Lost: one tabby cat, answers to Mittens, last seen '
  + 'near the vet on Pearl Street. There is no photo of her winter coat, and the '
  + 'image on this poster was taken in summer, so please do not go by colour '
  + 'alone. Call 555 0134 at any hour. She is chipped and she is missed.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readsAsRefusal.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES BOTH REPLIES THAT CORROBORATED EACH OTHER IN PRODUCTION. Neither was caught by '
            + 'the phrase list they were meant to trip, one by word order and one by a word sitting '
            + 'between two others, and a corroborated refusal reaches the translator and the judge as '
            + 'what the picture says',
          fn: async () => {
            expect(readsAsRefusal({ reading: QWEN_REFUSAL, },),).toBe(true,);
            expect(readsAsRefusal({ reading: KIMI_REFUSAL, },),).toBe(true,);
          },
        },),

        it({
          name: 'ACCEPTS A LONG TRANSCRIPTION that negates and names a picture. Length is checked '
            + 'first, so a passage that happens to discuss its own medium is never put to the word '
            + 'lists at all. Without this a real transcription mentioning a photo would be discarded',
          fn: async () => {
            expect(LONG_TRANSCRIPTION.length > 160,).toBe(true,);
            expect(readsAsRefusal({ reading: LONG_TRANSCRIPTION, },),).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS A SHORT READING THAT NEGATES WITHOUT NAMING THE PICTURE, since a transcript '
            + 'may say that something is not so. Refusing on a negation alone would discard any '
            + 'notice that reports an absence',
          fn: async () => {
            expect(readsAsRefusal({ reading: 'Mittens has not been seen since Tuesday.', },),).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS A SHORT READING THAT NAMES THE PICTURE WITHOUT NEGATING, since a poster may '
            + 'caption its own photograph. Refusing on the word alone would discard every transcript '
            + 'of a caption',
          fn: async () => {
            expect(readsAsRefusal({ reading: 'Photo of Mittens, taken last June.', },),).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS `note` AS ITS OWN WORD rather than reading `not` inside it. Substring '
            + 'matching would refuse a transcript of a handwritten note about a picture, which is '
            + 'exactly the kind of thing worth reading',
          fn: async () => {
            expect(latinWords({ text: 'Note on the picture', },),).toEqual([
              'note',
              'on',
              'the',
              'picture',
            ],);
            expect(readsAsRefusal({ reading: 'Note taped to the picture frame.', },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A CONTRACTION WRITTEN WITH A TYPOGRAPHIC APOSTROPHE, which is what a model '
            + 'writing prose reaches for. Folding it onto the straight apostrophe is what lets one '
            + 'spelling in the word list cover both',
          fn: async () => {
            expect(readsAsRefusal({ reading: 'I can’t make out any text here.', },),).toBe(true,);
            expect(readsAsRefusal({ reading: 'I can\'t make out any text here.', },),).toBe(true,);
          },
        },),

        it({
          name: 'READS AN ACCENTED WORD WHOLE, so Noël holds no "no" and a reading of a Noël card is a reading; '
            + 'ASCII letters alone cut it to "no" and "l", and the screen discarded the reading (ledger B18)',
          fn: async () => {
            expect(readsAsRefusal({ reading: 'The image shows a Noël card and a sleeping cat.', },),).toBe(false,);
            expect(readsAsRefusal({ reading: 'The picture shows a sign reading Café Noe\u{0308}l.', },),).toBe(false,);
            expect(latinWords({ text: 'Café Noe\u{0308}l, Émile', },),).toEqual(['café', 'noe\u{0308}l', 'émile',],);
          },
        },),

        it({
          name: 'REFUSES A REPLY PADDED WITH NEWLINES, because the padding is not the reply. Measuring '
            + 'length before trimming would let a refusal buy its way past the bound with whitespace',
          fn: async () => {
            expect(readsAsRefusal({
              reading: `${'\n'.repeat(200,)}No text here.${'\n'.repeat(200,)}`,
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: latinWords.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPLITS ON EVERYTHING THAT IS NOT A LETTER and lowercases what it keeps, so a word '
            + 'list needs one spelling of each word and matches whole words rather than substrings',
          fn: async () => {
            expect(latinWords({ text: 'No, TEXT: visible!  (none)', },),).toEqual([
              'no',
              'text',
              'visible',
              'none',
            ],);
          },
        },),

        it({
          name: 'KEEPS A CONTRACTION WHOLE rather than splitting it at the apostrophe, since `can` '
            + 'and `t` are not the word the list carries',
          fn: async () => {
            expect(latinWords({ text: 'I can\'t and it isn’t', },),).toEqual([
              'i',
              'can\'t',
              'and',
              'it',
              'isn\'t',
            ],);
          },
        },),

        it({
          name: 'RETURNS NOTHING FOR TEXT WITH NO LATIN LETTERS, which is what a Chinese transcript '
            + 'is. Every real reading in the measured sample lands here, which is why neither word '
            + 'list can reach one',
          fn: async () => {
            expect(latinWords({ text: '喵。喵喵！', },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: `${readingMakesSense.name} shape clause`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE CLAUSE BY WHAT THE REPLY SAYS, NOT BY WHICH TEST CAUGHT IT: the reply that '
            + 'slipped the phrase list in production and one the list catches are both absence reports, '
            + 'and a decline caught either way is a refusal',
          fn: async () => {
            /**
             Verdict on the reply that slipped the phrase list in production.
             */
            const shape = readingMakesSense({ reading: QWEN_REFUSAL, },);
            expect(shape.kind,).toBe('refused',);
            if (shape.kind !== 'refused')
              throw new Error('refused by construction',);
            expect(shape.clause,).toBe('reports-no-text',);

            /**
             Verdict on a reply the phrase list catches.
             */
            const phrase = readingMakesSense({ reading: 'No text is visible here.', },);
            expect(phrase.kind,).toBe('refused',);
            if (phrase.kind !== 'refused')
              throw new Error('refused by construction',);
            expect(phrase.clause,).toBe('reports-no-text',);

            /**
             Verdict on a decline, the one reply of fifteen on the probe of
             2026-09-04 that named no text.
             */
            const decline = readingMakesSense({ reading: 'I cannot read the image.', },);
            expect(decline.kind,).toBe('refused',);
            if (decline.kind !== 'refused')
              throw new Error('refused by construction',);
            expect(decline.clause,).toBe('reads-as-refusal',);
          },
        },),
      ],
    },),

    describe({
      name: refusalReportsAbsence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS ABSENCE for every wording the probe of 2026-09-04 recorded that names text: the '
            + 'three readers on Uekawakuyuurei\'s painting, drawing and photograph and on dogesir_\'s and '
            + 'gqt\'s photographs, 14 replies of 15',
          fn: async () => {
            for (const reading of [
              'There is no visible text in this image. It is a painting of ships at sea, and I cannot discern any '
                + 'words, names, signatures, dates, or inscriptions.',
              'I cannot read any text in this image. There are no visible words, names, dates, numbers, or '
                + 'addresses.',
              'There is no text visible in this image.',
              'No text is visible in this image. It contains only two pen-and-ink drawings of fluffy cats.',
              'I cannot see any words, text, names, handles, dates, numbers, or addresses in this image.',
              'There is no text visible in this image to transcribe.',
              'I can read the image, but there is no visible text to transcribe.',
              'No legible text is visible.',
            ]) {
              expect(refusalReportsAbsence({ reading, },),).toBe(true,);
            }
          },
        },),
        it({
          name: 'REPORTS INABILITY for a decline that names no text, and for one that names text it could '
            + 'not make out, since text a model cannot read is text',
          fn: async () => {
            for (const reading of [
              'I cannot read the image.',
              'I am unable to see the image.',
              'I cannot make out the text in this image; the resolution is too low.',
              'The text is too blurry to read.',
              'Unable to process the image.',
            ]) {
              expect(refusalReportsAbsence({ reading, },),).toBe(false,);
            }
          },
        },),
        it({
          name: 'READS AN INABILITY MARKER AS THE START OF A WORD (ledger B23): an absence report that mentions a '
            + 'boatload or equality holds none, while a marker with an ending, and each prefixed form that still '
            + 'names access, marks inability',
          fn: async () => {
            expect(refusalReportsAbsence({
              reading: 'There is no visible text in this image; it shows a boatload of cats.',
            },),).toBe(true,);
            expect(refusalReportsAbsence({
              reading: 'There is no visible text here, only a poster about equality for cats.',
            },),).toBe(true,);
            for (const reading of [
              'The text in this image could not be loaded; the file seems corrupted.',
              'The text in this image is inaccessible to me.',
              'No text can be read owing to the inaccessibility of the image.',
              'The text in this image is unaccessible.',
              'I cannot see any text; please upload the picture again.',
              'No text came through, since the picture could not be downloaded.',
              'I see no text; try reloading the picture.',
              'No text: the picture is unprocessable.',
              'No text: the picture is unloadable.',
            ]) {
              expect(refusalReportsAbsence({ reading, },),).toBe(false,);
            }
          },
        },),
      ],
    },),

    describe({
      name: negatesSomething.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS A NEGATION WORD standing on its own, and none inside another word or in a hull number',
          fn: async () => {
            expect(negatesSomething({ reading: 'I can\'t.', },),).toBe(true,);
            expect(negatesSomething({ reading: 'None.', },),).toBe(true,);
            expect(negatesSomething({ reading: 'DE581', },),).toBe(false,);
            expect(negatesSomething({ reading: 'A note: knot.', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
