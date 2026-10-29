import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type AskUserQuestionConfig,
  loadAskUserQuestionConfig,
} from './config.ts';
import {
  editorEnvironmentFromProcess,
  resolveEditorCommand,
} from './editor-command.ts';
import { createRequestRegistry, } from './request-registry.ts';
import { requestExternalAnswer, } from './request-external-answer.ts';
import {
  type ExternalAnswerRequester,
  registerAskUserQuestionTool,
} from './tool.ts';

//region Logger

/**
 Tagged logger for extension registration and shutdown.
 */
const l = tagged({ tag: 'ask-user-question:index', },);

//endregion Logger

//region Registration

/**
 Registers ask-user tool with session-scoped external-request cleanup.
 
 @param pi - host extension API
 
 @param config - optional loaded user configuration
 
 @param requestAnswer - optional test requester
 
 @example
 ```ts
 registerAskUserQuestionExtension({ pi });
 ```
 */
export function registerAskUserQuestionExtension(
  {
    pi,
    config,
    requestAnswer,
  }: {
    readonly pi: ExtensionAPI;
    readonly config?: AskUserQuestionConfig;
    readonly requestAnswer?: ExternalAnswerRequester;
  },
): void {
  /**
   Controllers for external helpers active in current extension session.
   */
  const registry = createRequestRegistry();
  /**
   Production requester or test boundary override.
   */
  const requester: ExternalAnswerRequester = requestAnswer
    ?? (function requestThroughDefaultTerminal({
      cwd,
      signal,
    },): ReturnType<ExternalAnswerRequester> {
      return requestExternalAnswer({
        cwd,
        registry,
        editorCommand: config?.editorCommand
          ?? resolveEditorCommand({
            env: editorEnvironmentFromProcess(process.env,),
            platform: process.platform,
          },),
        ...(signal === undefined ? {} : { signal, }),
      },);
    });
  registerAskUserQuestionTool({
    pi,
    requestAnswer: requester,
  },);
  pi.on(
    'session_shutdown',
    function abortPendingAnswers(): void {
      l.info('aborting pending answer helpers for session shutdown',);
      registry.abortAll();
    },
  );
}

/**
 Pi extension entry point.
 
 @param pi - host extension API
 
 @example
 ```ts
 askUserQuestionExtension(pi);
 ```
 */
export default async function askUserQuestionExtension(pi: ExtensionAPI,): Promise<void> {
  /**
   User-level editor override loaded once per Pi extension session.
   */
  const config = await loadAskUserQuestionConfig();
  registerAskUserQuestionExtension({
    pi,
    config,
  },);
}

//endregion Registration

export {
  createAnswerChannel,
  type AnswerChannel,
} from './answer-channel.ts';
export {
  isBlankAnswer,
  normalizeEditorAnswer,
} from './answer-text.ts';
export {
  AskUserQuestionConfigError,
  askUserQuestionConfigPath,
  loadAskUserQuestionConfig,
  type AskUserQuestionConfig,
  type LoadAskUserQuestionConfigOptions,
} from './config.ts';
export {
  GHOSTTY_HELIX_WARNING,
  isGhosttyHelixCombination,
} from './editor-compatibility.ts';
export {
  resolveEditorCommand,
  EditorCommandError,
  type EditorEnvironment,
} from './editor-command.ts';
export {
  AnswerLaunchError,
  resolveAnswerRuntime,
} from './helper-launch.ts';
export {
  readHelperRequest,
  type HelperRequest,
} from './helper-request.ts';
export {
  type HelperCompletion,
  HelperProtocolError,
  parseHelperCompletion,
  serializeHelperCompletion,
} from './helper-protocol.ts';
export {
  requestExternalAnswer,
  type AnswerCompatibilityWarner,
  type AnswerTerminalEntryIdResolver,
  type AnswerTerminalLauncher,
  type ExternalAnswerOutcome,
} from './request-external-answer.ts';
export {
  createRequestRegistry,
  type RequestRegistry,
} from './request-registry.ts';
export { visibleTerminalText, } from './terminal-text.ts';
export {
  ASK_USER_QUESTION_TOOL_NAME,
  AskUserQuestionUnavailableError,
  type AskUserQuestionParameters,
  type ExternalAnswerRequester,
} from './tool.ts';
export {
  buildAnsweredResult,
  buildCancelledResult,
  type AskUserQuestionDetails,
} from './tool-result.ts';
