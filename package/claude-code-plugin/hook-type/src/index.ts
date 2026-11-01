/**
 TypeScript type definitions for all Claude Code hook events.
 
 Covers every hook event in the Claude Code lifecycle with typed inputs, outputs,
 and tool-specific input shapes for built-in tools.
 
 @see [Hooks reference](https://docs.anthropic.com/en/docs/claude-code/hooks)
 
 @example
 ```ts
 import type { HookInput } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
 
 const event = JSON.parse(raw) as HookInput;
 switch (event.hook_event_name) {
   case 'PreToolUse':
     console.log(event.tool_name, event.tool_input);
     break;
   case 'Stop':
     console.log(event.stop_hook_active);
     break;
 }
 ```
 
 @module
 */

import type {
  StopInput,
  SubagentStartInput,
  SubagentStopInput,
  TaskCompletedInput,
  TeammateIdleInput,
} from './events-agent.ts';
import type {
  NotificationInput,
  PreCompactInput,
  UserPromptSubmitInput,
  WorktreeCreateInput,
  WorktreeRemoveInput,
} from './events-misc.ts';
import type {
  ConfigChangeInput,
  InstructionsLoadedInput,
  SessionEndInput,
  SessionStartInput,
} from './events-session.ts';
import type {
  PostToolUseFailureInput,
  PostToolUseInput,
} from './events-tool-post.ts';
import type {
  PermissionRequestInput,
  PreToolUseInput,
} from './events-tool.ts';

//region Re-exports

export type * from './common.ts';
export type * from './events-agent.ts';
export type * from './events-misc.ts';
export type * from './events-session.ts';
export type * from './events-tool-post.ts';
export type * from './events-tool.ts';
export type * from './tool-inputs-union.ts';
export type * from './tool-inputs.ts';
export type * from './transcript.ts';

//endregion

//region Discriminated unions

/**
 Discriminated union of all hook event inputs.
 Narrows via `hook_event_name`.
 
 @example
 ```ts
 function handle(event: HookInput) {
   if (event.hook_event_name === 'PreToolUse') {
     event.tool_name; // narrowed to PreToolUseInput
   }
 }
 ```
 */
type HookInput =
  | SessionStartInput
    | InstructionsLoadedInput
    | UserPromptSubmitInput
    | PreToolUseInput
    | PermissionRequestInput
    | PostToolUseInput
    | PostToolUseFailureInput
    | NotificationInput
    | SubagentStartInput
    | SubagentStopInput
    | StopInput
    | TeammateIdleInput
    | TaskCompletedInput
    | ConfigChangeInput
    | WorktreeCreateInput
    | WorktreeRemoveInput
    | PreCompactInput
    | SessionEndInput;

export type { HookInput, };

//endregion
