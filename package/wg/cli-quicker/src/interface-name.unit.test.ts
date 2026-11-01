import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseCliArgs, } from '../dist/final/node/cli-args.mjs';
import { loadConfig, } from '../dist/final/node/config.mjs';
import {
  assertInterfaceName,
  interfaceNameOfArgument,
  isValidInterfaceName,
} from '../dist/final/node/interface-name.mjs';

/**
 Longest name Linux accepts.
 */
const LONGEST = 15;

/**
 Names that are used, or could be used, for real interfaces.
 */
const VALID: readonly string[] = [
  'wg0',
  'fr-par-fr1',
  'mx-que-mx1',
  'a',
  'A_b.c-d.e-f',
  'a'.repeat(LONGEST,),
  'x.y',
];

/**
 Names that would be words, options, paths or code if they reached a command.
 */
const INVALID: readonly string[] = [
  '',
  'a'.repeat(LONGEST + 1,),
  'with space',
  'a\tb',
  'line\nbreak',
  'carriage\rreturn',
  'a;b',
  'a&&b',
  'a||b',
  'a|b',
  '$(touch injected)',
  '`touch injected`',
  ['$', '{HOME}',].join('',),
  'a\'b',
  'a"b',
  String.raw`a\b`,
  'a/b',
  'a:b',
  'a*b',
  'a?b',
  'a[b]',
  'a~b',
  'a#b',
  'a!b',
  'a<b',
  'a>b',
  'a\u0000b',
  'esc\u001B[31m',
  'bidi‮',
  'café',
  'a=b',
  'a+b',
  '=',
  '+',
  '-x',
  '--help',
  '-',
  '.',
  '..',
];

/**
 Reads what a call throws.

 @param run - Call expected to throw.

 @returns The thrown value,
 or nothing when it did not throw.

 @example
 ```ts
 await thrownBy({ run: () => Promise.reject(new Error('x')) });
 ```
 */
async function thrownBy({ run, }: { readonly run: () => unknown; },): Promise<unknown> {
  try {
    await run();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: isValidInterfaceName.name,
  children: [
    it({
      name: 'accepts the names of real interfaces, up to the 15 characters Linux allows',
      fn: async () => {
        for (const interfaceName of VALID)
          expect(isValidInterfaceName({ interfaceName, },),).toBe(true,);
      },
    },),
    it({
      name: 'refuses names with spaces, quotes, separators, substitutions, control characters, a leading dash, dots only, or more than 15 characters',
      fn: async () => {
        for (const interfaceName of INVALID)
          expect(isValidInterfaceName({ interfaceName, },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: assertInterfaceName.name,
  children: [
    it({
      name: 'throws a ConfigError that shows the refused name as a quoted string and states the rule',
      fn: async () => {
        /**
         What the check threw.
         */
        const thrown = await thrownBy({ run: () => assertInterfaceName({ interfaceName: 'a b;$(id)\n', },), },);
        expect((thrown as Error).name,).toBe('ConfigError',);
        // The name is shown escaped, so a newline in it cannot add a line to the message.
        expect(String(thrown,),).toContain(String.raw`"a b;$(id)\n" is not a usable interface name`,);
        expect(String(thrown,),).toContain('1 to 15 characters',);
        expect(String(thrown,).includes('\n',),).toBe(false,);
        /**
         The same for a control character of the C1 range and a direction mark,
         which JSON does not escape.
         */
        const odd = await thrownBy({ run: () => assertInterfaceName({ interfaceName: 'a\u009Bb\u202Ec', },), },);
        expect(String(odd,),).toContain(String.raw`"a\u{9b}b\u{202e}c" is not a usable interface name`,);
        expect(String(odd,).includes('\u009B',),).toBe(false,);
        expect(String(odd,).includes('\u202E',),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: interfaceNameOfArgument.name,
  children: [
    it({
      name: 'takes the bare name, or the basename of a config path without .conf',
      fn: async () => {
        expect(interfaceNameOfArgument({ arg: 'wg0', },),).toBe('wg0',);
        expect(interfaceNameOfArgument({ arg: '/etc/wireguard/fr-par-fr1.conf', },),).toBe('fr-par-fr1',);
        expect(interfaceNameOfArgument({ arg: 'dir/wg0', },),).toBe('wg0',);
        expect(interfaceNameOfArgument({ arg: 'wg0.conf', },),).toBe('wg0',);
        expect(interfaceNameOfArgument({ arg: '/etc/wireguard/a;b.conf', },),).toBe('a;b',);
      },
    },),
  ],
},);

await describe({
  name: parseCliArgs.name,
  children: [
    it({
      name: 'accepts up and down for a bare name and for a config path',
      fn: async () => {
        expect(parseCliArgs({ argv: ['up', 'wg0',], },),).toEqual({ subcommand: 'up', target: 'wg0', },);
        expect(parseCliArgs({ argv: ['down', '/etc/wireguard/fr-par-fr1.conf',], },),).toEqual({ subcommand: 'down', target: '/etc/wireguard/fr-par-fr1.conf', },);
      },
    },),
    it({
      name: 'refuses a target whose interface name has a space, a separator, a substitution, a quote or a newline, before anything asks for privilege',
      fn: async () => {
        for (const interfaceName of INVALID.filter((name,) => (name !== '') && (!name.includes('/',)) && (!name.endsWith('.conf',)) && (name !== '.') && (name !== '..'),)) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own error.
          const thrown = await thrownBy({ run: () => parseCliArgs({ argv: ['down', interfaceName,], },), },);
          expect((thrown as Error).name,).toBe('ConfigError',);
        }
        /**
         A config path whose file name is the hostile name.
         */
        const viaPath = await thrownBy({ run: () => parseCliArgs({ argv: ['up', '/etc/wireguard/$(touch injected).conf',], },), },);
        expect((viaPath as Error).name,).toBe('ConfigError',);
      },
    },),
    it({
      name: 'still reports a missing or unknown subcommand and a missing target as a usage error',
      fn: async () => {
        for (const argv of [[], ['sideways', 'wg0',], ['up',],]) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own error.
          const thrown = await thrownBy({ run: () => parseCliArgs({ argv, },), },);
          expect((thrown as Error).name,).toBe('CliUsageError',);
        }
      },
    },),
  ],
},);

await describe({
  name: loadConfig.name,
  children: [
    it({
      name: 'refuses a hostile interface name before it reads any file',
      fn: async () => {
        for (const arg of ['bad name', '$(touch injected)', 'a;b', '/nonexistent/dir/a b.conf',]) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own error.
          const thrown = await thrownBy({ run: () => loadConfig({ arg, expandAllowedIps: false, },), },);
          expect((thrown as Error).name,).toBe('ConfigError',);
          expect(String(thrown,),).toContain('is not a usable interface name',);
        }
      },
    },),
  ],
},);
