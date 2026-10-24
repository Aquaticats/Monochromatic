/**
 Tests for the command line of the editor fidelity probe: which flags it declares, who owns its build entry, and that a
 flag it does not declare is refused rather than ignored. The probe is the editor-seat counterpart of the judge
 fidelity probe and reads the same flags beside the amount it may spend.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  CLAUDE_KEY_VARIABLE,
  COMMAND_LINES,
  COMMAND_OWNERS,
} from '../../dist/final/node/index.mjs';
import { lineOf, } from './command-line.test-fixture.ts';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';

await describe({
  name: 'editor-fidelity-probe command line',
  children: [
    it({
      name: 'DECLARES the judge probe\'s entry, cap, damage and candidate flags, its candidates-alone switch, the '
        + 'amount it may spend and the number of trials an arm walks, and no context switch, which has no meaning for '
        + 'an editor',
      fn: async () => {
        /**
         The declaration under test.
         */
        const spec = COMMAND_LINES['editor-fidelity-probe'];
        expect(Object.keys(spec.valued,).toSorted(),).toEqual([
          'bench-spent-usd',
          'candidates',
          'cap',
          'damage',
          'max-usd',
          'only',
          'trials',
        ],);
        expect(Object.keys(spec.switches,),).toEqual(['candidates-alone',],);
        expect(Object.keys(spec.repeatable,),).toEqual([],);
        expect(spec.positionals.names,).toEqual([],);
      },
    },),

    it({
      name: 'IS OWNED by the probe package, beside the judge probe it mirrors',
      fn: async () => {
        expect(COMMAND_OWNERS['editor-fidelity-probe'],).toBe('probe',);
        expect(COMMAND_OWNERS['editor-fidelity-probe'],).toBe(COMMAND_OWNERS['judge-fidelity-probe'],);
      },
    },),

    it({
      name: 'READS every flag it declares as written',
      fn: async () => {
        /**
         A line carrying each flag once.
         */
        const line = lineOf({
          command: 'editor-fidelity-probe',
          typed: [
            '--cap',
            '4',
            '--only',
            'gqt',
            '--damage',
            'insertion',
            '--candidates',
            'claude-sonnet-5-5-effort-low',
            '--candidates-alone',
            '--max-usd',
            '1.5',
            '--bench-spent-usd',
            '3',
            '--trials',
            '57',
          ],
        },);
        expect(line.flag('trials',),).toEqual({
          kind: 'written',
          flag: '--trials',
          value: '57',
        },);
        expect(line.flag('max-usd',),).toEqual({
          kind: 'written',
          flag: '--max-usd',
          value: '1.5',
        },);
        expect(line.flag('bench-spent-usd',),).toEqual({
          kind: 'written',
          flag: '--bench-spent-usd',
          value: '3',
        },);
        expect(line.flag('cap',).kind,).toBe('written',);
        expect(line.switched('candidates-alone',),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES --context, which the judge probe reads and this probe does not, naming the command',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function readsContext(): void {
            lineOf({
              command: 'editor-fidelity-probe',
              typed: ['--context',],
            },);
          },
        },),).toContain('editor-fidelity-probe',);
      },
    },),

    it({
      name: 'NAMES the variable the Claude key is read from, which the probe keeps in the environment it builds its '
        + 'client over and leaves out every other provider\'s',
      fn: async () => {
        expect(CLAUDE_KEY_VARIABLE,).toBe('TRANSLATION_REPAIR_CLAUDE_API_KEY',);
      },
    },),
  ],
},);
