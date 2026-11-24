/**
 Opens the real Pi session that runs the literal-`rm`-path verification.
 
 The session loads Pi's own code-mode extension beside the built guardrail and a
 recorder extension, so the verification can ask what Pi hands an extension for a
 `codemode` call and for the calls a script makes.
 
 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  type AgentSession,
  createAgentSession,
  createCodemodeExtension,
  DefaultResourceLoader,
  ModelRuntime,
  SessionManager,
  SettingsManager,
} from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { mismatch, } from './pi-runtime-verifier-literal-rm-error.ts';
import {
  registerScriptedToolCallProvider,
  type ScriptedToolCall,
} from './pi-runtime-verifier-scripted-provider.ts';

//region Constants

/**
 Tool whose Bash calls this verification drives.
 */
const BASH_TOOL_NAME = 'bash';

/**
 Tool that writes a whole file.
 */
const WRITE_TOOL_NAME = 'write';

/**
 Tool that reads a file, which one code-mode script uses to build its command.
 */
const READ_TOOL_NAME = 'read';

/**
 Tool that runs a JavaScript script calling other tools.
 */
const CODEMODE_TOOL_NAME = 'codemode';

/**
 Folder the literal deletions name, which the session creates for itself.
 */
const DELETION_TARGET = 'scratch-target';

/**
 Script the session writes, whose deletion computes its path.
 */
const VIOLATING_SCRIPT_NAME = 'violating.sh';

/**
 Script the session writes, whose deletion writes its path out.
 */
const LITERAL_SCRIPT_NAME = 'literal.sh';

/**
 Script the session writes, whose deletion collapses to the filesystem root.
 */
const COLLAPSED_SCRIPT_NAME = 'collapsed.sh';

/**
 Notes file a code-mode script reads a command out of.
 */
const COMMAND_FILE_NAME = 'command.txt';

/**
 Command the notes file holds, which no guard sees until the script reads it.
 */
const COMMAND_FILE_CONTENT = 'rm -rf "$d"\n';

/**
 Environment variable handing the recorder extension its log path.
 */
const RECORDER_LOG_VARIABLE = 'PI_LITERAL_RM_PROBE_LOG';

/**
 File name the recorder extension is written to.
 */
const RECORDER_FILE_NAME = 'recorder.mjs';

/**
 Source of the recorder extension, which appends one JSON line per tool call.
 */
const RECORDER_SOURCE = `import { appendFileSync } from 'node:fs';

export default async function recordToolCalls(pi) {
  const logPath = process.env.${RECORDER_LOG_VARIABLE};
  pi.on('tool_call', function record(event) {
    appendFileSync(logPath, JSON.stringify({
      toolName: event.toolName,
      toolCallId: event.toolCallId,
      parentToolCallId: event.parentToolCallId ?? '',
      inputKeys: Object.keys(event.input ?? {}),
      commandPreview: typeof event.input?.command === 'string'
        ? event.input.command.slice(0, 80)
        : '',
      codePreview: typeof event.input?.code === 'string'
        ? event.input.code.slice(0, 80)
        : '',
    }) + '\\n');
    return undefined;
  });
}
`;

//endregion Constants

//region Types

/**
 Everything the literal-path verification needs from its caller.
 */
type LiteralRmSessionRequest = {
  /**
   Package directory whose manifest names the built extension.
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
   Working directory the session runs in, which holds the written scripts.
   */
  readonly workspaceRoot: string;
};

/**
 Open session and the log its recorder extension appends to.
 */
type OpenLiteralRmSession = {
  /**
   Session that owns the extension lifecycle and the tools under test.
   */
  readonly session: AgentSession;
  /**
   Path the recorder extension appends its events to.
   */
  readonly recorderLog: string;
};

//endregion Types

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
 Reads the scripted calls this verification runs, in order.
 
 @returns tool calls the scripted model asks for
 
 @example
 ```typescript
 scriptedCalls();
 ```
 */
