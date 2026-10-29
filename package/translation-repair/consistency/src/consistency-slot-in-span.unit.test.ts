/**
 Tests for the slot a frame's ballots agree on inside the span a place was located by: given as the span holds it with its
 offset when a strict majority gave it beside the agreed words and it stands once inside the span, and none when no majority
 gave one, when it shows a reader nothing, when it stands nowhere or twice in the span, when it is all of the span, or when it
 cuts markup. Each refusal stands beside a twin that differs from it in that one respect and gives its slot.

 Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ConsistencyVoice,
  foldedWordsOf,
  quotesByLabelOf,
  slotInSpan,
} from '../dist/final/node/index.mjs';

/**
 The span of the first place, as its English holds it, with a curly apostrophe.
 */
const SPAN = 'So she was a girl, wasn’t she';

/**
 Ballots that quote the span for the first place, each beside the slot given in its turn, as a list of zero or one.

 @param span - quoted words

 @param slots - each ballot's slot as a list of zero or one, in seat order

 @returns Ballots

 @example
 ```ts
 const voices = ballotsGiving({ span: SPAN, slots: [['a girl',], ['a girl',],], },);
 ```
 */
function ballotsGiving(
  {
    span,
    slots,
  }: {
    readonly span: string;
    readonly slots: readonly (readonly string[])[];
  },
): readonly ConsistencyVoice[] {
  return slots.map(function ballotOf(slot, index,): ConsistencyVoice {
    /**
     The slot this ballot gave, absent when it gave none.
     */
    const [given,] = slot;
    return {
      seat: `seat ${String(index,)}`,
      reason: 'It words the first place best.',
      quotes: [{ label: 'OCCURRENCE 1', quote: span, ...((given === undefined) ? {} : { slot: given, }), },],
      choice: 'OCCURRENCE 1',
      keep: [],
    };
  },);
}

/**
 The slot of the first place in a span whose ballots gave these slots, the words agreed being the span's own in the fold.

 @param span - span as its English holds it

 @param slots - each ballot's slot as a list of zero or one, in seat order

 @returns What `slotInSpan` gives

 @example
 ```ts
 const found = slotOf({ span: SPAN, slots: [['a girl',], ['a girl',],], },);
 ```
 */
function slotOf(
  {
    span,
    slots,
  }: {
    readonly span: string;
    readonly slots: readonly (readonly string[])[];
  },
): readonly unknown[] {
  return slotInSpan({
    voices: ballotsGiving({ span, slots, },),
    label: 'OCCURRENCE 1',
    words: foldedWordsOf({ text: span, },),
    span,
  },);
}

