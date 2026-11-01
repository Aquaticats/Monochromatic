/**
 Terminal title builder for pi events.
 
 Produces human-readable terminal tab titles from pi extension events
 using shared terminal-title engine entries.
 
 @module
 */

import {
  buildTerminalTitle,
  buildToolTitle,
  type ToolTitleInput,
  type ToolTitleTense,
} from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';
import { TOOL_TITLES, } from './tool-titles.ts';

/**
 Prefix prepended to every terminal title for pi visual identification.
 */
const TITLE_PREFIX = 'π';

/**
 Union of pi extension events that this extension handles.
 */
type HandledEventType =
  | 'tool_execution_start'
    | 'tool_execution_end'
    | 'session_start'
    | 'session_shutdown'
    | 'agent_end'
    | 'before_agent_start';

/**
 Data bag for event-specific fields passed to title builders.
 */
type EventData = {
  /**
   Tool name for tool execution events.
   */
  readonly toolName?: string;

  /**
   Tool input arguments for tool execution events.
   */
  readonly args?: ToolTitleInput;

  /**
   Session start reason from pi.
   */
  readonly reason?: string;

  /**
   User prompt text before agent start.
   */
  readonly prompt?: string;
};

//region Tool titles

/**
 Builds a terminal title body for a pi tool execution event.
 
 @param toolName - because pi built-ins and custom tools use string names
 
 @param args - because title entries inspect tool input fields
 
 @param tense - because start and end events use different lifecycle verbs
 
 @returns formatted title body without prefix
 
 selected registry formatter behavior against tool arguments
 
 @example
 ```ts
 titleForTool({ toolName: 'bash', args: { command: 'npm test' }, tense: 'pre' });
 // 'Running npm test'
 ```
 */
function titleForTool(
  {
    toolName,
    args,
    tense,
  }: Readonly<{
    toolName: string;
    args: ToolTitleInput;
    tense: ToolTitleTense;
  }>,
): string {
  return buildToolTitle({
    registry: TOOL_TITLES,
    toolName,
    input: args,
    tense,
  },);
}

//endregion Tool titles

//region Event title bodies

/**
 Builds title body for tool start event data.
 
 @param data - because pi event adapter normalizes event payloads into this shape
 
 @returns title body without prefix
 */
function toolStartBody(data: EventData,): string {
  return titleForTool({
    toolName: data.toolName
      ?? 'unknown',
    args: data.args
      ?? {},
    tense: 'pre',
  },);
}

/**
 Builds title body for tool end event data.
 
 @param data - because pi completion events use post tense
 
 @returns title body without prefix
 */
function toolEndBody(data: EventData,): string {
  return titleForTool({
    toolName: data.toolName
      ?? 'unknown',
    args: data.args
      ?? {},
    tense: 'post',
  },);
}

/**
 Builds title body for session start event data.
 
 @param data - because pi supplies session start reason when available
 
 @returns title body without prefix
 */
function sessionStartBody(data: EventData,): string {
  /**
   Session reason text shown after lifecycle verb.
   */
  const reason = data.reason
    ?? 'started';
  return `Started session: ${reason}`;
}

/**
 Builds title body for before-agent-start event data.
 
 @param data - because prompt text is the activity context
 
 @returns title body without prefix
 */
function promptBody(data: EventData,): string {
  /**
   Prompt text shown after lifecycle verb.
   */
  const prompt = data.prompt
    ?? '';
  if (prompt.length === 0)
    return 'Received prompt';
  return `Received prompt: ${prompt}`;
}

/**
 Maps event types to their title body builders.
 */
const EVENT_BODY_BUILDERS: Record<HandledEventType, (data: EventData,) => string> = {
  tool_execution_start: toolStartBody,
  tool_execution_end: toolEndBody,
  session_start: sessionStartBody,
  session_shutdown() {
    return 'Ended session';
  },
  agent_end() {
    return 'Stopped agent';
  },
  before_agent_start: promptBody,
};

//endregion Event title bodies

//region Event title API

/**
 Builds a terminal title from a pi extension event.
 
 @param eventType - discriminant for handled pi event
 
 @param data - event-specific data such as tool name, args, reason, or prompt
 
 @returns final title string with prefix before payload-boundary sanitizing
 
 @example
 ```ts
 titleForEvent({ eventType: 'tool_execution_start', data: { toolName: 'bash', args: { command: 'npm test' } } });
 // 'π Running npm test'
 ```
 */
function titleForEvent(
  {
    eventType,
    data,
  }: Readonly<{
    eventType: HandledEventType;
    data: EventData;
  }>,
): string {
  /**
   Event-specific body builder selected by event type.
   */
  const builder = EVENT_BODY_BUILDERS[eventType];
  return buildTerminalTitle({
    prefix: TITLE_PREFIX,
    body: builder(data,),
  },);
}

//endregion Event title API

export {
  TITLE_PREFIX,
  titleForEvent,
  titleForTool,
};

export type {
  EventData,
  HandledEventType,
};