function scriptedCalls(): readonly ScriptedToolCall[] {
  return [
    //region The Bash surface
    {
      toolName: BASH_TOOL_NAME,
      arguments: { command: 'rm -rf "$d"', },
    },
    {
      toolName: BASH_TOOL_NAME,
      arguments: { command: `rm --recursive ${join(DELETION_TARGET,)}`, },
    },
    {
      toolName: BASH_TOOL_NAME,
      arguments: { command: 'rmdir -p "$d"', },
    },
    {
      toolName: BASH_TOOL_NAME,
      arguments: { command: 'rm -rf /home', },
    },
    //endregion The Bash surface

    //region The write surface
    {
      toolName: WRITE_TOOL_NAME,
      arguments: {
        path: VIOLATING_SCRIPT_NAME,
        content: '#!/bin/sh\nrm -rf "$d"\n',
      },
    },
    {
      toolName: WRITE_TOOL_NAME,
      arguments: {
        path: LITERAL_SCRIPT_NAME,
        content: `#!/bin/sh\nrm --recursive ${join(DELETION_TARGET,)}\n`,
      },
    },
    {
      toolName: WRITE_TOOL_NAME,
      arguments: {
        path: COLLAPSED_SCRIPT_NAME,
        content: '#!/bin/sh\nrm -rf /\n',
      },
    },
    {
      toolName: WRITE_TOOL_NAME,
      arguments: {
        path: COMMAND_FILE_NAME,
        content: COMMAND_FILE_CONTENT,
      },
    },
    //endregion The write surface

    //region Code mode
    {
      toolName: CODEMODE_TOOL_NAME,
      arguments: {
        code: `const run = await tools.bash({ command: 'rm --recursive ${join(DELETION_TARGET,)}' });
text(run.exit_code);
`,
      },
    },
    {
      toolName: CODEMODE_TOOL_NAME,
      arguments: {
        code: `const word = "x";
const command = \`rm -rf \${word}\`;
const run = await tools.bash({ command });
text(run.exit_code);
`,
      },
    },
    {
      toolName: CODEMODE_TOOL_NAME,
      arguments: {
        code: `const command = await tools.read({ path: '${COMMAND_FILE_NAME}' });
const results = await Promise.allSettled([tools.bash({ command })]);
const statuses = results.map(function statusOf(result) { return result.status; },);
text(statuses.join(','));
`,
      },
    },
    //endregion Code mode
  ];
}

/**
 Opens the real Pi session that runs every scripted call.
 
 @param packageDirectory - built package directory whose manifest names its extension
 
 @param agentDirectory - disposable Pi global directory
 
 @param sessionDirectory - disposable session-file directory
 
 @param workspaceRoot - working directory the session writes its scripts in
 
 @returns the open session and the path of its recorder log
 
 @throws {@link LiteralRmVerificationError} when Pi reports extension load errors
 
 @example
 ```typescript
 const { session } = await openLiteralRmSession(request);
 ```
 */
async function openLiteralRmSession({
  packageDirectory,
  agentDirectory,
  sessionDirectory,
  workspaceRoot,
}: LiteralRmSessionRequest,): Promise<OpenLiteralRmSession> {
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
   Path the recorder extension appends its events to.
   */
  const recorderLog = join(
    workspaceRoot,
    'tool-calls.jsonl',
  );
  /**
   Path of the recorder extension itself.
   */
  const recorderPath = join(
    workspaceRoot,
    RECORDER_FILE_NAME,
  );
  await writeFile(
    recorderPath,
    RECORDER_SOURCE,
  );
  process.env[RECORDER_LOG_VARIABLE] = recorderLog;
  /**
   Folder the literal deletions name, created so the permitted calls succeed.
   */
  await mkdir(
    join(
      workspaceRoot,
      DELETION_TARGET,
    ),
    { recursive: true, },
  );
  /**
   Loader that discovers the package manifest, the recorder, and Pi's own code-mode extension.
   
   The recorder is loaded first so it sees every call, including the ones the
   guardrail blocks, because a blocked call stops the handler walk.
   */
  const resourceLoader = new DefaultResourceLoader({
    cwd: workspaceRoot,
    agentDir: agentDirectory,
    settingsManager,
    additionalExtensionPaths: [
      recorderPath,
      packageDirectory,
    ],
    extensionFactories: [
      createCodemodeExtension({ models: false, },),
    ],
  },);
  await resourceLoader.reload();
  /**
   Model registry isolated from this host's credentials and model files.
   */
  const modelRuntime: ForeignBorrowed<ModelRuntime> = await ModelRuntime.create({
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
   Scripted provider and its model.
   */
  const provider = registerScriptedToolCallProvider({
    modelRuntime,
    calls: scriptedCalls(),
  },);
  /**
   Session storage confined to the disposable directory.
   */
  const sessionManager = SessionManager.create(
    workspaceRoot,
    sessionDirectory,
  );
  /**
   Real session that owns the extension lifecycle and the tools under test.
   */
  const {
    session,
    extensionsResult,
  } = await createAgentSession({
    cwd: workspaceRoot,
    agentDir: agentDirectory,
    model: provider.model,
    tools: [
      BASH_TOOL_NAME,
      WRITE_TOOL_NAME,
      READ_TOOL_NAME,
      CODEMODE_TOOL_NAME,
    ],
    resourceLoader,
    modelRuntime,
    settingsManager,
    sessionManager,
  },);
  if (extensionsResult.errors
    .length
    > 0) {
    session.dispose();
    throw mismatch(`Pi reported extension errors: ${JSON.stringify(extensionsResult.errors,)}`,);
  }
  return {
    session,
    recorderLog,
  };
}

//endregion Session construction

export {
  COLLAPSED_SCRIPT_NAME,
  COMMAND_FILE_NAME,
  CODEMODE_TOOL_NAME,
  DELETION_TARGET,
  disposeSession,
  LITERAL_SCRIPT_NAME,
  openLiteralRmSession,
  READ_TOOL_NAME,
  scriptedCalls,
  VIOLATING_SCRIPT_NAME,
  WRITE_TOOL_NAME,
};
export type { LiteralRmSessionRequest, };
