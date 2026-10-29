/**
 Transcript record types.
 
 Hooks receive `transcript_path` and routinely read the session's JSONL to
 recover state the event payload does not carry. These types describe the
 fields such readers narrow on, so every reader shares one shape instead of
 redeclaring a partial view per call site.
 
 Partial by design: a transcript record carries far more than this, and
 widening the type to cover everything would weaken the narrowing readers rely
 on. Add fields here when a reader needs them.
 
 @module
 */

//region Transcript records

/**
 One content block inside a message.
 
 Assistant messages carry an array of these; user messages carry either an
 array or a plain string, which is why {@link TranscriptRecord} admits both.
 */
type TranscriptContentBlock = {
  /**
   Block kind, such as `text`, `tool_use`, or `tool_result`.
   */
  readonly type?: string;

  /**
   Tool name, present on `tool_use` blocks.
   */
  readonly name?: string;

  /**
   Rendered text, present on `text` blocks.
   */
  readonly text?: string;

  /**
   Reasoning text, present on `thinking` blocks.

   Stays on this machine: readers never send it to an external provider,
   and records derived from it carry hashes and indexes instead of text.
   */
  readonly thinking?: string;

  /**
   Tool arguments, present on `tool_use` blocks.
   
   Values stay `unknown` so each reader narrows the arguments it cares about
   rather than trusting a shape this type cannot verify.
   */
  readonly input?: Readonly<Record<string, unknown>>;
};

/**
 One record from a session transcript.
 
 Every field is optional because the JSONL mixes record kinds: user turns,
 assistant turns, tool results, attachments, and session metadata all share
 the file.
 */
type TranscriptRecord = {
  /**
   Record kind, such as `user`, `assistant`, or `attachment`.
   */
  readonly type?: string;

  /**
   Queue action on a `queue-operation` record: `enqueue`, `dequeue`, or `remove`.
   */
  readonly operation?: string;

  /**
   Queued text on a `queue-operation` record, such as a `<task-notification>` block.

   Absent on a `dequeue`, which takes the head of the queue without naming it.
   */
  readonly content?: string;

  /**
   `true` for records belonging to a subagent branch rather than the main thread.
   */
  readonly isSidechain?: boolean;

  /**
   Who produced the record; `kind` is `human` for a turn the user typed.
   */
  readonly origin?: {
    /**
     Producer of this record.
     */
    readonly kind?: string;
  };

  /**
   Result payload attached to a tool-result record.
   
   Present only on tool results, which makes its absence a reliable test for
   a record being a genuine turn rather than a tool result.
   */
  readonly toolUseResult?: {
    /**
     Task envelope returned by task-management tools.
     */
    readonly task?: {
      /**
       Identifier assigned to the task.
       */
      readonly id?: unknown;
    };
  };

  /**
   Message body, absent on attachments and session metadata.
   */
  readonly message?: {
    /**
     Message content, a string for hook feedback and an array otherwise.
     */
    readonly content?: string | readonly TranscriptContentBlock[];
  };
};

//endregion

export type {
  TranscriptContentBlock,
  TranscriptRecord,
};
