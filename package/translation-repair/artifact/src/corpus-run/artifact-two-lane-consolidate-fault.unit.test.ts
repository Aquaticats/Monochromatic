/**
 Tests for the error a consolidation record raises when a translate lane winner is marked shipped with no reason.

 WHAT THESE PIN is the error itself: the name a caller matches on, the flag that says its message is safe to forward,
 and the slice the message names. The case in `artifact-two-lane-consolidate.unit.test.ts` pins that the record
 builder throws it; this file pins what it carries, so a rename or a dropped flag fails here and not only there.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { TranslateWinnerReasonMissingError, } from '../../dist/final/node/index.mjs';

await describe({
  name: TranslateWinnerReasonMissingError.name,
  children: [
    it({
      name: 'IS AN ERROR WHOSE NAME A CALLER MATCHES ON, and declares its message safe to forward',
      fn: async () => {
        const fault = new TranslateWinnerReasonMissingError({ sliceIndex: 42, },);
        expect(Error.isError(fault,),).toBe(true,);
        expect({
          name: fault.name,
          messageNamesOnly: fault.messageNamesOnly,
        },).toStrictEqual({
          name: 'TranslateWinnerReasonMissingError',
          messageNamesOnly: true,
        },);
      },
    },),

    it({
      name: 'NAMES THE SLICE it was raised for and says the settlement marks a winner shipped without a reason',
      fn: async () => {
        const fault = new TranslateWinnerReasonMissingError({ sliceIndex: 42, },);
        expect({
          opensWithSlice: fault.message.startsWith('slice 42: ',),
          namesMark: fault.message.includes('marks the translate lane winner as shipped',),
          namesFault: fault.message.includes('names no reason',),
        },).toStrictEqual({
          opensWithSlice: true,
          namesMark: true,
          namesFault: true,
        },);
      },
    },),

    it({
      name: 'FOLLOWS THE SLICE INDEX it is given: another slice is named in the message, and the first is not',
      fn: async () => {
        const fault = new TranslateWinnerReasonMissingError({ sliceIndex: 0, },);
        expect({
          opensWithSlice: fault.message.startsWith('slice 0: ',),
          namesOtherSlice: fault.message.includes('slice 42',),
        },).toStrictEqual({
          opensWithSlice: true,
          namesOtherSlice: false,
        },);
      },
    },),
  ],
},);
