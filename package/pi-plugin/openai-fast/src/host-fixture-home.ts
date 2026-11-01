/**
 Disposable workspace and agent storage isolated from every real pi home.
 
 @module
 */
import {
  mkdir,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

//region Workspace ownership: remove only test-created storage.

/**
 Paths and cleanup authority for one independently owned fixture home.
 */
export type FixtureHome = {
  /**
   Root containing only files created by this fixture.
   */
  readonly root: string;
  /**
   Synthetic project directory given to createAgentSession.
   */
  readonly cwd: string;
  /**
   Disposable agent configuration directory, never the actual pi home.
   */
  readonly agentDir: string;
  /**
   Optional synthetic models.json consumed by the real host.
   */
  readonly modelsPath: string;
  /**
   Disposable cached catalog used to test cached-only discovery.
   */
  readonly modelsStorePath: string;
  /**
   Session persistence directory for real resume and branch tests.
   */
  readonly sessionDir: string;

  /**
   Delete only this fixture's temporary root after dependent sessions close.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Create independent workspace, config, cache, and session storage paths.
 
 @returns asynchronously disposable home for native-host scenarios
 
 @example
 ```ts
 await using home = await fixtureHome();
 ```
 */
export async function fixtureHome(): Promise<FixtureHome> {
  /**
   Newly created storage is owned immediately, even if initialization fails.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'openai-fast-host-',
  ),);
  /**
   Every derived storage path remains inside the owned temporary root.
   */
  const agentDir = join(
    root,
    'agent',
  );
  /**
   Frozen resource retains cleanup authority without mutable path fields.
   */
  const home: FixtureHome = Object.freeze({
    root,
    cwd: join(
      root,
      'workspace',
    ),
    agentDir,
    modelsPath: join(
      agentDir,
      'models.json',
    ),
    modelsStorePath: join(
      agentDir,
      'models-store.json',
    ),
    sessionDir: join(
      agentDir,
      'sessions',
    ),
    [Symbol.asyncDispose]: async function dispose(): Promise<void> {
      await rm(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  },);
  /**
   Ownership transfers only after both directories initialize successfully.
   */
  await using initialization = {
    transferred: false,
    [Symbol.asyncDispose]: async function dispose(): Promise<void> {
      if (!this.transferred)
        await home[Symbol.asyncDispose]();
    },
  };
  await Promise.all([
    mkdir(home.cwd,),
    mkdir(home.agentDir,),
  ],);
  initialization.transferred = true;
  return home;
}

//endregion Workspace ownership
