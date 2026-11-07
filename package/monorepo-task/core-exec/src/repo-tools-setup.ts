#!/usr/bin/env node

/**
 The step of `.github/actions/repo-tools-setup`: fetches the prebuilt `file-enforcer`, `mpm-shadow` and `cargo-shadow`
 binaries this repository publishes, checks each against its recorded SHA-256 and puts their folder on `PATH`.

 `mpm-shadow` comes from the archive row of the `mpm-shadow` tool in `toolchain.lock.json`, checked against the SHA-256
 recorded there. `file-enforcer` and `cargo-shadow` come from the release of the version the lock names for
 `cargo:monochromatic-file-enforcer-cli` and `cargo:monochromatic-cargo-shadow`, checked against the checksums file of
 that release. The developer-machine counterpart is `mpm-shadow.ts`: the same lock row, the same SHA-256 check.

 Reads `TOOLS` (space-separated tool names), `RUNNER_PLATFORM` (`<runner.os>-<runner.arch>`), `RUNNER_TEMP`,
 `GITHUB_REPOSITORY` and `GITHUB_PATH` from the environment and `toolchain.lock.json` from the current directory.
 Imports only `node:` modules and sibling files that do, because the action runs on a runner that has no dependencies
 installed. `curl`, `sha256sum` and `tar` run as they did in the shell step, so their output and statuses are unchanged;
 `jq` is no longer needed.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/repo-tools-setup.ts
 ```
 */

import {
  mkdir,
  mkdtemp,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  KNOWN_TOOLS,
  fetchTool,
  type PlatformNames,
  unpack,
} from './repo-tools-setup-archives.ts';
import {
  appendToRunnerFile,
  readJsonFile,
  requiredEnvironment,
  runStep,
  sayOnStandardError,
  StepExitError,
} from './github-actions-step-support.ts';

//region What each runner platform downloads

/**
 Runner platforms that have prebuilt archives, by `<runner.os>-<runner.arch>`.
 */
const PLATFORMS: ReadonlyMap<string, PlatformNames> = new Map([
  [
    'Linux-X64',
    {
      lockPlatform: 'linux-x64',
      target: 'x86_64-unknown-linux-gnu',
    },
  ],
  [
    'Linux-ARM64',
    {
      lockPlatform: 'linux-arm64',
      target: 'aarch64-unknown-linux-gnu',
    },
  ],
]);

/**
 Where the tools' versions and archives are recorded, relative to the checkout the step runs in.
 */
const LOCK_PATH = 'toolchain.lock.json';

//endregion What each runner platform downloads

//region The whole step

/**
 Reads the lock the first time a tool needs it and remembers it, so a step that installs nothing never opens it.

 @param cache - Holds the lock under one key once it is read

 @returns The parsed lock

 @throws {@link StepExitError} With status 2 when the lock cannot be read as JSON

 @example
 ```ts
 const lock = await lockOnce(new Map(),);
 ```
 */
async function lockOnce(cache: Map<string, unknown>,): Promise<unknown> {
  if (!cache.has(LOCK_PATH,))
    cache.set(
      LOCK_PATH,
      await readJsonFile(LOCK_PATH,),
    );
  return cache.get(LOCK_PATH,);
}

/**
 Fetches, checks and unpacks one requested tool.

 @param tool - Name from the `tools` input

 @param platform - Names this runner platform has in the lock and in archives

 @param binDirectory - Folder that receives the program

 @param lockCache - Holds the lock once read

 @throws {@link StepExitError} With the status the shell step would have ended with

 @example
 ```ts
 await installOneTool({ tool: 'mpm-shadow', platform, binDirectory, lockCache: new Map(), },);
 ```
 */
async function installOneTool(
  {
    tool,
    platform,
    binDirectory,
    lockCache,
  }: {
    readonly tool: string;
    readonly platform: PlatformNames;
    readonly binDirectory: string;
    readonly lockCache: Map<string, unknown>;
  },
): Promise<void> {
  /**
   Empty folder this tool downloads into, under the runner's own temporary folder.
   */
  const work = await mkdtemp(join(
    requiredEnvironment('RUNNER_TEMP',),
    'tool-',
  ),);
  if (!KNOWN_TOOLS.has(tool,)) {
    sayOnStandardError(`::error::repo-tools-setup: unknown tool '${tool}'`,);
    throw new StepExitError(1,);
  }
  await unpack({
    archive: await fetchTool({
      lock: await lockOnce(lockCache,),
      tool,
      platform,
      work,
    },),
    binDirectory,
    tool,
  },);
}

/**
 Installs every requested tool, one after another, and puts their folder on `PATH` through `GITHUB_PATH`.

 @throws {@link StepExitError} With the status the shell step would have ended with

 @example
 ```ts
 await installRepositoryTools();
 ```
 */
async function installRepositoryTools(): Promise<void> {
  /**
   The runner platform, such as `Linux-X64`.
   */
  const runnerPlatform = requiredEnvironment('RUNNER_PLATFORM',);
  /**
   Names of this platform in the lock and in archives.
   */
  const platform = PLATFORMS.get(runnerPlatform,);
  if (platform === undefined) {
    sayOnStandardError(`::error::repo-tools-setup: no prebuilt archive for runner ${runnerPlatform}`,);
    throw new StepExitError(1,);
  }
  /**
   Folder the tools are extracted into and that goes on `PATH`.
   */
  const binDirectory = join(
    requiredEnvironment('RUNNER_TEMP',),
    'repo-tools-bin',
  );
  await mkdir(
    binDirectory,
    { recursive: true, },
  );
  /**
   The lock, held once read.
   */
  const lockCache = new Map<string, unknown>();
  // The shell step split the input on blanks and line breaks, and a failed tool ended it before the next began.
  await requiredEnvironment('TOOLS',)
    .replaceAll(
    '\t',
    ' ',
    )
    .replaceAll(
    '\n',
    ' ',
    )
    .split(' ',)
    .filter(function isWord(word: string,) {
      return word !== '';
    },)
    .reduce(
    async function afterPrevious(
      previous: Promise<void>,
      tool: string,
    ) {
      await previous;
      await installOneTool({
        tool,
        platform,
        binDirectory,
        lockCache,
      },);
    },
    Promise.resolve(),
    );
  await appendToRunnerFile({
    variable: 'GITHUB_PATH',
    text: `${binDirectory}\n`,
  },);
}

//endregion The whole step

await runStep(installRepositoryTools,);
