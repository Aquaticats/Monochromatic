/**
 Real Pi session that runs scripted Bash calls against the built guardrail extension.
 
 The session is Pi's own agent loop with its own tool registry,
 loaded through Pi's own resource loader from the package manifest,
 so a denied call is denied where a model would reach it.
 
 @module
 */

import { join, } from 'node:path';

import {
  type AgentSession,
  createAgentSession,
  DefaultResourceLoader,
  ModelRuntime,
  SessionManager,
  SettingsManager,
} from '@earendil-works/pi-coding-agent';
import type { ToolResultMessage, } from '@earendil-works/pi-ai';

import {
  commitSubjects,
  hasNoCommit,
  type ThrowawayRepository,
} from './pi-runtime-verifier-repository.ts';
import {
  CLOSING_TEXT,
  registerScriptedGitGuardProvider,
  type ScriptedBashCall,
} from './pi-runtime-verifier-scripted-provider.ts';

//region Constants

/**
 Sentence the denial must carry, naming the command to run instead.
 */
const REMEDY_SENTENCE = 'git -C <repository-root>';

/**
 Prefix the shared denial reason starts with.
 */
const DENIAL_PREFIX = 'Blocked:';

/**
 Text Pi's Bash tool answers a blocked call with, which the model reads as the failure.
 */
const BLOCKED_CALL_TEXT = 'Blocked';

/**
 Commit message of the call the guardrail must refuse.
 */
const DENIED_COMMIT_MESSAGE = 'denied absolute-path commit';

/**
 Commit message of the same call spelled through the PATH wrapper, which must land.
 */
const ALLOWED_COMMIT_MESSAGE = 'commit through the PATH wrapper';

//endregion Constants

//region Types

/**
 Everything the real-session verification needs from its caller.
 */
type GuardedSessionRequest = {
  /**
   Package directory whose `pi.extensions` manifest names the built extension.
   */
  readonly packageDirectory: string;
  /**
   Disposable Pi global directory.
   */
  readonly agentDirectory: string;
  /**
   Disposable session-file directory.
   */
  readonly sessionDirectory: string;
  /**
   Throwaway repository the session works in and the Bash tool runs in.
   */
  readonly repository: ThrowawayRepository;
  /**
   Absolute path of this host's real Git binary, spelled the way a model would.
   */
  readonly realGitPath: string;
};

/**
 One message Pi reports for the session, of whatever role.
 */
type SessionMessage = AgentSession['messages'][number];

/**
 What the real session showed.
 */
type GuardedSessionOutcome = {
  /**
   Summary naming each scripted call and what Pi answered.
   */
  readonly summary: string;
};

//endregion Types

//region Message reads

/**
 Reads every text part of one tool result.
 
 @param message - tool result message
 
 @returns its text content, joined with newlines
 
 @example
 ```typescript
 toolResultText({ content: [{ type: 'text', text: 'done' }] });
 ```
 */
function toolResultText(message: ToolResultMessage,): string {
  return message.content
    .flatMap(function textParts(part,) {
      return part.type === 'text' ? [part.text,] : [];
    },)
    .join('\n',);
}

/**
 Collects the tool results of one session in the order Pi produced them.
 
 @param messages - session messages in order
 
 @returns every tool result message
 
 @example
 ```typescript
 const results = toolResultsOf(session.messages);
 ```
 */
function toolResultsOf(messages: readonly SessionMessage[],): readonly ToolResultMessage[] {
  return messages.flatMap(function toolResults(message,) {
    return message.role === 'toolResult' ? [message,] : [];
  },);
}

//endregion Message reads

//region Session construction

/**
 Creates the real Pi session that runs the scripted Bash calls.
 
 @param request - package, directories, repository, and real-Git path
 
 @returns session, its resource loader, and the scripted provider call count
 
 @throws when Pi reports extension load errors or loads a different count
 
 @example
 ```typescript
 const session = await openGuardedSession({ packageDirectory, agentDirectory, sessionDirectory, repository, realGitPath });
 ```
 */
