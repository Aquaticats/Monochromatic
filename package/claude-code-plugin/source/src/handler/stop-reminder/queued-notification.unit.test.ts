import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { hasQueuedTaskNotification, } from './queued-notification.ts';

/**
 Text Claude Code queues when the task `id` finishes.
 */
function notification(id: string,): string {
  return `<task-notification>\n<task-id>${id}</task-id>\n<status>completed</status>\n</task-notification>`;
}

/**
 Transcript line for one queue change.
 */
function queueLine({ operation, content, }: { readonly operation: string; readonly content?: string; },): string {
  return JSON.stringify({
    type: 'queue-operation',
    operation,
    ...((content === undefined) ? {} : { content, }),
  },);
}

/**
 Transcript line for a tool result returned to the model.
 */
const TOOL_RESULT = JSON.stringify({
  type: 'user',
  message: { content: [{ type: 'tool_result', tool_use_id: 'x', },], },
},);

/**
 Transcript line for the model's final prose.
 */
const TEXT = JSON.stringify({ type: 'assistant', message: { content: [{ type: 'text', text: 'done', },], }, },);

await describe({
  name: hasQueuedTaskNotification.name,
  children: [
    describe({
      name: 'reports a pending notification',
      children: [
        it({
          name: 'when it was queued after the last tool result and nothing removed it',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              TOOL_RESULT,
              queueLine({ operation: 'enqueue', content: notification('a',), },),
              TEXT,
            ],),).toBe(true,);
          },
        },),
        it({
          name: 'when a different notification was removed and this one was not',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'enqueue', content: notification('a',), },),
              queueLine({ operation: 'enqueue', content: notification('b',), },),
              queueLine({ operation: 'remove', content: notification('a',), },),
            ],),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'reports none',
      children: [
        it({
          name: 'for an empty tail',
          fn: async () => {
            expect(hasQueuedTaskNotification([],),).toBe(false,);
          },
        },),
        it({
          name: 'once the notification is removed by its text',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'enqueue', content: notification('a',), },),
              queueLine({ operation: 'remove', content: notification('a',), },),
            ],),).toBe(false,);
          },
        },),
        it({
          name: 'once a dequeue takes the head of the queue',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'enqueue', content: notification('a',), },),
              queueLine({ operation: 'dequeue', },),
            ],),).toBe(false,);
          },
        },),
        it({
          name: 'once popAll empties the queue',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'enqueue', content: notification('a',), },),
              queueLine({ operation: 'enqueue', content: notification('b',), },),
              queueLine({ operation: 'popAll', content: 'a b', },),
            ],),).toBe(false,);
          },
        },),
        it({
          name: 'for a leftover that a later tool result outdates',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'enqueue', content: notification('a',), },),
              TOOL_RESULT,
              TEXT,
            ],),).toBe(false,);
          },
        },),
        it({
          name: 'for a queued prompt that is not a task notification',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'enqueue', content: 'please also run the lint', },),
            ],),).toBe(false,);
          },
        },),
        it({
          name: 'for an enqueue that carries no text',
          fn: async () => {
            expect(hasQueuedTaskNotification([queueLine({ operation: 'enqueue', },),],),).toBe(false,);
          },
        },),
        it({
          name: 'for a removal whose entry left the window earlier',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'remove', content: notification('a',), },),
            ],),).toBe(false,);
          },
        },),
        it({
          name: 'for an operation it does not know',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'enqueue', content: notification('a',), },),
              queueLine({ operation: 'rearrange', content: notification('a',), },),
            ],),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'ordering and malformed input',
      children: [
        it({
          name: 'keeps a notification queued after a tool result pending, while an older one expires',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'enqueue', content: notification('old',), },),
              TOOL_RESULT,
              queueLine({ operation: 'enqueue', content: notification('new',), },),
            ],),).toBe(true,);
          },
        },),
        it({
          name: 'removes the entry a delivery names, even when a dequeue took another one',
          fn: async () => {
            /**
             Delivery record Claude Code writes when the notification becomes a turn.
             */
            const delivery = JSON.stringify({
              type: 'user',
              origin: { kind: 'task-notification', },
              message: { content: notification('b',), },
            },);
            /**
             Queue where a stale prompt sits ahead of the notification, so the dequeue takes the prompt.
             */
            const queued = [
              queueLine({ operation: 'enqueue', content: 'earlier prompt', },),
              queueLine({ operation: 'enqueue', content: notification('b',), },),
              queueLine({ operation: 'dequeue', },),
            ];

            expect(hasQueuedTaskNotification(queued,),).toBe(true,);
            expect(hasQueuedTaskNotification([...queued, delivery,],),).toBe(false,);
          },
        },),
        it({
          name: 'ignores a delivery that carries no text',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              queueLine({ operation: 'enqueue', content: notification('a',), },),
              JSON.stringify({ type: 'user', origin: { kind: 'task-notification', }, },),
            ],),).toBe(true,);
          },
        },),
        it({
          name: 'skips a truncated line and a tool result carrying a plain string',
          fn: async () => {
            expect(hasQueuedTaskNotification([
              '{"type":"queue-operation","operation":"enqu',
              JSON.stringify({ type: 'user', message: { content: 'tool_result', }, },),
              queueLine({ operation: 'enqueue', content: notification('a',), },),
            ],),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
