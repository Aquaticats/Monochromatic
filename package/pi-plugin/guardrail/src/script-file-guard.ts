/**
 Shell-script guard for pi's `write` and `edit` tool calls.
 
 A script is judged by the text the call would leave on disk,
 never by the new text alone:
 two edits could otherwise split one computed deletion between them,
 and a rewrite that keeps an older violation is no new violation.
 The comparison lives in the shared analyzer, so the Claude Code guardrail
 refuses the same changes.
 
 @module
 */

import { resolve, } from 'node:path';

import {
  literalRmPathFindingsAnEditWouldAdd,
  literalRmPathFindingsAWriteWouldAdd,
  literalRmPathRefusal,
  type TextEdit,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  GUARDRAIL_NOT_BLOCKED,
  type GuardrailDecision,
} from './types.ts';
import { isRecord, } from './value.ts';

//region Logging

/**
 Logger root for the shell-script guard.
 */
const parentLogger = tagged({ tag: 'pi-guardrail-file', },);

//endregion Logging

//region Constants

/**
 Sentinel returned when the input carries no file change to judge.
 */
const NO_FILE_CHANGE: unique symbol = Symbol('pi-guardrail/no-file-change',);

/**
 Tool name whose input is a whole file's text.
 */
const WRITE_TOOL_NAME = 'write';

/**
 Tool name whose input is a list of replacements.
 */
const EDIT_TOOL_NAME = 'edit';

//endregion Constants

//region Types

/**
 One replacement a pi `edit` call reports.
 */
type PiTextEdit = {
  /**
   Text the replacement finds.
   */
  readonly oldText: string;
  /**
   Text the replacement writes.
   */
  readonly newText: string;
};

/**
 File change a pi tool call asks for, as this guard judges it.
 */
type PiFileChange = {
  /**
   Absolute path of the file the call changes.
   */
  readonly path: string;
  /**
   Tool name that asked for it.
   */
  readonly toolName: string;
} & (
  | {
    /**
     The call writes the whole file.
     */
    readonly kind: 'write';
    /**
     Text the call leaves in the file.
     */
    readonly content: string;
  }
    | {
      /**
       The call replaces strings inside the file.
       */
      readonly kind: 'edits';
      /**
       Replacements the call makes, in the order it reports them.
       */
      readonly edits: readonly TextEdit[];
    }
);

//endregion Types

//region Readers

/**
 Reads one field of a tool input read defensively.
 
 @param input - tool input as the model wrote it
 
 @param field - field name to read
 
 @returns the field's value, or undefined when the input carries none
 
 @example
 ```typescript
 fieldOf({ input: { path: 'clean.sh', }, field: 'path', }); // 'clean.sh'
 ```
 */
function fieldOf({
  input,
  field,
}: {
  readonly input: Readonly<Record<string, unknown>>;
  readonly field: string;
},): unknown {
  return Object.hasOwn(
    input,
    field,
  )
    ? input[field]
    : undefined;
}

/**
 Reads one replacement from a pi `edit` list.
 
 @param value - value read from that list
 
 @returns the replacement, or {@link NO_FILE_CHANGE} when the value is not one
 
 @example
 ```typescript
 textEditOf({ oldText: 'a', newText: 'b' }); // { oldText: 'a', newText: 'b' }
 ```
 */
function textEditOf(value: unknown,): PiTextEdit | typeof NO_FILE_CHANGE {
  if (!isRecord(value,))
    return NO_FILE_CHANGE;
  /**
   Text the replacement finds.
   */
  const oldText = fieldOf({
    input: value,
    field: 'oldText',
  },);
  /**
   Text the replacement writes.
   */
  const newText = fieldOf({
    input: value,
    field: 'newText',
  },);
  if (((typeof oldText) !== 'string') || ((typeof newText) !== 'string'))
    return NO_FILE_CHANGE;
  return {
    oldText,
    newText,
  };
}

/**
 Reads the file change one pi tool call asks for.
 
 @param toolName - name pi reported for the tool
 
 @param input - tool input read defensively
 
 @param cwd - pi current working directory, which a relative path resolves against
 
 @returns the change to judge, or {@link NO_FILE_CHANGE} when the call changes no file
 
 @example
 ```typescript
 fileChangeOf({ toolName: 'write', input: { path: 'clean.sh', content: 'rm -rf "$d"\n' }, cwd: '/repo' });
 ```
 */
