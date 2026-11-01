/**
 REST API handlers for task CRUD operations.
 
 Mounted by server.ts as route handlers:
   POST   /api/tasks       -> handleCreateTask
   PUT    /api/tasks/:id   -> handleUpdateTask
   DELETE /api/tasks/:id   -> handleDeleteTask
 */
import {
  HTTP_BAD_REQUEST,
  HTTP_CREATED,
  HTTP_INTERNAL_SERVER_ERROR,
  HTTP_NOT_FOUND,
  HTTP_OK,
} from '@monochromatic-dev/const-core/ts';

import {
  createTask,
  deleteTask,
  updateTask,
} from '../../lib/db/tasks.ts';
import {
  type TaskComplexity,
  TASK_NOT_FOUND,
  type TaskPriority,
} from '../../lib/types.ts';
import { parseTaskUpdateInput, } from './tasks-parse-update.ts';
import {
  getPriorities,
  INVALID,
  isRecord,
  parseEnumValue,
  parseStringArray,
} from './tasks-parse.ts';

/**
 Wraps a payload in a JSON `Response` with the correct content type.
 
 @param options - Serializable payload and HTTP status.
 
 @returns JSON response with content-type header
 
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
 POST /api/tasks; creates a new task from the request body.
 Validates the body with {@link isRecord} and inserts via {@link createTask}.
 
 @param req - Incoming request with JSON body
 
 @returns 201 with the created task, or 400/500 on validation/server error
 
 @example
 ```ts
 const response = await handleCreateTask(request);
 ```
 */
export async function handleCreateTask(req: Request,): Promise<Response> {
  try {
    /**
     Loosely-typed body validated by {@link isRecord} before field-by-field access.
     */
    const body: unknown = await req.json();
    if (!isRecord(body,)) {
      return jsonResponse({
        payload: { error: 'Invalid request body', },
        status: HTTP_BAD_REQUEST,
      },);
    }

    /**
     Trimmed title; empty string short-circuits to the 400 branch below.
     */
    const title = (typeof body.title) === 'string' ? body.title
      .trim() : '';
    if (title.length
      === 0) {
      return jsonResponse({
        payload: { error: 'Task title is required', },
        status: HTTP_BAD_REQUEST,
      },);
    }

    /**
     Allowed priority values reused for both `priority` and `complexity` parsing.
     */
    const priorities = getPriorities();
    /**
     Validated tags; INVALID (not an array) falls back to an empty list.
     */
    const parsedTags = parseStringArray(body.tags,);
    /**
     Validated locations; INVALID falls back to an empty list.
     */
    const parsedLocations = parseStringArray(body.locations,);
    /**
     Validated priority string, or INVALID when absent/unrecognised.
     */
    const parsedPriority = parseEnumValue({
      value: body.priority,
      validValues: priorities,
    },);
    /**
     Validated complexity string, or INVALID when absent/unrecognised.
     */
    const parsedComplexity = parseEnumValue({
      value: body.complexity,
      validValues: priorities,
    },);
    /**
     Priority field, included only when a valid value was supplied.
     */
    const priorityField: { priority?: TaskPriority; } = parsedPriority === INVALID
      ? {}
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- member of the priorities set, which holds exactly the TaskPriority values
      : { priority: parsedPriority as TaskPriority, };
    /**
     Complexity field, included only when a valid value was supplied.
     */
    const complexityField: { complexity?: TaskComplexity; } = parsedComplexity === INVALID
      ? {}
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- member of the priorities set, whose low/medium/high values are exactly the TaskComplexity values
      : { complexity: parsedComplexity as TaskComplexity, };
    /**
     Created row returned by the data layer; serialised below with 201.
     */
    const task = await createTask({
      title,
      ...(((typeof body.description) === 'string') ? { description: body.description, } : {}),
      tags: parsedTags === INVALID ? [] : parsedTags,
      locations: parsedLocations === INVALID ? [] : parsedLocations,
      ...priorityField,
      ...complexityField,
    },);
    return jsonResponse({
      payload: task,
      status: HTTP_CREATED,
    },);
  }
  catch (error) {
    return jsonResponse({
      payload: { error: String(error,), },
      status: HTTP_INTERNAL_SERVER_ERROR,
    },);
  }
}

/**
 PUT /api/tasks/:id; applies a partial update to an existing task.
 Validates the body with {@link parseTaskUpdateInput} and applies it via
 {@link updateTask}.
 
 @param req - Incoming request with JSON body
 
 @param id - Task UUID from the route parameter
 
 @returns 200 with updated task, 400 on bad payload, 404 when missing
 
 @example
 ```ts
 const response = await handleUpdateTask({ req: request, id: 'abc-123', });
 ```
 */
export async function handleUpdateTask({
  req,
  id,
}: {
  readonly req: Request;
  readonly id: string;
},): Promise<Response> {
  try {
    /**
     Loosely-typed body validated by {@link parseTaskUpdateInput} below.
     */
    const body: unknown = await req.json();
    /**
     Normalised update payload; INVALID indicates the body failed validation.
     */
    const taskUpdateInput = parseTaskUpdateInput(body,);
    if (taskUpdateInput === INVALID) {
      return jsonResponse({
        payload: { error: 'Invalid update payload', },
        status: HTTP_BAD_REQUEST,
      },);
    }

    /**
     Result row; the not-found sentinel is surfaced as 404 below.
     */
    const task = await updateTask({
      id,
      input: taskUpdateInput,
    },);
    if (task === TASK_NOT_FOUND) {
      return jsonResponse({
        payload: { error: 'Task not found', },
        status: HTTP_NOT_FOUND,
      },);
    }

    return jsonResponse({ payload: task, },);
  }
  catch (error) {
    return jsonResponse({
      payload: { error: String(error,), },
      status: HTTP_INTERNAL_SERVER_ERROR,
    },);
  }
}

/**
 DELETE /api/tasks/:id; permanently removes a task via {@link deleteTask}.
 
 @param id - Task UUID from the route parameter
 
 @returns 200 on success, 404 when the task does not exist
 
 @example
 ```ts
 const response = await handleDeleteTask('abc-123');
 ```
 */
export async function handleDeleteTask(id: string,): Promise<Response> {
  /**
   Whether a row was actually removed; `false` is surfaced as 404 below.
   */
  const deleted = await deleteTask(id,);
  if (!deleted) {
    return jsonResponse({
      payload: { error: 'Task not found', },
      status: HTTP_NOT_FOUND,
    },);
  }

  return jsonResponse({ payload: { ok: true, }, },);
}
