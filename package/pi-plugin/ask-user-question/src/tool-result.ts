import {
  chmod,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_MAX_BYTES,
  DEFAULT_MAX_LINES,
  formatSize,
  truncateHead,
  withFileMutationQueue,
  type AgentToolResult,
} from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { scratchParent, } from './scratch-parent.ts';
//region Constants

/**
 Prefix for retained complete-answer directories.
 */
const RETAINED_ANSWER_PREFIX = 'pi-ask-user-answer-';

/**
 Filename used for complete answer fallback.
 */
const RETAINED_ANSWER_FILENAME = 'answer.txt';

/**
 Private directory mode for retained answer.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 Private file mode for retained answer.
 */
const PRIVATE_FILE_MODE = 0o600;

//endregion Constants

//region Logger

/**
 Tagged logger for model-visible answer formatting.
 */
const l = tagged({ tag: 'ask-user-question:tool-result', },);

//endregion Logger

//region Types

/**
 Persisted details for answered or cancelled question.
 */
export type AskUserQuestionDetails =
  | {
    /**
     User submitted nonblank answer.
     */
    readonly status: 'answered';
    /**
     Complete answer when visible,
     or visible prefix when truncated.
     */
    readonly answer: string;
    /**
     Private full-answer path when model-visible text was truncated.
     */
    readonly fullAnswerPath?: string;
  }
    | {
    /**
     User declined or answer helper disappeared.
     */
      readonly status: 'cancelled';
    };

//endregion Types

//region Result construction

/**
 Builds model-visible cancelled result.
 
 @returns successful tool result carrying cancellation status
 
 @example
 ```ts
 buildCancelledResult();
 ```
 */
export function buildCancelledResult(): AgentToolResult<AskUserQuestionDetails> {
  return {
    content: [{
      type: 'text',
      text: 'User cancelled the question.',
    },],
    details: { status: 'cancelled', },
  };
}

/**
 Builds context-bounded model result for submitted answer.
 
 Complete oversized answer remains in private temp file for model read tool.
 
 @param answer - normalized nonblank answer
 
 @returns tool result with answer or truncation path
 
 @example
 ```ts
 await buildAnsweredResult({ answer: 'Use the external editor.' });
 ```
 */
export async function buildAnsweredResult(
  { answer, }: { readonly answer: string; },
): Promise<AgentToolResult<AskUserQuestionDetails>> {
  /**
   Pi-standard context truncation over answer text.
   */
  const truncation = truncateHead(
    answer,
    {
      maxLines: DEFAULT_MAX_LINES,
      maxBytes: DEFAULT_MAX_BYTES,
    },
  );
  if (!truncation.truncated)
    return {
      content: [{
        type: 'text',
        text: `User answered:\n${truncation.content}`,
      },],
      details: {
        status: 'answered',
        answer,
      },
    };
  /**
   Private path retaining complete oversized answer.
   */
  const fullAnswerPath = await writeRetainedAnswer({ answer, },);
  /**
   Model-visible truncation notice with precise line and byte evidence.
   */
  const notice = `Answer truncated: showing ${String(truncation.outputLines,)} of ${String(truncation.totalLines,)} lines (${formatSize(truncation.outputBytes,)} of ${formatSize(truncation.totalBytes,)}). Full answer saved to: ${fullAnswerPath}`;
  l.warn(`truncated model-visible answer; full answer at ${fullAnswerPath}`,);
  return {
    content: [{
      type: 'text',
      text: [
        `User answered:\n${truncation.content}`,
        `[${notice}]`,
      ].join('\n\n',),
    },],
    details: {
      status: 'answered',
      answer: truncation.content,
      fullAnswerPath,
    },
  };
}

//endregion Result construction

//region Retained answer

/**
 Writes complete oversized answer to private temp file.
 
 @param answer - complete answer text
 
 @returns readable absolute path for model tool
 */
async function writeRetainedAnswer(
  { answer, }: { readonly answer: string; },
): Promise<string> {
  /**
   Unique retained-answer directory.
   */
  const directory = await mkdtemp(join(
    await scratchParent(),
    RETAINED_ANSWER_PREFIX,
  ),);
  await chmod(
    directory,
    PRIVATE_DIRECTORY_MODE,
  );
  /**
   Stable answer filename inside unique directory.
   */
  const path = join(
    directory,
    RETAINED_ANSWER_FILENAME,
  );
  await withFileMutationQueue(
    path,
    async function writeQueuedAnswer(): Promise<void> {
      await writeFile(
        path,
        answer,
        {
          encoding: 'utf8',
          mode: PRIVATE_FILE_MODE,
        },
      );
    },
  );
  return path;
}

//endregion Retained answer
