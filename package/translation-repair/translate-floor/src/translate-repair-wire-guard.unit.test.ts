/**
 Tests for the guard that admits a repair reply onto the ballot.

 WHY A SIBLING FILE rather than more children beside
 `translate-repair-wire.unit.test.ts`: that file reads the SHEET, this one
 reads the GUARD, and `await describe` throws, so a failure in the first suite
 of a file aborts the rest before it runs.

 WHAT WAS MEASURED. On 2026-08-25, making the guard ACCEPT a reply whose
 `resolution` is not a string at all failed no test in this package. Nothing
 said the field is required to be one, so a reply carrying a number, an object
 or nothing there would have reached the union check with a value it cannot
 name, and the guard's whole job is to keep such a reply off the ballot.

 EVERY REFUSAL IS PINNED SEPARATELY, since they are the guard: the union is
 enforced here rather than in the type, per the module's own note, and a
 revision carrying no translation is not a revision.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isTranslateRepairWire, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Replacement wording a revising model would send back.
 */
const REVISED_NAP = 'Mittens naps on the windowsill.[^1]';

/**
 Builds a reply, defaulting to a well-formed revision.

 @param over - fields this case replaces

 @returns Reply as a model would send it, before the guard reads it

 @example
 ```ts
 const reply = replyWith({ resolution: 'unable', },);
 ```
 */
function replyWith(over: Record<string, unknown> = {},): unknown {
  return {
    resolution: 'revised',
    translation: REVISED_NAP,
    translatorNote: 'restored the footnote marker',
    ...over,
  };
}

//endregion Fixtures

await describe({
  name: isTranslateRepairWire.name,
  children: [
    it({
      name: 'REFUSES a reply whose resolution is not a string, since the union check that follows can '
        + 'only compare names and a number or an object there would reach it as a value it cannot name',
      fn: async () => {
        /**
         Reply answering with a number where an answer belongs.
         */
        const numbered = replyWith({ resolution: 2, },);

        /**
         Reply answering with the shape of an answer rather than an answer.
         */
        const wrapped = replyWith({ resolution: { kind: 'revised', }, },);

        /**
         Reply that never named an answer at all.
         */
        const silent = replyWith({ resolution: undefined, },);

        expect(isTranslateRepairWire(numbered,),).toBe(false,);
        expect(isTranslateRepairWire(wrapped,),).toBe(false,);
        expect(isTranslateRepairWire(silent,),).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES a resolution outside the three answers, which is the union the flat schema cannot '
        + 'enforce by itself',
      fn: async () => {
        /**
         Reply naming a fourth answer nobody offered.
         */
        const invented = replyWith({ resolution: 'rewritten', },);

        /**
         Reply naming an empty answer, which is a string and not a name.
         */
        const blank = replyWith({ resolution: '', },);

        expect(isTranslateRepairWire(invented,),).toBe(false,);
        expect(isTranslateRepairWire(blank,),).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES a revision carrying no translation, since admitting one would put an empty '
        + 'candidate on the ballot in the name of repairing it',
      fn: async () => {
        /**
         Revision whose replacement wording is absent.
         */
        const empty = replyWith({ translation: '', },);

        /**
         Revision whose replacement wording is only whitespace, which reads as
         text and ships as nothing.
         */
        const spaces = replyWith({ translation: '   \n  ', },);

        /**
         Revision whose replacement wording is a zero-width space, which
         trim() keeps and the intake fold removes (ledger B40).
         */
        const zeroWidth = replyWith({ translation: '\u{200B}', },);

        /**
         Revision whose replacement wording is a Hangul filler, which nothing
         folds and nothing shows.
         */
        const filler = replyWith({ translation: '\u{3164}', },);

        expect(isTranslateRepairWire(empty,),).toBe(false,);
        expect(isTranslateRepairWire(spaces,),).toBe(false,);
        expect(isTranslateRepairWire(zeroWidth,),).toBe(false,);
        expect(isTranslateRepairWire(filler,),).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES a non-record and a reply missing either text field, so a stream that returned a '
        + 'bare string or a half-built object never reaches the ballot',
      fn: async () => {
        /**
         Well-formed reply put inside an array, which is not a record.
         */
        const listed = [replyWith(),];

        /**
         Reply whose replacement wording is a number.
         */
        const numberedText = replyWith({ translation: 2, },);

        /**
         Reply that said nothing about why.
         */
        const unexplained = replyWith({ translatorNote: undefined, },);

        expect(isTranslateRepairWire('revised',),).toBe(false,);
        expect(isTranslateRepairWire(null,),).toBe(false,);
        expect(isTranslateRepairWire(listed,),).toBe(false,);
        expect(isTranslateRepairWire(numberedText,),).toBe(false,);
        expect(isTranslateRepairWire(unexplained,),).toBe(false,);
      },
    },),
    it({
      name: 'ACCEPTS the two answers that carry no translation, because a model reporting that it '
        + 'cannot fix a finding, or that the finding is about the passage rather than its work, is '
        + 'making the report this turn exists to collect',
      fn: async () => {
        /**
         Reply reporting that the findings cannot be fixed.
         */
        const unable = replyWith({
          resolution: 'unable',
          translation: '',
        },);

        /**
         Reply reporting that the finding is about the passage.
         */
        const asIntended = replyWith({
          resolution: 'as-intended',
          translation: '',
        },);

        expect(isTranslateRepairWire(unable,),).toBe(true,);
        expect(isTranslateRepairWire(asIntended,),).toBe(true,);
      },
    },),
    it({
      name: 'ACCEPTS a revision that carries wording, which is the only shape that replaces a candidate',
      fn: async () => {
        /**
         Revision carrying a replacement.
         */
        const revised = replyWith();

        expect(isTranslateRepairWire(revised,),).toBe(true,);
      },
    },),
  ],
},);