async function openGuardedSession({
  packageDirectory,
  agentDirectory,
  sessionDirectory,
  repository,
  realGitPath,
}: GuardedSessionRequest,) {
  /**
   Settings with no compaction and no retry, so one scripted turn ends where it says.
   */
  const settingsManager = SettingsManager.inMemory({
    compaction: { enabled: false, },
    retry: {
      enabled: false,
      provider: { maxRetries: 0, },
    },
  },);
  /**
   Loader that discovers the package manifest and loads its declared extension.
   */
  const resourceLoader = new DefaultResourceLoader({
    cwd: repository.root,
    agentDir: agentDirectory,
    settingsManager,
    additionalExtensionPaths: [packageDirectory,],
  },);
  await resourceLoader.reload();
  /**
   Model registry isolated from this host's credentials and model files.
   */
  const modelRuntime = await ModelRuntime.create({
    authPath: join(
      agentDirectory,
      'auth.json',
    ),
    modelsPath: join(
      agentDirectory,
      'models.json',
    ),
  },);
  /**
   Bash calls the scripted model asks for, in the order the guardrail must see them.
   */
  const calls: readonly ScriptedBashCall[] = [
    { command: `${JSON.stringify(realGitPath,)} commit -m ${JSON.stringify(DENIED_COMMIT_MESSAGE,)}`, },
    { command: `${JSON.stringify(realGitPath,)} status --short`, },
    { command: `git commit -m ${JSON.stringify(ALLOWED_COMMIT_MESSAGE,)} ${repository.stagedFileName}`, },
  ];
  /**
   Scripted provider and its model.
   */
  const provider = registerScriptedGitGuardProvider({
    modelRuntime,
    calls,
  },);
  /**
   Session storage confined to the disposable directory.
   */
  const sessionManager = SessionManager.create(
    repository.root,
    sessionDirectory,
  );
  /**
   Real session that owns the extension lifecycle and the Bash tool.
   */
  const { session, extensionsResult, } = await createAgentSession({
    cwd: repository.root,
    agentDir: agentDirectory,
    model: provider.model,
    tools: ['bash',],
    resourceLoader,
    modelRuntime,
    settingsManager,
    sessionManager,
  },);
  if (extensionsResult.errors
    .length
    > 0) {
    session.dispose();
    throw new PiGuardrailVerificationError(`Pi reported extension errors: ${JSON.stringify(extensionsResult.errors,)}`,);
  }
  if (extensionsResult.extensions
    .length
    !== 1) {
    /**
     Loaded extension count, reported before the session is let go.
     */
    const loadedCount = extensionsResult.extensions
      .length;
    session.dispose();
    throw new PiGuardrailVerificationError(`expected one loaded guardrail extension, Pi loaded ${String(loadedCount,)}`,);
  }
  return {
    session,
    provider,
  };
}

/**
 A verification run that cannot answer its question.
 */
class PiGuardrailVerificationError extends Error {
  /**
   Name printed with the message.
   */
  override readonly name = 'PiGuardrailVerificationError';
}

//endregion Session construction

//region Verification

/**
 Runs the scripted Bash calls through a real Pi session and checks what the model saw.
 
 @param request - package, directories, repository, and real-Git path
 
 @returns the verified outcome
 
 @throws when Pi's answers or the repository's state differ from the guardrail's contract
 
 @example
 ```typescript
 await verifyAbsoluteGitGuardAtPiBoundary(request);
 ```
 */
