/**
 Miscellaneous hook event types: UserPromptSubmit, Notification, PreCompact, WorktreeCreate, WorktreeRemove.
 
 @module
 */

import type {
  HookInputBase,
  HookOutputBase,
} from './common.ts';

//region UserPromptSubmit

/**
 Input for `UserPromptSubmit` hooks.
 Fires when the user submits a prompt, before Claude processes it.
 */
type UserPromptSubmitInput = HookInputBase & {
  hook_event_name: 'UserPromptSubmit';

  /**
   Text the user submitted.
   */
  prompt: string;
};

/**
 Output for `UserPromptSubmit` hooks.
 Can block the prompt or add context.
 */
type UserPromptSubmitOutput = HookOutputBase & {
  /**
   `"block"` prevents the prompt from being processed and erases it from context.
   */
  decision?: 'block';

  /**
   Shown to the user when `decision` is `"block"`. Not added to context.
   */
  reason?: string;

  hookSpecificOutput?: {
    hookEventName: 'UserPromptSubmit';

    /**
     Context added to Claude's conversation.
     */
    additionalContext?: string;
  };
};

//endregion

//region Notification

/**
 Notification type that triggered the hook.
 */
type NotificationType =
  | 'permission_prompt'
    | 'idle_prompt'
    | 'auth_success'
    | 'elicitation_dialog';

/**
 Input for `Notification` hooks.
 Fires when Claude Code sends a notification.
 */
type NotificationInput = HookInputBase & {
  hook_event_name: 'Notification';

  /**
   Notification text.
   */
  message: string;

  /**
   Notification heading.
   */
  title?: string;

  /**
   Which notification type fired.
   */
  notification_type: NotificationType;
};

/**
 Output for `Notification` hooks.
 Cannot block or modify notifications.
 
 **Note**: despite being documented, `additionalContext` for Notification hooks
 is not implemented in Claude Code v2.1.76: the `hookSpecificOutput` switch
 statement has no `case "Notification"`, so any provided context is silently dropped.
 */
type NotificationOutput = HookOutputBase;

//endregion

//region PreCompact

/**
 What triggered context compaction.
 */
type PreCompactTrigger = 'manual' | 'auto';

/**
 Input for `PreCompact` hooks.
 Fires before context compaction.
 */
type PreCompactInput = HookInputBase & {
  hook_event_name: 'PreCompact';

  /**
   Whether compaction was triggered manually or automatically.
   */
  trigger: PreCompactTrigger;

  /**
   User instructions passed to `/compact`. Empty for `auto`.
   */
  custom_instructions: string;
};

/**
 Output for `PreCompact` hooks.
 No decision control.
 */
type PreCompactOutput = HookOutputBase;

//endregion

//region WorktreeCreate

/**
 Input for `WorktreeCreate` hooks.
 Fires when a worktree is being created.
 Replaces default git worktree behavior when configured.
 */
type WorktreeCreateInput = HookInputBase & {
  hook_event_name: 'WorktreeCreate';

  /**
   Slug identifier for the new worktree (e.g. `"bold-oak-a3f2"`).
   */
  name: string;
};

/* WorktreeCreate output is the absolute path printed to stdout. No JSON decision model. */

//endregion

//region WorktreeRemove

/**
 Input for `WorktreeRemove` hooks.
 Fires when a worktree is being removed.
 */
type WorktreeRemoveInput = HookInputBase & {
  hook_event_name: 'WorktreeRemove';

  /**
   Absolute path to the worktree being removed.
   */
  worktree_path: string;
};

/**
 Output for `WorktreeRemove` hooks.
 No decision control; cannot block removal.
 */
type WorktreeRemoveOutput = HookOutputBase;

//endregion

export type {
  NotificationInput,
  NotificationOutput,
  NotificationType,
  PreCompactInput,
  PreCompactOutput,
  PreCompactTrigger,
  UserPromptSubmitInput,
  UserPromptSubmitOutput,
  WorktreeCreateInput,
  WorktreeRemoveInput,
  WorktreeRemoveOutput,
};
