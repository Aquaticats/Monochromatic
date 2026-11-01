/**
 Tests for the per-call cost read off an OpenRouter stream.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COST_UNREPORTED,
  openRouterCostOf,
} from '../dist/final/node/index.mjs';

await describe({
  name: openRouterCostOf.name,
  children: [
    it({
      name: 'READS the cost the final chunk carries, ignoring chunks without usage, the [DONE] '
        + 'sentinel and a chunk that does not parse',
      fn: async () => {
        expect(openRouterCostOf({
          bodyText: [
            'data: {"id":"gen-1","choices":[{"delta":{"content":"{"}}]}',
            'data: {not json',
            'data: {"id":"gen-1","choices":[{"delta":{"content":"}"},"finish_reason":"stop"}],'
              + '"usage":{"prompt_tokens":574,"completion_tokens":306,"total_tokens":880,'
              + '"cost":0.000126255304,"is_byok":false}}',
            'data: [DONE]',
            '',
          ].join('\n\n',),
        },),).toBe(0.000126255304,);
      },
    },),

    it({
      name: 'REPORTS the named absence when no chunk carried a cost, a non-number, or a non-finite one',
      fn: async () => {
        expect(openRouterCostOf({
          bodyText: 'data: {"usage":{"prompt_tokens":1,"completion_tokens":1}}\n\ndata: [DONE]\n',
        },),).toBe(COST_UNREPORTED,);
        expect(openRouterCostOf({ bodyText: 'data: {"usage":{"cost":"0.1"}}\n', },),).toBe(COST_UNREPORTED,);
        expect(openRouterCostOf({ bodyText: 'data: {"usage":{"cost":null}}\n', },),).toBe(COST_UNREPORTED,);
        expect(openRouterCostOf({ bodyText: '', },),).toBe(COST_UNREPORTED,);
      },
    },),

    it({
      name: 'KEEPS the last cost when more than one chunk reports one, since a running figure ends on the total',
      fn: async () => {
        expect(openRouterCostOf({
          bodyText: 'data: {"usage":{"cost":0.1}}\n\ndata: {"usage":{"cost":0.25}}\n\ndata: [DONE]\n',
        },),).toBe(0.25,);
      },
    },),

    it({
      name: 'READS the cost off a chunk whose payload opens with spaces before its brace, which the '
        + 'completion reader parses and folds like any other chunk',
      fn: async () => {
        expect(openRouterCostOf({
          bodyText: 'data:   {"usage":{"cost":0.4}}\n\ndata: [DONE]\n',
        },),).toBe(0.4,);
      },
    },),

    it({
      name: 'SKIPS a cost that is not finite, since it states no spend',
      fn: async () => {
        expect(openRouterCostOf({
          bodyText: 'data: {"usage":{"cost":1e999}}\n\ndata: [DONE]\n',
        },),).toBe('unreported',);
      },
    },),
  ],
},);
