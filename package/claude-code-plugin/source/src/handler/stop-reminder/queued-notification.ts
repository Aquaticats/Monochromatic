/**
 Queued task notification release for forced continuation.

 `background_tasks` on the `Stop` event lists only tasks that are still running. A task
 that finished a moment before the stop is gone from that list, yet Claude Code has
 already queued its completion notification and delivers it as a new turn right after the
 stop. An agent that wrote "running in the background" a second before the job ended is
 then correct about the wait, and the hook saw an empty list. Observed on 2026-10-21: the
 type-check finished at 00:51:36.452, the stop came at 00:51:37, and the notification was
 delivered at 00:51:39.

 The transcript records the queue as `queue-operation` records, so the pending notification
 is visible there without asking anyone. A notification counts only when no tool result
 follows it. A real pending one arrives after the last tool result of the turn, so only the
 final reply can come after it. Across 46 transcripts, 31 notifications were never removed
 from the queue, 26 of them with at least 100 transcript lines after; counting those would
 release every later stop of that session.

 @module
 */

import type { TranscriptRecord, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';

import {
  parseRecord,
  UNPARSABLE,
} from './continuation-depth.ts';

/**
 Record kind Claude Code uses for changes to its pending-input queue.
 */
const QUEUE_OPERATION_TYPE = 'queue-operation' as const;

/**
 Opening tag of the text Claude Code queues when a background task finishes.
 */
const NOTIFICATION_MARKER = '<task-notification>' as const;

/**
 Origin kind Claude Code puts on the user record that delivers a queued notification.
 */
const NOTIFICATION_ORIGIN = 'task-notification' as const;

/**
 What `findIndex` returns when no entry matches.
 */
const NOT_FOUND = -1 as const;

/**
 One entry in the replayed queue.
 */
type QueueEntry = {
  /**
   Queued text; a `remove` names its entry by this text.
   */
  readonly content: string;

  /**
   Whether this entry is a task notification, delivered as a new turn after the stop.
   */
  readonly pendingNotification: boolean;
};

/**
 Reports whether a record is a tool result returned to the model.

 @param record - parsed transcript record

 @returns whether the record carries a `tool_result` block

 @example
 ```ts
 isToolResultRecord({ type: 'user', message: { content: [{ type: 'tool_result' }] } }); // true
 ```
 */
function isToolResultRecord(record: TranscriptRecord,): boolean {
  /**
   Message content, an array of blocks for tool results.
   */
  const content = record
    .message
    ?.content;

  if ((record.type !== 'user')
    || (content === undefined)
    || ((typeof content) === 'string')) {
    return false;
  }
  return content.some(function isToolResult(block,) {
    return block.type === 'tool_result';
  },);
}

/**
 Removes the first entry carrying the given text.

 @param queue - queue before the removal

 @param content - text of the entry to remove

 @returns the queue without that entry, or unchanged when it left the queue earlier

 @example
 ```ts
 removeByContent({ queue: [{ content: 'a', pendingNotification: false }], content: 'a' }); // []
 ```
 */
function removeByContent(
  {
    queue,
    content,
  }: {
    readonly queue: readonly QueueEntry[];
    readonly content: string;
  },
): readonly QueueEntry[] {
  /**
   Position of the first entry with this text.
   */
  const index = queue.findIndex(function sameContent(entry,) {
    return entry.content === content;
  },);

  if (index === NOT_FOUND) {
    return queue;
  }
  return queue.toSpliced(
    index,
    1,
  );
}

/**
 Applies one `queue-operation` record to the replayed queue.

 `dequeue` names nothing and takes the head. `remove` names its entry by content.
 `popAll` empties the queue. An `enqueue` without text is not an entry anyone can name, so it is skipped.

 @param queue - queue before this record

 @param record - parsed `queue-operation` record

 @returns the queue after this record

 @example
 ```ts
 applyQueueOperation({ queue: [], record: { operation: 'enqueue', content: '<task-notification>x' } }).length; // 1
 ```
 */
function applyQueueOperation(
  {
    queue,
    record,
  }: {
    readonly queue: readonly QueueEntry[];
    readonly record: TranscriptRecord;
  },
): readonly QueueEntry[] {
  if (record.operation === 'dequeue') {
    return queue.slice(1,);
  }
  if (record.operation === 'popAll') {
    return [];
  }

  /**
   Queued text, absent on operations that name nothing.
   */
  const { content, } = record;
  if (content === undefined) {
    return queue;
  }
  if (record.operation === 'enqueue') {
    return [
      ...queue,
      {
        content,
        pendingNotification: content.includes(NOTIFICATION_MARKER,),
      },
    ];
  }
  if (record.operation === 'remove') {
    return removeByContent({
      queue,
      content,
    },);
  }
  return queue;
}

/**
 Applies one transcript line to the replayed queue.

 @param queue - queue before this line

 @param line - raw transcript JSONL line

 @returns the queue after this line; unchanged for a line that is neither a queue change nor a tool result

 @example
 ```ts
 replayLine({ queue: [], line: '{"type":"assistant"}' }); // []
 ```
 */
function replayLine(
  {
    queue,
    line,
  }: {
    readonly queue: readonly QueueEntry[];
    readonly line: string;
  },
): readonly QueueEntry[] {
  // Cheap pre-filter: only queue records, delivered notifications, and tool results change the answer.
  if ((!line.includes(QUEUE_OPERATION_TYPE,))
    && (!line.includes(NOTIFICATION_ORIGIN,))
    && (!line.includes('"tool_result"',))) {
    return queue;
  }

  /**
   Parsed record for a line that might be a queue change, a delivery, or a tool result.
   */
  const record = parseRecord(line,);

  if (record === UNPARSABLE) {
    return queue;
  }
  if (record.type === QUEUE_OPERATION_TYPE) {
    return applyQueueOperation({
      queue,
      record,
    },);
  }

  /**
   Text of a delivered notification; the delivery names its entry even when a `dequeue` took another.
   */
  const delivered = record
    .message
    ?.content;
  /**
   Origin kind of the record, present only on records Claude Code attributes to a source.
   */
  const originKind = record
    .origin
    ?.kind;

  if ((originKind === NOTIFICATION_ORIGIN)
    && ((typeof delivered) === 'string')) {
    return removeByContent({
      queue,
      content: delivered,
    },);
  }
  // Everything queued is absorbed into a tool result, so what came before it cannot be pending.
  return isToolResultRecord(record,) ? [] : queue;
}

/**
 Reports whether a task notification is queued and will be delivered after this stop.

 Replays the queue operations oldest first. A tool result empties the replayed queue,
 because Claude Code absorbs queued input into the next tool result (6,609 `remove`
 records with reason `absorbed_mid_turn` in the transcripts surveyed), so an entry left
 over from before it is a leftover. Without the reset, a stale entry at the head of the
 queue made a later `dequeue` take the wrong entry and left a delivered notification
 looking pending.

 Reads only the tail it is given; a removal whose enqueue lies before the window is ignored.

 @param transcriptLines - transcript JSONL lines, oldest first

 @returns whether a task notification is waiting behind this stop

 @example
 ```ts
 hasQueuedTaskNotification([
   '{"type":"queue-operation","operation":"enqueue","content":"<task-notification>x</task-notification>"}',
 ]); // true
 ```
 */
function hasQueuedTaskNotification(transcriptLines: readonly string[],): boolean {
  /**
   Replayed queue after the last line, newest entry last.
   */
  const queue = transcriptLines.reduce<readonly QueueEntry[]>(
    function replay(
      replayed,
      line,
    ) {
      return replayLine({
        queue: replayed,
        line,
      },);
    },
    [],
  );

  return queue.some(function isPending(entry,) {
    return entry.pendingNotification;
  },);
}

export {
  hasQueuedTaskNotification,
  NOTIFICATION_MARKER,
};
