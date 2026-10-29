import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { withoutASentence, } from '../../dist/final/node/index.mjs';

//region Editor width control cut
// What the positive control actually asks the panel about.
//
// The gate in front of the width draw damages a passage and checks the panel
// prefers the intact one. Everything downstream rests on the damage being real:
// a cut that returned the passage unchanged would let the gate pass on a blind
// panel, and a cut that returned blank on everything would fail a working one.
//
// Fixtures are invented cat prose, never corpus text.

await describe({
  name: withoutASentence.name,
  children: [
    it({
      name: 'REMOVES THE FIRST SENTENCE at a full-width stop, which is the terminator most of the '
        + 'corpus actually uses, and keeps everything after it',
      fn: async function cutsAtAFullWidthStop() {
        expect(withoutASentence('橘猫睡在窗台上。灰猫在门口等饭。',),).toBe('灰猫在门口等饭。',);
      },
    },),

    it({
      name: 'REMOVES THE FIRST SENTENCE at an ASCII stop followed by a space, so an English '
        + 'passage is damaged as readily as a Chinese one',
      fn: async function cutsAtAnAsciiStop() {
        expect(withoutASentence('The tabby slept. The grey cat waited.',),).toBe('The grey cat waited.',);
      },
    },),

    it({
      name: 'REFUSES A PASSAGE HOLDING ONE SENTENCE by returning blank, because cutting its only '
        + 'sentence leaves nothing to judge and asking the panel about nothing measures nothing',
      fn: async function oneSentenceIsRefused() {
        expect(withoutASentence('橘猫睡在窗台上。',),).toBe('',);
      },
    },),

    it({
      name: 'REFUSES A PASSAGE WITH NO TERMINATOR AT ALL rather than cutting at a guessed '
        + 'position, since a cut mid-clause is damage of a different kind than a missing sentence',
      fn: async function noTerminatorIsRefused() {
        expect(withoutASentence('a heading with no stop in it',),).toBe('',);
      },
    },),

    it({
      name: 'CUTS AT THE EARLIEST TERMINATOR when several kinds appear, so the sentence removed is '
        + 'the first one rather than whichever punctuation happens to be checked first',
      fn: async function theEarliestTerminatorWins() {
        // The question mark sits later than the ASCII stop, so the stop decides.
        expect(withoutASentence('One. Two? Three.',),).toBe('Two? Three.',);
      },
    },),

    it({
      name: 'REMOVES THE FIRST SENTENCE at a stop that ends its line, so a passage written one sentence '
        + 'per line is damageable rather than read as one sentence and dropped from the control',
      fn: async function cutsAtAStopEndingItsLine() {
        expect(withoutASentence('The tabby slept.\nThe grey cat waited.\n',),).toBe('The grey cat waited.',);
      },
    },),

    it({
      name: 'TRIMS THE REMAINDER so the damaged passage does not open on the space that followed '
        + 'the cut, which would be a second difference the panel could notice instead',
      fn: async function theRemainderIsTrimmed() {
        // Two spaces, so the cut genuinely lands on whitespace and the trim has
        // something to remove; a single space is consumed by the terminator itself.
        expect(withoutASentence('First.  Second.',),).toBe('Second.',);
      },
    },),
  ],
},);

//endregion Editor width control cut
