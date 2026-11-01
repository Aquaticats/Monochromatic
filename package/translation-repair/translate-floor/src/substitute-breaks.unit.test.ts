/**
 Guards class forty-two (Mio25 slice 16, 2026-09-17): a verse the page
 rendered as a substitute paragraph still owes the original's explicit
 breaks, and a candidate that repeats a Han-carrying original untranslated is
 refused before any judge reads it. Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Single-block verse with two authored breaks.
 */
const VERSE = '> 猫醒了。  \n> 鸟唱了。  \n> 风停了。';

/**
 Page substitute: a farewell paragraph where the verse stands.
 */
const SUBSTITUTE = 'Sleep well, cat.';

/**
 Verse rendering whose physical lines carry no rendered break.
 */
const FLAT = '> The cat wakes.\n> The bird sings.\n> The wind stops.';

/**
 Verse rendering keeping both breaks.
 */
const BROKEN = '> The cat wakes.  \n> The bird sings.  \n> The wind stops.';

await describe({
  name: 'explicit breaks under a substitute page block (class forty-two)',
  children: [
    it({
      name: 'REFUSES a flat verse under the page paragraph where the page has no quote',
      fn: async () => {
        const result = validateTranslatedSlice({
          sourceText: VERSE,
          pageText: SUBSTITUTE,
          candidateText: `${SUBSTITUTE}\n\n${FLAT}`,
        },);
        expect(result.kind,).toBe('invalid',);
        if (result.kind === 'invalid')
          expect(result.findings.join('\n',),).toContain('explicit line break',);
      },
    },),
    it({
      name: 'ACCEPTS the verse with its breaks under the page paragraph',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: VERSE,
          pageText: SUBSTITUTE,
          candidateText: `${SUBSTITUTE}\n\n${BROKEN}`,
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'LEAVES a quote the page itself rendered without breaks to the page floor',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: VERSE,
          pageText: '> The cat wakes.\n> The bird sings.\n> The wind stops.',
          candidateText: FLAT,
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'REFUSES a candidate that repeats a Han-carrying original untranslated',
      fn: async () => {
        const result = validateTranslatedSlice({
          sourceText: VERSE,
          candidateText: VERSE,
        },);
        expect(result.kind,).toBe('invalid',);
        if (result.kind === 'invalid')
          expect(result.findings.join('\n',),).toContain('untranslated',);
      },
    },),
    it({
      name: 'ACCEPTS an original with nothing to translate returned as it stands',
      fn: async () => {
        const link = 'https://example.invalid/cat';
        expect(validateTranslatedSlice({
          sourceText: link,
          candidateText: link,
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
