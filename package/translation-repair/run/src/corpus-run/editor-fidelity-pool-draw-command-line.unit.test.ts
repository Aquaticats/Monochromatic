/**
 Tests for the command line of the editor fidelity pool draw: which flags it declares, who owns its build entry, and
 that a flag it does not declare is refused rather than ignored.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  COMMAND_LINES,
  COMMAND_OWNERS,
} from '../../dist/final/node/index.mjs';
import { lineOf, } from './command-line.test-fixture.ts';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';

await describe({
  name: 'editor-fidelity-pool-draw command line',
  children: [
    it({
      name: 'DECLARES the seed, the trials wanted and the directory the parts are written to or checked in, and the '
        + 'switch that checks instead of writing',
      fn: async () => {
        /**
         The declaration under test.
         */
        const spec = COMMAND_LINES['editor-fidelity-pool-draw'];
        expect(Object.keys(spec.valued,).toSorted(),).toEqual(['out', 'seed', 'want',],);
        expect(Object.keys(spec.switches,),).toEqual(['check',],);
        expect(Object.keys(spec.repeatable,),).toEqual([],);
        expect(spec.positionals.names,).toEqual([],);
      },
    },),

    it({
      name: 'IS OWNED by the probe package, beside the probe whose pool it draws',
      fn: async () => {
        expect(COMMAND_OWNERS['editor-fidelity-pool-draw'],).toBe('probe',);
      },
    },),

    it({
      name: 'READS every flag it declares as written',
      fn: async () => {
        /**
         A line carrying each flag once.
         */
        const line = lineOf({
          command: 'editor-fidelity-pool-draw',
          typed: ['--seed', 'round-4', '--want', '40', '--out', 'src', '--check',],
        },);
        expect(line.flag('seed',),).toEqual({
          kind: 'written',
          flag: '--seed',
          value: 'round-4',
        },);
        expect(line.flag('want',).kind,).toBe('written',);
        expect(line.flag('out',).kind,).toBe('written',);
        expect(line.switched('check',),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES a flag it does not declare, naming the command',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function readsClone(): void {
            lineOf({
              command: 'editor-fidelity-pool-draw',
              typed: ['--clone', 'elsewhere',],
            },);
          },
        },),).toContain('editor-fidelity-pool-draw',);
      },
    },),
  ],
},);
