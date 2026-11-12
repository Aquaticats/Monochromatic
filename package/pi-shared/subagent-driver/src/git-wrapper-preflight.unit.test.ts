/**
 Which program a search path resolves a name to, and the preflight that refuses a session whose `git` is not this repository's
 policy wrapper: the program it resolved, the folder it asked in, an answer that is the wrapper, an answer that is the system git,
 an answer that is some other program, an answer that never comes, and a search path holding no git at all.

 Every stand-in is a real program in a folder of its own, so the search, the question and the bound run as they run in a session.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  join,
} from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  GIT_WRAPPER_PREFLIGHT_TIMEOUT_MILLISECONDS,
  GitWrapperPreflightError,
  NO_SUCH_PROGRAM,
  WRAPPER_COMMAND,
  programOnAPath,
  refuseUnlessTheWrapperAnswersGit,
  runCommandWithChildProcess,
} from '../dist/final/node/index.mjs';

/**
 How long one refusal may take in the test that asks for a bound of its own, in milliseconds. It is short enough that a check
 which ignored its bound would still be waiting when the test's own deadline passed.
 */
const SHORT_BOUND_MILLISECONDS = 400;

/**
 How long a check that passed its bound may take before the test calls it a wait that never ended, in milliseconds.
 */
const PATIENCE_MILLISECONDS = 20_000;

/**
 A folder for one test, removed when the test ends.

 @returns the folder's path, and the disposer that removes it
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-preflight-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, {
        recursive: true,
        force: true,
      },);
    },
  };
}

/**
 Writes a stand-in program into a folder of its own.

 @param request - where the program goes, the name it answers to, what it does, and what it may be run as

 @returns the path of the program written

 @example
 ```ts
 const git = await writeProgram({ folder: bin, name: 'git', body: 'exit 0\n', },);
 ```
 */
async function writeProgram({
  folder,
  name,
  body,
  mode = 0o700,
}: {
  readonly folder: string;
  readonly name: string;
  readonly body: string;
  readonly mode?: number;
}): Promise<string> {
  /**
   The program's file.
   */
  const file = join(
    folder,
    name,
  );
  await mkdir(
    folder,
    { recursive: true, },
  );
  await writeFile(file, `#!/bin/sh\n${body}`,);
  await chmod(file, mode,);
  return file;
}

/**
 Runs a check that must refuse and hands back what it refused with.

 @param ask - the check, which must not answer

 @returns what the check threw

 @example
 ```ts
 const refusal = await refusalOf(async () => await theCheck(),);
 ```
 */
async function refusalOf(ask: () => Promise<unknown>,): Promise<Error> {
  try {
    await ask();
  }
  catch (error) {
    if (Error.isError(error,)) {
      return error;
    }
    throw new Error(`the check refused with something that is no error: ${String(error,)}`, { cause: error, },);
  }
  throw new Error('the check answered where the test expected a refusal',);
}

