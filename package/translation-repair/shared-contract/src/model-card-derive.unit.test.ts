/**
 Tests reading a roster model's decisions side (ledger T8): a decision seat
 hands back the side its card carries, and a chat seat, whose card carries
 none, is refused by name rather than read as undefined. The seats are drawn
 from the roster itself, so the case follows the roster as it changes. Also
 tests building a record over exactly the keys it is given.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DecisionsCardMissingError,
  decisionsCardOf,
  isDecisionSeat,
  recordOfDistinctIds,
  recordOver,
  MODEL_CARDS,
  ROSTER_MODEL_IDS,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: decisionsCardOf.name,
      children: [
        it({
          name: 'HANDS BACK A DECISION SEAT\'S SIDE, and refuses a chat seat, whose card carries none',
          fn: async () => {
            const decisionSeat = ROSTER_MODEL_IDS.find((modelId,) => isDecisionSeat({ modelId, },));
            const chatSeat = ROSTER_MODEL_IDS.find((modelId,) => !isDecisionSeat({ modelId, },));
            if ((decisionSeat === undefined) || (chatSeat === undefined))
              throw new Error('the roster needs a decision seat and a chat seat for this case',);
            expect(decisionsCardOf({ modelId: decisionSeat, },),).toBe(MODEL_CARDS[decisionSeat].decisions,);
            expect(() => decisionsCardOf({ modelId: chatSeat, },),).toThrow(DecisionsCardMissingError,);
          },
        },),
      ],
    },),

    describe({
      name: recordOfDistinctIds.name,
      children: [
        it({
          name: 'BUILDS the record where every id appears once, and REFUSES an id that repeats, naming the id '
            + 'and whose rows they are, since the later row would replace the earlier and drop a card',
          fn: async () => {
            expect(recordOfDistinctIds({
              entries: [
                ['cat', 3,],
                ['kitten', 6,],
              ],
              owner: 'tabby cards',
            },),).toEqual({
              cat: 3,
              kitten: 6,
            },);
            /**
             What building over a repeated id throws.
             */
            const refusal = caught(function repeats(): void {
              recordOfDistinctIds({
                entries: [
                  ['cat', 3,],
                  ['kitten', 6,],
                  ['cat', 9,],
                ],
                owner: 'tabby cards',
              },);
            },);
            expect(refusal,).toBeInstanceOf(RangeError,);
            expect(String(refusal,),).toBe('RangeError: tabby cards carry the id cat more than once',);
          },
        },),
      ],
    },),

    describe({
      name: recordOver.name,
      children: [
        it({
          name: 'REFUSES keys that repeat, naming them, since one record cannot hold one key twice, and BUILDS '
            + 'the record where they do not, each value read off its own key',
          fn: async () => {
            expect(recordOver({
              keys: ['cat', 'kitten',],
              of: function letters(key,): number {
                return key.length;
              },
            },),).toEqual({
              cat: 3,
              kitten: 6,
            },);
            /**
             What building over a repeated key throws.
             */
            const refusal = caught(function repeats(): void {
              recordOver({
                keys: ['cat', 'cat',],
                of: function letters(key,): number {
                  return key.length;
                },
              },);
            },);
            expect(refusal,).toBeInstanceOf(RangeError,);
            expect(String(refusal,),).toBe('RangeError: keys repeat: cat, cat',);
          },
        },),
      ],
    },),
  ],
},);
