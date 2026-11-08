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
import type {
  ImageContent,
  TextContent,
  ToolResultMessage,
} from '@earendil-works/pi-ai';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  commitSubjects,
  hasNoCommit,
  type ThrowawayRepository,
} from './pi-runtime-verifier-repository.ts';
import {
  CLOSING_TEXT,
  registerScriptedGitGuardProvider,
  type ScriptedBashCall,
  type ScriptedGitGuardProvider,
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
 Word the denial uses for what it did to the call.
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

/**
 Tool whose Bash calls this verification drives.
 */
const BASH_TOOL_NAME = 'bash';

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
 One message Pi reports for the session, as the harness owns it.
 */
type SessionMessage = AgentSession['messages'][number];

/**
 Open session under test and the provider that scripted its calls.
 */
type GuardedSession = {
  /**
   Real session that owns the loaded extension and the Bash tool.
   */
  readonly session: AgentSession;
  /**
   Scripted provider, whose remaining count shows whether every call ran.
   */
  readonly provider: ScriptedGitGuardProvider;
};

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
function toolResultText(message: ForeignBorrowed<ToolResultMessage>,): string {
  return message.content
    .flatMap(function textParts(part: ForeignBorrowed<TextContent | ImageContent>,) {
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
function toolResultsOf(
  messages: readonly ForeignBorrowed<SessionMessage>[],
): readonly ForeignBorrowed<ToolResultMessage>[] {
  return messages.flatMap(function toolResults(message,) {
    return message.role === 'toolResult' ? [message,] : [];
  },);
}

//endregion Message reads

//region Session construction

/**
 Releases the session and everything it holds.
 
 @param session - session to release
 
 @returns disposable owner for `using`
 
 @example
 ```typescript
 using _owner = disposeSession(session);
 ```
 */
function disposeSession(session: AgentSession,): Disposable {
  return {
    [Symbol.dispose]() {
      session.dispose();
    },
  };
}

/**
 Creates the real Pi session that runs the scripted Bash calls.
 
 @param packageDirectory - built package directory whose manifest names its extension
 
 @param agentDirectory - disposable Pi global directory
 
 @param sessionDirectory - disposable session-file directory
 
 @param repository - throwaway repository the Bash tool runs in
 
 @param realGitPath - absolute path of this host's real Git binary
 
 @returns session and its scripted provider
 
 @throws when Pi reports extension load errors or loads a different count
 
 @example
 ```typescript
 const guarded = await openGuardedSession({ packageDirectory, agentDirectory, sessionDirectory, repository, realGitPath });
 ```
 */
async function openGuardedSession({
  packageDirectory,
  agentDirectory,
  sessionDirectory,
  repository,
  realGitPath,
}: GuardedSessionRequest,): Promise<GuardedSession> {
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
    {
      command: `${JSON.stringify(realGitPath,)} commit -m ${JSON.stringify(DENIED_COMMIT_MESSAGE,)}`,
    },
    { command: `${JSON.stringify(realGitPath,)} status --short`, },
    {
      command: `git commit -m ${JSON.stringify(ALLOWED_COMMIT_MESSAGE,)} ${repository.stagedFileName}`,
    },
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
  const {
    session,
    extensionsResult,
  } = await createAgentSession({
    cwd: repository.root,
    agentDir: agentDirectory,
    model: provider.model,
    tools: [BASH_TOOL_NAME,],
    resourceLoader,
    modelRuntime,
    settingsManager,
    sessionManager,
  },);
  if (extensionsResult.errors
    .length
    > 0) {
    session.dispose();
    throw new PiGuardrailVerificationError(
      `Pi reported extension errors: ${JSON.stringify(extensionsResult.errors,)}`,
    );
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
    throw new PiGuardrailVerificationError(
      `expected one loaded guardrail extension, Pi loaded ${String(loadedCount,)}`,
    );
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

//region Assertions

/**
 Reports one mismatch between what Pi answered and what the guardrail promises.
 
 @param problem - what differed, in plain words
 
 @returns the error to throw
 
 @example
 ```typescript
 throw mismatch(`expected three tool results, Pi produced ${String(count,)}`);
 ```
 */
function mismatch(problem: string,): PiGuardrailVerificationError {
  return new PiGuardrailVerificationError(problem,);
}

/**
 Checks the result of the refused call and returns the text the model read.
 
 @param result - tool result of the denied call
 
 @returns the denial text
 
 @throws {@link PiGuardrailVerificationError} when the call was not refused with the remedy
 
 @example
 ```typescript
 const denialText = requireRefusal(deniedResult);
 ```
 */
function requireRefusal(result: ForeignBorrowed<ToolResultMessage>,): string {
  if ((result.toolName !== BASH_TOOL_NAME) || (!result.isError)) {
    throw mismatch(
      `the absolute-path commit was not refused: ${JSON.stringify({
        toolName: result.toolName,
        isError: result.isError,
        text: toolResultText(result,),
      },)}`,
    );
  }
  /**
   What the model read as the failure of the refused call.
   */
  const denialText = toolResultText(result,);
  if (!denialText.includes(DENIAL_PREFIX,))
    throw mismatch(`the denial carries no refusal reason: ${denialText}`,);
  if (!denialText.includes(BLOCKED_CALL_TEXT,))
    throw mismatch(`the denial does not say the call was blocked: ${denialText}`,);
  if (!denialText.includes(REMEDY_SENTENCE,)) {
    throw mismatch(
      `the denial does not name the remedy \`${REMEDY_SENTENCE}\`: ${denialText}`,
    );
  }
  return denialText;
}

/**
 Checks the result of the read-only probe and returns its output.
 
 @param result - tool result of the read-only call
 
 @param stagedFileName - staged file the probe must print
 
 @returns the probe's output
 
 @throws {@link PiGuardrailVerificationError} when the probe was refused or printed nothing staged
 
 @example
 ```typescript
 const statusText = requireReadOnlyOutput(readOnlyResult, stagedFileName);
 ```
 */
function requireReadOnlyOutput({
  result,
  stagedFileName,
}: {
  readonly result: ForeignBorrowed<ToolResultMessage>;
  readonly stagedFileName: string;
},): string {
  if (result.isError)
    throw mismatch(`the read-only absolute-path probe was refused: ${toolResultText(result,)}`,);
  /**
   Output of the probe through the real Git binary.
   */
  const readOnlyText = toolResultText(result,);
  if (!readOnlyText.includes(stagedFileName,)) {
    throw mismatch(
      `the read-only probe printed no staged file: ${readOnlyText}`,
    );
  }
  return readOnlyText;
}

//endregion Assertions

//region Verification

/**
 Runs the scripted Bash calls through a real Pi session and checks what the model saw.
 
 @param packageDirectory - built package directory whose manifest names its extension
 
 @param agentDirectory - disposable Pi global directory
 
 @param sessionDirectory - disposable session-file directory
 
 @param repository - throwaway repository the Bash tool runs in
 
 @param realGitPath - absolute path of this host's real Git binary
 
 @returns the verified outcome
 
 @throws when Pi's answers or the repository's state differ from the guardrail's contract
 
 @example
 ```typescript
 await verifyAbsoluteGitGuardAtPiBoundary({ packageDirectory, agentDirectory, sessionDirectory, repository, realGitPath });
 ```
 */
async function verifyAbsoluteGitGuardAtPiBoundary({
  packageDirectory,
  agentDirectory,
  sessionDirectory,
  repository,
  realGitPath,
}: GuardedSessionRequest,): Promise<GuardedSessionOutcome> {
  /**
   Open session under test and its scripted provider.
   */
  const {
    session,
    provider,
  } = await openGuardedSession({
    packageDirectory,
    agentDirectory,
    sessionDirectory,
    repository,
    realGitPath,
  },);
  /**
   Releases the session on every exit path of this verification.
   */
  using _owner = disposeSession(session,);

  await session.prompt('Run the scripted commands.',);
  /**
   Every tool result the real session produced, in the order Pi finished them.
   */
  const results = toolResultsOf(session.messages,);
  if (results.length !== provider.commandCount) {
    throw mismatch(
      `expected ${String(provider.commandCount,)} Bash tool results, `
        + `Pi produced ${String(results.length,)}`,
    );
  }
  /**
   The refused call, the read-only probe, and the call through the PATH wrapper, in that order.
   */
  const [
    deniedResult,
    readOnlyResult,
    wrapperResult,
  ] = results;
  if ((deniedResult === undefined)
    || (readOnlyResult === undefined)
    || (wrapperResult === undefined))
    throw mismatch('Bash tool results are missing',);

  requireRefusal(deniedResult,);
  requireReadOnlyOutput({
    result: readOnlyResult,
    stagedFileName: repository.stagedFileName,
  },);

  if (await hasNoCommit({ repository, },)) {
    throw mismatch(
      `the repository has no commit after the wrapper commit: ${toolResultText(wrapperResult,)}`,
    );
  }
  /**
   Commit subjects the guarded session left behind.
   */
  const subjects = await commitSubjects({ repository, },);
  if (JSON.stringify(subjects,) !== JSON.stringify([ALLOWED_COMMIT_MESSAGE,],)) {
    throw mismatch(
      `expected one commit named ${JSON.stringify(ALLOWED_COMMIT_MESSAGE,)}, `
        + `found ${JSON.stringify(subjects,)}`,
    );
  }
  if (provider.responsesLeft() !== 0) {
    throw mismatch(
      `expected every scripted response to be used, ${String(provider.responsesLeft(),)} left`,
    );
  }
  /**
   Last assistant text of the run.
   */
  const lastAssistantText = session.getLastAssistantText() ?? '';
  if (!lastAssistantText.includes(CLOSING_TEXT,)) {
    throw mismatch(
      `the run did not end on the scripted closing text: ${JSON.stringify(lastAssistantText,)}`,
    );
  }

  return {
    summary: [
      `refused ${JSON.stringify(DENIED_COMMIT_MESSAGE,)} through ${realGitPath}`,
      `ran ${JSON.stringify(`${JSON.stringify(realGitPath,)} status --short`)}`,
      `landed ${JSON.stringify(ALLOWED_COMMIT_MESSAGE,)} through the PATH wrapper`,
    ].join('; ',),
  };
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
