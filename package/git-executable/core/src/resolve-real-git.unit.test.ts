import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  delimiter,
  join,
} from 'node:path';
import { pathToFileURL, } from 'node:url';
import { promisify, } from 'node:util';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  RealGitNotFoundError,
  resolveRealGit,
  type ResolveRealGitOptions,
} from '../dist/final/node/index.mjs';

//region Disposable executable fixtures

/**
 File mode making fixture scripts executable by owner.
 */
const EXECUTABLE_MODE = 0o755;

/**
 File mode leaving fixture readable but not executable.
 */
const NON_EXECUTABLE_MODE = 0o644;

/**
 File mode allowing execution while denying read access.
 */
const UNREADABLE_EXECUTABLE_MODE = 0o111;

/**
 Prefix bytes placing self-shim marker beyond resolver inspection bound.
 */
const PADDED_SELF_SHIM_PREFIX_BYTES = 65_536;

/**
 Distinct successful resolutions needed to exceed bounded cache capacity.
 */
const CACHE_EVICTION_FIXTURE_COUNT = 17;

/**
 Child-process deadline proving special-file candidates cannot block resolver.
 */
const RESOLVER_CHILD_TIMEOUT_MS = 2_000;

/* oxlint-disable typescript/strict-void-return -- node:util.promisify intentionally accepts execFile while promise result excludes ChildProcess handle. */
/**
 Promise adapter for external fixture and isolated resolver processes.
 */
const executeFile = promisify(execFile,);
/* oxlint-enable typescript/strict-void-return */

/**
 Shell script standing in for external real Git.
 */
const REAL_GIT_CONTENT = `#!/bin/sh
echo real git "$@"
`;

/**
 Bundled entry marker emitted by pnpm command shims.
 */
const BUNDLED_ENTRY_SHIM_CONTENT = `#!/bin/sh
exec node "$basedir/../../package/git-policy/cli/dist/final/node/index.mjs" "$@"
`;

/**
 Cases proving every known Git policy self-shim marker is skipped.
 */
const SELF_SHIM_CASES: readonly {
  readonly name: string;
  readonly content: string;
}[] = [
  {
    name: 'package name marker',
    content: `#!/bin/sh
# @monochromatic-dev/git-policy-cli
`,
  },
  {
    name: 'bundled entry marker',
    content: BUNDLED_ENTRY_SHIM_CONTENT,
  },
  {
    name: 'Windows package marker',
    content: String.raw`@ECHO off
node "%~dp0\..\@monochromatic-dev\git-policy-cli\dist\final\node\index.mjs" %*
`,
  },
  {
    name: 'Windows bundled entry marker',
    content: String.raw`@ECHO off
node "%~dp0\..\..\package\git-policy\cli\dist\final\node\index.mjs" %*
`,
  },
];

/**
 Disposable temporary directory used by resolver tests.
 */
type TempDirectory = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates disposable temporary directory for executable fixtures.
 
 @returns Directory removed when async disposal completes.
 
 @example
 ```ts
 await using tempDirectory = await createTempDirectory();
 ```
 */
