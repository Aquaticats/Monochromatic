/**
 Comparing what a statement says it ran with the commands a session records: a built program wherever its build folder stands, a
 program a command runs rather than one it merely names, the exit code, the scripts a command runs, and whether a build ran.

 Every case hands the comparison a statement, a session reading and a review folder named by the test, so nothing is read from the
 machine and nothing is started.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
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
  type ReviewStatement,
  type SessionReadings,
  compareCommands,
} from '../dist/final/node/index.mjs';

/**
 The folder the fixture worktree stands for.

 @returns the folder's path

 @example
 ```ts
 const worktree = aWorktree();
 ```
 */
function aWorktree(): string {
  return '/worktrees/pi-review-a';
}

/**
 The program a review builds and runs inside its worktree.

 @returns the program's path

 @example
 ```ts
 const program = aBuiltProgram();
 ```
 */
function aBuiltProgram(): string {
  return `${aWorktree()}/package/git-policy/cli/target/debug/wrapper`;
}

/**
 The program a review builds in the shared build directory, outside every worktree.

 @returns the program's path

 @example
 ```ts
 const program = aSharedBuildProgram();
 ```
 */
function aSharedBuildProgram(): string {
  return '/var/mnt/encrypted/cargo-build/debug/deps/repro-abc123';
}

/**
 A statement naming one command.

 @param command - the command the statement lists

 @param ranBuiltProgram - the program the command ran, when it ran one

 @param exit - the code the command exited with

 @returns the statement

 @example
 ```ts
 const statement = aStatement(`${aBuiltProgram()} commit`);
 ```
 */
function aStatement(
  command: string,
  ranBuiltProgram?: string,
  exit = 0,
): ReviewStatement {
  return {
    commit: 'abc1234',
    diff: {
      lines: 12,
      linesRead: 4,
    },
    files: [],
    commands: [{
      command,
      exit,
      ...((ranBuiltProgram === undefined) ? {} : { ranBuiltProgram, }),
    },],
    findings: [],
  };
}

/**
 What a session records for one command.

 @param command - the command it recorded

 @param exitCode - the code a bash result carried, or nothing for a process start

 @param workingFolder - the folder the session started in

 @returns the reading

 @example
 ```ts
 const session = aSession('true');
 ```
 */
function aSession(
  command: string,
  exitCode?: number,
  workingFolder = aWorktree(),
): SessionReadings {
  return {
    workingFolder,
    commands: [{
      command,
      tool: (exitCode === undefined) ? 'process' : 'bash',
      ...((exitCode === undefined) ? {} : { exitCode, }),
    },],
    codemodeCalls: 0,
    reviewer: {
      provider: '',
      model: '',
    },
  };
}

/**
 A folder a test throws away, holding a script.

 @param name - the script's file name

 @param text - the script's text

 @returns the folder and the script's path

 @example
 ```ts
 await using folder = await aFolderWithScript({ name: 'repro.ts', text: '...' });
 ```
 */
