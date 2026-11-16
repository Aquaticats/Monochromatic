/**
 The class three walk: every spelling a start-up file can write a credential in is held against the value real `bash` exports for it.
 A spelling this reader reads must give exactly what the shell gives; one it cannot read must never arm a wrong text, so it either
 takes the value from the environment the driver was started with or stops the run.

 @module
 */

import { spawnSync, } from 'node:child_process';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  exportedCredentialValues,
  SecretEnvironmentUnreadableError,
  writtenValue,
} from '../dist/final/node/index.mjs';

/** The variable every spelling assigns. */
const NAME = 'STAND_IN_SHELL_TOKEN';

/**
 The spellings, each with what the shell does with it.
 */
const SPELLINGS: readonly {
  /**
   What the line is called in this table.
   */
  readonly called: string;

  /**
   The right-hand side of the assignment, as the file writes it.
   */
  readonly written: string;
}[] = [
  { called: 'a plain word', written: 'madeupsecretvalue', },
  { called: 'a double-quoted value', written: '"made up secret value"', },
  { called: 'a single-quoted value', written: '\'made up secret value\'', },
  { called: 'a value beginning with a hash', written: '#madeupsecretvalue', },
  { called: 'a value with a comment after it', written: 'madeupsecretvalue # a note', },
  { called: 'a value with a semicolon after it', written: 'madeupsecretvalue; :', },
  { called: 'a value with a command after it', written: 'madeupsecretvalue && true', },
  { called: 'a value with a pipe after it', written: 'madeupsecretvalue | true', },
  { called: 'a value with a redirect after it', written: 'madeupsecretvalue> /dev/null', },
  { called: 'a concatenation of a quoted and an unquoted part', written: '"madeup"secretvalue', },
  { called: 'an escaped space', written: String.raw`madeupsecret\ value`, },
  { called: 'a value the shell computes', written: '$(printf %s madeupsecretvalue)', },
  { called: 'a value from another variable', written: `"\${STAND_IN_OTHER}"`, },
  { called: 'a backslash inside double quotes', written: String.raw`"madeup\secretvalue"`, },
];

/**
 The value real bash exports for one spelling, or nothing when bash refuses the line.
 
 @param written - the right-hand side
 
 @returns what the shell exported
 */
function whatBashExports({ written, }: { readonly written: string },): string {
  /**
   A real shell, run with no environment of its own, printing what it exported.
   */
  const shell = spawnSync('bash', ['--noprofile', '--norc', '-c', `export ${NAME}=${written}\nprintf %s "\$${NAME}"`,], {
    encoding: 'utf8',
    env: {},
  },);
  if (shell.status !== 0) {
    throw new Error(`bash refused ${written}: ${shell.stderr}`,);
  }
  return shell.stdout;
}

await describe({
  name: 'a shell value is what the shell exports, or a stop',
  children: SPELLINGS.map(function eachSpelling(spelling,) {
    return it({
      name: `${spelling.called} is either the value bash exports or a line this reader refuses`,
      fn: async () => {
        /**
         What the shell exported for the line.
         */
        const exported = whatBashExports({ written: spelling.written, },);
        /**
         What this reader makes of the same line.
         */
        const read = writtenValue({ written: spelling.written, },);
        if ('value' in read) {
          // A line this reader reads must give exactly what the shell gives, and never a control operator the shell swallowed.
          expect(read.value,).toBe(exported,);
          return;
        }
        // A line it cannot read is one it never arms: the reader says so, and the run either finds the value by name or stops.
        expect(read,).not.toHaveProperty('value',);
        expect(() => {
          exportedCredentialValues({
            text: `export ${NAME}=${spelling.written}\n`,
            fileName: '.bashrc',
            environment: {},
          },);
        },).toThrow(SecretEnvironmentUnreadableError,);
        // The other answer: when the environment the driver was started with holds the name, that exact value is armed. An empty
        // exported value is nothing a run is asked to mask, so it is not walked here.
        if (exported === '') {
          return;
        }
        /**
         What the reader arms from the environment, by name.
         */
        const fromTheEnvironment = exportedCredentialValues({
          text: `export ${NAME}=${spelling.written}\n`,
          fileName: '.bashrc',
          environment: { [NAME]: exported, },
        },);
        expect(fromTheEnvironment,).toEqual([{ name: `.bashrc ${NAME}`, value: exported, },],);
      },
    },);
  },),
},);