async function verifyAbsoluteGitGuardAtPiBoundary(
  request: GuardedSessionRequest,
): Promise<GuardedSessionOutcome> {
  /**
   Open session under test and its scripted provider.
   */
  const { session, provider, } = await openGuardedSession(request,);
  try {
    await session.prompt('Run the scripted commands.',);
    /**
     Every tool result the real session produced.
     */
    const results = toolResultsOf(session.messages,);
    if (results.length !== 3) {
      throw new PiGuardrailVerificationError(
        `expected three Bash tool results, Pi produced ${String(results.length,)}: `
          + JSON.stringify(results.map(function commandOf(result,) {
            return result.toolName;
          },),),
      );
    }
    const [deniedResult, readOnlyResult, wrapperResult,] = results;
    if ((deniedResult === undefined) || (readOnlyResult === undefined) || (wrapperResult === undefined))
      throw new PiGuardrailVerificationError('Bash tool results are missing',);

    if (deniedResult.toolName !== 'bash' || !deniedResult.isError) {
      throw new PiGuardrailVerificationError(
        `the absolute-path commit was not denied: ${JSON.stringify({
          toolName: deniedResult.toolName,
          isError: deniedResult.isError,
          text: toolResultText(deniedResult,),
        },)}`,
      );
    }
    /**
     What the model read as the failure of the denied call.
     */
    const denialText = toolResultText(deniedResult,);
    if (!denialText.includes(DENIAL_PREFIX,))
      throw new PiGuardrailVerificationError(`denial does not start a reason: ${denialText}`,);
    if (!denialText.includes(REMEDY_SENTENCE,))
      throw new PiGuardrailVerificationError(`denial does not name the remedy \`${REMEDY_SENTENCE}\`: ${denialText}`,);
    if (!denialText.includes(BLOCKED_CALL_TEXT,))
      throw new PiGuardrailVerificationError(`denial does not say the call was blocked: ${denialText}`,);

    if (readOnlyResult.isError) {
      throw new PiGuardrailVerificationError(
        `the read-only absolute-path probe was refused: ${toolResultText(readOnlyResult,)}`,
      );
    }
    /**
     Output of the read-only probe through the real Git binary.
     */
    const readOnlyText = toolResultText(readOnlyResult,);
    if (!readOnlyText.includes(request.repository.stagedFileName,)) {
      throw new PiGuardrailVerificationError(
        `the read-only probe returned no staged file: ${readOnlyText}`,
      );
    }

    if (await hasNoCommit(request.repository,)) {
      throw new PiGuardrailVerificationError(
        `the repository still has no commit after the wrapper commit: ${toolResultText(wrapperResult,)}`,
      );
    }
    /**
     Commit subjects the guarded session left behind.
     */
    const subjects = await commitSubjects(request.repository,);
    if (JSON.stringify(subjects,) !== JSON.stringify([ALLOWED_COMMIT_MESSAGE,],)) {
      throw new PiGuardrailVerificationError(
        `expected one commit named ${JSON.stringify(ALLOWED_COMMIT_MESSAGE,)}, found ${JSON.stringify(subjects,)}`,
      );
    }
    if (provider.responsesLeft() !== 0) {
      throw new PiGuardrailVerificationError(
        `expected every scripted response to be used, ${String(provider.responsesLeft(),)} left`,
      );
    }
    /**
     Last assistant text of the run.
     */
    const lastAssistantText = session.getLastAssistantText() ?? '';
    if (!lastAssistantText.includes(CLOSING_TEXT,)) {
      throw new PiGuardrailVerificationError(
        `the run did not end on the scripted closing text: ${JSON.stringify(lastAssistantText,)}`,
      );
    }

    return {
      summary: [
        `denied ${JSON.stringify(DENIED_COMMIT_MESSAGE,)} through ${request.realGitPath}`,
        `allowed ${JSON.stringify(`${JSON.stringify(request.realGitPath,)} status --short`)}`,
        `landed ${JSON.stringify(ALLOWED_COMMIT_MESSAGE,)} through the PATH wrapper`,
      ].join('; ',),
    };
  }
  finally {
    session.dispose();
  }
}

//endregion Verification

export {
  DENIED_COMMIT_MESSAGE,
  PiGuardrailVerificationError,
  verifyAbsoluteGitGuardAtPiBoundary,
};
export type {
  GuardedSessionOutcome,
  GuardedSessionRequest,
};
