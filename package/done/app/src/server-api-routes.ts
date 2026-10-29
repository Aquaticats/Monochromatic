/**
 API route registration for the h3 application.
 
 Wires up all JSON API endpoints (CRUD, timers, AI autofill)
 to their respective handler functions.
 */
import {
  defineHandler,
  getRouterParam,
  type H3,
  type H3Event,
  HTTPError,
} from 'h3';
import { handleAutofill, } from './server/api/ai-autofill.ts';
import {
  handleCreateTask,
  handleDeleteTask,
  handleUpdateTask,
} from './server/api/tasks.ts';
import {
  handleCompleteTask,
  handleStartTimer,
  handleStopTimer,
} from './server/api/timer.ts';

import { HTTP_BAD_REQUEST, } from '@monochromatic-dev/const-core/ts';

/**
 Extracts a required route parameter, throwing 400 if missing.
 
 @param event - h3 event
 
 @param name - Parameter name from the route pattern
 
 @returns Parameter value
 
 @throws {@link HTTPError} 400 when parameter is missing
 
 @example
 ```ts
 const id = requireParam({ event, name: 'id', });
 ```
 */
function requireParam({
  event,
  name,
}: {
  readonly event: H3Event;
  readonly name: string;
},): string {
  /**
   Route value as returned by h3; converted to a thrown 400 below when absent.
   */
  const value = getRouterParam(
    event,
    name,
  );
  if (value === undefined) {
    throw new HTTPError({
      status: HTTP_BAD_REQUEST,
      message: `missing route parameter: ${name}`,
    },);
  }
  return value;
}

/**
 Registers all API routes on the given h3 application.
 
 @param app - h3 application instance to attach routes to
 
 @mutates app - `app.delete`, `app.post`, and `app.put` register and retain route handlers.
 
 @example
 ```ts
 registerApiRoutes(app);
 ```
 */
export function registerApiRoutes(app: H3,): void {
  app.post(
    '/api/tasks',
    defineHandler(function handleCreateTaskRoute(event,) {
      return handleCreateTask(event.req,);
    },),
  );

  app.put(
    '/api/tasks/:id',
    defineHandler(function handleUpdateTaskRoute(event,) {
      /**
       Required route slug; thrown as 400 by {@link requireParam} when absent.
       */
      const id = requireParam({
        event,
        name: 'id',
      },);
      return handleUpdateTask({
        req: event.req,
        id,
      },);
    },),
  );

  app.delete(
    '/api/tasks/:id',
    defineHandler(function handleDeleteTaskRoute(event,) {
      /**
       Required route slug; thrown as 400 by {@link requireParam} when absent.
       */
      const id = requireParam({
        event,
        name: 'id',
      },);
      return handleDeleteTask(id,);
    },),
  );

  app.post(
    '/api/tasks/:id/start',
    defineHandler(function handleStartTimerRoute(event,) {
      /**
       Required route slug; thrown as 400 by {@link requireParam} when absent.
       */
      const id = requireParam({
        event,
        name: 'id',
      },);
      return handleStartTimer(id,);
    },),
  );

  app.post(
    '/api/tasks/:id/stop',
    defineHandler(function handleStopTimerRoute(event,) {
      /**
       Required route slug; thrown as 400 by {@link requireParam} when absent.
       */
      const id = requireParam({
        event,
        name: 'id',
      },);
      return handleStopTimer(id,);
    },),
  );

  app.post(
    '/api/tasks/:id/complete',
    defineHandler(function handleCompleteTaskRoute(event,) {
      /**
       Required route slug; thrown as 400 by {@link requireParam} when absent.
       */
      const id = requireParam({
        event,
        name: 'id',
      },);
      return handleCompleteTask(id,);
    },),
  );

  app.post(
    '/api/ai/autofill',
    defineHandler(function handleAutofillRoute(event,) {
      return handleAutofill(event.req,);
    },),
  );
}
