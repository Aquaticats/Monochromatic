/**
 Tests that an applied edit never nests a blockquote deeper than its context
 allows (TianqiChen66621 slice 16).

 WHY. An editor's patch on TianqiChen66621 slice 16 resolved six accepted
 issues and left the last quoted line as `> >`, a quote inside a quote that
 neither the original nor the archive has, and the page shipped it. Over
 every artifact, ten repair-lane slices nested a quote two levels deep where
 neither side nests, and every one shipped; the translate lane did it none.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { applyPatchOperations, } from '../dist/final/node/index.mjs';

/**
 Applies one edit over one envelope in a text.

 @param targetText - whole text the envelope sits in

 @param start - envelope start offset

 @param end - envelope end offset

 @param newText - edit's replacement

 @param sourceText - original, which licenses nesting it has; omitted runs the
   gate-exempt path the naturalness lane takes

 @returns Patch outcome

 @example
 ```ts
 const outcome = editOne({ targetText, start: 0, end: targetText.length, newText, },);
 ```
 */
function editOne(
  {
    targetText,
    start,
    end,
    newText,
    sourceText,
  }: {
    readonly targetText: string;
    readonly start: number;
    readonly end: number;
    readonly newText: string;
    readonly sourceText?: string;
  },
): ReturnType<typeof applyPatchOperations> {
  /**
   Text the envelope covers.
   */
  const baseText = targetText.slice(start, end,);

  /**
   The one envelope.
   */
  const envelope = {
    envelopeId: 'envelope/quote',
    startOffset: start,
    endOffset: end,
    baseText,
    baseHash: hashContent({ content: baseText, },),
    issueIds: ['adjudicated/quote',],
  };
  return applyPatchOperations({
    targetText,
    envelopes: [envelope,],
    operations: [{ envelopeId: envelope.envelopeId, baseHash: envelope.baseHash, newText, },],
    preservation: (sourceText === undefined)
      ? { mode: 'skip', }
      : {
        mode: 'enforce',
        licensedQuotes: new Map([[envelope.envelopeId, [baseText,],],],),
        removableQuotes: new Map([[envelope.envelopeId, [],],],),
        sourceText,
      },
  },);
}

/**
 A quote of two paragraphs, one level deep.
 */
const QUOTED = '> The cat asked for a treat.\n>\n> She promised to be good.';

await describe({
  name: 'blockquote depth of an applied edit',
  children: [
    it({
      name: 'CLAMPS a line the edit nests a level deeper than the quote it sits in, when the original does not nest',
      fn: async () => {
        /** The edit, with its last line nested. */
        const outcome = editOne({
          targetText: QUOTED,
          start: 0,
          end: QUOTED.length,
          newText: '> The cat asked for one more treat.\n>\n> > She promised to be very good.',
          sourceText: '「猫咪想再要一块零食。」\n\n「她保证会很乖。」',
        },);

        expect(outcome.patchedText,).toBe('> The cat asked for one more treat.\n>\n> She promised to be very good.',);
      },
    },),

    it({
      name: 'CLAMPS on the naturalness lane\'s gate-exempt path too, from a region starting mid-line',
      fn: async () => {
        /** Where the region starts, inside the first quoted line. */
        const start = QUOTED.indexOf('for a treat',);
        /** The edit, adding a nested line after the region's first line. */
        const outcome = editOne({
          targetText: QUOTED,
          start,
          end: QUOTED.indexOf('\n',),
          newText: 'for one more treat.\n>\n> > Then she purred.',
        },);

        expect(outcome.patchedText,).toBe('> The cat asked for one more treat.\n>\n> Then she purred.\n>\n> She promised to be good.',);
      },
    },),

    it({
      name: 'LEAVES ALONE a region\'s first line when the region starts mid-line, since its markers lie outside it and a leading > there is words',
      fn: async () => {
        /** A quoted line ending in an emoticon. */
        const text = '> The cat purred >>_<< happily.';
        /** Where the region starts, at the emoticon. */
        const start = text.indexOf('>>_<<',);
        /** The edit, keeping the emoticon. */
        const outcome = editOne({ targetText: text, start, end: text.length, newText: '>>_<< very happily.', },);

        expect(outcome.patchedText,).toBe('> The cat purred >>_<< very happily.',);
      },
    },),

    it({
      name: 'KEEPS the nesting of the line a mid-line region starts in',
      fn: async () => {
        /** A line quoted two levels deep. */
        const nested = '> > The kitten said hi.';
        /** Where the region starts, inside it. */
        const start = nested.indexOf('hi.',);
        /** The edit, adding a line at the same depth. */
        const outcome = editOne({ targetText: nested, start, end: nested.length, newText: 'hi.\n> > Then it slept.', },);

        expect(outcome.patchedText,).toBe('> > The kitten said hi.\n> > Then it slept.',);
      },
    },),

    it({
      name: 'KEEPS nesting the original has',
      fn: async () => {
        /** The edit, nesting as the original does. */
        const outcome = editOne({
          targetText: QUOTED,
          start: 0,
          end: QUOTED.length,
          newText: '> The cat asked for a treat.\n>\n> > She promised to be very good.',
          sourceText: '> 猫咪想要零食。\n>\n> > 她保证会很乖。',
        },);

        expect(outcome.patchedText,).toBe('> The cat asked for a treat.\n>\n> > She promised to be very good.',);
      },
    },),

    it({
      name: 'KEEPS nesting the replaced text already had',
      fn: async () => {
        /** A quote whose second paragraph is nested already. */
        const nested = '> The cat asked for a treat.\n>\n> > She promised to be good.';
        /** The edit, keeping the nesting. */
        const outcome = editOne({
          targetText: nested,
          start: 0,
          end: nested.length,
          newText: '> The cat asked for a treat.\n>\n> > She promised to be very good.',
        },);

        expect(outcome.patchedText,).toBe('> The cat asked for a treat.\n>\n> > She promised to be very good.',);
      },
    },),

    it({
      name: 'KEEPS a single quote level an edit adds where the archive had none, since the page may render speech as a blockquote',
      fn: async () => {
        /** Unquoted speech. */
        const plain = 'The cat asked for a treat.';
        /** The edit, quoting it. */
        const outcome = editOne({ targetText: plain, start: 0, end: plain.length, newText: '> The cat asked for one more treat.', },);

        expect(outcome.patchedText,).toBe('> The cat asked for one more treat.',);
      },
    },),
  ],
},);
