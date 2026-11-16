import {
  execFile,
} from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  realpath,
  rm,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseArguments } from './native-build-fresh-arguments.ts';
import {
  buildFresh,
  type BuildDeadlines,
  NativeBuildFreshError,
  refuseAForeignTargetDirectory,
} from './native-build-fresh-steps.ts';
import {
  memberOf,
  workspaceOf,
} from './native-build-fresh-workspace.ts';
import {
  createCargoStandIn,
  recordedCargoCalls,
} from './stand-in-cargo-for-tests.ts';

/**
 Scratch parent of every fixture directory, resolved so a path Cargo names back is the same text as the fixture's own;
 never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  await realpath(join(
    homedir(),
    'temp',
    'agent',
  ),),
);

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 Runs one program and answers the line it printed.

 @param run - the program, its arguments and its directory

 @returns what the program wrote on standard output, trimmed

 @example
 ```ts
 const printed = await printedBy({ program: '/w/probe', args: [], directory: '/w' });
 ```
 */
async function printedBy({
  program,
  args,
  directory,
}: {
  readonly program: string;
  readonly args: readonly string[];
  readonly directory: string;
}): Promise<string> {
  /**
   What the program printed, which is what a case compares.
   */
  const done = await execFileAsync(
    program,
    [...args],
    {
      encoding: 'utf8',
      cwd: directory,
      maxBuffer: 4_194_304,
    },
  );
  return done.stdout.trim();
}

/**
 Writes a crate whose program prints one word.

 @param directory - the crate directory to write

 @param word - what the program prints

 @example
 ```ts
 await writeCrate({ directory: '/w', word: 'FROM_THESE_SOURCES' });
 ```
 */
async function writeCrate({
  directory,
  word,
}: {
  readonly directory: string;
  readonly word: string;
}): Promise<void> {
  await mkdir(
    join(directory, 'src',),
    { recursive: true },
  );
  await writeFile(
    join(directory, 'Cargo.toml',),
    '[package]\nname = "probe"\nversion = "0.1.0"\nedition = "2021"\n\n[workspace]\n',
  );
  await writeFile(
    join(directory, 'src/main.rs',),
    `fn main() {\n    println!("${word}");\n}\n`,
  );
}

/**
 A fixture directory of this file's own, removed when the case ends.

 @returns the directory and what removes it

 @example
 ```ts
 await using scratch = await scratchDirectory();
 ```
 */
async function scratchDirectory(): Promise<AsyncDisposable & { readonly path: string; }> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  /**
   The directory itself.
   */
  const path = await mkdtemp(join(
    scratchParent,
    'git-policy-build-fresh-case-',
  ));
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 A bound that stops the step at once, for a case that proves what a stopped step says.
 */
const atOnce: BuildDeadlines = {
  metadata: 1,
  clean: 1,
  build: 1,
};

