/**
 Exercises the literal-`rm`-path guard through a real Pi session.
 
 The session runs Pi's own agent loop, its `bash`, `write`, `read` and `codemode`
 tools, and the built extension, so a refusal here is a refusal where a model
 meets it. A recorder extension beside the guardrail records every `tool_call`
 the pipeline dispatches, which is how this verification answers what Pi hands an
 extension for a `codemode` call and for the calls a script makes.
 
 @module
 */

import {
  access,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import type { AgentSession, } from '@earendil-works/pi-coding-agent';
import type {
  ImageContent,
  TextContent,
  ToolResultMessage,
} from '@earendil-works/pi-ai';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { mismatch, } from './pi-runtime-verifier-literal-rm-error.ts';
import {
  disposeSession,
  LITERAL_SCRIPT_NAME,
  openLiteralRmSession,
  scriptedCalls,
  VIOLATING_SCRIPT_NAME,
  type LiteralRmSessionRequest,
} from './pi-runtime-verifier-literal-rm-session.ts';
import { CLOSING_TEXT, } from './pi-runtime-verifier-scripted-provider.ts';

//region Logging

/**
 Logger root for this verification.
 */
const parentLogger = tagged({ tag: 'pi-guardrail-literal-rm-verifier', },);

//endregion Logging

//region Constants

/**
 Prefix every shared denial reason starts with.
 */
const DENIAL_PREFIX = 'Blocked:';

/**
 Remedy sentence the literal-path refusal carries.
 */
const REMEDY_SENTENCE = '`rm --recursive /home/user/temp/agent/<folder>/scratch`';

/**
 Text a refused nested call leaves in the script that made it.
 */
const SCRIPT_REJECTION_TEXT = 'rejected';

/**
 Field separator of the tool-call record the summary prints.
 */
const RECORD_FIELD_SEPARATOR = '|';

/**
 Text fields every recorded tool call carries.
 */
const RECORDED_CALL_TEXT_FIELDS: readonly string[] = [
  'toolName',
  'toolCallId',
  'parentToolCallId',
  'commandPreview',
  'codePreview',
];

//endregion Constants

//region Types

/**
 One message Pi reports for the session, as the harness owns it.
 */
type SessionMessage = AgentSession['messages'][number];

/**
 One `tool_call` event the recorder extension saw.
 */
type RecordedToolCall = {
  /**
   Tool the pipeline was about to run.
   */
  readonly toolName: string;
  /**
   Identifier Pi assigned the call.
   */
  readonly toolCallId: string;
  /**
   Identifier of the call that issued it, or empty text for a call the model made.
   */
  readonly parentToolCallId: string;
  /**
   Field names the tool input carried.
   */
  readonly inputKeys: readonly string[];
  /**
   First characters of a Bash command the event carried, or empty text for another tool.
   */
  readonly commandPreview: string;
  /**
   First characters of a code-mode script the event carried, or empty text for another tool.
   */
  readonly codePreview: string;
};

/**
 What the real session showed.
 */
type LiteralRmSessionOutcome = {
  /**
   Summary naming each scripted call and what Pi answered.
   */
  readonly summary: string;
  /**
   Every tool call the recorder extension saw, in order.
   */
  readonly recordedCalls: readonly RecordedToolCall[];
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
 Reports whether one parsed log line is a recorded tool call.
 
 @param value - value parsed from one line of the recorder log
 
 @returns whether the value carries the fields a recorded call has
 
 @example
 ```typescript
 isRecordedToolCall(JSON.parse(line));
 ```
 */
function isRecordedToolCall(value: unknown,): value is RecordedToolCall {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  for (const field of RECORDED_CALL_TEXT_FIELDS) {
    if (!hasTextField({
      value,
      field,
    },))
      return false;
  }
  return isFieldNameList(
    Reflect.get(
      value,
      'inputKeys',
    ),
  );
}

/**
 Reports whether one object carries a field whose value is text.
 
 @param value - object read from parsed JSON
 
 @param field - field name to look for
 
 @returns whether the field is present and holds a string
 
 @example
 ```typescript
 hasTextField(record, 'toolName');
 ```
 */
function hasTextField({
  value,
  field,
}: {
  readonly value: object;
  readonly field: string;
},): boolean {
  return (field in value)
    && ((typeof Reflect.get(
      value,
      field,
    )) === 'string');
}

/**
 Reports whether one value is a list of field names.
 
 @param value - value read from parsed JSON
 
 @returns whether the value is an array of strings
 
 @example
 ```typescript
 isFieldNameList(['command']);
 ```
 */
function isFieldNameList(value: unknown,): boolean {
  if (!Array.isArray(value,))
    return false;
  return value.every(function isFieldName(field,): boolean {
    return (typeof field) === 'string';
  },);
}

/**
 Reports whether a path exists.
 
 @param path - absolute path to look for
 
 @returns whether anything is there
 
 @example
 ```typescript
 await pathExists('/repo/literal.sh');
 ```
 */
async function pathExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error: unknown) {
    /**
     Function-tagged logger reporting a path the check could not find.
     */
    const innerLogger = tagged({
      tag: pathExists.name,
      l: parentLogger,
    },);
    innerLogger.debug(`path not present: ${path}: ${caughtValueText(error,)}`,);
    return false;
  }
}

