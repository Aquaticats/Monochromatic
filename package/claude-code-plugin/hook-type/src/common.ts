/**
 Common types shared across all Claude Code hook events.
 
 @module
 */

//region Permission mode

/**
 Permission mode the session is running in.
 
 @see [Claude Code permissions](https://docs.anthropic.com/en/docs/claude-code/permissions)
 */
type PermissionMode =
  | 'default'
    | 'plan'
    | 'acceptEdits'
    | 'dontAsk'
    | 'bypassPermissions';

//endregion

//region Base input

/**
 Fields present in every hook event's JSON input, regardless of the event type.
 
 Command hooks receive this as JSON on stdin.
 HTTP hooks receive this as the POST request body.
 
 @example
 ```ts
 const event = JSON.parse(await readStdin()) as HookInputBase;
 console.log(event.session_id, event.cwd);
 ```
 */
type HookInputBase = {
  /**
   Current session identifier.
   */
  session_id: string;

  /**
   Path to the conversation transcript JSONL file.
   */
  transcript_path: string;

  /**
   Working directory when the hook was invoked.
   */
  cwd: string;

  /**
   Permission mode active in the session.
   */
  permission_mode: PermissionMode;

  /**
   Name of the hook event that fired.
   */
  hook_event_name: HookEventName;

  /**
   Unique identifier for the subagent, when the hook fires inside a subagent call.
   Absent on the main thread.
   */
  agent_id?: string;

  /**
   Agent name (e.g. `"Explore"`, `"security-reviewer"`).
   Present when the session uses `--agent` or the hook fires inside a subagent.
   For subagents, the subagent's type takes precedence over the session's `--agent` value.
   */
  agent_type?: string;
};

//endregion

//region Base output

/**
 Universal JSON output fields available to all hook events.
 Return this object (or a superset) as JSON on stdout from a command hook,
 or as the HTTP response body from an HTTP hook.
 
 @example
 ```ts
 const output: HookOutputBase = { continue: false, stopReason: 'Build failed' };
 process.stdout.write(JSON.stringify(output));
 ```
 */
type HookOutputBase = {
  /**
   When `false`, Claude stops processing entirely after the hook runs.
   Takes precedence over any event-specific decision fields.
   
   @defaultValue true
   */
  continue?: boolean;

  /**
   Message shown to the user when `continue` is `false`. Not shown to Claude.
   */
  stopReason?: string;

  /**
   When `true`, hides stdout from verbose mode output.
   
   @defaultValue false
   */
  suppressOutput?: boolean;

  /**
   Warning message shown to the user.
   */
  systemMessage?: string;
};

//endregion

//region Hook event name

/**
 All supported hook event names in Claude Code's lifecycle.
 */
type HookEventName =
  | 'SessionStart'
    | 'InstructionsLoaded'
    | 'UserPromptSubmit'
    | 'PreToolUse'
    | 'PermissionRequest'
    | 'PostToolUse'
    | 'PostToolUseFailure'
    | 'Notification'
    | 'SubagentStart'
    | 'SubagentStop'
    | 'Stop'
    | 'TeammateIdle'
    | 'TaskCompleted'
    | 'ConfigChange'
    | 'WorktreeCreate'
    | 'WorktreeRemove'
    | 'PreCompact'
    | 'SessionEnd';

//endregion

export type {
  HookEventName,
  HookInputBase,
  HookOutputBase,
  PermissionMode,
};
