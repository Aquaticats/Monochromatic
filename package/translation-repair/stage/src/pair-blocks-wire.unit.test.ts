/**
 Tests for block pairing: what the sheet carries, and what the reader refuses.

 WHY REFUSAL MATTERS MORE THAN ACCEPTANCE HERE. A wrong pairing puts two
 passages that were never about the same thing in front of the critics, which
 then report differences between them and are right to. It is known that a
 wrong pairing is worse than no pairing for exactly this reason, so every
 malformed reply in this file must throw rather than be tidied into something usable.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BlockPairingError,
  buildBlockPairingMessages,
  isBlockPairingWire,
  readBlockPairing,
  requireBlockPairingRefusal,
} from '../dist/final/node/index.mjs';

/**
 Three blocks standing in for an original side.
 */
const SOURCE = [
  {
    index: 0,
    text: '猫睡了。',
  },
  {
    index: 1,
    text: '它喜欢盒子。',
  },
  {
    index: 2,
    text: '它不喜欢水。',
  },
];

/**
 Four blocks standing in for a translation that split one paragraph.
 */
const TARGET = [
  {
    index: 0,
    text: 'The cat slept.',
  },
  {
    index: 1,
    text: 'She loves boxes.',
  },
  {
    index: 2,
    text: 'Cardboard ones especially.',
  },
  {
    index: 3,
    text: 'She does not love water.',
  },
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildBlockPairingMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NUMBERS both sides so a reply can name them',
          fn: async () => {
            const messages = buildBlockPairingMessages({
              sourceBlocks: SOURCE,
              targetBlocks: TARGET,
            },);
            const sheet = String(messages[1]?.content,);
            expect(sheet,).toContain('ORIGINAL BLOCKS',);
            expect(sheet,).toContain('TRANSLATION BLOCKS',);
            expect(sheet,).toContain('[2]',);
            expect(sheet,).toContain('[3]',);
          },
        },),
        it({
          name: 'TELLS the model that leaving a block out is a correct answer',
          fn: async () => {
            const messages = buildBlockPairingMessages({
              sourceBlocks: SOURCE,
              targetBlocks: TARGET,
            },);
            expect(String(messages[0]?.content,),).toContain('LEAVE IT OUT',);
          },
        },),
        it({
          name: 'CARRIES WHAT THE PICTURES HERE SAY and tells the model a translation block rendering '
            + 'a picture has no original counterpart (class thirty-four, 2026-09-16): Mio17 paired the '
            + 'archive\'s chat translation with the poem standing where the picture stands',
          fn: async () => {
            const messages = buildBlockPairingMessages({
              sourceBlocks: SOURCE,
              targetBlocks: TARGET,
              pictureContext: 'PICTURE nap.webp\nhf:cat/Whiskers:\n[left] are you asleep\n[right] nearly',
            },);
            const rules = String(messages[0]?.content,);
            const sheet = String(messages[1]?.content,);
            expect(rules,).toContain('WHAT THE PICTURES SAY',);
            expect(rules,).toContain('translates THE PICTURE',);
            expect(sheet,).toContain('WHAT THE PICTURES HERE SAY',);
            expect(sheet,).toContain('[left] are you asleep',);
          },
        },),
        it({
          name: 'LEAVES THE SHEET WITHOUT A PICTURE SECTION when no picture context is given, so a '
            + 'page without pictures asks the question it always asked',
          fn: async () => {
            const messages = buildBlockPairingMessages({
              sourceBlocks: SOURCE,
              targetBlocks: TARGET,
            },);
            expect(String(messages[0]?.content,),).not.toContain('WHAT THE PICTURES SAY',);
            expect(String(messages[1]?.content,),).not.toContain('WHAT THE PICTURES HERE SAY',);
          },
        },),
        it({
          name: 'CHOOSES a fence no block can close, since blocks are arbitrary prose',
          fn: async () => {
            /**
             A block carrying a fence run of the kind the sheet itself uses.
             */
            const awkward = 'A block whose own line reads ===== and continues.';
            const messages = buildBlockPairingMessages({
              sourceBlocks: [
                {
                  index: 0,
                  text: awkward,
                },
              ],
              targetBlocks: TARGET,
            },);

            /**
             Delimiter the sheet chose.

             The sheet reads header, blank, `[0]`, fence, so the fence is the
             fourth line. Reading the second returns the blank line, and
             `includes('')` is true of everything, which is a test that cannot fail.
             */
            const fence = String(messages[1]?.content,)
              .split('\n',)
              .at(3,);
            expect(fence,).toBeDefined();

            /**
             Whether the enclosed text could close the sheet's own delimiter.
             */
            const closable = awkward.includes(String(fence,),);
            // THE INVARIANT, not the character: whatever it picked, the enclosed
            // text must not be able to close it.
            expect(closable,).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: readBlockPairing.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a monotone pairing that leaves a block unpaired',
          fn: async () => {
            const pairs = readBlockPairing({
              value: {
                pairs: [
                  {
                    source: 0,
                    target: 0,
                  },
                  {
                    source: 2,
                    target: 3,
                  },
                ],
              },
              sourceCount: 3,
              targetCount: 4,
            },);
            expect(pairs.length,).toBe(2,);
          },
        },),
        it({
          name: 'ACCEPTS one original rendered by TWO translation blocks',
          fn: async () => {
            const pairs = readBlockPairing({
              value: {
                pairs: [
                  {
                    source: 1,
                    target: 1,
                  },
                  {
                    source: 1,
                    target: 2,
                  },
                ],
              },
              sourceCount: 3,
              targetCount: 4,
            },);
            expect(pairs.length,).toBe(2,);
          },
        },),
        it({
          name: 'ACCEPTS definition pairs that cross when told which blocks are definitions, since a page '
            + 'renders its notes by reference order (the third yuki418330012 launch of 2026-09-08 lost six of '
            + 'eight voices to the order rule), and still refuses a body pair that steps backwards',
          fn: async () => {
            /**
             Body in order, the two definitions paired by content, crossing.
             */
            const crossing = {
              pairs: [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 1,
                },
                {
                  source: 2,
                  target: 4,
                },
                {
                  source: 3,
                  target: 3,
                },
              ],
            };
            /**
             Which blocks are definitions on each side.
             */
            const freeOrder = {
              source: new Set([
                2,
                3,
              ],),
              target: new Set([
                3,
                4,
              ],),
            };
            expect(readBlockPairing({
              value: crossing,
              sourceCount: 4,
              targetCount: 5,
              freeOrder,
            },).length,).toBe(4,);
            expect(function readsWithoutTheExemption() {
              readBlockPairing({
                value: crossing,
                sourceCount: 4,
                targetCount: 5,
              },);
            },).toThrow(BlockPairingError,);
            expect(function readsABodyStepBack() {
              readBlockPairing({
                value: {
                  pairs: [
                    {
                      source: 1,
                      target: 1,
                    },
                    {
                      source: 2,
                      target: 4,
                    },
                    {
                      source: 0,
                      target: 0,
                    },
                    {
                      source: 3,
                      target: 3,
                    },
                  ],
                },
                sourceCount: 4,
                targetCount: 5,
                freeOrder,
              },);
            },).toThrow(BlockPairingError,);
          },
        },),
        it({
          name: 'REFUSES a pair that joins a footnote definition with a body block',
          fn: async () => {
            expect(function readsMixed() {
              readBlockPairing({
                value: {
                  pairs: [
                    {
                      source: 0,
                      target: 0,
                    },
                    {
                      source: 1,
                      target: 3,
                    },
                  ],
                },
                sourceCount: 4,
                targetCount: 5,
                freeOrder: {
                  source: new Set([
                    2,
                    3,
                  ],),
                  target: new Set([
                    3,
                    4,
                  ],),
                },
              },);
            },).toThrow(BlockPairingError,);
          },
        },),
        it({
          name: 'REFUSES a pairing that moves backwards on the original side',
          fn: async () => {
            expect(function readsBackwards() {
              readBlockPairing({
                value: {
                  pairs: [
                    {
                      source: 2,
                      target: 0,
                    },
                    {
                      source: 1,
                      target: 1,
                    },
                  ],
                },
                sourceCount: 3,
                targetCount: 4,
              },);
            },).toThrow(BlockPairingError,);
          },
        },),
        it({
          name: 'REFUSES every CROSSING ownership, where one original owns translations either side of '
            + 'another original’s, so grouping never has to resolve one. An item was once opened for a '
            + 'crossing refusal downstream; monotone on both sides already makes the shape unsayable, '
            + 'and this is where that is decided',
          fn: async () => {
            /**
             The three ways a reply can cross, each one a real reading a model
             could hold and none of them expressible in document order.
             */
            const crossings = [
              [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 1,
                },
                {
                  source: 0,
                  target: 2,
                },
              ],
              [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 1,
                },
                {
                  source: 2,
                  target: 0,
                },
              ],
              [
                {
                  source: 0,
                  target: 1,
                },
                {
                  source: 1,
                  target: 0,
                },
              ],
            ];
            for (const pairs of crossings)
              expect(function readsCrossing() {
                readBlockPairing({
                  value: { pairs, },
                  sourceCount: 4,
                  targetCount: 4,
                },);
              },).toThrow(BlockPairingError,);
          },
        },),
        it({
          name: 'ACCEPTS one translation block rendering TWO originals, which is a merge',
          fn: async () => {
            // AN EARLIER VERSION REFUSED THIS, holding that a passage renders one
            // place. A live run refuted it: on `lintong` all six models
            // independently paired one translation block with two originals, every
            // reply was refused, and the entry fell back to scoring and collapsed
            // to a single slice. A translation may merge two paragraphs exactly as
            // it may split one.
            const pairs = readBlockPairing({
              value: {
                pairs: [
                  {
                    source: 0,
                    target: 1,
                  },
                  {
                    source: 1,
                    target: 1,
                  },
                ],
              },
              sourceCount: 3,
              targetCount: 4,
            },);
            expect(pairs.length,).toBe(2,);
          },
        },),
        it({
          name: 'REFUSES the same correspondence twice, which describes nothing new',
          fn: async () => {
            expect(function readsRepeat() {
              readBlockPairing({
                value: {
                  pairs: [
                    {
                      source: 1,
                      target: 1,
                    },
                    {
                      source: 1,
                      target: 1,
                    },
                  ],
                },
                sourceCount: 3,
                targetCount: 4,
              },);
            },).toThrow(BlockPairingError,);
          },
        },),
        it({
          name: 'REFUSES a definition pair named again after another pair, which the free order of definitions lets '
            + 'through the order rule, since one voice naming it twice would count as two voices agreeing on it',
          fn: async () => {
            /**
             The refusal for a definition pair repeated after another one.
             */
            const refusal = caught(function readsDefinitionRepeat(): void {
              readBlockPairing({
                value: {
                  pairs: [
                    {
                      source: 2,
                      target: 2,
                    },
                    {
                      source: 3,
                      target: 3,
                    },
                    {
                      source: 2,
                      target: 2,
                    },
                  ],
                },
                sourceCount: 4,
                targetCount: 4,
                freeOrder: {
                  source: new Set([
                    2,
                    3,
                  ],),
                  target: new Set([
                    2,
                    3,
                  ],),
                },
              },);
            },);
            expect(refusal,).toBeInstanceOf(BlockPairingError,);
            expect(caughtValueText(refusal,),).toBe('pairing repeats the same correspondence at position 2',);
          },
        },),
        it({
          name: 'REFUSES an index no block carries, on either side, naming the side',
          fn: async () => {
            /**
             The refusal for an original index past the end.
             */
            const sourceRefusal = caught(function readsOutOfRange(): void {
              readBlockPairing({
                value: {
                  pairs: [
                    {
                      source: 9,
                      target: 0,
                    },
                  ],
                },
                sourceCount: 3,
                targetCount: 4,
              },);
            },);
            /**
             The refusal for a translation index past the end.
             */
            const targetRefusal = caught(function readsTargetOutOfRange(): void {
              readBlockPairing({
                value: {
                  pairs: [
                    {
                      source: 0,
                      target: 9,
                    },
                  ],
                },
                sourceCount: 3,
                targetCount: 4,
              },);
            },);
            // THE CLASS IS PART OF THE CONTRACT: `readBlockPairingOutcomes`
            // records a `BlockPairingError` as that seat's unusable reply and
            // rethrows anything else.
            expect(sourceRefusal,).toBeInstanceOf(BlockPairingError,);
            expect(targetRefusal,).toBeInstanceOf(BlockPairingError,);
            expect(caughtValueText(sourceRefusal,),).toBe('pairing names original block 9, and there are 3',);
            expect(caughtValueText(targetRefusal,),).toBe('pairing names translation block 9, and there are 4',);
          },
        },),
        it({
          name: 'REFUSES a reply that is not a pairing at all',
          fn: async () => {
            expect(function readsRubbish() {
              readBlockPairing({
                value: { pairs: 'all of them', },
                sourceCount: 3,
                targetCount: 4,
              },);
            },).toThrow(BlockPairingError,);
          },
        },),
        it({
          name: 'ACCEPTS an empty pairing, which says nothing corresponds',
          fn: async () => {
            expect(
              readBlockPairing({
                value: { pairs: [], },
                sourceCount: 3,
                targetCount: 4,
              },).length,
            ).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: requireBlockPairingRefusal.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'hands back a pairing refusal as it is, and rethrows anything else unchanged',
          fn: async () => {
            /**
             A refusal the reader raises.
             */
            const refusal = new BlockPairingError({ message: 'pairing names original block 9, and there are 3', },);
            /**
             A failure that is not a bad reply.
             */
            const stray = new TypeError('the cat sat on the keyboard',);
            expect(requireBlockPairingRefusal({ error: refusal, },),).toBe(refusal,);
            expect(caught(function narrowStray(): void {
              requireBlockPairingRefusal({ error: stray, },);
            },),).toBe(stray,);
          },
        },),
      ],
    },),

    describe({
      name: isBlockPairingWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES non-integer indices, which cannot name a block',
          fn: async () => {
            expect(isBlockPairingWire({
              pairs: [
                {
                  source: 0.5,
                  target: 1,
                },
              ],
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
