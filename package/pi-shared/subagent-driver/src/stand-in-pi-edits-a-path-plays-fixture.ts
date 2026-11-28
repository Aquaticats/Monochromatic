/**
 What the stand-in's Pi does with an `edit` call: it plays Pi's own pipeline around one call, so a test sees whether the call
 reaches the file or is refused before any tool runs.

 The order is Pi's: `tool_execution_start`, then the `tool_call` hook, where a refusal ends the call before the tool runs, then
 `tool_execution_end` with what the tool answered. A call nothing refused is written by this play, which is the whole of what Pi's
 edit tool would have done to the file.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { dirname, } from 'node:path';
import { env, } from 'node:process';
import {
  answer,
  emit,
} from './stand-in-pi-records-fixture.ts';
import { standInMainCheckoutGuard, } from './stand-in-pi-main-checkout-guard-fixture.ts';

/**
 The scenario in which a run's Pi makes one `edit` call and ends.
 */
export const EDITS_A_PATH_SCENARIO = 'edits-a-path';

/**
 The variable naming the path this play's `edit` call names.
 */
const PATH_VARIABLE = 'STAND_IN_PI_EDIT_PATH';

/**
 The variable holding the text the call would write.
 */
const TEXT_VARIABLE = 'STAND_IN_PI_EDIT_TEXT';

/**
 The variable that says which tool the call is: `edit` unless a test asks for the other tool this guard reads.
 */
const TOOL_VARIABLE = 'STAND_IN_PI_EDIT_TOOL';

/**
 The variable that says whether the path comes last in the call's arguments, as a model that wrote a long text first leaves it.
 */
const PATH_LAST_VARIABLE = 'STAND_IN_PI_EDIT_PATH_LAST';

/**
 The identifier of the one tool call this play makes.
 */
const TOOL_CALL_ID = 'call-1';

/**
 The text a play writes when a test names none.
 */
const DEFAULT_TEXT = 'a line the stand-in wrote\n';

/**
 Reads the path this play's `edit` call names.

 @returns the path, as the call spells it

 @throws Error when the test named no path
 */
function editedPath(): string {
  /**
   The path the test named.
   */
  const path = env[PATH_VARIABLE];
  if ((path === undefined) || (path === '')) {
    throw new Error(`the ${EDITS_A_PATH_SCENARIO} scenario needs ${PATH_VARIABLE}`,);
  }
  return path;
}

/**
 Plays one `edit` or `write` call: the start, the hook's answer, and what Pi keeps of it.

 @param id - the prompt's identifier, answered before the play so the driver's watch begins

 @example
 ```ts
 await editOnePath({ id: 'c1', });
 ```
 */
export async function editOnePath({ id, }: { readonly id: unknown },): Promise<void> {
  /**
   The path this call names.
   */
  const path = editedPath();
  /**
   The tool the call is, which is `edit` unless a test asks for a write.
   */
  const toolName = env[TOOL_VARIABLE] ?? 'edit';
  /**
   The text this call would write.
   */
  const text = env[TEXT_VARIABLE] ?? DEFAULT_TEXT;
  /**
   The call's arguments, in the order a test asked for: a model that writes a long text first leaves the path below the cut a
   feed line keeps.
   */
  const args = (env[PATH_LAST_VARIABLE] === 'yes')
    ? {
      content: text,
      path,
    }
    : {
      path,
      content: text,
    };
  answer({ id, },);
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit({
    type: 'tool_execution_start',
    toolCallId: TOOL_CALL_ID,
    toolName,
    args,
  },);
  /**
   What the run's guard answered this call with, which is nothing when the call may go.
   */
  const answered = await standInMainCheckoutGuard()
    .refusalOf({
      toolName,
      input: args,
    },);
  if ('allowed' in answered) {
    await mkdir(
      dirname(path,),
      { recursive: true, },
    );
    await writeFile(
      path,
      text,
    );
  }
  /**
   What Pi keeps of the call: the refusal's words, or the words its own edit tool answers a landed call with, built as
   `createEditTool` builds them (`dist/core/tools/edit.js` of `@earendil-works/pi-coding-agent` 1.6.0).
   */
  const kept = ('refusal' in answered) ? answered.refusal : `Successfully replaced 1 block(s) in ${path}.`;
  emit({
    type: 'tool_execution_end',
    toolCallId: TOOL_CALL_ID,
    toolName,
    isError: 'refusal' in answered,
    result: {
      content: [
        {
          type: 'text',
          text: kept,
        },
      ],
    },
  },);
  emit({
    type: 'agent_end',
    messages: [],
  },);
  emit({ type: 'agent_settled', },);
}
