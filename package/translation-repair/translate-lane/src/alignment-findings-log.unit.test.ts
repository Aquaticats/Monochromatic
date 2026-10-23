/**
 Tests for the alignment findings written to the run log.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { logAlignmentFindings, } from '../dist/final/node/index.mjs';

/**
 Logger forwarding everything but warnings, which it keeps.

 @param warnings - sink receiving each warn message

 @returns Logger writing its warnings to the sink

 @example
 ```ts
 const l = keeping({ warnings: [], },);
 ```
 */
function keeping({ warnings, }: { readonly warnings: string[]; },): Logger {
  return {
    ...tagged({ tag: 'alignment-findings-log-test', },),
    warn: function keep(message: string,): void {
      warnings.push(message,);
    },
  };
}

await describe({
  name: logAlignmentFindings.name,
  children: [
    it({
      name: 'WRITES a warn line for each alignment and section-pairing finding, in the order recorded',
      fn: async () => {
        /**
         Warnings written.
         */
        const warnings: string[] = [];
        logAlignmentFindings({
          findings: [
            'section-pairing paired 1 of 2 original and 1 translation sections, from 4 usable voices of 4 heard',
            'alignment structure-mismatch (source section 1: source-only (roster-unpaired))',
            'section-pairing fell back to the deterministic aligner',
          ],
          l: keeping({ warnings, },),
        },);
        expect(warnings,).toEqual([
          'alignment finding recorded: section-pairing paired 1 of 2 original and 1 translation sections, '
          + 'from 4 usable voices of 4 heard',
          'alignment finding recorded: alignment structure-mismatch (source section 1: source-only (roster-unpaired))',
          'alignment finding recorded: section-pairing fell back to the deterministic aligner',
        ],);
      },
    },),

    it({
      name: 'LEAVES OUT the block-pairing and stage findings, which the rounds that made them already log',
      fn: async () => {
        /**
         Warnings written.
         */
        const warnings: string[] = [];
        logAlignmentFindings({
          findings: [
            'block-pairing section 0 paired 2 of 2 original and 2 of 2 translation blocks',
            'stage-voice-lost (block-pairing minimax-m3)',
            'footnotes: left out of the relabel reading: slice 4 references 1 distinct note in the original',
            'page title lexicon settled 6 of 6 repeated titles from 3 voices',
          ],
          l: keeping({ warnings, },),
        },);
        expect(warnings,).toEqual([],);
      },
    },),

    it({
      name: 'WRITES nothing for a preparation that recorded no findings',
      fn: async () => {
        /**
         Warnings written.
         */
        const warnings: string[] = [];
        logAlignmentFindings({
          findings: [],
          l: keeping({ warnings, },),
        },);
        expect(warnings,).toEqual([],);
      },
    },),
  ],
},);
