import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  HelperProtocolError,
  parseHelperCompletion,
  serializeHelperCompletion,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: serializeHelperCompletion.name,
      children: [
        it({
          name: 'serializes error completion with message',
          fn: async () => {
            expect(serializeHelperCompletion({
              completion: {
                status: 'error',
                message: 'editor unavailable',
              },
            },),)
              .toBe('{"status":"error","message":"editor unavailable"}',);
          },
        },),
      ],
    },),
    describe({
      name: parseHelperCompletion.name,
      children: [
        ...[
          {
            name: 'maps disconnect after authentication to cancellation',
            payload: '',
            expected: { status: 'cancelled', },
          },
          {
            name: 'accepts submitted completion',
            payload: '{"status":"submitted"}',
            expected: { status: 'submitted', },
          },
          {
            name: 'accepts cancelled completion',
            payload: '{"status":"cancelled"}',
            expected: { status: 'cancelled', },
          },
          {
            name: 'accepts error completion',
            payload: '{"status":"error","message":"editor unavailable"}',
            expected: {
              status: 'error',
              message: 'editor unavailable',
            },
          },
        ].map(function toAcceptedPayloadTest(testCase,) {
          return it({
            name: testCase.name,
            fn: async () => {
              expect(parseHelperCompletion({ payload: testCase.payload, }),)
                .toEqual(testCase.expected,);
            },
          },);
        },),
        ...[
          'not-json',
          '{}',
          '{"status":"unknown"}',
          '{"status":"error"}',
          '{"status":"error","message":""}',
        ].map(function toRejectedPayloadTest(payload,) {
          return it({
            name: `rejects ${payload}`,
            fn: async () => {
              /**
               Captured protocol failure.
               */
              const caught: { value?: unknown; } = {};
              try {
                parseHelperCompletion({ payload, },);
              }
              catch (error: unknown) {
                caught.value = error;
              }
              expect(caught.value,)
                .toBeInstanceOf(HelperProtocolError,);
            },
          },);
        },),
      ],
    },),
  ],
},);
