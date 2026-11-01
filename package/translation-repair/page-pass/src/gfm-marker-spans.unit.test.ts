/**
 Tests for the footnote marker reader the relabel rewrite checks each supplied
 label with, read through that rewrite: a label is one marker only where its
 own backslashes leave the closing bracket a closing bracket.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { FootnoteRewriteError, } from '@monochromatic-dev/translation-repair-document-structure';
import { applyFootnoteRelabel, } from '../dist/final/node/index.mjs';

/**
 Page with one reference and its definition under the label `1`.
 */
const PAGE = 'A cat[^1] naps.\n\n[^1]: The cat.\n';

await describe({
  name: applyFootnoteRelabel.name,
  children: [
    it({
      name: 'READS A LABEL ENDING IN AN ESCAPED BACKSLASH as one marker, since the escaped backslash leaves the '
        + 'closing bracket to close it, and REFUSES one ending in a lone backslash, which escapes that bracket',
      fn: async () => {
        expect(applyFootnoteRelabel({
          text: PAGE,
          map: [{ from: '1', to: String.raw`a\\`, },],
        },),).toBe('A cat[^a\\\\] naps.\n\n[^a\\\\]: The cat.\n',);
        /**
         What the rewrite refused a label ending in one backslash with.
         */
        const refusal = caught(function relabelToLoneBackslash(): unknown {
          return applyFootnoteRelabel({
            text: PAGE,
            map: [{ from: '1', to: 'a\\', },],
          },);
        },);
        expect(refusal,).toBeInstanceOf(FootnoteRewriteError,);
        expect(String(refusal,),).toBe(
          'FootnoteRewriteError: footnote rewrite: a supplied label does not encode exactly one GFM identifier; '
            + 'supply a valid label before retrying',
        );
      },
    },),
  ],
},);
