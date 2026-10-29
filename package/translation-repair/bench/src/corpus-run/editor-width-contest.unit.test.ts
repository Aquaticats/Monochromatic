import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  armInSeat,
  seatThatWon,
} from '../../dist/final/node/index.mjs';

//region Editor width contest attribution
// Who a contest round credited, and the collision that made the naive reading
// wrong.
//
// An arm that declined to repair offers the untouched translation. The fallback
// a panel falls back to when it will not separate the pair is also the untouched
// translation. The two are byte-identical, so reading the winner by matching
// shipped text credited every indecision to whichever arm had declined. The wide
// arm declines more often, because it splits one selection minimum across twice
// the candidates, so the bias ran toward the very conclusion the draw is meant
// to test.
//
// Fixtures are invented cat prose, never corpus text.

/**
 Translation as it stood, which a declining arm re-offers unchanged.
 */
const UNTOUCHED = 'The tabby waited by the door.';

/**
 A real rewrite, distinguishable from the incumbent.
 */
const REWRITTEN = 'The tabby sat waiting at the door.';

/**
 Builds an arm offering some text.

 @param patchedText - what this arm shipped

 @returns Arm shaped as the contest reads it

 @example
 ```ts
 const arm = armOffering(UNTOUCHED,);
 ```
 */
function armOffering(patchedText: string,) {
  return {
    text: patchedText,
    patch: {
      patchedText,
      applied: [],
      rejected: [],
    },
    heard: 3,
    producers: [],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: seatThatWon.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE SEAT WHOSE TEXT SHIPPED when a candidate actually won, which the stage '
            + 'reports by marking the shipped producer a composite',
          fn: async function adecidedRoundNamesItsSeat() {
            expect(
              seatThatWon({
                shippedProducer: {
                  kind: 'composite',
                  contributors: [],
                },
                shipped: REWRITTEN,
                first: armOffering(UNTOUCHED,),
                second: armOffering(REWRITTEN,),
              },),
            ).toBe('second',);
          },
        },),

        it({
          name: 'CREDITS NOBODY WHEN THE PANEL DECLINED even though the shipped bytes match an arm '
            + 'exactly, which is the collision this reader exists for: a declining arm offers the '
            + 'untouched translation and so does the fallback, so text alone cannot tell them apart',
          fn: async function indecisionIsNotAWinForTheDecliningArm() {
            expect(
              seatThatWon({
                // `incumbent` is what the indecision fallback carries, and no arm is
                // ever seated as one.
                shippedProducer: {
                  kind: 'incumbent',
                  matched: [],
                },
                shipped: UNTOUCHED,
                first: armOffering(UNTOUCHED,),
                second: armOffering(REWRITTEN,),
              },),
            ).toBe('none',);
          },
        },),

        it({
          name: 'CREDITS NOBODY WHEN THE SLATE WAS REJECTED, which the stage reports as unattributed '
            + 'rather than as any candidate',
          fn: async function rejectionCreditsNobody() {
            expect(
              seatThatWon({
                shippedProducer: { kind: 'unattributed', },
                shipped: UNTOUCHED,
                first: armOffering(UNTOUCHED,),
                second: armOffering(REWRITTEN,),
              },),
            ).toBe('none',);
          },
        },),

        it({
          name: 'REFUSES A COMPOSITE WHOSE SHIPPED TEXT IS NEITHER SEAT\'S, since the two seats are the only '
            + 'composites on the slate and a winner ships one of their patches',
          fn: async function aThirdWordingFromACompositeIsRefused() {
            const refusal = caught(function act(): unknown {
              return seatThatWon({
                shippedProducer: {
                  kind: 'composite',
                  contributors: [],
                },
                shipped: 'a third wording neither arm offered',
                first: armOffering(UNTOUCHED,),
                second: armOffering(REWRITTEN,),
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe(
              'Error: unreachable: the stage shipped a composite whose text is neither seat\'s, and the two '
                + 'seats are the only composites on the slate',
            );
          },
        },),
      ],
    },),

    describe({
      name: armInSeat.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAPS THE SAME SEAT TO OPPOSITE ARMS ACROSS THE TWO ORDERS, which is the whole '
            + 'mechanism cancelling position bias: winning the first seat means narrow in one order '
            + 'and wide in the other',
          fn: async function oneSeatMeansOppositeArmsAcrossOrders() {
            expect(armInSeat({ seat: 'first', firstArm: 'narrow', },),).toBe('narrow',);
            expect(armInSeat({ seat: 'first', firstArm: 'wide', },),).toBe('wide',);
            expect(armInSeat({ seat: 'second', firstArm: 'narrow', },),).toBe('wide',);
            expect(armInSeat({ seat: 'second', firstArm: 'wide', },),).toBe('narrow',);
          },
        },),

        it({
          name: 'CARRIES A NONE THROUGH rather than resolving it to whichever arm sat somewhere, so a '
            + 'round nobody won cannot become a win downstream',
          fn: async function noneStaysNone() {
            expect(armInSeat({ seat: 'none', firstArm: 'narrow', },),).toBe('none',);
            expect(armInSeat({ seat: 'none', firstArm: 'wide', },),).toBe('none',);
          },
        },),
      ],
    },),
  ],
},);

//endregion Editor width contest attribution
