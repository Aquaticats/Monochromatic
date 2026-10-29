import { fileURLToPath, } from 'node:url';

import { launchTerminal, } from '@monochromatic-dev/terminal-exec-cli/ts';
import { resolveTerminal, } from '@monochromatic-dev/terminal-exec-cli/ts/resolve.ts';
import { NO_TERMINAL, } from '@monochromatic-dev/terminal-exec-cli/ts/validate.ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { createAnswerChannel, } from './answer-channel.ts';
import {
  createAnswerWorkspace,
  readWorkspaceAnswer,
  writeHelperRequest,
} from './answer-workspace.ts';
import {
  isBlankAnswer,
  normalizeEditorAnswer,
} from './answer-text.ts';
import {
  GHOSTTY_HELIX_WARNING,
  isGhosttyHelixCombination,
} from './editor-compatibility.ts';
import {
  editorEnvironmentFromProcess,
  resolveEditorCommand,
} from './editor-command.ts';
import {
  prepareAnswerHelper,
  resolveAnswerRuntime,
} from './helper-launch.ts';
import type { RequestRegistry, } from './request-registry.ts';
import { ANSWER_BOOTSTRAP, } from './helper-bootstrap.ts';

//region Constants

/**
 Stable helper bundle emitted next to extension entry.
 */
const ANSWER_HELPER_FILENAME = 'answer-helper.mjs';

/**
 Terminal title documents save-and-exit submission gesture.
 */
const ANSWER_TERMINAL_TITLE = 'Pi answer: save and exit to submit';

/**
 Sentinel for terminal-exec finding no usable terminal identity.
 */
const TERMINAL_ENTRY_ID_UNAVAILABLE: unique symbol = Symbol('ask-user-question/terminal-entry-id-unavailable',);

//endregion Constants

//region Logger

/**
 Tagged logger for detached answer request lifecycle.
 */
const l = tagged({ tag: 'ask-user-question:request-external-answer', },);

//endregion Logger

//region Types

/**
 Result returned from external answer interaction.
 */
export type ExternalAnswerOutcome =
  | {
    readonly status: 'answered';
    readonly answer: string
  }
    | { readonly status: 'cancelled'; };

/**
 Terminal launcher shape injected by tests.
 */
export type AnswerTerminalLauncher = (
  options: {
    readonly dir: string;
    readonly command: readonly string[];
    readonly title: string;
  },
) => Promise<void>;

/**
 Resolves terminal-exec selected entry identity for compatibility checks.
 */
export type AnswerTerminalEntryIdResolver = () => Promise<string | typeof TERMINAL_ENTRY_ID_UNAVAILABLE>;

/**
 Emits one compatibility warning through production logger or test capture.
 */
export type AnswerCompatibilityWarner = (message: string) => void;

//endregion Types

//region Defaults

/**
 Resolves default terminal identity through terminal-exec.
 
 @returns selected entry ID,
 or undefined when no terminal resolves
 */
async function resolveDefaultTerminalEntryId(): Promise<string | typeof TERMINAL_ENTRY_ID_UNAVAILABLE> {
  /**
   terminal-exec resolution result.
   */
  const terminal = await resolveTerminal();
  return terminal === NO_TERMINAL
    ? TERMINAL_ENTRY_ID_UNAVAILABLE
    : terminal.entryId;
}

/**
 Emits compatibility warning through tagged module logger.
 
 @param message - detected terminal and editor risk
 */
function logCompatibilityWarning(message: string,): void {
  l.warn(message,);
}

//endregion Defaults

//region Request

/**
 Launches default terminal and awaits editor completion through private channel.
 
 @param cwd - Pi working directory for launched terminal
 
 @param signal - tool cancellation signal
 
 @param registry - session-scoped request cleanup registry
 
 @param editorCommand - effective editor command passed privately to helper
 
 @param resolveTerminalEntryId - injectable terminal identity resolver
 
 @param warn - injectable warning sink
 
 @param launch - injectable terminal launcher
 
 @returns normalized answer or cancellation
 
 @throws when terminal,
 helper,
 channel,
 or editor reports operational failure
 
 @example
 ```ts
 await requestExternalAnswer({ cwd: '/project', registry, launch: launchTerminal });
 ```
 */
