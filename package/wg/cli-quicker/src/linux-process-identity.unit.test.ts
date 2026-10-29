import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { processArgumentsMatch, } from '../dist/final/node/tunnel-bypass.mjs';

/**
 Stable watcher arguments expected independently from Node installation path.
 */
const expected = [
  '/package/bypass-watch.mjs',
  '/run/wg-quicker/interface.json',
] as const;

/**
 Live process identity carrying prior Node installation argument.
 */
const identity = {
  commandLine: [
    '/runtime/node-previous',
    ...expected,
  ],
  state: 'S',
  startTime: '1234',
} as const;

await describe({
  name: processArgumentsMatch.name,
  children: [
    it({
      name: 'accepts exact arguments from prior executable path',
      fn: async () => {
        expect(processArgumentsMatch({ identity, expected, },)).toBe(true,);
      },
    },),
    it({
      name: 'accepts exact arguments from current executable path',
      fn: async () => {
        expect(processArgumentsMatch({
          identity: {
            ...identity,
            commandLine: [
              '/runtime/node-current',
              ...expected,
            ],
          },
          expected,
        },)).toBe(true,);
      },
    },),
    it({
      name: 'rejects missing executable argument',
      fn: async () => {
        expect(processArgumentsMatch({
          identity: {
            ...identity,
            commandLine: expected,
          },
          expected,
        },)).toBe(false,);
      },
    },),
    it({
      name: 'rejects empty executable argument',
      fn: async () => {
        expect(processArgumentsMatch({
          identity: {
            ...identity,
            commandLine: [
              '',
              ...expected,
            ],
          },
          expected,
        },)).toBe(false,);
      },
    },),
    it({
      name: 'rejects different watcher script argument',
      fn: async () => {
        expect(processArgumentsMatch({
          identity: {
            ...identity,
            commandLine: [
              '/runtime/node-previous',
              '/package/other-script.mjs',
              expected[1],
            ],
          },
          expected,
        },)).toBe(false,);
      },
    },),
    it({
      name: 'rejects different state argument',
      fn: async () => {
        expect(processArgumentsMatch({
          identity: {
            ...identity,
            commandLine: [
              '/runtime/node-previous',
              expected[0],
              '/run/wg-quicker/other.json',
            ],
          },
          expected,
        },)).toBe(false,);
      },
    },),
  ],
},);
