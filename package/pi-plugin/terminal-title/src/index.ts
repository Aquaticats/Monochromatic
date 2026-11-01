/**
 Terminal Title pi extension entry point.
 
 Sets the terminal window/tab title to reflect the current agent activity:
 tool executions, session lifecycle, and user prompts.
 
 Uses pi's `ctx.ui.setTitle()` API instead of raw OSC 0 escape sequences.
 The title is composed by the title-builder module using a tool-name registry
 with tense-aware formatting.
 
 @example
 ```typescript
 // Auto-discovered from ~/.pi/agent/extensions/terminal-title/index.ts
 // or loaded via pi install / pi -e
 pi -e ./packages/pi-plugin/terminal-title/src/index.ts
 ```
 
 @module
 */

import type {
  AgentEndEvent,
  BeforeAgentStartEvent,
  ExtensionAPI,
  SessionShutdownEvent,
  SessionStartEvent,
} from '@earendil-works/pi-coding-agent';
import { safeTerminalTitlePayload, } from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { titleForEvent, } from './title-builder.ts';

/**
 Immutable empty tool-argument record used when pi omits event arguments.
 */
const EMPTY_TOOL_ARGS: Readonly<Record<string, unknown>> = Object.freeze({},);

/**
 Minimal context shape needed by all event handlers (just `ui.setTitle()`).
 */
type TitleContext = {
  readonly ui: {
    readonly setTitle: (title: string,) => void;
  };
};

/**
 Minimal tool execution start event shape verified against pi 0.80.3 internal declarations.
 */
type ToolExecutionStartEvent = {
  /**
   Event discriminant.
   */
  readonly type: 'tool_execution_start';

  /**
   Stable pi tool call identifier.
   */
  readonly toolCallId: string;

  /**
   pi tool name.
   */
  readonly toolName: string;

  /**
   Raw tool arguments from pi.
   */
  readonly args: unknown;
};

/**
 Minimal tool execution end event shape verified against pi 0.80.3 internal declarations.
 */
type ToolExecutionEndEvent = {
  /**
   Event discriminant.
   */
  readonly type: 'tool_execution_end';

  /**
   Stable pi tool call identifier.
   */
  readonly toolCallId: string;

  /**
   pi tool name.
   */
  readonly toolName: string;
};

//region Tool argument helpers

/**
 Checks whether unknown pi event args are a string-keyed object.
 
 @param value - because pi event args arrive from extension events without specific types
 
 @returns whether value can be read as tool arguments
 
 @example
 ```ts
 isToolArgs({ command: 'ls -l' });
 ```
 */
function isToolArgs(value: unknown,): value is Readonly<Record<string, unknown>> {
  if (value === null)
    return false;
  if ((typeof value) !== 'object')
    return false;
  return !Array.isArray(value,);
}

/**
 Converts unknown pi event args into a read-only argument record, narrowing
 with {@link isToolArgs}.
 
 @param args - because pi may omit args on completion events
 
 @returns tool arguments, or an empty record when absent or non-object
 
 @example
 ```ts
 toolArgsFromUnknown({ command: 'ls -l' });
 ```
 */
function toolArgsFromUnknown(args: unknown,): Readonly<Record<string, unknown>> {
  if (isToolArgs(args,))
    return args;
  return EMPTY_TOOL_ARGS;
}

//endregion Tool argument helpers

//region Terminal title output

/**
 Sends safe title payload text through pi's terminal title API.
 Control sanitizing and byte capping happen at this output boundary so OSC
 controls cannot leak and terminals such as Ghostty do not leave stale titles.
 
 @param ctx - because pi owns the UI title side effect
 
 @param title - because callers build host-specific display text first
 
 @example
 ```ts
 setTerminalTitle({ ctx, title: 'π Reading index.ts' });
 ```
 */
function setTerminalTitle(
  {
    ctx,
    title,
  }: Readonly<{
    ctx: TitleContext;
    title: string;
  }>,
): void {
  ctx.ui
    .setTitle(
    safeTerminalTitlePayload({ value: title, },),
    );
}

//endregion Terminal title output

//region Extension entry point