await describe({
  name: 'the build step that leaves this crate\'s own program at the path it names',
  children: [
    //region Reading Cargo's own answer

    it({
      name: 'reads the workspace root, the two directories and the members out of cargo metadata\'s answer',
      fn: async () => {
        /**
         What this step reads out of the answer.
         */
        const seen = workspaceOf(JSON.stringify({
          workspace_root: '/w',
          target_directory: '/w/target',
          build_directory: '/b/xx/yyyy',
          packages: [
            {
              name: 'probe',
              manifest_path: '/w/Cargo.toml',
            },
          ],
        },),);
        expect(seen,).toEqual({
          workspaceRoot: '/w',
          targetDirectory: '/w/target',
          buildDirectory: '/b/xx/yyyy',
          packages: [
            {
              name: 'probe',
              manifestPath: '/w/Cargo.toml',
            },
          ],
        },);
        expect(memberOf({
          seen,
          packageDirectory: '/w',
        },),).toBe('probe',);
      },
    }),
    it({
      name: 'leaves the build directory out when this Cargo names none, so a Cargo that predates build-dir reads the same',
      fn: async () => {
        /**
         The answer of a Cargo that names no build directory.
         */
        const seen = workspaceOf(JSON.stringify({
          workspace_root: '/w',
          target_directory: '/w/target',
          packages: [
            {
              name: 'probe',
              manifest_path: '/w/Cargo.toml',
            },
          ],
        },),);
        expect(seen,).toEqual({
          workspaceRoot: '/w',
          targetDirectory: '/w/target',
          packages: [
            {
              name: 'probe',
              manifestPath: '/w/Cargo.toml',
            },
          ],
        },);
      },
    }),
    it({
      name: 'refuses an answer that names no packages, naming what it could not say',
      fn: async () => {
        /**
         What was thrown.
         */
        let caught: unknown;
        try {
          workspaceOf('{"workspace_root":"/w","target_directory":"/w/target"}',);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('named no packages',);
      },
    }),
    it({
      name: 'refuses a member list that holds no member of this directory, naming the manifest it looked for',
      fn: async () => {
        /**
         What was thrown.
         */
        let caught: unknown;
        try {
          memberOf({
            seen: workspaceOf('{"workspace_root":"/w","target_directory":"/w/target","packages":[{"name":"other","manifest_path":"/w/other/Cargo.toml"}]}',),
            packageDirectory: '/w',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('/w/Cargo.toml',);
      },
    }),
    //endregion Reading Cargo's own answer

    //region The command line

    it({
      name: 'builds the dev profile of this package\'s own executable when the command line names nothing',
      fn: async () => {
        expect(parseArguments({
          words: [],
          ownPackage: '/repo/package/git-policy/cli',
        },),).toEqual({
          profile: 'dev',
          binName: 'monochromatic-git-policy-cli',
          packageDirectory: '/repo/package/git-policy/cli',
          cargoProgram: 'cargo',
        },);
      },
    }),
    it({
      name: 'takes every flag it knows, one value after the flag',
      fn: async () => {
        expect(parseArguments({
          words: [
            '--release',
            '--bin',
            'other',
            '--directory',
            '/elsewhere',
            '--cargo',
            '/opt/cargo',
          ],
          ownPackage: '/repo/package/git-policy/cli',
        },),).toEqual({
          profile: 'release',
          binName: 'other',
          packageDirectory: '/elsewhere',
          cargoProgram: '/opt/cargo',
        },);
      },
    }),
    it({
      name: 'refuses a word it does not know, and lists the flags it takes',
      fn: async () => {
        /**
         What was thrown.
         */
        let caught: unknown;
        try {
          parseArguments({
            words: ['--bins'],
            ownPackage: '/repo/package/git-policy/cli',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('--release',);
      },
    }),
    it({
      name: 'refuses a flag left without a value',
      fn: async () => {
        /**
         What was thrown.
         */
        let caught: unknown;
        try {
          parseArguments({
            words: ['--bin'],
            ownPackage: '/repo/package/git-policy/cli',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('--bin needs a value',);
      },
    }),
    //endregion The command line

    //region What the step asks Cargo

    it({
      name: 'asks Cargo where everything is, removes the program first, and cleans this crate before it builds',
      fn: async () => {
        await using scratch = await scratchDirectory();
        /**
         The program the stand-in writes, which is the path the step promises.
         */
        const artifact = join(
          scratch.path,
          'target/debug/monochromatic-git-policy-cli',
        );
        await mkdir(
          join(scratch.path, 'target/debug',),
          { recursive: true },
        );
        await writeFile(
          artifact,
          '#!/bin/sh\necho an earlier build left this one\n',
          { mode: ownerOnlyMode },
        );
        await using standIn = await createCargoStandIn({
          targetDirectory: join(scratch.path, 'target',),
          buildDirectory: join(scratch.path, 'build',),
          workspaceRoot: scratch.path,
          member: 'probe',
          manifestPath: join(scratch.path, 'Cargo.toml',),
          artifact,
        },);
        /**
         What the build came to.
         */
        const built = await buildFresh({
          packageDirectory: scratch.path,
          cargoProgram: standIn.program,
          binName: 'monochromatic-git-policy-cli',
          profile: 'dev',
          environment: standIn.environment,
        },);
        expect(built.artifactPath,).toBe(artifact,);
        expect(built.member,).toBe('probe',);
        expect(built.buildDirectory,).toBe(join(scratch.path, 'build',),);
        /**
         What the stand-in was asked, in the order it was asked.
         */
        const calls = await recordedCargoCalls({ directory: standIn.directory, },);
        expect(calls,).toHaveLength(3,);
        expect(calls[0]?.args,).toEqual([
          'metadata',
          '--locked',
          '--offline',
          '--no-deps',
          '--format-version',
          '1',
        ],);
        expect(calls[1]?.args,).toEqual([
          'clean',
          '--offline',
          '--package',
          'probe',
        ],);
        expect(calls[2]?.args,).toEqual([
          'build',
          '--locked',
          '--offline',
          '--bin',
          'monochromatic-git-policy-cli',
        ],);
        // The cleanup is what makes the promise true: no call that changes anything sees a program an earlier build
        // left, and the metadata read before it changes nothing.
        expect(calls.slice(1,).map(function sawArtifact(call): boolean {
          return call.artifactThere;
        },),).toEqual([
          false,
          false,
        ],);
      },
    }),
    it({
      name: 'asks for the release profile and its own artifact directory when the command line names the release',
      fn: async () => {
        await using scratch = await scratchDirectory();
        /**
         The program the stand-in writes for the release profile.
         */
        const artifact = join(
          scratch.path,
          'target/release/monochromatic-git-policy-cli',
        );
        await using standIn = await createCargoStandIn({
          targetDirectory: join(scratch.path, 'target',),
          workspaceRoot: scratch.path,
          member: 'probe',
          manifestPath: join(scratch.path, 'Cargo.toml',),
          artifact,
        },);
        /**
         What the build came to.
         */
        const built = await buildFresh({
          packageDirectory: scratch.path,
          cargoProgram: standIn.program,
          binName: 'monochromatic-git-policy-cli',
          profile: 'release',
          environment: standIn.environment,
        },);
        expect(built.artifactPath,).toBe(artifact,);
        /**
         What the stand-in was asked.
         */
        const calls = await recordedCargoCalls({ directory: standIn.directory, },);
        // Cargo's `clean` takes the release profile only when it is told, and a clean of the dev profile leaves the
        // release units a release build would call fresh. Measured 2026-11-16: without this flag the release task
        // printed `Finished` in 0.12 s and linked the artifact of a state built from other sources.
        expect(calls[1]?.args,).toEqual([
          'clean',
          '--release',
          '--offline',
          '--package',
          'probe',
        ],);
        expect(calls[2]?.args,).toEqual([
          'build',
          '--locked',
          '--offline',
          '--release',
          '--bin',
          'monochromatic-git-policy-cli',
        ],);
      },
    }),
    //endregion What the step asks Cargo

    //region What the step refuses

    it({
      name: 'refuses a target directory that is not this package\'s own, naming what moved it, because the program would land elsewhere',
      fn: async () => {
        /**
         What was thrown.
         */
        let caught: unknown;
        try {
          refuseAForeignTargetDirectory({
            packageDirectory: '/repo/package/git-policy/cli',
            targetDirectory: '/elsewhere/target',
            movedBy: 'CARGO_TARGET_DIR names /elsewhere/target',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NativeBuildFreshError,);
        expect((caught as Error).message,).toContain('/elsewhere/target',);
        expect((caught as Error).message,).toContain('CARGO_TARGET_DIR',);
      },
    }),
    it({
      name: 'refuses a build that ends successfully with no program at the path it names, naming the path and both directories',
      fn: async () => {
        await using scratch = await scratchDirectory();
        await using standIn = await createCargoStandIn({
          targetDirectory: join(scratch.path, 'target',),
          buildDirectory: join(scratch.path, 'build',),
          workspaceRoot: scratch.path,
          member: 'probe',
          manifestPath: join(scratch.path, 'Cargo.toml',),
        },);
        /**
         What was thrown.
         */
        let caught: unknown;
        try {
          await buildFresh({
            packageDirectory: scratch.path,
            cargoProgram: standIn.program,
            binName: 'monochromatic-git-policy-cli',
            profile: 'dev',
            environment: standIn.environment,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NativeBuildFreshError,);
        expect((caught as Error).message,).toContain(join(scratch.path, 'target/debug/monochromatic-git-policy-cli',),);
        expect((caught as Error).message,).toContain(join(scratch.path, 'build',),);
      },
    }),
    it({
      name: 'stops when the clean fails, quoting Cargo\'s own words, and never reaches the build',
      fn: async () => {
        await using scratch = await scratchDirectory();
        await using standIn = await createCargoStandIn({
          targetDirectory: join(scratch.path, 'target',),
          workspaceRoot: scratch.path,
          member: 'probe',
          manifestPath: join(scratch.path, 'Cargo.toml',),
          cleanFails: true,
        },);
        /**
         What was thrown.
         */
        let caught: unknown;
        try {
          await buildFresh({
            packageDirectory: scratch.path,
            cargoProgram: standIn.program,
            binName: 'monochromatic-git-policy-cli',
            profile: 'dev',
            environment: standIn.environment,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NativeBuildFreshError,);
        expect((caught as Error).message,).toContain('failed to remove',);
        /**
         What the stand-in was asked, which stops at the failing clean.
         */
        const calls = await recordedCargoCalls({ directory: standIn.directory, },);
        expect(calls,).toHaveLength(2,);
        expect(calls[1]?.args[0],).toBe('clean',);
      },
    }),
    it({
      name: 'stops a step that runs past its bound and says nothing was built',
      fn: async () => {
        await using scratch = await scratchDirectory();
        await using standIn = await createCargoStandIn({
          targetDirectory: join(scratch.path, 'target',),
          workspaceRoot: scratch.path,
          member: 'probe',
          manifestPath: join(scratch.path, 'Cargo.toml',),
          delayMilliseconds: 20_000,
        },);
        /**
         What was thrown.
         */
        let caught: unknown;
        try {
          await buildFresh({
            packageDirectory: scratch.path,
            cargoProgram: standIn.program,
            binName: 'monochromatic-git-policy-cli',
            profile: 'dev',
            environment: standIn.environment,
            deadlines: atOnce,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NativeBuildFreshError,);
        expect((caught as Error).message,).toContain('ran past the 1 ms',);
        expect((caught as Error).message,).toContain('nothing was built',);
      },
      timeout: 30_000,
    }),
    //endregion What the step refuses

    //region A program another build left there

    it({
      name: 'compiles these sources even when the build state was written after they were last modified, as a snapshot worktree hands it over',
      fn: async () => {
        await using scratch = await scratchDirectory();
        /**
         The crate whose program is this case's own.
         */
        const crate = join(scratch.path, 'crate',);
        await writeCrate({
          directory: crate,
          word: 'FROM_THE_STATE_AN_EARLIER_BUILD_MADE',
        },);

        // A person's earlier build of this crate: the state Cargo calls fresh.
        await execFileAsync(
          'cargo',
          ['build', '--offline'],
          {
            encoding: 'utf8',
            cwd: crate,
          },
        );
        await writeCrate({
          directory: crate,
          word: 'FROM_THESE_SOURCES',
        });
        // A worktree of this repository keeps the source checkout's modification times, which can be earlier than the
        // build state another checkout made (`doc/troubleshooting/cargo-fresh-build-leaves-another-checkouts-program.md`).
        /**
         The time the sources carry, which is earlier than the state above.
         */
        const earlier = new Date('2026-11-11T19:00:00Z',);
        await utimes(
          join(crate, 'src/main.rs',),
          earlier,
          earlier,
        );
        /**
         What this build came to.
         */
        const built = await buildFresh({
          packageDirectory: crate,
          cargoProgram: 'cargo',
          binName: 'probe',
          profile: 'dev',
          environment: process.env,
        },);
        expect(built.artifactPath,).toBe(join(crate, 'target/debug/probe',),);
        expect(await printedBy({
          program: built.artifactPath,
          args: [],
          directory: crate,
        },),).toBe('FROM_THESE_SOURCES',);
      },
      timeout: 300_000,
    }),
    //endregion A program another build left there
  ],
},);