await describe({
  name: slotInSpan.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'GIVES THE SLOT A STRICT MAJORITY GAVE BESIDE THE AGREED WORDS, as the span holds it and with its offset, a ballot '
        + 'that gave none counting against it, and reads the quotes the same when the caller hands them in',
      fn: async () => {
        /**
         Three ballots, two of which gave the slot.
         */
        const voices = ballotsGiving({ span: SPAN, slots: [['a girl',], ['a girl',], [],], },);
        expect({
          read: slotInSpan({ voices, label: 'OCCURRENCE 1', words: foldedWordsOf({ text: SPAN, },), span: SPAN, },),
          handed: slotInSpan({
            voices,
            label: 'OCCURRENCE 1',
            words: foldedWordsOf({ text: SPAN, },),
            span: SPAN,
            quotesByLabel: quotesByLabelOf({ voices, },),
          },),
        },).toEqual({
          read: [{ slot: 'a girl', slotStart: 11, },],
          handed: [{ slot: 'a girl', slotStart: 11, },],
        },);
      },
    },),
    it({
      name: 'GIVES NO SLOT WHEN NO STRICT MAJORITY OF THE BALLOTS GAVE ONE',
      fn: async () => {
        expect(slotOf({ span: SPAN, slots: [['a girl',], ['a boy',], [],], },),).toEqual([],);
      },
    },),
    it({
      name: 'GIVES NO SLOT THAT SHOWS A READER NOTHING, as a zero-width space folds to no words',
      fn: async () => {
        expect(slotOf({ span: 'So she was\u{200B} a girl', slots: [['\u{200B}',], ['\u{200B}',],], },),).toEqual([],);
      },
    },),
    it({
      name: 'GIVES NO SLOT THAT STANDS NOWHERE OR MORE THAN ONCE IN THE SPAN, beside a slot standing once in the same span',
      fn: async () => {
        expect({
          nowhere: slotOf({ span: SPAN, slots: [['a dog',], ['a dog',],], },),
          twice: slotOf({ span: 'the cat and the cat naps', slots: [['the cat',], ['the cat',],], },),
          once: slotOf({ span: 'the cat and the cat naps', slots: [['naps',], ['naps',],], },),
        },).toEqual({
          nowhere: [],
          twice: [],
          once: [{ slot: 'naps', slotStart: 20, },],
        },);
      },
    },),
    it({
      name: 'GIVES NO SLOT THAT IS ALL OF THE SPAN, beside the same slot in a longer span',
      fn: async () => {
        expect({
          whole: slotOf({ span: 'a girl', slots: [['a girl',], ['a girl',],], },),
          part: slotOf({ span: 'a girl naps', slots: [['a girl',], ['a girl',],], },),
        },).toEqual({
          whole: [],
          part: [{ slot: 'a girl', slotStart: 0, },],
        },);
      },
    },),
    it({
      name: 'READS THE SLOT WITH THE DEFINITIONS IT IS GIVEN, as it reads them from a page: a shortcut reference in the span whose '
        + 'definition stands elsewhere makes `a[b]c` one word, so the slot `b` is no stand, where the span read alone has one',
      fn: async () => {
        /**
         The span, with a reference in it.
         */
        const span = 'a[b]c';
        /**
         The definition of the reference, as the page holds it in another slice.
         */
        const definitions = '[b]: https://x.example/b';
        /**
         Three ballots that gave the slot.
         */
        const voices = ballotsGiving({ span, slots: [['b',], ['b',], ['b',],], },);
        /**
         The arguments every reading shares.
         */
        const shared = { voices, label: 'OCCURRENCE 1', words: foldedWordsOf({ text: span, },), span, };
        expect({
          alone: slotInSpan(shared,),
          given: slotInSpan({ ...shared, definitions, },),
          pageRead: slotInSpan({ ...shared, page: { text: `${span}\n\n${definitions}`, offset: 0, }, },),
          unrelated: slotInSpan({ ...shared, definitions: '[z]: https://x.example/z', },),
        },).toEqual({
          alone: [{ slot: 'b', slotStart: 2, },],
          given: [],
          pageRead: [],
          unrelated: [{ slot: 'b', slotStart: 2, },],
        },);
      },
    },),
    it({
      name: 'READS THE SPAN WITH THE EARLIER READINGS A RECOUNT NAMES, those of an artifact written before joints and line endings '
        + 'counted: a slot given beside a quote that holds the words before a joint counts under the edges of letters and not by '
        + 'default, and a slot beside inline code padded by line endings is a stand under the padding of spaces and not by default',
      fn: async () => {
        /**
         Ballots that quote a longer text with the slot `big`, and one that quoted the words alone.
         */
        const voices: readonly ConsistencyVoice[] = [
          { quote: 'big cat\'s bowl', slot: 'big', },
          { quote: 'big cat\'s bowl', slot: 'big', },
          { quote: 'big cat', },
        ].map(function ballotOf({ quote, slot, }, index,): ConsistencyVoice {
          return {
            seat: `seat ${String(index,)}`,
            reason: 'It words the first place best.',
            quotes: [{ label: 'OCCURRENCE 1', quote, ...((slot === undefined) ? {} : { slot, }), },],
            choice: 'OCCURRENCE 1',
            keep: [],
          };
        },);
        /**
         The span of code padded by line endings, whose ballots all gave the slot `kit`.
         */
        const padded = 'The kit`\ntens\n` ate';
        expect({
          byDefault: slotInSpan({ voices, label: 'OCCURRENCE 1', words: 'big cat', span: 'big cat', },),
          letters: slotInSpan({ voices, label: 'OCCURRENCE 1', words: 'big cat', span: 'big cat', edges: 'letters', },),
          codeByDefault: slotOf({ span: padded, slots: [['kit',], ['kit',],], },),
          codeBySpaces: slotInSpan({
            voices: ballotsGiving({ span: padded, slots: [['kit',], ['kit',],], },),
            label: 'OCCURRENCE 1',
            words: foldedWordsOf({ text: padded, },),
            span: padded,
            codePadding: 'spaces',
          },),
        },).toEqual({
          byDefault: [],
          letters: [{ slot: 'big', slotStart: 0, },],
          codeByDefault: [],
          codeBySpaces: [{ slot: 'kit', slotStart: 4, },],
        },);
      },
    },),
    it({
      name: 'READS A SPAN ALONE AS DEFINING NOTHING, so an empty reference `[][r]` after the slot is text where no page and no '
        + 'definitions are given and a link where they define it, and the slot is found in the first and not in the second',
      fn: async () => {
        /**
         The span, whose slot is followed by an empty reference and a word.
         */
        const span = 'a kit[][r]tens';
        /**
         Three ballots that gave the slot.
         */
        const voices = ballotsGiving({ span, slots: [['kit',], ['kit',], ['kit',],], },);
        /**
         The arguments every reading shares.
         */
        const shared = { voices, label: 'OCCURRENCE 1', words: foldedWordsOf({ text: span, },), span, };
        expect({
          alone: slotInSpan(shared,),
          defined: slotInSpan({ ...shared, definitions: '[r]: https://x.example/r', },),
          pageDefines: slotInSpan({ ...shared, page: { text: `${span}\n\n[r]: https://x.example/r`, offset: 0, }, },),
          pageDefinesOther: slotInSpan({ ...shared, page: { text: `${span}\n\n[x]: https://x.example/x`, offset: 0, }, },),
        },).toEqual({
          alone: [{ slot: 'kit', slotStart: 2, },],
          defined: [],
          pageDefines: [],
          pageDefinesOther: [{ slot: 'kit', slotStart: 2, },],
        },);
      },
    },),
    it({
      name: 'GIVES THE SLOT AT ITS WORDS ALONE, leaving an apostrophe beside it outside the slot, as the span already holds the '
        + 'apostrophes of the located words',
      fn: async () => {
        expect({
          opening: slotOf({ span: '’round the cat', slots: [['round',], ['round',],], },),
          closingOfPlural: slotOf({ span: 'the parents’ cat', slots: [['parents',], ['parents',],], },),
          closingOfCats: slotOf({ span: 'the cats’ bowl', slots: [['cats',], ['cats',],], },),
        },).toEqual({
          opening: [{ slot: 'round', slotStart: 1, },],
          closingOfPlural: [{ slot: 'parents', slotStart: 4, },],
          closingOfCats: [{ slot: 'cats', slotStart: 4, },],
        },);
      },
    },),
    it({
      name: 'GIVES NO SLOT THAT CUTS MARKUP, beside a slot of the same span that cuts none',
      fn: async () => {
        expect({
          cutting: slotOf({ span: 'a *black cat* naps', slots: [['cat* naps',], ['cat* naps',],], },),
          clear: slotOf({ span: 'a *black cat* naps', slots: [['naps',], ['naps',],], },),
        },).toEqual({
          cutting: [],
          clear: [{ slot: 'naps', slotStart: 14, },],
        },);
      },
    },),
  ],
},);