async function aFolderWithScript({
  name,
  text,
}: {
  readonly name: string;
  readonly text: string;
}): Promise<{
  readonly path: string;
  readonly script: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-command-matching-',),);
  const script = join(path, name,);
  await mkdir(join(path, 'unused',), { recursive: true, },);
  await writeFile(script, text, 'utf8',);
  return {
    path,
    script,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: compareCommands.name,
      children: [
        it({
          name: 'confirms a command the session records, its exit code, and the built program in the worktree it ran',
          fn: async () => {
            const command = `${aBuiltProgram()} commit -m a`;
            const compared = await compareCommands({
              statement: aStatement(command, aBuiltProgram(),),
              session: aSession(command, 0,),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).toEqual([],);
            expect(compared.builtPrograms,).toEqual([aBuiltProgram(),],);
            expect(compared.buildCommandRan,).toBe(false,);
          },
        },),
        it({
          name: 'does not confirm a built program outside the worktree, in a build directory tied to no worktree (F3 limit)',
          fn: async () => {
            const command = `${aSharedBuildProgram()} --version`;
            const compared = await compareCommands({
              statement: aStatement(command, aSharedBuildProgram(),),
              session: aSession(command, 0,),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).not.toEqual([],);
            expect(compared.problems[0],).toContain('not a program a build wrote in this review\'s worktree or its own folder');
            expect(compared.builtPrograms,).toEqual([],);
          },
        },),
        it({
          name: 'confirms a built program although the session folder and the program are written through two names of this machine (F6)',
          fn: async () => {
            /**
             The program under a name of this machine the session's folder does not prefix.
             */
            const program = '/var/home/user/worktrees/pi-review-a/package/a/target/debug/a';
            const command = `${program} --version`;
            const compared = await compareCommands({
              statement: aStatement(command, program,),
              session: aSession(command, 0, '/home/user/worktrees/pi-review-a',),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).toEqual([],);
            expect(compared.builtPrograms,).toEqual([program,],);
          },
        },),
        it({
          name: 'does not count a file merely named under a build folder as a program that ran (F7)',
          fn: async () => {
            const command = 'ls package/a/dist/x.js';
            const compared = await compareCommands({
              statement: aStatement(command,),
              session: aSession(command, 0,),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).toEqual([],);
            expect(compared.builtPrograms,).toEqual([],);
          },
        },),
        it({
          name: 'counts a program run as a script the interpreter runs',
          fn: async () => {
            const command = 'node package/a/dist/x.js';
            const compared = await compareCommands({
              statement: aStatement(command, `${aWorktree()}/package/a/dist/x.js`,),
              session: aSession(command, 0,),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).toEqual([],);
            expect(compared.builtPrograms,).toEqual([`${aWorktree()}/package/a/dist/x.js`,],);
          },
        },),
        it({
          name: 'confirms a program a script the command runs holds, and not one held by a script it merely names',
          fn: async () => {
            await using folder = await aFolderWithScript({
              name: 'repro.ts',
              text: `const WRAPPER = '${aBuiltProgram()}';\n`,
            },);
            const run = 'node repro.ts';
            const ran = await compareCommands({
              statement: aStatement(run, aBuiltProgram(),),
              session: aSession(run, 0,),
              reviewFolder: folder.path,
            },);
            expect(ran.problems,).toEqual([],);
            expect(ran.builtPrograms,).toEqual([aBuiltProgram(),],);
            const named = 'cat repro.ts';
            const read = await compareCommands({
              statement: aStatement(named, aBuiltProgram(),),
              session: aSession(named, 0,),
              reviewFolder: folder.path,
            },);
            expect(read.problems.join('\n',),).toContain('neither the command nor a script it runs holds that path');
          },
        },),
        it({
          name: 'says a build ran when a command runs a build tool, or a script it runs holds one',
          fn: async () => {
            await using folder = await aFolderWithScript({
              name: 'build.sh',
              text: 'exec mise exec -- cargo build\n',
            },);
            const direct = await compareCommands({
              statement: aStatement('mise run //package/a:native:build',),
              session: aSession('mise run //package/a:native:build', 0,),
              reviewFolder: folder.path,
            },);
            expect(direct.buildCommandRan,).toBe(true,);
            const scripted = await compareCommands({
              statement: aStatement('bash build.sh',),
              session: aSession('bash build.sh', 0,),
              reviewFolder: folder.path,
            },);
            expect(scripted.buildCommandRan,).toBe(true,);
            const neither = await compareCommands({
              statement: aStatement('rg --count change package/a/src',),
              session: aSession('rg --count change package/a/src', 0,),
              reviewFolder: folder.path,
            },);
            expect(neither.buildCommandRan,).toBe(false,);
          },
        },),
        it({
          name: 'names a stated command the session records nowhere',
          fn: async () => {
            const compared = await compareCommands({
              statement: aStatement(aBuiltProgram(),),
              session: aSession('rg --count change package/a/src', 0,),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems[0],).toContain('the session records no call whose text holds it');
          },
        },),
        it({
          name: 'names an exit code that is not the one the session records',
          fn: async () => {
            const command = `${aBuiltProgram()} commit`;
            const compared = await compareCommands({
              statement: aStatement(command, aBuiltProgram(), 0,),
              session: aSession(command, 1,),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems[0],).toContain('exited 0 and the session records 1');
          },
        },),
        it({
          name: 'says a process start\'s exit code cannot be compared, and still confirms the program it ran',
          fn: async () => {
            const command = `${aBuiltProgram()} commit`;
            const compared = await compareCommands({
              statement: aStatement(command, aBuiltProgram(),),
              session: aSession(command,),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).toEqual([],);
            expect(compared.uncompared[0],).toContain('records no exit code');
            expect(compared.builtPrograms,).toEqual([aBuiltProgram(),],);
          },
        },),
        it({
          name: 'refuses a path with no build output segment, and a session-run built program the statement does not name',
          fn: async () => {
            const plain = await compareCommands({
              statement: aStatement('/usr/local/bin/wrapper commit', '/usr/local/bin/wrapper',),
              session: aSession('/usr/local/bin/wrapper commit', 0,),
              reviewFolder: '/reviews/a',
            },);
            expect(plain.problems[0],).toContain('not a program a build wrote');
            const unstated = await compareCommands({
              statement: aStatement(`${aBuiltProgram()} commit`,),
              session: aSession(`${aBuiltProgram()} commit`, 0,),
              reviewFolder: '/reviews/a',
            },);
            expect(unstated.problems[0],).toContain('does not name that command as one that ran a built program');
          },
        },),
        it({
          name: 'confirms a command whose line breaks differ from the session\'s, which a reviewer writes across lines',
          fn: async () => {
            const compared = await compareCommands({
              statement: aStatement(`cd ${aWorktree()} \\\n  && ${aBuiltProgram()} commit`, aBuiltProgram(),),
              session: aSession(`cd ${aWorktree()} && ${aBuiltProgram()} commit`, 0,),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).toEqual([],);
            expect(compared.builtPrograms,).toEqual([aBuiltProgram(),],);
          },
        },),
      ],
    },),
  ],
},);
