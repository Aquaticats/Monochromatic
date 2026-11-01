import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { decoyCut, } from '../../dist/final/node/index.mjs';

//region Coverage control decoy cut
// The cut that must not change the roster's answer.
//
// The absence control's whole claim rests on the two cuts being comparable: the
// same document, the same passage, the same number of characters, differing
// only in whether the deleted text is what the roster pointed at. A decoy that
// silently overlapped the anchored spans would delete the rendering too and
// report the wire as trigger-happy when it was reading correctly.
//
// Fixtures are invented cat prose, never corpus text.

await describe({
  name: decoyCut.name,
  children: [
    it({
      name: 'TAKES A CUT OF THE REQUESTED SIZE clear of the anchored span, which is what makes it '
        + 'comparable with the targeted cut it is measured against',
      fn: async function takesAClearCut() {
        expect(
          decoyCut({
            text: 'The tabby slept by the window all afternoon.',
            avoid: ['tabby',],
            chars: 10,
          },),
        ).toEqual({ span: 'afternoon.', at: 34, },);
      },
    },),

    it({
      name: 'TAKES IT AS LATE IN THE PAGE AS IT FITS, since the front of a page carries its '
        + 'frontmatter and title and deleting those is structural damage of another kind',
      fn: async function takesTheLatestCut() {
        // Nothing is anchored, so every window is available and the offset
        // alone says which end it chose.
        expect(
          decoyCut({
            text: 'The tabby slept by the window all afternoon.',
            avoid: [],
            chars: 10,
          },).at,
        ).toBe(34,);
      },
    },),

    it({
      name: 'SKIPS PAST EVERY OCCURRENCE of a repeated anchored span rather than the first, '
        + 'because a rendering the roster can still find anywhere is one the cut must not touch',
      fn: async function skipsEveryOccurrence() {
        expect(
          decoyCut({
            text: 'cat dog cat dog cat',
            avoid: ['cat',],
            chars: 4,
          },),
        ).toEqual({ span: 'dog ', at: 12, },);
      },
    },),

    it({
      name: 'REFUSES when every window overlaps an anchored span, rather than cutting one anyway: '
        + 'a decoy that deletes the rendering measures the same thing as the targeted cut',
      fn: async function noRoomIsRefused() {
        expect(
          decoyCut({
            text: 'The tabby slept.',
            avoid: ['The tabby slept.',],
            chars: 4,
          },),
        ).toEqual({ span: '', at: -1, },);
      },
    },),

    it({
      name: 'REFUSES a cut larger than the document, which is the shape a page shorter than its '
        + 'own anchored evidence would hand it',
      fn: async function tooLargeIsRefused() {
        expect(
          decoyCut({
            text: 'The tabby.',
            avoid: [],
            chars: 20,
          },),
        ).toEqual({ span: '', at: -1, },);
      },
    },),

    it({
      name: 'REFUSES a cut of no size, since deleting nothing and calling it a control would pass '
        + 'a wire that never reads the document at all',
      fn: async function noSizeIsRefused() {
        expect(
          decoyCut({
            text: 'The tabby slept.',
            avoid: [],
            chars: 0,
          },),
        ).toEqual({ span: '', at: -1, },);
      },
    },),
  ],
},);

//endregion Coverage control decoy cut
