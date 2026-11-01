/**
 Application entry point.
 
 Boot sequence:
 1. Side-effect import of `./lib/db.ts` opens the SQLite database and runs migrations
 2. CSS is compiled from `src/client/styles.css` -\> `dist/css/styles.css`
 3. Start h3 HTTP server with page routes and API routes
 
 Client JS bundles are built separately via `mise run build:js:client` (tsdown).
 */
import { buildCss, } from '@monochromatic-dev/css-cli/ts';
import { HTTP_BAD_REQUEST, } from '@monochromatic-dev/const-core/ts';
import {
  defineHandler,
  getRouterParam,
  H3,
  type H3Event,
  HTTPError,
  serve,
} from 'h3';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: opens SQLite database and runs schema migrations
import './lib/db.ts';
import {
  ARGUMENT_ABSENT,
  getArgumentValue,
} from './lib/args.ts';
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
import { inProgressPage, } from './server/page/in-progress.ts';
import { inboxPage, } from './server/page/inbox.ts';
import { searchPage, } from './server/page/search.ts';
import { settingsPage, } from './server/page/settings.ts';
import { taskDetailsPage, } from './server/page/task-details.ts';
import { staticHandler, } from './server/static.ts';

/**
 Default HTTP port when neither `--port=` nor `PORT` env var is provided.
 */
const DEFAULT_PORT = 3_000;

/**
 Radix for decimal integer parsing.
 */
const DECIMAL_RADIX = 10;

/**
 Resolves the HTTP listen port from CLI arguments, environment, or default.
 
 @returns Resolved port number
 */
function resolvePort(): number {
  /**
   Port supplied via `--port=` CLI flag, if present.
   */
  const argumentPort = getArgumentValue('port',);
  /**
   Port from `PORT` environment variable, used when no CLI flag is given.
   */
  const environmentPort = process.env
    .PORT;
  /**
   Whichever source (CLI takes precedence) provided a value, or `undefined`.
   */
  const rawPort = argumentPort === ARGUMENT_ABSENT ? environmentPort : argumentPort;
  if (rawPort === undefined)
    return DEFAULT_PORT;
  /**
   Numeric port after `parseInt`; `NaN` falls back to {@link DEFAULT_PORT}.
   */
  const parsedPort = Number.parseInt(
    rawPort,
    DECIMAL_RADIX,
  );
  return Number.isNaN(parsedPort,) ? DEFAULT_PORT : parsedPort;
}

/**
 Extracts a required route parameter, throwing 400 if missing.
 
 @param event - h3 event
 
 @param name - Parameter name from the route pattern
 
 @returns Parameter value
 
 @throws {@link HTTPError} 400 when parameter is missing
 
 @example
 ```ts
 const id = requireParam({ event, name: 'id' });
 ```
 */
function requireParam(
  {
    event,
    name,
  }: {
    readonly event: H3Event;
    readonly name: string;
  },
): string {
  /**
   Raw parameter value from h3's router; `undefined` when missing.
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

await buildCss({
  input: './src/client/styles.css',
  output: './dist/css/styles.css',
},);

/**
 h3 application instance routing HTTP requests to handlers.
 */
const app = new H3();

app.get(
  '/',
  defineHandler(function handleInbox() {
    return inboxPage();
  },),
);
app.get(
  '/in-progress',
  defineHandler(function handleInProgress() {
    return inProgressPage();
  },),
);
app.get(
  '/tasks/:id',
  defineHandler(function handleTaskDetails(event,) {
    return taskDetailsPage(
      requireParam({
        event,
        name: 'id',
      },),
    );
  },),
);
app.get(
  '/search',
  defineHandler(function handleSearch(event,) {
    return searchPage(event.url,);
  },),
);
app.get(
  '/settings',
  defineHandler(function handleSettings() {
    return settingsPage();
  },),
);

app.post(
  '/api/tasks',
  defineHandler(function handleCreateTaskRoute(event,) {
    return handleCreateTask(event.req,);
  },),
);
app.put(
  '/api/tasks/:id',
  defineHandler(function handleUpdateTaskRoute(event,) {
    return handleUpdateTask({
      req: event.req,
      id: requireParam({
        event,
        name: 'id',
      },),
    },);
  },),
);
app.delete(
  '/api/tasks/:id',
  defineHandler(function handleDeleteTaskRoute(event,) {
    return handleDeleteTask(
      requireParam({
        event,
        name: 'id',
      },),
    );
  },),
);
app.post(
  '/api/tasks/:id/start',
  defineHandler(function handleStartTimerRoute(event,) {
    return handleStartTimer(
      requireParam({
        event,
        name: 'id',
      },),
    );
  },),
);
app.post(
  '/api/tasks/:id/stop',
  defineHandler(function handleStopTimerRoute(event,) {
    return handleStopTimer(
      requireParam({
        event,
        name: 'id',
      },),
    );
  },),
);
app.post(
  '/api/tasks/:id/complete',
  defineHandler(function handleCompleteTaskRoute(event,) {
    return handleCompleteTask(
      requireParam({
        event,
        name: 'id',
      },),
    );
  },),
);
app.post(
  '/api/ai/autofill',
  defineHandler(function handleAutofillRoute(event,) {
    return handleAutofill(event.req,);
  },),
);

app.get(
  '/dist/client/**',
  staticHandler,
);

/**
 Running HTTP server instance.
 */
const server = serve(
  app,
  { port: resolvePort(), },
);
console.log(`Listening on ${server.url}`,);
