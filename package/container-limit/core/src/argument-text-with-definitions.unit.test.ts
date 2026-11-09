import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { argumentTextWithBoundInitializers, } from './argument-text-with-definitions.ts';

await describe({
  name: argumentTextWithBoundInitializers.name,
  children: [
    it({
      name: 'reads a word the argument text writes itself',
      fn: async () => {
        expect(argumentTextWithBoundInitializers({
          argumentText: "'run', ...containerLimitArguments(bound)",
          definitions: new Map(),
        },).includes('containerLimitArguments',),).toBe(true,);
      },
    },),
    it({
      name: 'reaches a definition the argument text names',
      fn: async () => {
        const reaching = argumentTextWithBoundInitializers({
          argumentText: "'run', '--rm', ...bounds",
          definitions: new Map([['bounds', "['--blkio-weight=10']",],]),
        },);
        expect(reaching.includes('--blkio-weight=10',),).toBe(true,);
      },
    },),
    it({
      name: 'follows a chain of definitions to its end',
      fn: async () => {
        const reaching = argumentTextWithBoundInitializers({
          argumentText: '...first',
          definitions: new Map([
            ['first', '...second',],
            ['second', 'containerLimitArguments(bound)',],
          ],),
        },);
        expect(reaching.includes('containerLimitArguments',),).toBe(true,);
      },
    },),
    it({
      name: 'reads a name the file imports from the definitions it was given',
      fn: async () => {
        const reaching = argumentTextWithBoundInitializers({
          argumentText: "'build', ...buildBounds",
          definitions: new Map(),
          importedDefinitions: new Map([['buildBounds', "['--memory=2g']",],]),
        },);
        expect(reaching.includes('--memory=2g',),).toBe(true,);
      },
    },),
    it({
      name: 'keeps the argument text when a name has no definition anywhere',
      fn: async () => {
        expect(argumentTextWithBoundInitializers({
          argumentText: "'run', ...args",
          definitions: new Map(),
        },),).toBe("'run', ...args",);
      },
    },),
    it({
      name: 'stops at the resolved-name cap instead of reading a whole file of definitions',
      fn: async () => {
        /**
         Definitions whose names each reach the next one, ten times longer than the cap allows.
         */
        const definitions = new Map<string, string>();
        for (let step = 0; step < 100; step += 1)
          definitions.set(`step${String(step,)}`, `...step${String(step + 1,)}`,);
        definitions.set('step100', 'THE_LAST_WORD',);
        const reaching = argumentTextWithBoundInitializers({
          argumentText: '...step0',
          definitions,
        },);
        expect(reaching.includes('THE_LAST_WORD',),).toBe(false,);
      },
    },),
  ],
},);
