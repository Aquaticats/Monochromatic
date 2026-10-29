/**
 Guards class ninety-two (XingZ626, 2026-09-23): a candidate that drops a
 footnote marker the original passage carries is refused before any judge
 reads it, since the note it cites is defined elsewhere on the page and the
 assembly trims that definition as an orphan once nothing cites it. The
 other direction stays open: a marker the page carries beyond the original
 is the page's own apparatus. Cat-themed invention throughout; no corpus
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
 Original passage citing a note defined elsewhere on the page.
 */
const CITING = '猫在窗台上打盹[^3]。';

/**
 Rendering that keeps the marker.
 */
const KEPT = 'The cat naps on the windowsill[^3].';

/**
 Rendering that dropped the marker.
 */
const DROPPED = 'The cat naps on the windowsill.';

await describe({
  name: 'a footnote marker the original passage carries (class ninety-two)',
  children: [
    it({
      name: 'REFUSES a candidate that drops a marker the ORIGINAL carries, naming the marker',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: CITING,
          candidateText: DROPPED,
        },),).toEqual({
          kind: 'invalid',
          findings: [
            'Your translation drops footnote 3: the ORIGINAL passage carries the marker [^3] and your translation '
              + 'does not. Keep the marker where the note is cited, since the note is defined elsewhere on the page '
              + 'and loses its place without it, and leave the definition out.',
          ],
        },);
      },
    },),
    it({
      name: 'REFUSES it where the page slice dropped the marker too, since the archive\'s omission '
        + 'is what the pass is repairing',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: CITING,
          pageText: DROPPED,
          candidateText: DROPPED,
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'REFUSES it under the full-width convention too, since the identifiers are one note',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '猫在窗台上打盹〔3〕。',
          candidateText: DROPPED,
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'ACCEPTS the marker kept, and ACCEPTS a marker the page carries beyond the ORIGINAL',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: CITING,
          candidateText: KEPT,
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: '猫在窗台上打盹。',
          pageText: 'The cat naps on the windowsill[^1].',
          candidateText: 'The cat naps on the windowsill[^1].',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
