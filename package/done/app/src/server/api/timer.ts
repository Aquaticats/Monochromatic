/**
 REST API handlers for timer and task completion.
 
 Mounted by server.ts as route handlers:
   POST /api/tasks/:id/start    -\> handleStartTimer
   POST /api/tasks/:id/stop     -\> handleStopTimer
   POST /api/tasks/:id/complete -\> handleCompleteTask
 */
import {
  HTTP_CONFLICT,
  HTTP_NOT_FOUND,
  HTTP_OK,
} from '@monochromatic-dev/const-core/ts';

import {
  completeTask,
  startTaskTimer,
  stopTaskTimer,
} from '../../lib/db/tasks.ts';
import { TASK_NOT_FOUND, } from '../../lib/types.ts';

/**
 Wraps a payload in a JSON `Response` with the correct content type.
 
 @param options - Serializable payload and HTTP status.
 
 @returns JSON response
 
 @mutates options - `Fetch commit 586cd2a4 Response.json serializes data and reads response initialization`
 may invoke serialization hooks reachable from `options.payload`.
 
 @example
 ```ts
 return jsonResponse({ payload: { ok: true, }, });
 return jsonResponse({ payload: { error: 'not found', }, status: HTTP_NOT_FOUND, });
 ```
 */
function jsonResponse(options: {
  readonly payload: unknown;
  readonly status?: number;
},): Response {
  /**
   Response fields separated after boundary contract attaches to their containing input.
   */
  const {
    payload,
    status = HTTP_OK,
  } = options;
  return Response.json(
    payload,
    { status, },
  );
}

/**
 POST /api/tasks/:id/start; starts the task timer via {@link startTaskTimer}.
 
 @param id - Task UUID from the route parameter
 
 @returns 200 with updated task, or 404 when not found
 
 @example
 ```ts
 const response = await handleStartTimer('abc-123');
 ```
 */
export async function handleStartTimer(id: string,): Promise<Response> {
  /**
   Updated row; the not-found sentinel distinguishes a missing task from a successful start.
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
 POST /api/tasks/:id/stop; stops the running timer and accumulates tracked
 time via {@link stopTaskTimer}.
 
 @param id - Task UUID from the route parameter
 
 @returns 200 with updated task, or 404 when not found
 
 @example
 ```ts
 const response = await handleStopTimer('abc-123');
 ```
 */
export async function handleStopTimer(id: string,): Promise<Response> {
  /**
   Updated row; the not-found sentinel distinguishes a missing task from a successful stop.
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
 POST /api/tasks/:id/complete; completes the task via {@link completeTask}
 if all blockers are resolved.
 
 @param id - Task UUID from the route parameter
 
 @returns 200 on success, 404 when missing, 409 when blocked
 
 @example
 ```ts
 const response = await handleCompleteTask('abc-123');
 ```
 */
export async function handleCompleteTask(id: string,): Promise<Response> {
  /**
   Discriminated outcome distinguishing completion, missing row, and blocker errors.
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
