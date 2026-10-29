/**
 Tests for rendering a caught value without repeating what it refused.

 THE ABSENCE CASES ARE THE POINT, and one of them carries its own positive
 control. V8 quotes the first ten characters of the text a `JSON.parse`
 refusal was given, so the control asserts that the RAW message carries the
 fixture word before the guarded case asserts that the rendered text does not.
 Without the control, an assertion of absence would pass against a probe that
 could never have shown a difference.

 MEASURED WHILE WRITING THESE: V8 quotes only where the text stops being JSON
 near its start. A file truncated at its tail yields a positional message that
 quotes nothing, so a fixture failing late would silently test nothing.

 Fixture wording is cat-themed invention, so no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  namesWithoutQuoting,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure';
import { RunJsonUnreadableError, } from '../dist/final/node/index.mjs';

//region Refusal text tests

/**
 Word appearing nowhere else in this file, so an assertion of absence cannot
 pass by accident.

 TEN CHARACTERS EXACTLY, because that is the width of V8's quote window. A
 shorter word would be quoted whole and a longer one truncated, and a
 truncated word would be absent from the message for a reason that has nothing
 to do with the guard under test.
 */
const FIXTURE_WORD = 'Pouncewick';

/**
 Text that stops being JSON at its first character, so V8 quotes it.
 */
const UNPARSEABLE = `${FIXTURE_WORD} was never JSON`;

/**
 Raises the parse refusal these cases are about.

 @returns V8's own refusal, caught rather than raised so a case can read it

 @throws {@link Error} where the fixture parsed, which would mean it no longer
 exercises anything

 @example
 ```ts
 expect(parseRefusal().message.includes(FIXTURE_WORD,),).toBe(true,);
 ```
 */
function parseRefusal(): SyntaxError {
  try {
    JSON.parse(UNPARSEABLE,);
  }
  catch (error) {
    if (error instanceof SyntaxError)
      return error;

    throw error;
  }

  throw new Error('the fixture parsed, so it is no longer a refusal fixture',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: refusalText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CONTROL: V8 quotes the fixture word, so absence is provable',
          fn: async () => {
            /**
             V8's own message, which every absence case is measured against.
             */
            const raw = parseRefusal().message;

            expect(raw.includes(FIXTURE_WORD,),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES to repeat a parse message, which quotes the text',
          fn: async () => {
            /**
             Same refusal, rendered through the guard.
             */
            const rendered = refusalText({ error: parseRefusal(), },);

            expect(rendered.includes(FIXTURE_WORD,),).toBe(false,);
          },
        },),
        it({
          name: 'NAMES an unmarked class rather than saying what it said',
          fn: async () => {
            /**
             Ordinary refusal carrying the fixture word in its message.
             */
            const plain = new SyntaxError(UNPARSEABLE,);

            expect(refusalText({ error: plain, },),).toBe('refused by SyntaxError',);
          },
        },),
        it({
          name: 'FORWARDS a message from a class declaring it names rather than quotes',
          fn: async () => {
            /**
             Refusal whose message is the file, the class and an offset.
             */
            const named = new RunJsonUnreadableError({
              file: 'whiskerfield.json',
              failure: 'SyntaxError',
              at: 27,
            },);

            expect(refusalText({ error: named, },),)
              .toBe('could not read whiskerfield.json as JSON (SyntaxError at byte 27)',);
          },
        },),
        it({
          name: 'NAMES a thrown value that is not an Error at all',
          fn: async () => {
            expect(refusalText({ error: UNPARSEABLE, },),)
              .toBe('refused by a thrown value that is not an Error',);
          },
        },),
        it({
          name: 'SAYS IN PLAIN WORDS THAT THE ENGINE RAN OUT OF CALL STACK, since the class name alone does not tell a '
            + 'reader what ran out',
          fn: async () => {
            /**
             What the engine raises where a runaway recursion exhausts its stack.
             */
            const exhaustion = new RangeError('Maximum call stack size exceeded',);

            expect(refusalText({ error: exhaustion, },),).toBe('refused because the engine ran out of call stack space',);
          },
        },),
        it({
          name: 'KEEPS NAMING ANOTHER RangeError BY ITS CLASS, and a stack exhaustion message on another class too, so '
            + 'only the engine\'s own exhaustion is described as one',
          fn: async () => {
            expect([
              refusalText({ error: new RangeError('Invalid array length',), },),
              refusalText({ error: new Error('Maximum call stack size exceeded',), },),
            ],).toEqual(['refused by RangeError', 'refused by Error',],);
          },
        },),
      ],
    },),

    describe({
      name: namesWithoutQuoting.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS an error carrying the declaration',
          fn: async () => {
            /**
             Refusal from the guarded reader, which declares the property.
             */
            const declared = new RunJsonUnreadableError({
              file: 'whiskerfield.json',
              failure: 'ENOENT',
              at: 'unstated',
            },);

            expect(namesWithoutQuoting(declared,),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES a plain object shaped like one, which JSON can produce',
          fn: async () => {
            /**
             Everything a forged marker would carry, and not an Error.
             */
            const forged = {
              name: 'RunJsonUnreadableError',
              message: UNPARSEABLE,
              messageNamesOnly: true,
            };

            expect(namesWithoutQuoting(forged,),).toBe(false,);
          },
        },),
        it({
          name: 'REFUSES an ordinary Error, which declares nothing',
          fn: async () => {
            /**
             Error carrying the fixture word and no declaration.
             */
            const undeclared = new Error(UNPARSEABLE,);

            expect(namesWithoutQuoting(undeclared,),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Refusal text tests
