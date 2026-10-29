/**
 Tests for stating a local program's failure from a code, never from the
 rejection's own words. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  LocalProgramFailedError,
  localProgramFailureOf,
  readerFailureText,
} from '../dist/final/node/index.mjs';

/**
 Builds the rejection a failed program raises: a message that quotes a path and
 the tool's output, and the code Node attaches.

 @param code - value of the `code` property, or nothing for none

 @returns Rejection

 @example
 ```ts
 const failure = rejectionWith({ code: 'ENOSPC', },);
 ```
 */
function rejectionWith({ code, }: { readonly code?: unknown; },): Error {
  /**
   Rejection whose message quotes what must not be repeated.
   */
  const failure = new Error('Command failed: tesseract /home/whiskers/scratch/cat.png\nthe cat sat on the scanner',);
  if (code !== undefined)
    Object.defineProperty(
      failure,
      'code',
      { value: code, },
    );
  return failure;
}

await describe({
  name: 'local program failure',
  children: [
    it({
      name: 'READS THE KIND OFF THE CODE: not installed, a filesystem code, an exit code, or neither',
      fn: async () => {
        expect(localProgramFailureOf({ error: rejectionWith({ code: 'ENOENT', },), },),).toEqual({ kind: 'missing', },);
        expect(localProgramFailureOf({ error: rejectionWith({ code: 'ENOSPC', },), },),).toEqual({
          kind: 'code',
          code: 'ENOSPC',
        },);
        expect(localProgramFailureOf({ error: rejectionWith({ code: 2, },), },),).toEqual({
          kind: 'exit',
          exitCode: 2,
        },);
        expect(localProgramFailureOf({ error: rejectionWith({}), },),).toEqual({ kind: 'unnamed', },);
        expect(localProgramFailureOf({ error: 'the cat', },),).toEqual({ kind: 'unnamed', },);
      },
    },),
    it({
      name: 'NEVER REPEATS A CODE THAT IS NOT AN UPPER-CASE WORD, nor a fractional one',
      fn: async () => {
        expect(localProgramFailureOf({ error: rejectionWith({ code: 'the cat sat on the scanner', },), },),)
          .toEqual({ kind: 'unnamed', },);
        expect(localProgramFailureOf({ error: rejectionWith({ code: 1.5, },), },),).toEqual({ kind: 'unnamed', },);
      },
    },),
    it({
      name: 'REPEATS A CODE OF TWO TO 32 CHARACTERS, and never one of one character or of 33',
      fn: async () => {
        /**
         An upper-case code at the longest the reader repeats.
         */
        const longest = `E${'X'.repeat(31,)}`;
        expect([
          'EX',
          longest,
          'E',
          `${longest}X`,
        ].map((code,) => localProgramFailureOf({ error: rejectionWith({ code, },), },)),).toEqual([
          {
            kind: 'code',
            code: 'EX',
          },
          {
            kind: 'code',
            code: longest,
          },
          { kind: 'unnamed', },
          { kind: 'unnamed', },
        ],);
      },
    },),
    it({
      name: 'STATES EACH KIND IN AN ACCOUNT OF ITS OWN, quoting nothing the rejection wrote',
      fn: async () => {
        /**
         Rejection the failures are built over.
         */
        const cause = rejectionWith({ code: 'ENOSPC', },);
        /**
         Failure stated for this kind.
         */
        const notInstalled = new LocalProgramFailedError({
          program: 'tesseract',
          failure: { kind: 'missing', },
          cause,
        },);
        expect(String(notInstalled,),).toBe('LocalProgramFailedError: tesseract is not installed',);
        /**
         Failure stated for this kind.
         */
        const codeFailure = new LocalProgramFailedError({
          program: 'tesseract',
          failure: { kind: 'code', code: 'ENOSPC', },
          cause,
        },);
        expect(String(codeFailure,),).toBe('LocalProgramFailedError: tesseract failed with filesystem code ENOSPC',);
        /**
         Failure stated for this kind.
         */
        const exitFailure = new LocalProgramFailedError({
          program: 'tesseract',
          failure: { kind: 'exit', exitCode: 3, },
          cause,
        },);
        expect(String(exitFailure,),).toBe('LocalProgramFailedError: tesseract exited with code 3',);
        /**
         Failure that names no code, whose cause is kept.
         */
        const unnamed = new LocalProgramFailedError({
          program: 'tesseract',
          failure: { kind: 'unnamed', },
          cause,
        },);
        expect(String(unnamed,),).toBe(
          'LocalProgramFailedError: tesseract failed, and its failure names neither a filesystem code nor an exit code',
        );
        expect(unnamed.cause,).toBe(cause,);
      },
    },),
    it({
      name: 'RENDERS A READER FAILURE BY ITS OWN SENTENCE where marked, by class and code otherwise, never by message',
      fn: async () => {
        expect(readerFailureText({
          error: new NoProviderForModelError({ modelId: 'hf:cat', reason: 'every provider reads dry', },),
        },),).toBe('no provider can take hf:cat: every provider reads dry',);
        expect(readerFailureText({ error: rejectionWith({ code: 'ENOSPC', },), },),).toBe(
          'refused by Error with code ENOSPC',
        );
        expect(readerFailureText({ error: rejectionWith({}), },),).toBe('refused by Error',);
        expect(readerFailureText({ error: rejectionWith({ code: 1, },), },),).toBe('refused by Error',);
      },
    },),
  ],
},);
