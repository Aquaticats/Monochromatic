import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { withFileMutationQueue, } from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { HelperRequest, } from './helper-request.ts';

import { scratchParent, } from './scratch-parent.ts';
//region Constants

/**
 Prefix for private answer workspaces under operating-system temp storage.
 */
const WORKSPACE_PREFIX = 'pi-ask-user-question-';

/**
 File edited by user in detached terminal.
 */
const ANSWER_FILENAME = 'ANSWER.md';

/**
 Private coordination file consumed by answer helper.
 */
const REQUEST_FILENAME = 'request.json';

/**
 Private directory mode for answer workspace.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 Private file mode for answer and coordination files.
 */
const PRIVATE_FILE_MODE = 0o600;

//endregion Constants

//region Logger

/**
 Tagged logger for answer workspace lifecycle.
 */
const l = tagged({ tag: 'ask-user-question:answer-workspace', },);

//endregion Logger

//region Types

/**
 Disposable private workspace for one pending question.
 */
export type AnswerWorkspace = AsyncDisposable & {
  /**
   Directory containing request artifacts.
   */
  readonly directory: string;
  /**
   User-edited answer path.
   */
  readonly answerPath: string;
  /**
   Helper-readable request path.
   */
  readonly requestPath: string;
};

//endregion Types

//region Workspace lifecycle

/**
 Creates empty private files for one external answer session.
 
 @returns disposable workspace paths
 
 @example
 ```ts
 await using workspace = await createAnswerWorkspace();
 ```
 */
export async function createAnswerWorkspace(): Promise<AnswerWorkspace> {
  /**
   Unique directory hidden from other operating-system users.
   */
  const directory = await mkdtemp(join(
    await scratchParent(),
    WORKSPACE_PREFIX,
  ),);
  await chmod(
    directory,
    PRIVATE_DIRECTORY_MODE,
  );
  /**
   Empty answer file opened by configured editor.
   */
  const answerPath = join(
    directory,
    ANSWER_FILENAME,
  );
  /**
   Coordination file populated after channel starts listening.
   */
  const requestPath = join(
    directory,
    REQUEST_FILENAME,
  );
  await withFileMutationQueue(
    answerPath,
    async function writeEmptyAnswer(): Promise<void> {
      await writeFile(
        answerPath,
        '',
        {
          encoding: 'utf8',
          mode: PRIVATE_FILE_MODE,
        },
      );
    },
  );
  l.debug(`created answer workspace: ${directory}`,);
  return {
    directory,
    answerPath,
    requestPath,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
      l.debug(`removed answer workspace: ${directory}`,);
    },
  };
}

/**
 Writes helper request after channel endpoint becomes available.
 
 @param workspace - private answer workspace
 
 @param request - authenticated helper endpoint
 
 @returns request path passed to detached helper
 
 @example
 ```ts
 await writeHelperRequest({ workspace, request: { host: '127.0.0.1', port: 1234, token: 'token', answerPath: workspace.answerPath, editorCommand: ['nano'] } });
 ```
 */
export async function writeHelperRequest(
  {
    workspace,
    request,
  }: {
    readonly workspace: AnswerWorkspace;
    readonly request: HelperRequest;
  },
): Promise<string> {
  await withFileMutationQueue(
    workspace.requestPath,
    async function writeRequest(): Promise<void> {
      await writeFile(
        workspace.requestPath,
        JSON.stringify(request,),
        {
          encoding: 'utf8',
          mode: PRIVATE_FILE_MODE,
        },
      );
    },
  );
  return workspace.requestPath;
}

/**
 Reads final UTF-8 answer after helper reports successful editor exit.
 
 @param workspace - workspace containing editor file
 
 @returns raw file text before final-line normalization
 
 @example
 ```ts
 await readWorkspaceAnswer({ workspace });
 ```
 */
export function readWorkspaceAnswer(
  { workspace, }: { readonly workspace: AnswerWorkspace; },
): Promise<string> {
  return readFile(
    workspace.answerPath,
    'utf8',
  );
}

//endregion Workspace lifecycle
