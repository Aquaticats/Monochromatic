/**
 Session lifecycle hook event types: SessionStart, SessionEnd, InstructionsLoaded, ConfigChange.
 
 @module
 */

import type {
  HookInputBase,
  HookOutputBase,
} from './common.ts';

//region SessionStart

/**
 How the session was initiated.
 */
type SessionStartSource = 'startup' | 'resume' | 'clear' | 'compact';

/**
 Input for `SessionStart` hooks.
 Fires when a session begins or resumes.
 */
type SessionStartInput = HookInputBase & {
  hook_event_name: 'SessionStart';

  /**
   How the session was initiated.
   */
  source: SessionStartSource;

  /**
   Model identifier for the session.
   */
  model: string;

  /**
   Agent name when started with `claude --agent <name>`.
   */
  agent_type?: string;
};

/**
 Output for `SessionStart` hooks.
 Text on stdout or `additionalContext` is added to Claude's context.
 */
type SessionStartOutput = HookOutputBase & {
  hookSpecificOutput?: {
    hookEventName: 'SessionStart';

    /**
     Context added to Claude's conversation.
     */
    additionalContext?: string;
  };
};

//endregion

//region SessionEnd

/**
 Why the session ended.
 */
type SessionEndReason =
  | 'clear'
    | 'logout'
    | 'prompt_input_exit'
    | 'bypass_permissions_disabled'
    | 'other';

/**
 Input for `SessionEnd` hooks.
 Fires when a session terminates. Cannot block termination.
 */
type SessionEndInput = HookInputBase & {
  hook_event_name: 'SessionEnd';

  /**
   Why the session ended.
   */
  reason: SessionEndReason;
};

/**
 Output for `SessionEnd` hooks.
 No decision control; used for cleanup and logging.
 */
type SessionEndOutput = HookOutputBase;

//endregion

//region InstructionsLoaded

/**
 Why an instruction file was loaded.
 */
type InstructionsLoadReason =
  | 'session_start'
    | 'nested_traversal'
    | 'path_glob_match'
    | 'include';

/**
 Scope of a loaded instruction file.
 */
type InstructionsMemoryType = 'User' | 'Project' | 'Local' | 'Managed';

/**
 Input for `InstructionsLoaded` hooks.
 Fires when a CLAUDE.md or `.claude/rules/*.md` file is loaded.
 Cannot block or modify loading; used for observability.
 */
type InstructionsLoadedInput = HookInputBase & {
  hook_event_name: 'InstructionsLoaded';

  /**
   Absolute path to the loaded instruction file.
   */
  file_path: string;

  /**
   Scope of the file.
   */
  memory_type: InstructionsMemoryType;

  /**
   Why the file was loaded.
   */
  load_reason: InstructionsLoadReason;

  /**
   Path glob patterns from `paths:` frontmatter, for `path_glob_match` loads.
   */
  globs?: string[];

  /**
   Path to the file whose access triggered this load, for lazy loads.
   */
  trigger_file_path?: string;

  /**
   Path to the parent instruction file, for `include` loads.
   */
  parent_file_path?: string;
};

/**
 Output for `InstructionsLoaded` hooks.
 No decision control.
 */
type InstructionsLoadedOutput = HookOutputBase;

//endregion

//region ConfigChange

/**
 Which configuration type changed.
 */
type ConfigChangeSource =
  | 'user_settings'
    | 'project_settings'
    | 'local_settings'
    | 'policy_settings'
    | 'skills';

/**
 Input for `ConfigChange` hooks.
 Fires when a configuration file changes during a session.
 */
type ConfigChangeInput = HookInputBase & {
  hook_event_name: 'ConfigChange';

  /**
   Which configuration type changed.
   */
  source: ConfigChangeSource;

  /**
   Path to the specific file that was modified.
   */
  file_path?: string;
};

/**
 Output for `ConfigChange` hooks.
 Can block changes (except `policy_settings` which cannot be blocked).
 */
type ConfigChangeOutput = HookOutputBase & {
  /**
   `"block"` prevents the config change from being applied.
   */
  decision?: 'block';

  /**
   Explanation shown to the user when blocked.
   */
  reason?: string;
};

//endregion

export type {
  ConfigChangeInput,
  ConfigChangeOutput,
  ConfigChangeSource,
  InstructionsLoadedInput,
  InstructionsLoadedOutput,
  InstructionsLoadReason,
  InstructionsMemoryType,
  SessionEndInput,
  SessionEndOutput,
  SessionEndReason,
  SessionStartInput,
  SessionStartOutput,
  SessionStartSource,
};
