/**
 Tests for the upstream endpoint read off an OpenRouter stream.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ENDPOINT_UNREPORTED,
  openRouterEndpointOf,
} from '../dist/final/node/index.mjs';

await describe({
  name: openRouterEndpointOf.name,
  children: [
    it({
      name: 'READS the upstream name the chunks carry, past a chunk that does not parse and the '
        + '[DONE] sentinel, so a run log can say which endpoint served a call',
      fn: async () => {
        expect(openRouterEndpointOf({
          bodyText: [
            ': OPENROUTER PROCESSING',
            'data: {not json',
            'data: {"id":"gen-1","provider":"ModelRun","choices":[{"delta":{"content":"{"}}]}',
            'data: {"id":"gen-1","provider":"ModelRun","choices":[{"delta":{"content":"}"},'
              + '"finish_reason":"stop"}],"usage":{"cost":0.001}}',
            'data: [DONE]',
            '',
          ].join('\n\n',),
        },),).toEqual({
          reported: true,
          name: 'ModelRun',
        },);
      },
    },),

    it({
      name: 'REPORTS the named absence when no chunk names one, names it empty, or names it with '
        + 'something other than a string, as a record no upstream name can collide with',
      fn: async () => {
        expect(openRouterEndpointOf({
          bodyText: 'data: {"choices":[{"delta":{"content":"x"}}]}\n\ndata: [DONE]\n',
        },),).toEqual(ENDPOINT_UNREPORTED,);
        expect(openRouterEndpointOf({ bodyText: 'data: {"provider":""}\n', },),).toEqual({ reported: false, },);
        expect(openRouterEndpointOf({ bodyText: 'data: {"provider":7}\n', },),).toEqual({ reported: false, },);
        expect(openRouterEndpointOf({ bodyText: '', },),).toEqual({ reported: false, },);
      },
    },),

    it({
      name: 'KEEPS the first name when chunks disagree, since the first is the upstream that accepted '
        + 'the request',
      fn: async () => {
        expect(openRouterEndpointOf({
          bodyText: 'data: {"provider":"Parasail"}\n\ndata: {"provider":"ModelRun"}\n\ndata: [DONE]\n',
        },),).toEqual({
          reported: true,
          name: 'Parasail',
        },);
      },
    },),
  ],
},);