await describe({
  name: '',
  children: [
    describe({
      name: programOnAPath.name,
      children: [
        it({
          name: 'takes the first folder of the search path that holds a program this user may run',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A folder whose git nobody made executable.
             */
            const locked = join(
              folder.path,
              'locked',
            );
            await writeProgram({
              folder: locked,
              name: 'git',
              body: 'exit 0\n',
              mode: 0o600,
            },);
            /**
             The folder holding the program the search may run.
             */
            const runnable = join(
              folder.path,
              'runnable',
            );
            /**
             The program the search must answer with.
             */
            const program = await writeProgram({
              folder: runnable,
              name: 'git',
              body: 'exit 0\n',
            },);
            expect(await programOnAPath({
              name: 'git',
              path: [
                locked,
                runnable,
              ].join(delimiter,),
              folder: folder.path,
            },),).toBe(program,);
          },
        },),
        it({
          name: 'passes a folder holding a folder of the program name, because a folder cannot run',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A folder named after the program, which is executable in the POSIX sense without being one.
             */
            const aFolder = join(
              folder.path,
              'a-folder',
            );
            await mkdir(
              join(
                aFolder,
                'git',
              ),
              { recursive: true, },
            );
            /**
             The program the search must answer with.
             */
            const runnable = join(
              folder.path,
              'runnable',
            );
            /**
             The program of the second folder.
             */
            const program = await writeProgram({
              folder: runnable,
              name: 'git',
              body: 'exit 0\n',
            },);
            expect(await programOnAPath({
              name: 'git',
              path: [
                aFolder,
                runnable,
              ].join(delimiter,),
              folder: folder.path,
            },),).toBe(program,);
          },
        },),
        it({
          name: 'takes an empty entry as the folder the search starts in',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The program in the folder the search starts in.
             */
            const program = await writeProgram({
              folder: folder.path,
              name: 'git',
              body: 'exit 0\n',
            },);
            /**
             The folder holding the program the search must not reach.
             */
            const other = join(
              folder.path,
              'other',
            );
            await writeProgram({
              folder: other,
              name: 'git',
              body: 'exit 0\n',
            },);
            expect(await programOnAPath({
              name: 'git',
              path: [
                '',
                other,
              ].join(delimiter,),
              folder: folder.path,
            },),).toBe(program,);
          },
        },),
        it({
          name: 'takes a relative entry of the search path against the folder the search starts in',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder a relative entry names.
             */
            const relative = join(
              folder.path,
              'relative',
            );
            /**
             The program the search must answer with.
             */
            const program = await writeProgram({
              folder: relative,
              name: 'git',
              body: 'exit 0\n',
            },);
            expect(await programOnAPath({
              name: 'git',
              path: relative.slice(folder.path.length + 1,),
              folder: folder.path,
            },),).toBe(program,);
          },
        },),
        it({
          name: 'answers nothing when no folder of the search path holds the program',
          fn: async () => {
            await using folder = await makeFolder();
            expect(await programOnAPath({
              name: 'git',
              path: [
                join(
                  folder.path,
                  'empty',
                ),
                join(
                  folder.path,
                  'also-empty',
                ),
              ].join(delimiter,),
              folder: folder.path,
            },),).toBe(NO_SUCH_PROGRAM,);
          },
        },),
        it({
          name: 'lets a failure of the search itself through instead of answering that no folder holds the program',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A name no file system accepts, which is a failure other than a program that is missing or unrunnable.
             */
            const tooLong = 'a'.repeat(300,);
            await expect(programOnAPath({
              name: tooLong,
              path: folder.path,
              folder: folder.path,
            },),).rejects.toThrow('ENAMETOOLONG',);
          },
        },),
        it({
          name: 'stops at the program a session would run, so a later folder it never reaches cannot refuse the session',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder holding the program the session would run.
             */
            const bin = join(
              folder.path,
              'bin',
            );
            /**
             The program the search must answer with, even though a later folder cannot be looked at.
             */
            const program = await writeProgram({
              folder: bin,
              name: 'git',
              body: 'exit 0\n',
            },);
            expect(await programOnAPath({
              name: 'git',
              path: [
                bin,
                'a'.repeat(300,),
              ].join(delimiter,),
              folder: folder.path,
            },),).toBe(program,);
          },
        },),
        it({
          name: 'reads a folder holding a link and .. the way the kernel does, not the way tidying the string would',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder the link points into, which is where the program is.
             */
            const deep = join(
              folder.path,
              'deep',
            );
            await writeProgram({
              folder: join(
                deep,
                'bin',
              ),
              name: 'git',
              body: 'exit 0\n',
            },);
            await mkdir(join(
              deep,
              'a',
            ), { recursive: true, },);
            // A link named a, pointing at deep/a: `a/..` is deep when the kernel reads it, and the folder holding that link when
            // the string is tidied first, which holds no bin.
            await symlink(
              join(
                deep,
                'a',
              ),
              join(
                folder.path,
                'a',
              ),
            );
            expect(await programOnAPath({
              name: 'git',
              path: `${folder.path}/a/../bin`,
              folder: folder.path,
            },),).toBe(`${folder.path}/a/../bin/git`,);
          },
        },),
      ],
    },),
    describe({
      name: refuseUnlessTheWrapperAnswersGit.name,
      children: [
        it({
          name: 'answers with the git the session would run, and asks it for the policy namespace help in the folder the session works in',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder of the session's own search path.
             */
            const bin = join(
              folder.path,
              'bin',
            );
            /**
             The folder the session would work in.
             */
            const work = join(
              folder.path,
              'work',
            );
            await mkdir(
              work,
              { recursive: true, },
            );
            /**
             Where the stand-in records the folder it ran in and the question it was asked.
             */
            const asked = join(
              folder.path,
              'asked.log',
            );
            /**
             The wrapper the session would run.
             */
            const git = await writeProgram({
              folder: bin,
              name: 'git',
              body: `pwd -P >> '${asked}'\nprintf '  %s\\n' "$*" >> '${asked}'\nprintf '%s\\n' 'Usage: git monochromatic-git-policy-cli <command> [options]'\nexit 0\n`,
            },);
            expect(await refuseUnlessTheWrapperAnswersGit({
              run: runCommandWithChildProcess,
              sessionPath: [
                bin,
                '/usr/bin',
              ].join(delimiter,),
              workingFolder: work,
            },),).toBe(git,);
            expect((await readFile(asked, 'utf8',)).trim(),).toBe(
              `${await realpath(work,)}\n  ${WRAPPER_COMMAND} --help`,
            );
          },
        },),
        it({
          name: 'refuses, naming the git it found and the fix, when that git answers as the system git',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder of the session's own search path.
             */
            const bin = join(
              folder.path,
              'bin',
            );
            /**
             The system git's own answer to the wrapper's subcommand.
             */
            const git = await writeProgram({
              folder: bin,
              name: 'git',
              body: `printf '%s\\n' "git: 'monochromatic-git-policy-cli' is not a git command. See 'git --help'." >&2\nexit 1\n`,
            },);
            /**
             What the check refused with.
             */
            const refusal = await refusalOf(async () => await refuseUnlessTheWrapperAnswersGit({
              run: runCommandWithChildProcess,
              sessionPath: bin,
              workingFolder: folder.path,
            },),);
            expect(refusal,).toBeInstanceOf(GitWrapperPreflightError,);
            expect(refusal.message,).toContain(git,);
            expect(refusal.message,).toContain('answered 1',);
            expect(refusal.message,).toContain('is not a git command',);
            expect(refusal.message,).toContain('.cache/monochromatic-git-policy-cli-bin',);
          },
        },),
        it({
          name: 'refuses a program that answers with status zero, the wrapper\'s own command word and something other than its help',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder of the session's own search path.
             */
            const bin = join(
              folder.path,
              'bin',
            );
            /**
             A git that prints its own version and nothing about the policies.
             */
            const git = await writeProgram({
              folder: bin,
              name: 'git',
              body: `printf '%s\\n' "'monochromatic-git-policy-cli' is aliased to 'commit --message'"\nexit 0\n`,
            },);
            /**
             What the check refused with.
             */
            const refusal = await refusalOf(async () => await refuseUnlessTheWrapperAnswersGit({
              run: runCommandWithChildProcess,
              sessionPath: bin,
              workingFolder: folder.path,
            },),);
            expect(refusal,).toBeInstanceOf(GitWrapperPreflightError,);
            expect(refusal.message,).toContain(git,);
            expect(refusal.message,).toContain('answered 0',);
            expect(refusal.message,).toContain('policy namespace help',);
          },
        },),
        it({
          name: 'refuses within its bound when the git it found never answers, and names the bound',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder of the session's own search path.
             */
            const bin = join(
              folder.path,
              'bin',
            );
            /**
             A git that never answers. It replaces itself with the sleep, so the process the bound ends is the one still running
             and no wait outlives the test.
             */
            const git = await writeProgram({
              folder: bin,
              name: 'git',
              body: 'exec sleep 600\n',
            },);
            /**
             When the check was asked.
             */
            const started = Date.now();
            /**
             What the check refused with.
             */
            const refusal = await refusalOf(async () => await refuseUnlessTheWrapperAnswersGit({
              run: runCommandWithChildProcess,
              sessionPath: [
                bin,
                '/usr/bin',
              ].join(delimiter,),
              workingFolder: folder.path,
              timeoutMilliseconds: SHORT_BOUND_MILLISECONDS,
            },),);
            expect(Date.now() - started,).toBeLessThan(PATIENCE_MILLISECONDS,);
            expect(refusal,).toBeInstanceOf(GitWrapperPreflightError,);
            expect(refusal.message,).toContain(git,);
            expect(refusal.message,).toContain(`did not answer within ${String(SHORT_BOUND_MILLISECONDS,)} milliseconds`,);
          },
        },),
        it({
          name: 'refuses when no folder of the session PATH holds a git this user may run, and names the PATH it searched',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder of the session's own search path, which holds no program.
             */
            const bin = join(
              folder.path,
              'bin',
            );
            await mkdir(
              bin,
              { recursive: true, },
            );
            /**
             What the check refused with.
             */
            const refusal = await refusalOf(async () => await refuseUnlessTheWrapperAnswersGit({
              run: runCommandWithChildProcess,
              sessionPath: bin,
              workingFolder: folder.path,
            },),);
            expect(refusal,).toBeInstanceOf(GitWrapperPreflightError,);
            expect(refusal.message,).toContain(bin,);
            expect(refusal.message,).toContain('.cache/monochromatic-git-policy-cli-bin',);
          },
        },),
        it({
          name: 'refuses a session with no PATH at all, rather than guessing one',
          fn: async () => {
            await using folder = await makeFolder();
            expect((await refusalOf(async () => await refuseUnlessTheWrapperAnswersGit({
              run: runCommandWithChildProcess,
              workingFolder: folder.path,
            },),)).message,).toContain('no PATH at all',);
          },
        },),
        it({
          name: 'searches an empty PATH as the folder the session works in, and refuses by that folder when it holds no git',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What the check refused with.
             */
            const refusal = await refusalOf(async () => await refuseUnlessTheWrapperAnswersGit({
              run: runCommandWithChildProcess,
              sessionPath: '',
              workingFolder: folder.path,
            },),);
            expect(refusal,).toBeInstanceOf(GitWrapperPreflightError,);
            expect(refusal.message,).toContain('no folder of the PATH it would get holds a git',);
            expect(refusal.message,).toContain('empty, so it searches only the folder the session works in',);
          },
        },),
        it({
          name: 'answers with a wrapper an empty PATH finds in the folder the session works in',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The wrapper standing in the folder the session works in, which an empty PATH searches alone.
             */
            const git = await writeProgram({
              folder: folder.path,
              name: 'git',
              body: `printf '%s\\n' 'Usage: git monochromatic-git-policy-cli <command> [options]'\nexit 0\n`,
            },);
            expect(await refuseUnlessTheWrapperAnswersGit({
              run: runCommandWithChildProcess,
              sessionPath: '',
              workingFolder: folder.path,
            },),).toBe(git,);
          },
        },),
        it({
          name: 'refuses a session whose working folder is not there, rather than asking a question about no folder',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A folder the session would work in, which is not there.
             */
            const gone = join(
              folder.path,
              'gone',
            );
            expect((await refusalOf(async () => await refuseUnlessTheWrapperAnswersGit({
              run: runCommandWithChildProcess,
              sessionPath: folder.path,
              workingFolder: gone,
            },),)).message,).toContain(gone,);
          },
        },),
        it({
          name: 'checks the wrapper within the time this repository ships, so a silent git cannot hold a launch forever',
          fn: async () => {
            expect(GIT_WRAPPER_PREFLIGHT_TIMEOUT_MILLISECONDS,).toBeGreaterThan(0,);
            expect(GIT_WRAPPER_PREFLIGHT_TIMEOUT_MILLISECONDS,).toBeLessThan(60_000,);
          },
        },),
      ],
    },),
  ],
},);