/**
 Terminal Title pi extension.
 
 Subscribes to agent lifecycle events and updates the terminal window/tab
 title to reflect the current activity, delegating title text to
 {@link titleForEvent}.
 
 Handler types for `tool_execution_start` and `tool_execution_end` are
 inferred by the `pi.on()` overload signatures; those event types are not
 re-exported from the package's top-level index but are available via the
 `on()` method's parameter types.
 
 @param pi - the pi extension API
 
 @mutates pi - `pi.on` stores tool, session, and agent lifecycle registrations in the Pi host
 
 @example
 ```typescript
 // In ~/.pi/agent/settings.json:
 { "packages": ["./packages/pi-plugin/terminal-title"] }
 ```
 */
export default function terminalTitle(pi: ForeignBorrowed<ExtensionAPI>,): void {
  /**
   Tool arguments captured at start time. Pi completion events expose result
   metadata but not original args, so terminal titles need this per-call cache
   to show details such as `ls -l` after completion.
   */
  const toolArgsByCallId = new Map<string, Readonly<Record<string, unknown>>>();

  pi.on(
    'tool_execution_start',
    function handleToolExecutionStart(
      event: Readonly<ToolExecutionStartEvent>,
      ctx: TitleContext,
    ) {
      /**
       Event arguments normalized to a string-keyed record so the title builder can sample fields by name.
       */
      const args = toolArgsFromUnknown(event.args,);
      toolArgsByCallId.set(
        event.toolCallId,
        args,
      );
      setTerminalTitle({
        ctx,
        title: titleForEvent({
          eventType: 'tool_execution_start',
          data: {
            toolName: event.toolName,
            args,
          },
        },),
      },);
    },
  );
  pi.on(
    'tool_execution_end',
    function handleToolExecutionEnd(
      event: Readonly<ToolExecutionEndEvent>,
      ctx: TitleContext,
    ) {
      /**
       Original args recovered from the start event because pi's end event omits them.
       */
      const cachedArgs = toolArgsByCallId.get(event.toolCallId,);
      /**
       Completion title args from start-event cache, or empty fallback when no matching start exists.
       */
      const args = cachedArgs
        ?? EMPTY_TOOL_ARGS;
      toolArgsByCallId.delete(event.toolCallId,);
      setTerminalTitle({
        ctx,
        title: titleForEvent({
          eventType: 'tool_execution_end',
          data: {
            toolName: event.toolName,
            args,
          },
        },),
      },);
    },
  );
  pi.on(
    'session_start',
    function handleSessionStart(
      event: Readonly<Pick<SessionStartEvent, 'reason'>>,
      ctx: TitleContext,
    ) {
      toolArgsByCallId.clear();
      setTerminalTitle({
        ctx,
        title: titleForEvent({
          eventType: 'session_start',
          data: {
            reason: event.reason,
          },
        },),
      },);
    },
  );
  pi.on(
    'session_shutdown',
    function handleSessionShutdown(
      _event: Readonly<Pick<SessionShutdownEvent, 'type'>>,
      ctx: TitleContext,
    ) {
      toolArgsByCallId.clear();
      setTerminalTitle({
        ctx,
        title: titleForEvent({
          eventType: 'session_shutdown',
          data: {},
        },),
      },);
    },
  );
  pi.on(
    'agent_end',
    function handleAgentEnd(
      _event: Readonly<Pick<AgentEndEvent, 'type'>>,
      ctx: TitleContext,
    ) {
      toolArgsByCallId.clear();
      setTerminalTitle({
        ctx,
        title: titleForEvent({
          eventType: 'agent_end',
          data: {},
        },),
      },);
    },
  );
  pi.on(
    'before_agent_start',
    function handleBeforeAgentStart(
      event: Readonly<BeforeAgentStartEvent>,
      ctx: TitleContext,
    ) {
      // Only user-triggered runs carry a prompt; custom-triggered runs get no prompt title.
      if (event.messageType !== 'user')
        return;
      setTerminalTitle({
        ctx,
        title: titleForEvent({
          eventType: 'before_agent_start',
          data: {
            prompt: event.prompt,
          },
        },),
      },);
    },
  );
}

//endregion
