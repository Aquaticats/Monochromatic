import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  CONTINUATION_MARKER,
  FEEDBACK_PREFIX,
} from './continuation-depth.ts';
import {
  hasRunningBackgroundTask,
  hasRunningSubagent,
  recordCarriesToolUse,
  workedSinceLastForcedContinuation,
} from './continuation-progress.ts';

/**
 Transcript line standing for one forced-continuation feedback record.
 */
const BLOCK_LINE = `{"type":"user","message":{"content":"${FEEDBACK_PREFIX}:\\n${CONTINUATION_MARKER}."}}`;

/**
 Assistant record that issued a tool call.
 */
const TOOL_LINE = '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash"}]}}';

/**
 Assistant record that only produced prose.
 */
const TEXT_LINE = '{"type":"assistant","message":{"content":[{"type":"text","text":"still blocked on run 008"}]}}';

await describe({
  name: 'forced-continuation progress release',
  children: [
    describe({
      name: workedSinceLastForcedContinuation.name,
      children: [
        it({
          name: 'allows the first block, since nothing has been pushed yet',
          fn: async () => {
            expect(workedSinceLastForcedContinuation([TEXT_LINE,],),).toBe(true,);
          },
        },),
        it({
          name: 'reports work when the pushed turn issued a tool call',
          fn: async () => {
            expect(workedSinceLastForcedContinuation([BLOCK_LINE, TOOL_LINE, TEXT_LINE,],),).toBe(true,);
          },
        },),
        it({
          name: 'reports no work when the pushed turn only restated the blocker',
          fn: async () => {
            // The real failure: a session waiting on an external process, where every
            // forced turn produces another restatement and no work at all.
            expect(workedSinceLastForcedContinuation([BLOCK_LINE, TEXT_LINE,],),).toBe(false,);
          },
        },),
        it({
          name: 'looks only at the most recent push, not at earlier productive ones',
          fn: async () => {
            expect(
              workedSinceLastForcedContinuation([BLOCK_LINE, TOOL_LINE, BLOCK_LINE, TEXT_LINE,],),
            ).toBe(false,);
          },
        },),
        it({
          name: 'counts a read-only tool as work, since orienting is not idleness',
          fn: async () => {
            const readLine = '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read"}]}}';

            expect(workedSinceLastForcedContinuation([BLOCK_LINE, readLine,],),).toBe(true,);
          },
        },),
        it({
          name: 'reports work for an empty transcript rather than suppressing the first block',
          fn: async () => {
            expect(workedSinceLastForcedContinuation([],),).toBe(true,);
          },
        },),
        it({
          name: 'errs toward blocking when the block itself fell outside the tail',
          fn: async () => {
            // The opposite direction from the task-list release, which refuses to read a
            // truncated tail because its absence reads as finished. Here an unseen block
            // reads as work, so the hook keeps pushing, which the depth guard bounds.
            expect(workedSinceLastForcedContinuation([TEXT_LINE,],),).toBe(true,);
          },
        },),
        it({
          name: 'skips a truncated final line rather than throwing',
          fn: async () => {
            expect(workedSinceLastForcedContinuation([BLOCK_LINE, '{"type":"assist',],),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: hasRunningSubagent.name,
      children: [
        it({
          name: 'reports a running subagent',
          fn: async () => {
            expect(hasRunningSubagent([{ id: 'a', type: 'subagent', status: 'running', },],),).toBe(true,);
          },
        },),
        it({
          name: 'ignores a running shell task, a running monitor, and a finished subagent',
          fn: async () => {
            expect(hasRunningSubagent([
              { id: 'a', type: 'shell', status: 'running', },
              { id: 'b', type: 'monitor', status: 'running', },
              { id: 'c', type: 'subagent', status: 'completed', },
            ],),).toBe(false,);
          },
        },),
        it({
          name: 'reports no subagent when the list is empty or absent',
          fn: async () => {
            expect(hasRunningSubagent([],),).toBe(false,);
            expect(hasRunningSubagent(),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: hasRunningBackgroundTask.name,
      children: [
        it({
          name: 'reports no wait when the session has no background tasks',
          fn: async () => {
            expect(hasRunningBackgroundTask([],),).toBe(false,);
            expect(hasRunningBackgroundTask(),).toBe(false,);
          },
        },),
        it({
          name: 'reports a wait while a shell task is running',
          fn: async () => {
            // Shape captured from a live Stop payload on Claude Code 2.1.224.
            expect(hasRunningBackgroundTask([
              {
                id: 'b6ldjvy2v',
                type: 'shell',
                status: 'running',
                description: 'Sleep for 60 seconds in background',
                command: 'sleep 60',
              },
            ],),).toBe(true,);
          },
        },),
        it({
          name: 'reports no wait once every task has left the running state',
          fn: async () => {
            expect(hasRunningBackgroundTask([
              { id: 'a', type: 'shell', status: 'completed', },
              { id: 'b', type: 'shell', status: 'failed', },
            ],),).toBe(false,);
          },
        },),
        it({
          name: 'reports a wait when only one of several tasks is still running',
          fn: async () => {
            expect(hasRunningBackgroundTask([
              { id: 'a', type: 'shell', status: 'completed', },
              { id: 'b', type: 'shell', status: 'running', },
            ],),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: recordCarriesToolUse.name,
      children: [
        it({
          name: 'ignores a user record that happens to carry tool_use blocks',
          fn: async () => {
            expect(
              recordCarriesToolUse({ type: 'user', message: { content: [{ type: 'tool_use', },], }, },),
            ).toBe(false,);
          },
        },),
        it({
          name: 'ignores an assistant record whose content is a plain string',
          fn: async () => {
            expect(recordCarriesToolUse({ type: 'assistant', message: { content: 'done', }, },),).toBe(false,);
          },
        },),
        it({
          name: 'ignores a record with no message at all',
          fn: async () => {
            expect(recordCarriesToolUse({ type: 'assistant', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