//endregion Message reads

//region Assertions

/**
 Checks that one result was refused for a computed deletion.
 
 @param result - tool result of the refused call
 
 @param label - which call this result belongs to, named in a mismatch
 
 @returns the denial text
 
 @example
 ```typescript
 requireRefusal({ result, label: 'computed-path bash call' });
 ```
 */
function requireRefusal({
  result,
  label,
}: {
  readonly result: ForeignBorrowed<ToolResultMessage>;
  readonly label: string;
},): string {
  if (!result.isError) {
    throw mismatch(
      `the ${label} was not refused: ${JSON.stringify({
        toolName: result.toolName,
        text: toolResultText(result,),
      },)}`,
    );
  }
  /**
   What the model read as the failure of the refused call.
   */
  const denialText = toolResultText(result,);
  if (!denialText.includes(DENIAL_PREFIX,))
    throw mismatch(`the ${label} denial carries no refusal reason: ${denialText}`,);
  if (!denialText.includes(REMEDY_SENTENCE,)) {
    throw mismatch(
      `the ${label} denial does not name the remedy \`${REMEDY_SENTENCE}\`: ${denialText}`,
    );
  }
  return denialText;
}

/**
 Checks that a refused write left no file behind.
 
 @param path - absolute path of the script the refused call would have written
 
 @example
 ```typescript
 await requireAbsent('/repo/violating.sh');
 ```
 */
async function requireAbsent(path: string,): Promise<void> {
  if (await pathExists(path,))
    throw mismatch(`the refused write left the file on disk: ${path}`,);
}

/**
 Checks that a permitted write landed its text.
 
 @param path - absolute path of the script the permitted call wrote
 
 @returns the text the file holds
 
 @example
 ```typescript
 const text = await requireWritten('/repo/literal.sh');
 ```
 */
async function requireWritten(path: string,): Promise<string> {
  if (!(await pathExists(path,)))
    throw mismatch(`the permitted write left no file at ${path}`,);
  return await readFile(
    path,
    'utf8',
  );
}

//endregion Assertions

//region Recorded calls

/**
 Reads the tool-call events the recorder extension logged.
 
 @param recorderLog - path the recorder appended to
 
 @returns one record per event, in order
 
 @example
 ```typescript
 const recorded = await recordedToolCalls(recorderLog);
 ```
 */
async function recordedToolCalls(recorderLog: string,): Promise<readonly RecordedToolCall[]> {
  if (!(await pathExists(recorderLog,)))
    return [];
  /**
   Log text, one JSON object per line.
   */
  const logText = await readFile(
    recorderLog,
    'utf8',
  );
  /**
   Every non-empty line of the log, in order.
   */
  const lines = logText.split('\n',);
  /**
   Records parsed from those lines.
   */
  const records: RecordedToolCall[] = [];
  for (const line of lines) {
    if (line.trim() === '')
      continue;
    /**
     Value parsed from one line of the log.
     */
    const parsed: unknown = JSON.parse(line,);
    if (isRecordedToolCall(parsed,))
      records.push(parsed,);
  }
  return records;
}

/**
 Spells one recorded call so a reader can see which call issued it.
 
 @param call - recorded tool call
 
 @returns the tool name, its input fields, and the call that issued it
 
 @example
 ```typescript
 callSentence({ toolName: 'bash', toolCallId: 'a/1', parentToolCallId: 'a', inputKeys: ['command'], commandPreview: '', codePreview: '' });
 ```
 */
function callSentence(call: RecordedToolCall,): string {
  /**
   Who issued the call, which is the model or the tool call that ran the script.
   */
  const origin = (call.parentToolCallId === '')
    ? 'from the model'
    : `under ${call.parentToolCallId}`;
  /**
   Input field names of that call, joined for one line of the summary.
   */
  const fields = call.inputKeys
    .join(RECORD_FIELD_SEPARATOR,);
  return `${call.toolName}(${fields}) ${origin}`;
}

//endregion Recorded calls

//region Verification

