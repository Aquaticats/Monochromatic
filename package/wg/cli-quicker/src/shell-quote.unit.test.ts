import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runAllowingFailure, } from '../dist/final/node/runner.mjs';
import { shellQuote, } from '../dist/final/node/shell-quote.mjs';

/**
 Values a hostile or careless environment could put into a path or a name,
 each of which must come back from a shell as that one word.
 NUL is left out because no process argument or path can hold it.
 */
const ADVERSARIAL: readonly string[] = [
  'plain',
  'with space',
  'two  spaces and a trailing space ',
  '; touch injected',
  'a;b',
  'a&&b',
  'a||b',
  'a|b',
  'a & b',
  '$(touch injected)',
  '`touch injected`',
  ['$', '{HOME}',].join('',),
  '$HOME',
  '"double"',
  '\'single\'',
  'it\'s',
  '\'\'',
  '\\backslash\\',
  String.raw`\'`,
  '*',
  '?',
  '[a-z]',
  '~',
  '~root',
  '#comment',
  '!history',
  '> redirect',
  '< redirect',
  '--option',
  '-n',
  '=',
  'a=b',
  '',
  'tab\there',
  'line one\nline two',
  '\ntrailing newline\n',
  'carriage\rreturn',
  'escape\u001B[31mred',
  'bell\u0007',
  'delete\u007F',
  'c1\u0085next-line',
  'bidi‮override',
  'café 日本語',
  'x'.repeat(4_096,),
];

/**
 Runs a shell that prints the quoted value with `printf`,
 and returns what the shell printed.

 @param shell - Shell program.

 @param value - Value to quote and print.

 @returns Standard output of the shell.

 @example
 ```ts
 await printedBy({ shell: 'bash', value: 'a b' });
 ```
 */
async function printedBy({ shell, value, }: { readonly shell: string; readonly value: string; },): Promise<string> {
  /**
   What the shell ran.
   */
  const result = await runAllowingFailure({
    command: shell,
    args: ['-c', `printf '%s' ${shellQuote({ value, },)}`,],
  },);
  return result.exitCode === 0 ? result.stdout : `shell failed with ${String(result.exitCode,)}: ${result.stderr}`;
}

await describe({
  name: shellQuote.name,
  children: [
    it({
      name: 'prints a value of safe characters as it is, and an empty value as two quotes',
      fn: async () => {
        expect(shellQuote({ value: 'fr-par-fr1', },),).toBe('fr-par-fr1',);
        expect(shellQuote({ value: '/run/wg-quicker/interface-0123.json', },),).toBe('/run/wg-quicker/interface-0123.json',);
        expect(shellQuote({ value: '', },),).toBe('\'\'',);
      },
    },),
    it({
      name: 'makes bash read back every adversarial value as the one word it was',
      fn: async () => {
        for (const value of ADVERSARIAL) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case starts its own shell.
          expect(await printedBy({ shell: 'bash', value, },),).toBe(value,);
        }
      },
    },),
    it({
      name: 'makes sh read back every adversarial value without a control character as the one word it was',
      fn: async () => {
        for (const value of ADVERSARIAL.filter((candidate,) => !Array.from(candidate,).some((character,) => ((character.codePointAt(0,) ?? 0) < 0x20) || (((character.codePointAt(0,) ?? 0) >= 0x7F) && ((character.codePointAt(0,) ?? 0) <= 0x9F)),),)) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case starts its own shell.
          expect(await printedBy({ shell: 'sh', value, },),).toBe(value,);
        }
      },
    },),
    it({
      name: 'prints no control character and no line break for any value, so a message line is one line and cannot rewrite the terminal',
      fn: async () => {
        for (const value of ADVERSARIAL) {
          /**
           What is printed for the value.
           */
          const printed = shellQuote({ value, },);
          expect(Array.from(printed,).some((character,) => {
            /**
             Code point of the character.
             */
            const code = character.codePointAt(0,) ?? 0;
            return (code < 0x20) || ((code >= 0x7F) && (code <= 0x9F));
          },),).toBe(false,);
        }
      },
    },),
  ],
},);
