/**
 Exercises the built pi guardrail package through a real Pi session in a throwaway repository.
 
 The session runs Pi's own agent loop and Bash tool against the built extension,
 so a denied Bash call is denied where a model would reach it,
 and the refusal names the command to run instead.
 
 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { resolveRealGit, } from '@monochromatic-dev/git-executable-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  createThrowawayRepository,
  type ThrowawayRepository,
} from './pi-runtime-verifier-repository.ts';
import { verifyAbsoluteGitGuardAtPiBoundary, } from './pi-runtime-verifier-session.ts';

//region Types

/**
 Disposable directory owner removed through `await using`.
 */
type AsyncTemporaryDirectory = {
  /**
   Directory root every verification resource lives in.
   */
  readonly path: string;
  /**
   Removes the directory and everything below it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

//endregion Types

//region Scratch

/**
 Creates a disposable directory below the configured scratch root.
 
 @returns asynchronous temporary directory owner
 
 @example
 ```typescript
 await using temporary = await createTemporaryDirectory();
 ```
 */
async function createTemporaryDirectory(): Promise<AsyncTemporaryDirectory> {
  /**
   Unique root containing every verifier-controlled resource.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'pi-guardrail-runtime-',
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

//endregion Scratch

//region Verification

/**
 Runs the real-session verification of the absolute real-Git path guard.
 
 The Pi global directory is disposable and holds empty settings,
 so this host's own extension configuration takes no part.
 
 @param packageDirectory - built package directory whose manifest names its extension
 
 @param temporaryRoot - disposable directory for every resource
 
 @returns the verified outcome summary
 
 @throws when Pi reports extension errors or its answers differ from the guardrail's contract
 
 @example
 ```typescript
 await verifyPiGuardrailRuntime({ packageDirectory: process.cwd(), temporaryRoot: temporary.path });
 ```
 */
async function verifyPiGuardrailRuntime({
  packageDirectory,
  temporaryRoot,
}: {
  readonly packageDirectory: string;
  readonly temporaryRoot: string;
},): Promise<string> {
  /**
   Disposable Pi global directory.
   */
  const agentDirectory = join(
    temporaryRoot,
    'agent',
  );
  /**
   Disposable persisted session directory.
   */
  const sessionDirectory = join(
    temporaryRoot,
    'sessions',
  );
  /**
   Throwaway repository the session works in.
   */
  const repositoryRoot = join(
    temporaryRoot,
    'repository',
  );
  await mkdir(
    agentDirectory,
    { recursive: true, },
  );
  await mkdir(
    sessionDirectory,
    { recursive: true, },
  );
  await writeFile(
    join(
      agentDirectory,
      'settings.json',
    ),
    '{}\n',
  );
  /**
   Repository with one staged file and no commit yet.
   */
  const repository: ThrowawayRepository = await createThrowawayRepository({ root: repositoryRoot, },);
  /**
   Real Git binary of this host, spelled the way a model spells it.
   */
  const realGitPath = await resolveRealGit();
  /**
   What the real session showed.
   */
  const outcome = await verifyAbsoluteGitGuardAtPiBoundary({
    packageDirectory,
    agentDirectory,
    sessionDirectory,
    repository,
    realGitPath,
  },);
  return outcome.summary;
}

//endregion Verification

/**
 Disposable root removed after every outcome.
 */
await using temporary = await createTemporaryDirectory();

console.log(await verifyPiGuardrailRuntime({
  packageDirectory: process.cwd(),
  temporaryRoot: temporary.path,
},),);
