/**
 Tests for the lines an editor fidelity arm writes before it asks anything: the manifest, the pool, the ceiling of each
 card, the spend guard and the decision rule, in the order the read step finds them.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  ceilingLine,
  EDITOR_DAMAGE_KINDS,
  EDITOR_FIDELITY_POOL,
  editorCeilings,
  editorDecisionRuleLine,
  editorFidelityTrials,
  guardLine,
  manifestDigest,
  manifestLine,
  openingLines,
  poolLine,
} from '../../dist/final/node/index.mjs';
import {
  inventedReference,
  LOW_CARD,
} from '../editor-fidelity.test-fixture.ts';

/**
 Two pinned specifications of the pool.
 */
const SPECS = EDITOR_FIDELITY_POOL.slice(
  0,
  2,
);

await describe({
  name: openingLines.name,
  children: [
    it({
      name: 'SAYS THE MANIFEST WITH ITS DIGEST, the digest of the specifications as the plan keeps it, with the noun in '
        + 'the form for the count',
      fn: async () => {
        expect(manifestLine({ specs: SPECS, },),).toBe(
          `manifest: 2 specifications, digest ${manifestDigest({ specs: SPECS, },)}`,
        );
        expect(manifestLine({ specs: SPECS.slice(0, 1,), },).startsWith('manifest: 1 specification, digest ',),).toBe(
          true,
        );
        expect(manifestDigest({ specs: SPECS, },),).not.toBe(manifestDigest({ specs: SPECS.slice(0, 1,), },),);
      },
    },),

    it({
      name: 'WRITES THE MANIFEST, POOL, CEILING OF EACH CARD, GUARD AND RULE LINES in that order and no others, '
        + 'each by the writer the read step reads it from',
      fn: async () => {
        /**
         The trials of the invented reference.
         */
        const trials = editorFidelityTrials({
          references: [inventedReference(),],
          damageKinds: EDITOR_DAMAGE_KINDS,
        },);
        /**
         The ceiling of one card over two sheets.
         */
        const ceilings = editorCeilings({
          cardIds: [LOW_CARD,],
          sheetChars: [27_000, 27_000,],
        },);
        expect(openingLines({
          specs: SPECS,
          trials,
          ceilings,
          maxUsd: 3,
          benchSpentUsd: 1.5,
        },),).toStrictEqual([
          manifestLine({ specs: SPECS, },),
          poolLine({ trials, },),
          ...ceilings.map(function lineOf(ceiling,): string {
            return ceilingLine({ ceiling, },);
          },),
          guardLine({ maxUsd: 3, benchSpentUsd: 1.5, },),
          editorDecisionRuleLine(),
        ],);
      },
    },),
  ],
},);
