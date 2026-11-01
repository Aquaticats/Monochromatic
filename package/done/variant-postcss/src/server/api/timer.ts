/**
 REST API handlers for timer and task completion.
 
 Mounted by server.ts as route handlers:
   POST /api/tasks/:id/start    -\> handleStartTimer
   POST /api/tasks/:id/stop     -\> handleStopTimer
   POST /api/tasks/:id/complete -\> handleCompleteTask
 */
import {
  completeTask,
  startTaskTimer,
  stopTaskTimer,
} from '../../lib/db/tasks.ts';
import { TASK_NOT_FOUND, } from '../../lib/types.ts';
import {
  HTTP_CONFLICT,
  HTTP_NOT_FOUND,
  jsonResponse,
} from './http-utils.ts';

/**
 POST /api/tasks/:id/start; starts the task timer.
 
 @param id - Task UUID from the route parameter
 
 @returns 200 with updated task, or 404 when not found
 
 @example
 ```ts
 const response = await handleStartTimer('uuid-123');
 ```
 */
export async function handleStartTimer(id: string,): Promise<Response> {
  /**
   Updated task with the freshly set `timerStartedAt`; {@link TASK_NOT_FOUND} triggers a 404 response.
   */
  const task = await startTaskTimer(id,);
  if (task === TASK_NOT_FOUND) {
    return jsonResponse({
      payload: { error: 'Task not found', },
      status: HTTP_NOT_FOUND,
    },);
  }
  return jsonResponse({ payload: task, },);
}

/**
 POST /api/tasks/:id/stop; stops the running timer and accumulates tracked time.
 
 @param id - Task UUID from the route parameter
 
 @returns 200 with updated task, or 404 when not found
 
 @example
 ```ts
 const response = await handleStopTimer('uuid-123');
 ```
 */
export async function handleStopTimer(id: string,): Promise<Response> {
  /**
   Updated task with accumulated tracked time; {@link TASK_NOT_FOUND} triggers a 404 response.
   */
  const task = await stopTaskTimer(id,);
  if (task === TASK_NOT_FOUND) {
    return jsonResponse({
      payload: { error: 'Task not found', },
      status: HTTP_NOT_FOUND,
    },);
  }
  return jsonResponse({ payload: task, },);
}

/**
 POST /api/tasks/:id/complete; completes the task if all blockers are resolved.
 
 @param id - Task UUID from the route parameter
 
 @returns 200 on success, 404 when missing, 409 when blocked
 
 @example
 ```ts
 const response = await handleCompleteTask('uuid-123');
 ```
 */
export async function handleCompleteTask(id: string,): Promise<Response> {
  /**
   Completion outcome carrying both the success flag and any blockers refusing completion.
   */
  const result = await completeTask(id,);
  if (result.notFound) {
    return jsonResponse({
      payload: { error: 'Task not found', },
      status: HTTP_NOT_FOUND,
    },);
  }
  if (!result.completed) {
    return jsonResponse({
      payload: {
        error: 'Task is blocked',
        blockedBy: result.blockedBy,
      },
      status: HTTP_CONFLICT,
    },);
  }
  return jsonResponse({ payload: { ok: true, }, },);
}