async function createTempDirectory(): Promise<TempDirectory> {
  /**
   Absolute fixture root unique to current test.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'git-executable-',
  ),);

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
 Writes executable fixture bytes.
 
 @param path - Absolute fixture path.
 
 @param content - Script or native-format bytes.
 
 @returns Nothing after fixture becomes executable.
 
 @example
 ```ts
 await writeExecutable({ path: '/tmp/git', content: '#!/bin/sh\n' });
 ```
 */
async function writeExecutable({
  path,
  content,
}: {
  readonly path: string;
  readonly content: string | Uint8Array;
},): Promise<void> {
  await writeFile(
    path,
    content,
    { mode: EXECUTABLE_MODE, },
  );
  await chmod(
    path,
    EXECUTABLE_MODE,
  );
}

/**
 Captures resolver rejection without promise matcher indirection.
 
 @param options - Disposable resolver inputs expected to reject.
 
 @returns Caught value or undefined when lookup unexpectedly resolves.
 
 @example
 ```ts
 const caught = await captureResolutionError({ pathEnv: '/missing' });
 ```
 */
async function captureResolutionError(
  options: ResolveRealGitOptions,
): Promise<unknown> {
  try {
    await resolveRealGit(options,);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Resolves Git in isolated process with deadline protecting FIFO regression tests.
 
 @param pathEnv - PATH-like candidate directories passed to child resolver.
 
 @param commonGitPaths - Preferred paths passed to child resolver.
 
 @returns Resolved path plus captured verbose diagnostics.
 
 @throws When resolver blocks beyond deadline or child lookup fails.
 
 @example
 ```ts
 await resolveRealGitInChild({
   pathEnv: '/tmp/bin:/usr/bin',
   commonGitPaths: ['/usr/bin/git'],
 });
 ```
 */
async function resolveRealGitInChild({
  pathEnv,
  commonGitPaths,
}: {
  readonly pathEnv: string;
  readonly commonGitPaths: readonly string[];
},): Promise<{
  readonly resolvedPath: string;
  readonly diagnostics: string;
}> {
  /**
   Built package interface loaded by child process.
   */
  const moduleUrl = pathToFileURL(join(
    import.meta.dirname,
    '..',
    'dist',
    'final',
    'node',
    'index.mjs',
  ),)
    .href;
  /**
   Destination-JavaScript-safe serialized resolver inputs.
   */
  const serializedOptions = JSON.stringify({
    pathEnv,
    commonGitPaths,
  },);
  /**
   Child module source using only JSON-encoded interpolations.
   */
  const childSource = `import { resolveRealGit } from ${JSON.stringify(moduleUrl,)};\nprocess.stdout.write(await resolveRealGit(${serializedOptions}));\n`;
  /**
   Isolated resolver output.
   */
  const {
    stdout,
    stderr,
  } = await executeFile(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      childSource,
    ],
    {
      timeout: RESOLVER_CHILD_TIMEOUT_MS,
      env: {
        ...process.env,
        MONOCHROMATIC_VERBOSE: 'true',
      },
    },
  );
  return {
    resolvedPath: stdout,
    diagnostics: stderr,
  };
}

/**
 Creates executable FIFO candidate for non-blocking resolver tests.
 
 @param path - Absolute FIFO path to create.
 
 @returns Nothing after FIFO is executable.
 
 @example
 ```ts
 await createExecutableFifo('/tmp/git');
 ```
 */
async function createExecutableFifo(path: string,): Promise<void> {
  await executeFile(
    'mkfifo',
    [path,],
  );
  await chmod(
    path,
    EXECUTABLE_MODE,
  );
}

//endregion Disposable executable fixtures

await describe({
  name: resolveRealGit.name,
  concurrency: 1,
  children: [
    //region Selection priority and self-shim exclusion

    it({
      name: 'promotes PATH-exposed common Git over earlier external Git',
      fn: async function promotesCommonGit(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Earlier ordinary PATH directory.
         */
        const ordinaryBin = join(tempDirectory.path, 'ordinary-bin',);
        /**
         Later common PATH directory.
         */
        const commonBin = join(tempDirectory.path, 'common-bin',);
        await Promise.all([
          mkdir(ordinaryBin,),
          mkdir(commonBin,),
        ],);
        /**
         Earlier external executable that common priority must supersede.
         */
        const ordinaryGit = join(ordinaryBin, 'git',);
        /**
         Later preferred executable exposed through PATH.
         */
        const commonGit = join(commonBin, 'git',);
        await Promise.all([
          writeExecutable({ path: ordinaryGit, content: REAL_GIT_CONTENT, },),
          writeExecutable({ path: commonGit, content: REAL_GIT_CONTENT, },),
        ],);

        expect(await resolveRealGit({
          pathEnv: [ordinaryBin, commonBin,].join(delimiter,),
          commonGitPaths: [commonGit,],
        },),).toBe(commonGit,);
      },
    },),
    it({
      name: 'skips four absent package-bin candidates before common Git',
      fn: async function skipsAbsentPackageBins(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         PATH directory containing selected common Git fixture.
         */
        const commonBin = join(tempDirectory.path, 'common-bin',);
        await mkdir(commonBin,);
        /**
         Selected executable after absent workspace candidates.
         */
        const commonGit = join(commonBin, 'git',);
        await writeExecutable({ path: commonGit, content: REAL_GIT_CONTENT, },);
        /**
         Broad PATH-like sequence reproducing package-local bin misses.
         */
        const pathEnv = [
          join(tempDirectory.path, 'package-a', 'node_modules', '.bin',),
          join(tempDirectory.path, 'package-b', 'node_modules', '.bin',),
          join(tempDirectory.path, 'package-c', 'node_modules', '.bin',),
          join(tempDirectory.path, 'package-d', 'node_modules', '.bin',),
          commonBin,
        ].join(delimiter,);

        expect(await resolveRealGit({
          pathEnv,
          commonGitPaths: [commonGit,],
        },),).toBe(commonGit,);
      },
    },),
    it({
      name: 'rejects executable FIFO candidate without waiting for a writer',
      skip: process.platform === 'win32',
      fn: async function rejectsExecutableFifo(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Earlier PATH directory containing executable FIFO candidate.
         */
        const fifoBin = join(tempDirectory.path, 'fifo-bin',);
        /**
         Later PATH directory containing regular executable candidate.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        await Promise.all([
          mkdir(fifoBin,),
          mkdir(externalBin,),
        ],);
        /**
         Executable FIFO that must not be opened in blocking mode.
         */
        const fifoGit = join(fifoBin, 'git',);
        /**
         Regular executable selected after FIFO rejection.
         */
        const externalGit = join(externalBin, 'git',);
        await Promise.all([
          createExecutableFifo(fifoGit,),
          writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },),
        ],);

        /**
         Isolated result proving FIFO candidate cannot block lookup.
         */
        const result = await resolveRealGitInChild({
          pathEnv: [fifoBin, externalBin,].join(delimiter,),
          commonGitPaths: [],
        },);

        expect(result.resolvedPath,).toBe(externalGit,);
      },
    },),
    it({
      name: 'does not inspect later unreadable candidate after common Git resolves',
      skip: process.platform === 'win32',
      fn: async function doesNotInspectLaterUnreadableCandidate(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Earlier PATH directory containing preferred common executable.
         */
        const commonBin = join(tempDirectory.path, 'common-bin',);
        /**
         Later PATH directory containing executable without read permission.
         */
        const unreadableBin = join(tempDirectory.path, 'unreadable-bin',);
        await Promise.all([
          mkdir(commonBin,),
          mkdir(unreadableBin,),
        ],);
        /**
         Preferred regular executable that must end sequential lookup.
         */
        const commonGit = join(commonBin, 'git',);
        /**
         Later candidate that emits path-specific diagnostic if inspected.
         */
        const unreadableGit = join(unreadableBin, 'git',);
        await Promise.all([
          writeExecutable({ path: commonGit, content: REAL_GIT_CONTENT, },),
          writeFile(
            unreadableGit,
            REAL_GIT_CONTENT,
            { mode: UNREADABLE_EXECUTABLE_MODE, },
          ),
        ],);
        /**
         Child result carrying positive-control resolver log.
         */
        const result = await resolveRealGitInChild({
          pathEnv: [commonBin, unreadableBin,].join(delimiter,),
          commonGitPaths: [commonGit,],
        },);

        expect(result.resolvedPath,).toBe(commonGit,);
        expect(result.diagnostics,).toContain('resolved real Git at',);
        expect(result.diagnostics,).not.toContain(unreadableGit,);
      },
    },),
    ...SELF_SHIM_CASES.map(function mapSelfShimCase(selfShimCase,) {
      return it({
        name: `skips ${selfShimCase.name} and resolves later external Git`,
        fn: async function skipsSelfShim(): Promise<void> {
          await using tempDirectory = await createTempDirectory();
          /**
           Directory containing self-referential wrapper fixture.
           */
          const selfBin = join(tempDirectory.path, 'self-bin',);
          /**
           Directory containing external Git fixture.
           */
          const externalBin = join(tempDirectory.path, 'external-bin',);
          await Promise.all([
            mkdir(selfBin,),
            mkdir(externalBin,),
          ],);
          /**
           Self-referential executable rejected by marker inspection.
           */
          const selfGit = join(selfBin, 'git',);
          /**
           External executable selected after wrapper rejection.
           */
          const externalGit = join(externalBin, 'git',);
          await Promise.all([
            writeExecutable({ path: selfGit, content: selfShimCase.content, },),
            writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },),
          ],);

          expect(await resolveRealGit({
            pathEnv: [selfBin, externalBin,].join(delimiter,),
            commonGitPaths: [],
          },),).toBe(externalGit,);
        },
      },);
    },),
    it({
      name: 'rejects padded self shim whose marker follows inspection bound',
      fn: async function rejectsPaddedSelfShim(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Earlier PATH directory containing oversized non-native wrapper.
         */
        const paddedBin = join(tempDirectory.path, 'padded-bin',);
        /**
         Later PATH directory containing external Git fixture.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        await Promise.all([
          mkdir(paddedBin,),
          mkdir(externalBin,),
        ],);
        /**
         Oversized wrapper whose self marker is outside bounded prefix.
         */
        const paddedGit = join(paddedBin, 'git',);
        /**
         External executable selected after oversized wrapper rejection.
         */
        const externalGit = join(externalBin, 'git',);
        /**
         Non-native wrapper bytes placing package marker after bound.
         */
        const paddedSelfShim = Buffer.concat([
          Buffer.from('#!/bin/sh\n',),
          Buffer.alloc(PADDED_SELF_SHIM_PREFIX_BYTES, '#',),
          Buffer.from('\n@monochromatic-dev/git-policy-cli\n',),
        ],);
        await Promise.all([
          writeExecutable({ path: paddedGit, content: paddedSelfShim, },),
          writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },),
        ],);

        expect(await resolveRealGit({
          pathEnv: [paddedBin, externalBin,].join(delimiter,),
          commonGitPaths: [],
        },),).toBe(externalGit,);
      },
    },),
    it({
      name: 'skips prioritized common self shim before ordinary PATH Git',
      fn: async function skipsCommonSelfShim(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Earlier ordinary PATH directory.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        /**
         Later promoted self-shim directory.
         */
        const selfBin = join(tempDirectory.path, 'self-bin',);
        await Promise.all([
          mkdir(externalBin,),
          mkdir(selfBin,),
        ],);
        /**
         External executable selected after promoted self shim is rejected.
         */
        const externalGit = join(externalBin, 'git',);
        /**
         Promoted wrapper fixture.
         */
        const selfGit = join(selfBin, 'git',);
        await Promise.all([
          writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },),
          writeExecutable({ path: selfGit, content: BUNDLED_ENTRY_SHIM_CONTENT, },),
        ],);

        expect(await resolveRealGit({
          pathEnv: [externalBin, selfBin,].join(delimiter,),
          commonGitPaths: [selfGit,],
        },),).toBe(externalGit,);
      },
    },),
    it({
      name: 'falls back to PATH order when preferred path is not exposed',
      fn: async function fallsBackToPathOrder(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         PATH directory containing fallback executable.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        await mkdir(externalBin,);
        /**
         PATH executable selected because preferred path is absent.
         */
        const externalGit = join(externalBin, 'git',);
        await writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },);

        expect(await resolveRealGit({
          pathEnv: externalBin,
          commonGitPaths: [join(tempDirectory.path, 'missing', 'git',),],
        },),).toBe(externalGit,);
      },
    },),
    it({
      name: 'skips non-executable candidate and selects later Git',
      fn: async function skipsNonExecutableCandidate(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Earlier directory containing readable non-executable candidate.
         */
        const nonExecutableBin = join(tempDirectory.path, 'non-executable-bin',);
        /**
         Later directory containing usable executable.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        await Promise.all([
          mkdir(nonExecutableBin,),
          mkdir(externalBin,),
        ],);
        /**
         Candidate rejected because execute permission is absent.
         */
        const nonExecutableGit = join(nonExecutableBin, 'git',);
        /**
         Later selected executable.
         */
        const externalGit = join(externalBin, 'git',);
        await Promise.all([
          writeFile(
            nonExecutableGit,
            REAL_GIT_CONTENT,
            { mode: NON_EXECUTABLE_MODE, },
          ),
          writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },),
        ],);

        expect(await resolveRealGit({
          pathEnv: [nonExecutableBin, externalBin,].join(delimiter,),
          commonGitPaths: [],
        },),).toBe(externalGit,);
      },
    },),

    //endregion Selection priority and self-shim exclusion

    //region Platform and path semantics

    it({
      name: 'follows Windows PATHEXT and common-path identity case-insensitively',
      fn: async function followsWindowsPathExtensions(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Directory containing Windows-named executable fixtures.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        await mkdir(externalBin,);
        /**
         Earlier PATHEXT candidate superseded by promoted common candidate.
         */
        const comGit = join(externalBin, 'git.COM',);
        /**
         Case-insensitively matched preferred executable.
         */
        const exeGit = join(externalBin, 'git.EXE',);
        await Promise.all([
          writeExecutable({ path: comGit, content: REAL_GIT_CONTENT, },),
          writeExecutable({ path: exeGit, content: REAL_GIT_CONTENT, },),
        ],);

        expect(await resolveRealGit({
          pathEnv: [
            join(tempDirectory.path, 'missing-bin',),
            externalBin,
          ].join(';',),
          platform: 'win32',
          pathExtensions: '.COM;.EXE;.BAT;.CMD',
          commonGitPaths: [exeGit.toLowerCase(),],
        },),).toBe(exeGit,);
      },
    },),
    it({
      name: 'resolves relative PATH entry against injected working directory',
      fn: async function resolvesRelativePathEntry(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Relative PATH directory under injected cwd.
         */
        const externalBin = join(tempDirectory.path, 'bin',);
        await mkdir(externalBin,);
        /**
         Absolute result expected from relative PATH input.
         */
        const externalGit = join(externalBin, 'git',);
        await writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },);

        expect(await resolveRealGit({
          pathEnv: 'bin',
          cwd: tempDirectory.path,
          commonGitPaths: [],
        },),).toBe(externalGit,);
      },
    },),
    it({
      name: 'resolves empty PATH entry against injected working directory',
      fn: async function resolvesEmptyPathEntry(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Executable exposed by empty PATH entry at injected cwd.
         */
        const externalGit = join(tempDirectory.path, 'git',);
        await writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },);

        expect(await resolveRealGit({
          pathEnv: '',
          cwd: tempDirectory.path,
          commonGitPaths: [],
        },),).toBe(externalGit,);
      },
    },),
    it({
      name: 'uses injected environment PATH when explicit path is absent',
      fn: async function usesInjectedEnvironmentPath(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Directory exposed only through injected process environment.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        await mkdir(externalBin,);
        /**
         Executable selected from injected environment.
         */
        const externalGit = join(externalBin, 'git',);
        await writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },);

        expect(await resolveRealGit({
          environment: { PATH: externalBin, },
          commonGitPaths: [],
        },),).toBe(externalGit,);
      },
    },),

    //endregion Platform and path semantics

    //region Cache and failure lifetime

    it({
      name: 'shares in-flight resolution across concurrent equal calls',
      fn: async function sharesInflightResolution(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Directory containing concurrently resolved executable.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        await mkdir(externalBin,);
        /**
         Executable expected from both concurrent calls.
         */
        const externalGit = join(externalBin, 'git',);
        await writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },);
        /**
         Concurrent equal lookup results.
         */
        const resolved = await Promise.all([
          resolveRealGit({ pathEnv: externalBin, commonGitPaths: [], },),
          resolveRealGit({ pathEnv: externalBin, commonGitPaths: [], },),
        ],);

        expect(resolved,).toEqual([externalGit, externalGit,],);
      },
    },),
    it({
      name: 'reuses successful resolution after executable disappears',
      fn: async function reusesSuccessfulResolution(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Directory containing cache fixture.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        await mkdir(externalBin,);
        /**
         Executable removed after initial successful lookup.
         */
        const externalGit = join(externalBin, 'git',);
        await writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },);
        /**
         Initial result persisted by candidate-sequence identity.
         */
        const initial = await resolveRealGit({
          pathEnv: externalBin,
          commonGitPaths: [],
        },);
        await rm(externalGit,);

        expect(initial,).toBe(externalGit,);
        expect(await resolveRealGit({
          pathEnv: externalBin,
          commonGitPaths: [],
        },),).toBe(externalGit,);
      },
    },),
    it({
      name: 'separates successful resolutions for changed effective candidates',
      fn: async function separatesChangedInputs(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         First candidate directory.
         */
        const firstBin = join(tempDirectory.path, 'first-bin',);
        /**
         Second candidate directory.
         */
        const secondBin = join(tempDirectory.path, 'second-bin',);
        await Promise.all([
          mkdir(firstBin,),
          mkdir(secondBin,),
        ],);
        /**
         First candidate result.
         */
        const firstGit = join(firstBin, 'git',);
        /**
         Second candidate result.
         */
        const secondGit = join(secondBin, 'git',);
        await Promise.all([
          writeExecutable({ path: firstGit, content: REAL_GIT_CONTENT, },),
          writeExecutable({ path: secondGit, content: REAL_GIT_CONTENT, },),
        ],);

        expect(await resolveRealGit({
          pathEnv: firstBin,
          commonGitPaths: [],
        },),).toBe(firstGit,);
        expect(await resolveRealGit({
          pathEnv: secondBin,
          commonGitPaths: [],
        },),).toBe(secondGit,);
      },
    },),
    it({
      name: 'evicts least-recently-used success after cache capacity',
      fn: async function evictsLeastRecentlyUsedSuccess(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Distinct PATH directories exceeding documented cache capacity.
         */
        const cacheBinPaths = Array.from(
          { length: CACHE_EVICTION_FIXTURE_COUNT, },
          function cacheBinPath(_unused, index,) {
            return join(
              tempDirectory.path,
              `cache-bin-${String(index,)}`,
            );
          },
        );
        await Promise.all(cacheBinPaths.map(async function createCachedGit(cacheBinPath,) {
          await mkdir(cacheBinPath,);
          await writeExecutable({
            path: join(cacheBinPath, 'git',),
            content: REAL_GIT_CONTENT,
          },);
        },),);

        for (const cacheBinPath of cacheBinPaths) {
          // oxlint-disable-next-line no-await-in-loop -- insertion order establishes deterministic LRU eviction
          await resolveRealGit({
            pathEnv: cacheBinPath,
            commonGitPaths: [],
          },);
        }

        /**
         Oldest successful candidate expected to have been evicted.
         */
        const [oldestCacheBinPath,] = cacheBinPaths;
        if (oldestCacheBinPath === undefined)
          throw new Error('Cache eviction fixture did not create oldest candidate.',);
        await rm(join(oldestCacheBinPath, 'git',),);
        /**
         Fresh lookup failure proving oldest success no longer remains cached.
         */
        const caught = await captureResolutionError({
          pathEnv: oldestCacheBinPath,
          commonGitPaths: [],
        },);

        expect(caught,).toBeInstanceOf(RealGitNotFoundError,);
      },
    },),
    it({
      name: 'retries rejected resolution after executable appears',
      fn: async function retriesRejectedResolution(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Existing directory whose Git candidate initially does not exist.
         */
        const externalBin = join(tempDirectory.path, 'external-bin',);
        await mkdir(externalBin,);
        /**
         Candidate created only after initial rejection.
         */
        const externalGit = join(externalBin, 'git',);
        /**
         Initial no-candidate failure.
         */
        const caught = await captureResolutionError({
          pathEnv: externalBin,
          commonGitPaths: [],
        },);
        await writeExecutable({ path: externalGit, content: REAL_GIT_CONTENT, },);

        expect(caught,).toBeInstanceOf(RealGitNotFoundError,);
        expect(await resolveRealGit({
          pathEnv: externalBin,
          commonGitPaths: [],
        },),).toBe(externalGit,);
      },
    },),
    it({
      name: 'throws dedicated error when PATH exposes no usable candidate',
      fn: async function throwsDedicatedError(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /**
         Captured dedicated lookup failure.
         */
        const caught = await captureResolutionError({
          pathEnv: join(tempDirectory.path, 'missing-bin',),
          commonGitPaths: [],
        },);

        expect(caught,).toBeInstanceOf(RealGitNotFoundError,);
        expect((caught as Error).name,).toBe('RealGitNotFoundError',);
        expect((caught as Error).message,).toContain('examining 1 PATH candidates',);
        expect((caught as Error).message,).toContain('PATH/PATHEXT',);
      },
    },),

    //endregion Cache and failure lifetime
  ],
},);
