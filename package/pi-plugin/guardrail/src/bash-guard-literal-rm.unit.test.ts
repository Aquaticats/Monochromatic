/**
 Tests for pi's literal-`rm`-path guard on the Bash tool.
 
 @module
 */

import {
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  evaluateBashGuard,
  GUARDRAIL_NOT_BLOCKED,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Disposable working directory the guard resolves script operands against.
 */
const fixtureCwd = await mkdtemp(join(scratchRoot(), 'pi-bash-guard-literal-rm-',),);

/**
 Script holding a computed deletion, which one command runs.
 */
const VIOLATING_SCRIPT = join(fixtureCwd, 'deploy.sh',);

await writeFile(VIOLATING_SCRIPT, '#!/bin/sh\nrm -rf "$d"\n',);

/**
 Commands that delete a path they compute, with the offending word.
 */
const REFUSED_COMMANDS: readonly { readonly command: string; readonly word: string; }[] = [
  { command: 'rm -rf "$d"', word: '(word `"$d"`)', },
  { command: 'sudo rm -rf "$litter"', word: '(word `"$litter"`)', },
  { command: 'env rm -f "$d/$n"', word: '(word `"$d/$n"`)', },
  { command: `bash -c 'rm -rf "\${HOME}/scratch"'`, word: `(word \`"\${HOME}/scratch"\`)`, },
  { command: 'eval \'rm -rf "$d"\'', word: '(word `"$d"`)', },
  { command: String.raw`find . -name build -exec rm -rf {} \;`, word: '(word `rm`)', },
  { command: String.raw`printf '%s\n' build | xargs rm -rf`, word: '(word `rm`)', },
];

/**
 Commands whose deletion writes every path out, which stay allowed.
 */
const ALLOWED_COMMANDS: readonly string[] = [
  'rm --recursive /home/user/temp/agent/scratch',
  'rm -rf \'$d\'',
  'rm -rf ~/temp/agent/scratch',
  'git rm --cached "$file"',
  'rmdir "$d"',
];

//endregion Fixtures

//region Expectations

/**
 Reads the refusal reason of a guard decision.
 
 @param decision - decision the guard returned
 
 @returns the reason text, empty when the call is not blocked
 
 @example
 ```ts
 reasonOf(decision);
 ```
 */
function reasonOf(decision: unknown,): string {
  if (decision === GUARDRAIL_NOT_BLOCKED)
    return '';
  if ((typeof decision) !== 'object')
    return '';
  if (decision === null)
    return '';
  return ('reason' in decision) && ((typeof decision.reason) === 'string')
    ? decision.reason
    : '';
}

//endregion Expectations

await describe({
  name: 'bash literal rm path guard',
  children: [
    //region Refused commands
    describe({
      name: 'a command that deletes a path it computes',
      children: REFUSED_COMMANDS.map(function refusedCommand(fixture,) {
        return it({
          name: `blocks \`${fixture.command}\``,
          fn: async function testRefusedCommand() {
            /**
             Reason the guard gave for the call.
             */
            const reason = reasonOf(await evaluateBashGuard({
              input: { command: fixture.command, },
              cwd: fixtureCwd,
            },),);
            expect(reason,).toContain(fixture.word,);
            expect(reason,).toContain('`rm --recursive /home/user/temp/agent/<folder>/scratch`',);
          },
        },);
      },),
    },),
    //endregion Refused commands

    //region Script files
    describe({
      name: 'a command that runs a script file',
      children: [
        it({
          name: 'blocks a script whose deletion computes its path',
          fn: async function testScriptFile() {
            /**
             Reason the guard gave for the call.
             */
            const reason = reasonOf(await evaluateBashGuard({
              input: { command: `bash ${VIOLATING_SCRIPT}`, },
              cwd: fixtureCwd,
            },),);
            expect(reason,).toContain(VIOLATING_SCRIPT,);
          },
        },),
        it({
          name: 'blocks a here-document body the command writes',
          fn: async function testHereDocument() {
            expect(
              reasonOf(await evaluateBashGuard({
                input: { command: 'bash <<\'EOF\'\nrm -rf "$d"\nEOF', },
                cwd: fixtureCwd,
              },),),
            ).toContain('(word `"$d"`)',);
          },
        },),
      ],
    },),
    //endregion Script files

    //region Allowed commands
    describe({
      name: 'a command that writes every path out',
      children: ALLOWED_COMMANDS.map(function allowedCommand(command,) {
        return it({
          name: `allows \`${command}\``,
          fn: async function testAllowedCommand() {
            expect(
              await evaluateBashGuard({
                input: { command, },
                cwd: fixtureCwd,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },);
      },),
    },),
    //endregion Allowed commands
  ],
},);