/**
 Runs the scripted calls through a real Pi session and checks what the model saw.
 
 @param packageDirectory - built package directory whose manifest names its extension
 
 @param agentDirectory - disposable Pi global directory
 
 @param sessionDirectory - disposable session-file directory
 
 @param workspaceRoot - working directory the session writes its scripts in
 
 @returns the verified outcome and every recorded tool call
 
 @throws {@link LiteralRmVerificationError} when Pi's answers or the workspace differ from the guardrail's contract
 
 @example
 ```typescript
 await verifyLiteralRmGuardAtPiBoundary(request);
 ```
 */
async function verifyLiteralRmGuardAtPiBoundary({
  packageDirectory,
  agentDirectory,
  sessionDirectory,
  workspaceRoot,
}: LiteralRmSessionRequest,): Promise<LiteralRmSessionOutcome> {
  /**
   Open session and the log its recorder extension appends to.
   */
  const {
    session,
    recorderLog,
  } = await openLiteralRmSession({
    packageDirectory,
    agentDirectory,
    sessionDirectory,
    workspaceRoot,
  },);
  /**
   Releases the session on every exit path of this verification.
   */
  using _owner = disposeSession(session,);

  await session.prompt('Run the scripted commands.',);
  /**
   Every tool result the real session produced, in the order Pi finished them.
   */
  const results: ForeignBorrowed<ToolResultMessage>[] = [];
  for (const message of session.messages) {
    if (message.role === 'toolResult')
      results.push(message,);
  }
  /**
   How many results the scripted run asked for.
   */
  const calls = scriptedCalls();
  /**
   How many results the scripted run asked for.
   */
  const expectedCallCount = calls.length;
  if (results.length !== expectedCallCount) {
    throw mismatch(
      `expected ${String(expectedCallCount,)} tool results, Pi produced ${String(results.length,)}`,
    );
  }
  /**
   The Bash, write and code-mode results, in the order the script asked for them.
   */
  const [
    deniedBashResult,
    allowedBashResult,
    deniedWriteResult,
    allowedWriteResult,
    commandFileResult,
    literalCodemodeResult,
    buildingCodemodeResult,
    nestedCodemodeResult,
  ] = results;
  if ((deniedBashResult === undefined)
    || (allowedBashResult === undefined)
    || (deniedWriteResult === undefined)
    || (allowedWriteResult === undefined)
    || (commandFileResult === undefined)
    || (literalCodemodeResult === undefined)
    || (buildingCodemodeResult === undefined)
    || (nestedCodemodeResult === undefined))
    throw mismatch('tool results are missing',);

  requireRefusal({
    result: deniedBashResult,
    label: 'computed-path bash call',
  },);
  if (allowedBashResult.isError) {
    throw mismatch(
      `the literal-path bash call was refused: ${toolResultText(allowedBashResult,)}`,
    );
  }
  requireRefusal({
    result: deniedWriteResult,
    label: 'computed-path write call',
  },);
  await requireAbsent(join(
    workspaceRoot,
    VIOLATING_SCRIPT_NAME,
  ),);
  await requireWritten(join(
    workspaceRoot,
    LITERAL_SCRIPT_NAME,
  ),);
  if (commandFileResult.isError) {
    throw mismatch(
      `the notes-file write was refused: ${toolResultText(commandFileResult,)}`,
    );
  }
  if (literalCodemodeResult.isError) {
    throw mismatch(
      `the literal-path code-mode script was refused: ${toolResultText(literalCodemodeResult,)}`,
    );
  }
  requireRefusal({
    result: buildingCodemodeResult,
    label: 'code-mode script that builds its command',
  },);
  if (nestedCodemodeResult.isError) {
    throw mismatch(
      `the code-mode script that reads its command from a file was refused: ${
        toolResultText(nestedCodemodeResult,)}`,
    );
  }
  if (!toolResultText(nestedCodemodeResult,)
    .includes(SCRIPT_REJECTION_TEXT,)) {
    throw mismatch(
      `the nested Bash call of the third code-mode script was not refused: ${
        toolResultText(nestedCodemodeResult,)}`,
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
      'refused `rm -rf "$d"` on bash',
      'ran the literal-path bash call',
      'refused the computed-path write and left no file',
      'landed the literal-path write',
      `landed the notes file that holds a deletion no write guard reads`,
      'ran the code-mode script with a literal-path bash call',
      'refused the code-mode script that builds its command',
      'refused the nested bash call of a code-mode script that reads its command from a file',
    ].join('; ',),
    recordedCalls: await recordedToolCalls(recorderLog,),
  };
}

//endregion Verification

export { callSentence, };
export { verifyLiteralRmGuardAtPiBoundary, };
export { DELETION_TARGET, } from './pi-runtime-verifier-literal-rm-session.ts';
export { LiteralRmVerificationError, } from './pi-runtime-verifier-literal-rm-error.ts';
export type { LiteralRmSessionOutcome, };
export type { RecordedToolCall, };
