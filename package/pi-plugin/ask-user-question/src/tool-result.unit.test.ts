import {
  readFile,
  rm,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

import { DEFAULT_MAX_BYTES, } from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildAnsweredResult,
  buildCancelledResult,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: buildCancelledResult.name,
      children: [
        it({
          name: 'returns model-visible cancellation',
          fn: async () => {
            expect(buildCancelledResult(),)
              .toEqual({
                content: [{
                  type: 'text',
                  text: 'User cancelled the question.',
                },],
                details: { status: 'cancelled', },
              },);
          },
        },),
      ],
    },),
    describe({
      name: buildAnsweredResult.name,
      children: [
        it({
          name: 'returns complete answer under Pi output limit',
          fn: async () => {
            expect(await buildAnsweredResult({ answer: 'first\nsecond', }),)
              .toEqual({
                content: [{
                  type: 'text',
                  text: 'User answered:\nfirst\nsecond',
                },],
                details: {
                  status: 'answered',
                  answer: 'first\nsecond',
                },
              },);
          },
        },),
        it({
          name: 'retains complete answer when visible result truncates',
          fn: async () => {
            /**
             Text one byte beyond Pi visible tool-output cap.
             */
            const answer = 'a'.repeat(DEFAULT_MAX_BYTES + 1,);
            /**
             Truncated model result with retained path.
             */
            const result = await buildAnsweredResult({ answer, },);
            if (result.details.status !== 'answered')
              throw new Error('Expected answered details for truncated result.',);
            if (result.details.fullAnswerPath === undefined)
              throw new Error('Expected complete-answer path for truncated result.',);
            /**
             Retained full-answer path for cleanup.
             */
            const { fullAnswerPath, } = result.details;
            expect(await readFile(
              fullAnswerPath,
              'utf8',
            ),)
              .toBe(answer,);
            expect(result.content[0]?.type === 'text'
              ? result.content[0].text
              : '',)
              .toContain(fullAnswerPath,);
            await rm(
              dirname(fullAnswerPath,),
              {
                recursive: true,
                force: true,
              },
            );
          },
        },),
      ],
    },),
  ],
},);
