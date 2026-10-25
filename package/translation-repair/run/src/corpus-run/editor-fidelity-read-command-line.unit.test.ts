/**
 Tests for the command line of the editor fidelity read step: which arm logs it takes, who owns its build entry, and
 that a flag it does not declare is refused rather than ignored. The step checks the logs of the arms before any
 figure of them is read.

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
  name: 'editor-fidelity-read command line',
  children: [
    it({
      name: 'TAKES THE LOGS OF THE ARMS as positional paths, at least one, and no flag',
      fn: async () => {
        /**
         The declaration under test.
         */
        const spec = COMMAND_LINES['editor-fidelity-read'];
        expect(Object.keys(spec.valued,),).toEqual([],);
        expect(Object.keys(spec.switches,),).toEqual([],);
        expect(Object.keys(spec.repeatable,),).toEqual([],);
        expect(spec.positionals.names,).toEqual(['arm log',],);
        expect(spec.positionals.least,).toBe(1,);
        expect(spec.positionals.rest,).toBe(true,);
      },
    },),

    it({
      name: 'IS OWNED by the probe package, beside the probe whose arms it reads',
      fn: async () => {
        expect(COMMAND_OWNERS['editor-fidelity-read'],).toBe('probe',);
      },
    },),

    it({
      name: 'READS the logs as written, in the order given',
      fn: async () => {
        const line = lineOf({
          command: 'editor-fidelity-read',
          typed: ['low.log', 'medium-a.log',],
        },);
        expect(line.positionals,).toEqual(['low.log', 'medium-a.log',],);
      },
    },),

    it({
      name: 'REFUSES a flag it does not declare, naming the command',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function readsFlag(): void {
            lineOf({
              command: 'editor-fidelity-read',
              typed: ['low.log', '--cap', '3',],
            },);
          },
        },),).toContain('editor-fidelity-read',);
      },
    },),
  ],
},);
