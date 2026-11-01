/**
 Tests that a judge is told deleting page-only content is a fault.

 WHAT THIS FILE EXISTS TO STOP, measured on the sixth consolidation bed. One
 slice's ORIGINAL is a single parenthetical note; its archive page is that note
 followed by an entire will. A consolidation rendered the note alone and the
 gate voted six to nil to ship it, deleting the will from the page.

 TWO BALLOTS ON THAT SLICE REASONED IN OPPOSITE DIRECTIONS, and both were
 correct under the rules they had. One objected that rival candidates add a
 whole document "not present in the ORIGINAL". Another objected that rival
 candidates "omit the body of the will". Nothing said which reading governs.

 THE GAP WAS NARROWER THAN IT LOOKED. The policy already said a detail the
 archive supplies is not unsupported, so keeping was licensed. But DROPPED was
 defined purely as omitting something THE CHINESE says, so deleting page-only
 content was not a fault of any named kind, and its examples were all small: a
 name, a spelled-out referent. Facing a whole will, a judge did not read them
 as covering it.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { CONTEST_POLICY, } from '../dist/final/node/index.mjs';

await describe({
  name: 'dropped covers the page',
  children: [
    it({
      name: 'TELLS a judge that omitting what only the archive carries is dropped content',
      fn: async () => {
        // The half that was missing. Keeping was already licensed; dropping was
        // not a fault of any kind the ballot named.
        expect(CONTEST_POLICY,).toContain(
          'DROPPED ALSO COVERS WHAT THE ARCHIVE CARRIES AND THE CHINESE DOES NOT SAY.',
        );
      },
    },),

    it({
      name: 'BOUNDS the rule to a silent original, not a contradicting one',
      fn: async () => {
        // Without this bound the rule would forbid replacing a badly translated
        // span, which is most of what this pipeline exists to do.
        expect(CONTEST_POLICY,).toContain(
          'Where the Chinese is SILENT rather than contradicting',
        );
      },
    },),

    it({
      name: 'SCOPES the silent-keeps sentence to apparatus and whole regions, never a narrative detail (ledger S6)',
      fn: async () => {
        // "Dropping it is a fault, keeping it is correct" once covered any
        // detail the archive carried, which contradicted the narrative bound
        // later in the same policy: an invented event read as a keep the
        // Chinese is silent about.
        expect(CONTEST_POLICY,).toContain(
          'dropping page apparatus the archive carries, or a whole region of the page',
        );
        expect(CONTEST_POLICY,).toContain(
          'a narrative detail the archive adds to a passage is not kept on the archive\'s word',
        );
      },
    },),

    it({
      name: 'NAMES a whole region as the severest case, since the small examples did not cover a will',
      fn: async () => {
        // A judge reading only "a name or a spelled-out referent" did not apply
        // the rule to an entire document region.
        expect(CONTEST_POLICY,).toContain('THE SEVEREST CASE IS A WHOLE REGION.',);
        expect(CONTEST_POLICY,).toContain('details section',);
      },
    },),

    it({
      name: 'KEEPS the older half saying archive detail is not unsupported',
      fn: async () => {
        // The two halves have to arrive together: one says keeping is allowed,
        // the other says dropping is a fault. Either alone leaves the split
        // that shipped the deletion.
        expect(CONTEST_POLICY,).toContain(
          'is NOT unsupported: keeping it is correct.',
        );
      },
    },),
  ],
},);