export async function requestExternalAnswer(
  {
    cwd,
    signal,
    registry,
    editorCommand = resolveEditorCommand({
      env: editorEnvironmentFromProcess(process.env,),
      platform: process.platform,
    },),
    resolveTerminalEntryId = resolveDefaultTerminalEntryId,
    warn = logCompatibilityWarning,
    launch = launchTerminal,
  }: {
    readonly cwd: string;
    readonly signal?: AbortSignal;
    readonly registry: RequestRegistry;
    readonly editorCommand?: readonly string[];
    readonly resolveTerminalEntryId?: AnswerTerminalEntryIdResolver;
    readonly warn?: AnswerCompatibilityWarner;
    readonly launch?: AnswerTerminalLauncher;
  },
): Promise<ExternalAnswerOutcome> {
  /**
   Session-scoped cancellation covers terminal resolution as well as the pending helper.
   */
  using request = registry.open();
  /**
   Tool abort and session-shutdown abort combined for one request.
   */
  const requestSignal = signal === undefined
    ? request.signal
    : AbortSignal.any([
      signal,
      request.signal,
    ],);
  requestSignal.throwIfAborted();
  /**
   terminal-exec selected terminal identity.
   */
  const terminalEntryId = await resolveTerminalEntryId();
  requestSignal.throwIfAborted();
  if (((typeof terminalEntryId) !== 'symbol') && isGhosttyHelixCombination({
    terminalEntryId,
    editorCommand,
  },))
    warn(GHOSTTY_HELIX_WARNING,);
  /**
   Executable retained by live Pi process where supported.
   */
  const runtimePath = await resolveAnswerRuntime();
  /**
   Private answer and coordination files.
   */
  await using workspace = await createAnswerWorkspace();
  /**
   Request-owned helper survives replacement of installed bundle during launch.
   */
  const helperPath = await prepareAnswerHelper({
    workspace,
    sourcePath: fileURLToPath(new URL(
      ANSWER_HELPER_FILENAME,
      import.meta.url,
    ),),
  },);
  /**
   Authenticated one-shot helper return channel.
   */
  await using channel = await createAnswerChannel();
  await writeHelperRequest({
    workspace,
    request: {
      host: channel.host,
      port: channel.port,
      token: channel.token,
      answerPath: workspace.answerPath,
      editorCommand,
    },
  },);
  requestSignal.throwIfAborted();
  /**
   Wait begins before detached launch to avoid missing fast helper connection.
   */
  const completionTask = channel.wait({ signal: requestSignal, },);
  /**
   Observe launch and channel together, including cancellation while launch is pending.
   A late launcher still receives a bootstrap that safely handles expired requests.
   */
  async function launchAnswerTerminal(): Promise<void> {
    try {
      await launch({
        dir: cwd,
        title: ANSWER_TERMINAL_TITLE,
        command: [
          runtimePath,
          '--input-type=module',
          '--eval',
          ANSWER_BOOTSTRAP,
          helperPath,
          '--request',
          workspace.requestPath,
        ],
      },);
    }
    catch (error: unknown) {
      request.abort();
      l.error(`answer terminal launch failed: ${String(error,)}`,);
      throw error;
    }
  }
  /**
   Both tasks are observed immediately, so cancellation cannot leave a rejected wait unhandled.
   */
  const [, completion,] = await Promise.all([
    launchAnswerTerminal(),
    completionTask,
  ],);
  if (completion.status === 'cancelled') {
    l.info('answer helper cancelled',);
    return { status: 'cancelled', };
  }
  if (completion.status === 'error')
    throw new Error(`Answer helper failed: ${completion.message}`,);
  /**
   Raw editor file after successful attached editor exit.
   */
  const rawAnswer = await readWorkspaceAnswer({ workspace, },);
  /**
   Answer with one editor-added final line ending removed.
   */
  const answer = normalizeEditorAnswer({ text: rawAnswer, },);
  if (isBlankAnswer({ text: answer, })) {
    l.info('answer helper submitted blank content as cancellation',);
    return { status: 'cancelled', };
  }
  l.info(`answer helper submitted ${String(Buffer.byteLength(
    answer,
    'utf8',
  ),)} UTF-8 bytes`,);
  return {
    status: 'answered',
    answer,
  };
}

//endregion Request
