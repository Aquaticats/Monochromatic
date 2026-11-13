/**
 Comparing what a statement says it ran with the commands a session records: the command text, the exit code, the built program a
 command ran, and the claims the comparison can only report as unconfirmed.

 Every case hands the comparison a statement, a session reading and a review folder the test names, so nothing is read or started.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
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
 The program a review builds and runs.

 @returns the program's path

 @example
 ```ts
 const program = aBuiltProgram();
 ```
 */
function aBuiltProgram(): string {
  return `${aWorktree()}/package/git-policy/cli/target/debug/monochromatic-git-policy-cli`;
}

/**
 A statement naming one command that ran the built program.

 @returns the statement

 @example
 ```ts
 const statement = aStatement();
 ```
 */
function aStatement(): ReviewStatement {
  return {
    commit: 'abc1234',
    diff: {
      lines: 12,
      linesRead: 4,
    },
    files: [],
    commands: [{
      command: `${aBuiltProgram()} commit -m a`,
      exit: 0,
      ranBuiltProgram: aBuiltProgram(),
    },],
    findings: [],
  };
}

/**
 What a session records for one command.

 @param command - the command it recorded

 @param exitCode - the code a bash result carried, or nothing for a process start

 @returns the reading

 @example
 ```ts
 const session = aSession({ command: 'true' });
 ```
 */
function aSession({
  command,
  exitCode,
}: {
  readonly command: string;
  readonly exitCode?: number;
},): SessionReadings {
  return {
    workingFolder: aWorktree(),
    commands: [{
      command,
      tool: (exitCode === undefined) ? 'process' : 'bash',
      ...((exitCode === undefined) ? {} : { exitCode, }),
    },],
    codemodeCalls: 0,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: compareCommands.name,
      children: [
        it({
          name: 'confirms a command the session records, its exit code, and the built program it names',
          fn: async () => {
            const compared = await compareCommands({
              statement: aStatement(),
              session: aSession({
                command: `${aBuiltProgram()} commit -m a`,
                exitCode: 0,
              },),
              reviewFolder: '/reviews/a',
            },);
            expect(compared,).toEqual({
              problems: [],
              uncompared: [],
              builtPrograms: [aBuiltProgram(),],
            },);
          },
        },),
        it({
          name: 'confirms a command whose line breaks differ from the session\'s, which a reviewer writes across lines',
          fn: async () => {
            const compared = await compareCommands({
              statement: {
                ...aStatement(),
                commands: [{
                  command: `cd ${aWorktree()} \\\n  && ${aBuiltProgram()} commit -m a`,
                  exit: 0,
                  ranBuiltProgram: aBuiltProgram(),
                },],
              },
              session: aSession({
                command: `cd ${aWorktree()} && ${aBuiltProgram()} commit -m a`,
                exitCode: 0,
              },),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).toEqual([],);
            expect(compared.builtPrograms,).toEqual([aBuiltProgram(),],);
          },
        },),
        it({
          name: 'names a stated command the session records nowhere',
          fn: async () => {
            const compared = await compareCommands({
              statement: aStatement(),
              session: aSession({
                command: 'rg --count change package/a/src',
                exitCode: 0,
              },),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems[0],).toContain('the session records no call whose text holds it');
            expect(compared.builtPrograms,).toEqual([],);
          },
        },),
        it({
          name: 'names an exit code that is not the one the session records',
          fn: async () => {
            const compared = await compareCommands({
              statement: aStatement(),
              session: aSession({
                command: `${aBuiltProgram()} commit -m a`,
                exitCode: 1,
              },),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems[0],).toContain('exited 0 and the session records 1');
          },
        },),
        it({
          name: 'says a process start\'s exit code cannot be compared, and still confirms the program it ran',
          fn: async () => {
            const compared = await compareCommands({
              statement: aStatement(),
              session: aSession({ command: `${aBuiltProgram()} commit -m a`, },),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).toEqual([],);
            expect(compared.uncompared[0],).toContain('records no exit code');
            expect(compared.builtPrograms,).toEqual([aBuiltProgram(),],);
          },
        },),
        it({
          name: 'refuses a built-program path outside a build folder of the worktree',
          fn: async () => {
            const compared = await compareCommands({
              statement: {
                ...aStatement(),
                commands: [{
                  command: 'monochromatic-git-policy-cli commit -m a',
                  exit: 0,
                  ranBuiltProgram: '/usr/local/bin/monochromatic-git-policy-cli',
                },],
              },
              session: aSession({
                command: 'monochromatic-git-policy-cli commit -m a',
                exitCode: 0,
              },),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems[0],).toContain('not a path under a build folder of the review\'s worktree');
          },
        },),
        it({
          name: 'refuses a statement that names no built program although the session ran one',
          fn: async () => {
            const compared = await compareCommands({
              statement: {
                ...aStatement(),
                commands: [{
                  command: `${aBuiltProgram()} commit -m a`,
                  exit: 0,
                },],
              },
              session: aSession({
                command: `${aBuiltProgram()} commit -m a`,
                exitCode: 0,
              },),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems[0],).toContain('does not name that command as one that ran a built program');
          },
        },),
        it({
          name: 'says nothing about a session command that ran no built program',
          fn: async () => {
            const compared = await compareCommands({
              statement: aStatement(),
              session: aSession({
                command: 'rg --count change package/a/src',
                exitCode: 0,
              },),
              reviewFolder: '/reviews/a',
            },);
            expect(compared.problems,).toHaveLength(1,);
            expect(compared.problems[0],).toContain('the session records no call whose text holds it');
          },
        },),
      ],
    },),
  ],
},);
