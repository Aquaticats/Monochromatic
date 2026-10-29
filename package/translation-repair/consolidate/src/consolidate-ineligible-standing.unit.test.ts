/**
 Tests for the one-line rendering of a deterministic verdict on a standing
 text, written for the run log.

 WHAT THESE PIN: each of the verdict's three kinds renders on its own line,
 so a consolidation warning names the rule that refused a standing rather
 than saying only that it "fails publication eligibility", which is what
 the 2026-09-04 luxuanwen3 log said about a link destination the archive
 had rewritten.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '@monochromatic-dev/translation-repair-translate-floor';
import { describeStandingVerdict, } from '../dist/final/node/index.mjs';

await describe({
  name: describeStandingVerdict.name,
  children: [
    it({
      name: 'NAMES A PASS in one line',
      fn: async () => {
        expect(describeStandingVerdict({
          validation: {
            kind: 'valid',
            pageGrammar: 'strict',
          },
        },),).toBe('passes the deterministic publication rule',);
      },
    },),

    it({
      name: 'JOINS THE FINDINGS of a refusal',
      fn: async () => {
        expect(describeStandingVerdict({
          validation: {
            kind: 'invalid',
            findings: [
              'The ORIGINAL carries link-url https://cats.test/a and your translation does not.',
              'Your translation carries link-url https://cats.test/b and the ORIGINAL does not.',
            ],
          },
        },),).toBe(
          'The ORIGINAL carries link-url https://cats.test/a and your translation does not. '
            + 'Your translation carries link-url https://cats.test/b and the ORIGINAL does not.',
        );
      },
    },),

    it({
      name: 'NAMES THE REASON no comparison was possible',
      fn: async () => {
        expect(describeStandingVerdict({
          validation: {
            kind: 'unknown',
            detail: 'original could not be read: cats',
          },
        },),).toBe('no comparison was possible: original could not be read: cats',);
      },
    },),

    it({
      name: 'RENDERS A LIVE VERDICT on a candidate that carries neither rendering of a link',
      fn: async () => {
        /**
         Verdict on a rendering that dropped the link where the original
         carries one destination and the page another.
         */
        const validation = validateTranslatedSlice({
          sourceText: '她的头像由[画师](https://twitter.com/cat)绘制。',
          candidateText: 'Her avatar was drawn by the artist.',
          pageText: 'Her avatar was drawn by [the artist](https://x.com/cat).',
        },);
        /**
         Rendered line for the log.
         */
        const line = describeStandingVerdict({ validation, },);
        expect(validation.kind,).toBe('invalid',);
        expect(line,).toContain('must carry exactly 1 of these, taken from either side; it carries 0',);
      },
    },),
  ],
},);
