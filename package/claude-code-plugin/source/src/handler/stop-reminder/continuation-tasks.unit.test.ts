import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  FINISHED_STATUSES,
  INITIAL_STATUS,
  isFinishedStatus,
  taskListState,
} from './continuation-tasks.ts';

/**
 Builds a `TaskCreate` result record announcing a task id.
 
 @param id - task identifier Claude Code assigned
 
 @returns transcript line for a successful creation
 
 @example
 ```ts
 created('1');
 ```
 */
function created(id: string,): string {
  return JSON.stringify({ type: 'user', toolUseResult: { task: { id, subject: 'x', }, }, },);
}

/**
 Builds an assistant record issuing a `TaskUpdate` call.
 
 @param id - task identifier being updated
 @param status - status applied by the call
 
 @returns transcript line for the update
 
 @example
 ```ts
 updated('1', 'completed');
 ```
 */
function updated(id: string, status: string,): string {
  return JSON.stringify({
    type: 'assistant',
    message: { content: [{ type: 'tool_use', name: 'TaskUpdate', input: { taskId: id, status, }, },], },
  },);
}

await describe({
  name: 'forced-continuation task-list release',
  children: [
    describe({
      name: isFinishedStatus.name,
      children: [
        it({
          name: 'treats completed and deleted as needing no further work',
          fn: async () => {
            for (const status of FINISHED_STATUSES) {
              expect(isFinishedStatus(status,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'treats the observed unfinished statuses as work remaining',
          fn: async () => {
            for (const status of ['pending', 'in_progress',]) {
              expect(isFinishedStatus(status,),).toBe(false,);
            }
          },
        },),
        it({
          name: 'treats the initial status as unfinished',
          fn: async () => {
            expect(isFinishedStatus(INITIAL_STATUS,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: taskListState.name,
      children: [
        it({
          name: 'reports no task list when the session never created one',
          fn: async () => {
            // Most sessions never create a task; releasing here would disable the
            // mechanism everywhere rather than only where work is genuinely done.
            expect(taskListState([],),).toBe('no-task-list',);
          },
        },),
        it({
          name: 'reports work remaining for a freshly created task',
          fn: async () => {
            expect(taskListState([created('1',),],),).toBe('work-remains',);
          },
        },),
        it({
          name: 'reports work remaining while any task is in progress',
          fn: async () => {
            expect(taskListState([
              created('1',),
              created('2',),
              updated('1', 'completed',),
              updated('2', 'in_progress',),
            ],),).toBe('work-remains',);
          },
        },),
        it({
          name: 'reports all finished once every task reaches a terminal status',
          fn: async () => {
            expect(taskListState([
              created('1',),
              created('2',),
              updated('1', 'completed',),
              updated('2', 'deleted',),
            ],),).toBe('all-finished',);
          },
        },),
        it({
          name: 'lets a later update win over an earlier one',
          fn: async () => {
            expect(taskListState([
              created('1',),
              updated('1', 'completed',),
              updated('1', 'in_progress',),
            ],),).toBe('work-remains',);
          },
        },),
        it({
          name: 'tracks a task updated without its creation record in range',
          fn: async () => {
            // A tail read can start after the creation; the update alone still proves
            // the task exists.
            expect(taskListState([updated('7', 'in_progress',),],),).toBe('work-remains',);
          },
        },),
        it({
          name: 'skips a truncated final line rather than throwing',
          fn: async () => {
            expect(taskListState([created('1',), updated('1', 'completed',), '{"type":"assist',],),)
              .toBe('all-finished',);
          },
        },),
      ],
    },),
  ],
},);
