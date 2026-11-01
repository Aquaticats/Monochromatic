/**
 Unit tests for the plain-text Advisor theme.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { dshAdvisorTheme, } from '../dist/final/node/index.mjs';

await describe({
  name: 'Advisor theme pass-through',
  children: [
    it({
      name: 'returns the text unchanged for fg',
      fn: async () => {
        expect(
          dshAdvisorTheme.fg(
            'blue' as Parameters<typeof dshAdvisorTheme.fg>[0],
            'visible',
          ),
        ).toBe('visible',);
      },
    },),
    it({
      name: 'returns the text unchanged for bold',
      fn: async () => {
        expect(dshAdvisorTheme.bold('loud',),).toBe('loud',);
      },
    },),
  ],
},);
