/**
 Tests for the ways the pass's footnote relabel leaves the archive's labels or
 definitions standing, each with the finding the artifact carries.

 WHAT THIS FILE PINS (ledger T8, ninth batch): evidence that disagrees with
 itself, read off the definitions the roster paired or off the paired slices,
 withholds the relabel and names the disagreement; and definitions a
 paragraph sits among keep their order, said as a finding. The widening to
 the slices has its own file (`pass-footnote-relabel-widen.unit.test.ts`).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { relabelArchiveFootnotes, } from '../../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

//region Fixtures

/**
 Original: two notes, the cat's spot first, the sparrow second.
 */
const SOURCE_TEXT = '猫猫[^1]睡在窗台上。\n\n鸟儿[^2]在外面唱歌。\n\n'
  + '[^1]: 它最喜欢的位置。\n\n[^2]: 一只麻雀。\n';

/**
 Slice budget at which each paragraph is its own slice, so the body
 paragraphs pair one to one.
 */
const SLICE_BUDGET = 40;

//endregion Fixtures

await describe({
  name: relabelArchiveFootnotes.name,
  children: [
    it({
      name: 'LEAVES THE ARCHIVE\'S LABELS STANDING where the definitions the roster paired disagree, naming the '
        + 'pair that contradicts an earlier one',
      fn: async () => {
        /**
         Archive under the original's labels.
         */
        const archiveText = 'The cat[^1] slept on the sill.\n\nThe bird[^2] sang outside.\n\n'
          + '[^1]: Her favourite spot.\n\n[^2]: A sparrow.\n';
        /**
         Lines the relabel logged.
         */
        const messages: string[] = [];

        expect(relabelArchiveFootnotes({
          entryId: 'invented-definitions-disagree',
          slices: prepareDocumentPair({
            sourceText: SOURCE_TEXT,
            targetText: archiveText,
            sliceCharBudget: SLICE_BUDGET,
          },).slices,
          // The archive's `[^1]` paired with the original's `[^1]`, then with
          // its `[^2]`: one archive note cannot be two of the original's.
          definitionPairs: [
            {
              sourceLabel: '1',
              targetLabel: '1',
            },
            {
              sourceLabel: '2',
              targetLabel: '1',
            },
          ],
          sourceText: SOURCE_TEXT,
          archiveText,
          l: capturingLogger({ messages, },),
        },),).toEqual({
          archiveText,
          changed: false,
          findings: [
            'footnotes: archive labels stand, since the definitions the roster paired disagree: definition pair 1 '
              + 'maps archive [^1] to original [^2], where definition pair 0 mapped that archive label to original [^1]',
          ],
          withheld: 'correspondence',
        },);
        expect(messages,).toContain(
          `[${relabelArchiveFootnotes.name}] FOOTNOTES entry=invented-definitions-disagree archive labels stand, `
            + 'since the definitions the roster paired disagree: definition pair 1 maps archive [^1] to original '
            + '[^2], where definition pair 0 mapped that archive label to original [^1]',
        );
      },
    },),
    it({
      name: 'LEAVES THE ARCHIVE\'S LABELS STANDING where the roster paired no definitions and the paired slices '
        + 'disagree, naming the slice that contradicts an earlier one',
      fn: async () => {
        /**
         Archive giving both of the original's notes the one label `[^a]`.
         */
        const archiveText = 'The cat[^a] slept on the sill.\n\nThe bird[^a] sang outside.\n\n[^a]: A note.\n';
        /**
         Preparation over the pair.
         */
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: archiveText,
          sliceCharBudget: SLICE_BUDGET,
        },);
        /**
         The slice the bird's paragraph sits in, whose `[^a]` meets the
         original's `[^2]` after the cat's met its `[^1]`.
         */
        const birdSlice = prepared.slices.findIndex(function carriesBird(slice,): boolean {
          return slice.target
            .text
            .includes('The bird',);
        },);
        /**
         The slice the cat's paragraph sits in, the earlier claim the detail
         names.
         */
        const catSlice = prepared.slices.findIndex(function carriesCat(slice,): boolean {
          return slice.target
            .text
            .includes('The cat',);
        },);

        expect(catSlice,).toBeGreaterThanOrEqual(0,);
        expect(birdSlice,).toBeGreaterThan(catSlice,);
        expect(relabelArchiveFootnotes({
          entryId: 'invented-slices-disagree',
          slices: prepared.slices,
          definitionPairs: [],
          sourceText: SOURCE_TEXT,
          archiveText,
          l: capturingLogger({ messages: [], },),
        },),).toEqual({
          archiveText,
          changed: false,
          findings: [
            `footnotes: archive labels stand, since the paired slices disagree: slice ${String(birdSlice,)} maps `
              + `archive [^a] to original [^2], where slice ${String(catSlice,)} mapped that archive label to original [^1]`,
          ],
          withheld: 'correspondence',
        },);
      },
    },),
    it({
      name: 'KEEPS THE DEFINITIONS\' ORDER where a paragraph sits among them, and says so as a finding, rather than '
        + 'moving blocks across it',
      fn: async () => {
        /**
         Archive under the original's labels, its definitions out of the
         original's order with a closing line between them.
         */
        const archiveText = 'The cat[^1] slept on the sill.\n\nThe bird[^2] sang outside.\n\n'
          + '[^2]: A sparrow.\n\nThe end of the afternoon.\n\n[^1]: Her favourite spot.\n';
        /**
         Lines the relabel logged.
         */
        const messages: string[] = [];

        expect(relabelArchiveFootnotes({
          entryId: 'invented-interleaved',
          slices: prepareDocumentPair({
            sourceText: SOURCE_TEXT,
            targetText: archiveText,
            sliceCharBudget: SLICE_BUDGET,
          },).slices,
          definitionPairs: [],
          sourceText: SOURCE_TEXT,
          archiveText,
          l: capturingLogger({ messages, },),
        },),).toEqual({
          archiveText,
          changed: false,
          findings: [
            'footnotes: archive definitions stand: the footnote definitions are interleaved with other blocks, so '
              + 'they keep their order',
          ],
        },);
        expect(messages,).toContain(
          `[${relabelArchiveFootnotes.name}] FOOTNOTES entry=invented-interleaved definitions stand: the footnote `
            + 'definitions are interleaved with other blocks, so they keep their order',
        );
      },
    },),
    it({
      name: 'RELABELS THE ARCHIVE AND LEAVES ITS DEFINITIONS STANDING where moving them would change how the page '
        + 'parses them (ledger T8, eighteenth batch): the refused move once threw, and the whole relabel was kept '
        + 'back with it, so the markers stayed under the archive\'s labels',
      fn: async () => {
        /**
         Archive numbering its notes the other way, its note on the sill
         written as a fence whose lines stand outside the definition, so a
         paragraph definition moved after it would take them as its own.
         */
        const archiveText = 'The cat[^2] slept on the sill.\n\nThe bird[^1] sang outside.\n\n'
          + '[^1]: A sparrow.\n\n[^2]: ```\nsill\n```\n';

        expect(relabelArchiveFootnotes({
          entryId: 'invented-unmovable',
          slices: prepareDocumentPair({
            sourceText: SOURCE_TEXT,
            targetText: archiveText,
            sliceCharBudget: SLICE_BUDGET,
          },).slices,
          definitionPairs: [
            {
              sourceLabel: '1',
              targetLabel: '2',
            },
            {
              sourceLabel: '2',
              targetLabel: '1',
            },
          ],
          sourceText: SOURCE_TEXT,
          archiveText,
          l: capturingLogger({ messages: [], },),
        },),).toEqual({
          archiveText: 'The cat[^1] slept on the sill.\n\nThe bird[^2] sang outside.\n\n'
            + '[^2]: A sparrow.\n\n[^1]: ```\nsill\n```\n',
          changed: true,
          findings: [
            'footnotes: archive relabelled [^2]->[^1], [^1]->[^2] off the definitions the roster paired to follow '
              + 'the original\'s labels',
            'footnotes: archive definitions stand: moving the footnote definitions would change how the page '
              + 'parses them, so they keep their order',
          ],
        },);
      },
    },),
  ],
},);