function fileChangeOf({
  toolName,
  input,
  cwd,
}: {
  readonly toolName: string;
  readonly input: unknown;
  readonly cwd: string;
},): PiFileChange | typeof NO_FILE_CHANGE {
  if (!isRecord(input,))
    return NO_FILE_CHANGE;
  /**
   Path of the file the call changes.
   */
  const path = fieldOf({
    input,
    field: 'path',
  },);
  if ((typeof path) !== 'string')
    return NO_FILE_CHANGE;
  /**
   Absolute path the guard reads and judges.
   */
  const absolutePath = resolve(
    cwd,
    path,
  );

  if (toolName === WRITE_TOOL_NAME) {
    /**
     Text the call leaves in the file.
     */
    const content = fieldOf({
      input,
      field: 'content',
    },);
    if ((typeof content) !== 'string')
      return NO_FILE_CHANGE;
    return {
      kind: 'write',
      toolName,
      path: absolutePath,
      content,
    };
  }

  if (toolName !== EDIT_TOOL_NAME)
    return NO_FILE_CHANGE;

  /**
   Value of the replacement list the call reports.
   */
  const listed = fieldOf({
    input,
    field: 'edits',
  },);
  if (!Array.isArray(listed,))
    return NO_FILE_CHANGE;

  /**
   Replacements read from that list.
   */
  const edits: TextEdit[] = [];
  for (const value of listed) {
    /**
     One replacement read from the list.
     */
    const edit = textEditOf(value,);
    if ((typeof edit) === 'symbol')
      return NO_FILE_CHANGE;
    edits.push(edit,);
  }
  return {
    kind: 'edits',
    toolName,
    path: absolutePath,
    edits,
  };
}

//endregion Readers

//region Guard evaluation

/**
 Refuses a file change that adds a computed deletion to a shell script.
 
 @param change - file change the call asks for
 
 @returns block decision when the change adds a violation, otherwise {@link GUARDRAIL_NOT_BLOCKED}
 
 @example
 ```typescript
 await evaluateFileChange({ change: { kind: 'write', path: '/repo/clean.sh', content: 'rm -rf "$d"\n' } });
 ```
 */
async function evaluateFileChange(change: PiFileChange,): Promise<GuardrailDecision> {
  /**
   Violations the call would add to the file.
   */
  const findings = (change.kind === 'write')
    ? await literalRmPathFindingsAWriteWouldAdd({
      path: change.path,
      content: change.content,
    },)
    : await literalRmPathFindingsAnEditWouldAdd({
      path: change.path,
      edits: change.edits,
    },);
  if (findings.length === 0)
    return GUARDRAIL_NOT_BLOCKED;
  return {
    block: true,
    reason: literalRmPathRefusal({ findings, },),
  };
}

/**
 Applies the shell-script guard to one pi tool call.
 
 Only `write` and `edit` reach it;
 every other tool answers {@link GUARDRAIL_NOT_BLOCKED} without reading a file.
 
 @param toolName - name pi reported for the tool
 
 @param input - pi tool input read defensively
 
 @param cwd - pi current working directory
 
 @returns block decision when the call adds a computed deletion to a script
 
 @example
 ```typescript
 await evaluateScriptFileGuard({ toolName: 'write', input: { path: 'clean.sh', content: 'rm -rf "$d"\n' }, cwd: '/repo' });
 ```
 */
async function evaluateScriptFileGuard({
  toolName,
  input,
  cwd,
}: {
  readonly toolName: string;
  readonly input: unknown;
  readonly cwd: string;
},): Promise<GuardrailDecision> {
  /**
   Change this call asks for, absent for every other tool.
   */
  const change = fileChangeOf({
    toolName,
    input,
    cwd,
  },);
  if ((typeof change) === 'symbol') {
    /**
     Function-tagged logger reporting a call this guard could not read.
     */
    const innerLogger = tagged({
      tag: evaluateScriptFileGuard.name,
      l: parentLogger,
    },);
    innerLogger.debug(`no file change read from ${toolName} input`,);
    return GUARDRAIL_NOT_BLOCKED;
  }
  return await evaluateFileChange(change,);
}

//endregion Guard evaluation

export {
  evaluateScriptFileGuard,
  fileChangeOf,
  NO_FILE_CHANGE,
};
export type { PiFileChange, };
