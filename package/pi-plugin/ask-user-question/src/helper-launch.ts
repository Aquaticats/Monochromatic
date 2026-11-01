import { constants, } from 'node:fs';
import {
  access,
  chmod,
  copyFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { AnswerWorkspace, } from './answer-workspace.ts';

//region Launch diagnostics

/**
 Owner-only permissions for executable source stored with answer data.
 */
const PRIVATE_FILE_MODE = 0o600;

/**
 Reports missing or inaccessible inputs before opening detached terminal.
 */
export class AnswerLaunchError extends Error {
  /**
   Preserves filesystem failure alongside user-facing remediation.

   @param message - affected input and recovery instructions

   @param cause - filesystem failure without request contents
   */
  constructor({
    message,
    cause,
  }: {
    readonly message: string;
    readonly cause: unknown
  },) {
    super(
      message,
      { cause, },
    );
    this.name = 'AnswerLaunchError';
  }
}

/**
 Tagged logger for dependencies crossing detached-process boundary.
 */
const l = tagged({ tag: 'ask-user-question:helper-launch', },);

//endregion Launch diagnostics

//region Runtime selection

/**
 Reuses running Linux executable through procfs so upgrades cannot unlink launch target.
 Falls back to original executable when procfs is unavailable or on other platforms.

 @param platform - operating system owning executable handle

 @param execPath - original runtime path

 @param pid - live Pi process retaining executable inode

 @returns executable path checked before detached launch

 @throws when no executable runtime is accessible

 @example
 ```ts
 await resolveAnswerRuntime();
 ```
 */
export async function resolveAnswerRuntime({
  platform = process.platform,
  execPath = process.execPath,
  pid = process.pid,
}: {
  readonly platform?: NodeJS.Platform;
  readonly execPath?: string;
  readonly pid?: number;
} = {},): Promise<string> {
  /**
   Function-local runtime diagnostics omit all answer content.
   */
  const rl = tagged({
    tag: resolveAnswerRuntime.name,
    l,
  },);
  if (platform === 'linux') {
    /**
     Parent remains alive throughout detached helper lifetime.
     */
    const liveExecutable = `/proc/${String(pid,)}/exe`;
    try {
      await access(
        liveExecutable,
        constants.X_OK,
      );
      rl.debug(`using live runtime: ${liveExecutable}`,);
      return liveExecutable;
    }
    catch (error: unknown) {
      rl.warn(`live runtime ${liveExecutable} unavailable; checking ${execPath}: ${String(error,)}`,);
    }
  }
  try {
    await access(
      execPath,
      constants.X_OK,
    );
  }
  catch (error: unknown) {
    throw new AnswerLaunchError({
      message: `Cannot launch the answer helper: Pi's runtime ${execPath} is missing or not executable. Restart Pi using an installed runtime, or restore executable access, then retry the question.`,
      cause: error,
    },);
  }
  rl.debug(`using runtime path: ${execPath}`,);
  return execPath;
}

//endregion Runtime selection

//region Request-owned helper

/**
 Copies self-contained helper into private request workspace before terminal launch.
 Rebuilds can replace installed bundle without deleting pending launch target.

 @param workspace - request-owned directory surviving until completion or cancellation

 @param sourcePath - installed helper bundle

 @returns private helper path passed to runtime

 @throws when helper cannot be copied or protected

 @example
 ```ts
 await prepareAnswerHelper({ workspace, sourcePath: '/installed/answer-helper.mjs' });
 ```
 */
export async function prepareAnswerHelper({
  workspace,
  sourcePath,
}: {
  readonly workspace: AnswerWorkspace;
  readonly sourcePath: string;
},): Promise<string> {
  /**
   Function-local preparation diagnostics identify filesystem inputs.
   */
  const rl = tagged({
    tag: prepareAnswerHelper.name,
    l,
  },);
  /**
   Private launch target owned by same scope as request and answer.
   */
  const helperPath = join(
    workspace.directory,
    'answer-helper.mjs',
  );
  try {
    await copyFile(
      sourcePath,
      helperPath,
      constants.COPYFILE_EXCL,
    );
    await chmod(
      helperPath,
      PRIVATE_FILE_MODE,
    );
  }
  catch (error: unknown) {
    throw new AnswerLaunchError({
      message: `Cannot prepare the answer helper from ${sourcePath} in ${workspace.directory}. Check file access and available storage; if the bundle is missing or being rebuilt, finish rebuilding the ask-user-question package and retry.`,
      cause: error,
    },);
  }
  rl.debug(`prepared private helper: ${helperPath}`,);
  return helperPath;
}

//endregion Request-owned helper
