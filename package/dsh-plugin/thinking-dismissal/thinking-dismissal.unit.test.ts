import {
  describe,
  expect as verify,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  findSizeDismissal,
  thinkingDismissalProjection,
} from './index.js';

await describe({
  name: 'thinking-dismissal dsh plugin',
  children: [
    it({
      name: 'flags a sentence that carries cost and a dismissal together',
      fn: async () => {
        const hits = findSizeDismissal('The scan is quadratic, but the rows are short.',);
        verify(hits.length,).toBe(1,);
        verify(hits[0]?.pattern,).toBe('size-dismissal',);
        verify(hits[0]?.sentenceIndex,).toBe(0,);
        verify(hits[0]?.blockHash.length,).toBe(16,);
      },
    },),
    it({
      name: 'leaves a block that names cost without dismissing it unflagged',
      fn: async () => {
        const hits = findSizeDismissal('The scan is quadratic. Rewrite it as one linear pass.',);
        verify(hits.length,).toBe(0,);
      },
    },),
    it({
      name: 'folds reasoning blocks into hits and counts, and ignores other events',
      fn: async () => {
        const initial = thinkingDismissalProjection.init();
        verify(initial.reasoningBlocks,).toBe(0,);
        const folded = thinkingDismissalProjection.apply(initial, {
          type: 'assistant/message',
          seq: 1,
          data: {
            turn: 1,
            step: 1,
            message: {
              content: [
                { type: 'reasoning', text: 'Quadratic, but the rows are short.', },
                { type: 'text', text: 'Done.', },
              ],
            },
          },
        },);
        verify(folded.reasoningBlocks,).toBe(1,);
        verify(folded.hits.length,).toBe(1,);
        const untouched = thinkingDismissalProjection.apply(folded, {
          type: 'turn/start',
          seq: 2,
          data: {},
        },);
        verify(untouched === folded,).toBe(true,);
      },
    },),
  ],
},);
