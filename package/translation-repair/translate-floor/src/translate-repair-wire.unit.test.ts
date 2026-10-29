/**
 Tests for what the repair follow-up says the structural check compared.

 WHY IT MATTERS THAT THE SHEET NAMES BOTH TEXTS. The findings a repair round
 carries are written by `validateTranslatedSlice`, which checks a candidate
 against the ORIGINAL and against the PAGE AS IT STANDS and says which one
 each finding came from. A follow-up that announces the comparison was against
 the ORIGINAL tells the model to revise toward the source's shape, which is
 how a slice whose archive merged two blocks spends a repair round moving
 further from what the guard wants.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { buildTranslateRepairMessages, } from '../dist/final/node/index.mjs';

/**
 Follow-up turn, which is the last message the builder appends.
 */
const followUp = buildTranslateRepairMessages({
  priorMessages: [
    {
      role: 'user',
      content: 'render this',
    },
  ],
  priorTranslation: '> The cat sleeps on the windowsill.',
  findings: ['ORIGINAL: one block quote is missing',],
},)
  .map(function toContent(message,): string {
    return message.content;
  },)
  .join('\n',);

await describe({
  name: 'translate repair wire',
  children: [
    it({
      name: 'NAMES the page as one of the two texts the check compared',
      fn: async () => {
        expect(followUp.includes('PAGE AS IT STANDS',),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES to present the original as the only comparand',
      fn: async () => {
        // The wording that told a model its work had been measured against one
        // text, when the guard measures against two.
        expect(followUp.includes('compared against the ORIGINAL by a mechanical',),).toBe(false,);
      },
    },),
    it({
      name: 'KEEPS the three answers a model is allowed to give',
      fn: async () => {
        expect(followUp.includes('"revised"',),).toBe(true,);
        expect(followUp.includes('"unable"',),).toBe(true,);
        expect(followUp.includes('"as-intended"',),).toBe(true,);
      },
    },),
  ],
},);
