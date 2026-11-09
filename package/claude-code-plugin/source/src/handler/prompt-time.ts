import type {
  UserPromptSubmitInput,
  UserPromptSubmitOutput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';

import {
  makeHookInputParser,
  makeJsonWriter,
  type Parser,
  type JsonWriter,
} from '../runtime/handler-runtime.ts';
import type { ReadonlyDeep, } from 'type-fest';

import { formatTimeContext, } from '@monochromatic-dev/agent-harness-shared-current-time-context/ts';

/**
 Output of the prompt-time handler, a {@link UserPromptSubmitOutput}.
 
 Always populates `hookSpecificOutput.additionalContext`: the handler never
 blocks the prompt or chooses an empty response.
 */
type PromptTimeOutput = UserPromptSubmitOutput;

/**
 Injects the current local system time into Claude's conversation context
 each time the user submits a prompt.
 
 Reads the wall clock at handler-call time; non-deterministic by design.
 The pure {@link formatTimeContext} helper is what unit tests target.
 
 @param _event - parsed {@link UserPromptSubmitInput} event from Claude Code (unused; the
   handler does not inspect the prompt text)
 
 @returns response carrying `<time>HH:MM</time>` as `additionalContext`
 
 @example
 ```ts
 promptTimeHandler({
   hook_event_name: 'UserPromptSubmit',
   prompt: 'hello',
   session_id: 's',
   transcript_path: '/t',
   cwd: '/c',
 });
 // { hookSpecificOutput: { hookEventName: 'UserPromptSubmit',
 //                         additionalContext: '<time>20:48</time>' } }
 ```
 */
function promptTimeHandler(_event: ReadonlyDeep<UserPromptSubmitInput>,): PromptTimeOutput {
  return {
    hookSpecificOutput: {
      hookEventName: 'UserPromptSubmit',
      additionalContext: formatTimeContext(new Date(),),
    },
  };
}

/**
 Parses raw stdin as a {@link UserPromptSubmitInput}.
 
 Input is trusted; it comes from Claude Code's hook dispatch system.
 
 @example
 ```ts
 const event = promptTimeParser(await text(process.stdin));
 ```
 */
const promptTimeParser: Parser<UserPromptSubmitInput> = makeHookInputParser<UserPromptSubmitInput>();

/**
 Serializes the prompt-time output for stdout.
 
 No trailing newline; matches Claude Code's wire convention.
 
 @example
 ```ts
 process.stdout.write(promptTimeWriter(output));
 ```
 */
const promptTimeWriter: JsonWriter<PromptTimeOutput> = makeJsonWriter<PromptTimeOutput>();

export type { PromptTimeOutput, };

export { formatTimeContext, } from '@monochromatic-dev/agent-harness-shared-current-time-context/ts';

export {
  promptTimeHandler,
  promptTimeParser,
  promptTimeWriter,
};
