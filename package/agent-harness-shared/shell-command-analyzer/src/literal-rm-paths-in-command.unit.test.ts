/**
 Tests for the script files one shell command reads from disk.
 
 @module
 */

import {
  mkdir,
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
  findLiteralRmPathsInCommand,
  NO_SCRIPT_FILE_OPERAND,
  scriptFilesACommandRuns,
  shellFileOperand,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 Working directory the fixtures were built in.
 */
const fixtureRoot = await mkdtemp(join(scratchRoot(), 'literal-rm-script-files-',),);

/**
 Shell script holding a computed deletion.
 */
const VIOLATING_SCRIPT_PATH = join(fixtureRoot, 'deploy.sh',);

/**
 Shell script holding only literal paths.
 */
const CLEAN_SCRIPT_PATH = join(fixtureRoot, 'clean.sh',);

/**
 Executable with no extension whose first line asks for a shell.
 */
const SHEBANG_SCRIPT_PATH = join(fixtureRoot, 'cleanup',);

/**
 Executable with no extension whose first line asks for another language.
 */
const PYTHON_SCRIPT_PATH = join(fixtureRoot, 'tool',);

/**
 Shell script the command reads through a here-document rather than a word.
 */
const HERE_DOCUMENT_SCRIPT_PATH = join(fixtureRoot, 'generated.sh',);

await writeFile(VIOLATING_SCRIPT_PATH, '#!/bin/sh\nrm -rf "$d"\n',);
await writeFile(CLEAN_SCRIPT_PATH, '#!/bin/sh\nrm -rf /home/user/temp/agent/scratch\n',);
await writeFile(SHEBANG_SCRIPT_PATH, '#!/bin/bash\nrm -f "$d/$n"\n',);
await writeFile(PYTHON_SCRIPT_PATH, '#!/usr/bin/python3\nprint(1)\n',);
await writeFile(HERE_DOCUMENT_SCRIPT_PATH, '#!/bin/sh\nrm -rf "$generated"\n',);

/**
 Folder the here-document is written into, which the command creates first.
 */
const GENERATED_FOLDER = join(fixtureRoot, 'generated',);

await mkdir(GENERATED_FOLDER,);

//endregion Fixtures

//region Expectations

/**
 Reads the offending words of the files one command reads.
 
 @param command - shell command to read
 
 @returns offending words, in order
 
 @example
 ```ts
 await wordsOfCommand('bash clean.sh');
 ```
 */
async function wordsOfCommand(command: string,): Promise<readonly string[]> {
  return (await findLiteralRmPathsInCommand({
    command,
    cwd: fixtureRoot,
  },))
    .map(function wordOf(finding,): string {
      return finding.word;
    },);
}

/**
 Reads the paths of the files one command reads.
 
 @param command - shell command to read
 
 @returns absolute paths of the candidates, in order
 
 @example
 ```ts
 pathsOfCommand('bash clean.sh');
 ```
 */
function pathsOfCommand(command: string,): readonly string[] {
  return scriptFilesACommandRuns({
    command,
    cwd: fixtureRoot,
  },)
    .map(function pathOf(candidate,): string {
      return candidate.path;
    },);
}

//endregion Expectations

await describe({
  name: '',
  children: [
    describe({
      name: shellFileOperand.name,
      children: [
        it({
          name: 'reads the file operand of a shell named first',
          fn: async function testPlainShell() {
            expect(shellFileOperand(['bash', 'clean.sh',],),).toBe('clean.sh',);
          },
        },),
        it({
          name: 'skips shell flags, and the values of the options that take one',
          fn: async function testShellFlags() {
            expect(shellFileOperand(['bash', '-x', '-o', 'pipefail', 'clean.sh',],),).toBe('clean.sh',);
          },
        },),
        it({
          name: 'reads no file when the shell takes its script from a flag cluster',
          fn: async function testScriptFlag() {
            expect(shellFileOperand(['bash', '-lc', 'echo hi',],),).toBe(NO_SCRIPT_FILE_OPERAND,);
            expect(shellFileOperand(['sh', '-eu', 'clean.sh',],),).toBe('clean.sh',);
          },
        },),
        it({
          name: 'reads the file a source builtin names',
          fn: async function testSourceBuiltin() {
            expect(shellFileOperand(['source', './env.sh',],),).toBe('./env.sh',);
            expect(shellFileOperand(['.', './env.sh',],),).toBe('./env.sh',);
          },
        },),
        it({
          name: 'reads no file when only a program is named',
          fn: async function testProgramOnly() {
            expect(shellFileOperand(['bash',],),).toBe(NO_SCRIPT_FILE_OPERAND,);
          },
        },),
      ],
    },),
    describe({
      name: scriptFilesACommandRuns.name,
      children: [
        it({
          name: 'names the script a shell reads',
          fn: async function testShellScript() {
            expect(pathsOfCommand('bash deploy.sh',),).toEqual([VIOLATING_SCRIPT_PATH,],);
          },
        },),
        it({
          name: 'names the script a shell reached through a wrapper reads',
          fn: async function testWrappedShell() {
            expect(pathsOfCommand(`env sh ${CLEAN_SCRIPT_PATH}`,),).toEqual([CLEAN_SCRIPT_PATH,],);
          },
        },),
        it({
          name: 'names the script a busybox applet reads',
          fn: async function testBusyboxShell() {
            expect(pathsOfCommand('busybox sh deploy.sh',),).toEqual([VIOLATING_SCRIPT_PATH,],);
          },
        },),
        it({
          name: 'names the file a source builtin reads',
          fn: async function testSourcedFile() {
            expect(pathsOfCommand('source clean.sh',),).toEqual([CLEAN_SCRIPT_PATH,],);
          },
        },),
        it({
          name: 'names a path word that carries a separator',
          fn: async function testPathWord() {
            expect(pathsOfCommand('./cleanup',),).toEqual([SHEBANG_SCRIPT_PATH,],);
          },
        },),
        it({
          name: 'names a script a nested shell string would read',
          fn: async function testNestedShellString() {
            expect(pathsOfCommand('bash -c \'bash deploy.sh\'',),).toEqual([VIOLATING_SCRIPT_PATH,],);
          },
        },),
        it({
          name: 'names no file for a command that reads none',
          fn: async function testNoFile() {
            expect(pathsOfCommand('ls -la',),).toEqual([],);
          },
        },),
        it({
          name: 'terminates on a command that runs its own text',
          fn: async function testSelfLaunching() {
            expect(pathsOfCommand('eval "$0"',),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: findLiteralRmPathsInCommand.name,
      children: [
        it({
          name: 'refuses a deletion the command text spells',
          fn: async function testCommandText() {
            expect(await wordsOfCommand('rm -rf "$d"',),).toEqual(['"$d"',],);
          },
        },),
        it({
          name: 'refuses a deletion inside the script the command runs',
          fn: async function testScriptFile() {
            expect(await wordsOfCommand('bash deploy.sh',),).toEqual(['"$d"',],);
          },
        },),
        it({
          name: 'refuses a deletion inside a shebang script named by path',
          fn: async function testShebangFile() {
            expect(await wordsOfCommand('./cleanup',),).toEqual(['"$d/$n"',],);
          },
        },),
        it({
          name: 'refuses a deletion inside a file a source builtin reads',
          fn: async function testSourcedViolation() {
            await writeFile(join(fixtureRoot, 'env.sh',), 'rm -rf "$d"\n',);
            expect(await wordsOfCommand('source env.sh',),).toEqual(['"$d"',],);
          },
        },),
        it({
          name: 'allows a script whose paths are all written out',
          fn: async function testCleanScript() {
            expect(await wordsOfCommand('bash clean.sh',),).toEqual([],);
          },
        },),
        it({
          name: 'allows a path word whose first line names another language',
          fn: async function testOtherLanguage() {
            expect(await wordsOfCommand('./tool',),).toEqual([],);
          },
        },),
        it({
          name: 'gives no finding from a script file that cannot be read',
          fn: async function testMissingFile() {
            expect(await wordsOfCommand('bash missing.sh',),).toEqual([],);
          },
        },),
        it({
          name: 'refuses a deletion inside a here-document body the command writes',
          fn: async function testHereDocumentBody() {
            expect(
              await wordsOfCommand('bash <<\'EOF\'\nrm -rf "$d"\nEOF',),
            ).toEqual(['"$d"',],);
          },
        },),
      ],
    },),
  ],
},);
