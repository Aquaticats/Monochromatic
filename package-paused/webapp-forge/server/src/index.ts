/**
 * Application entry point for the forge server.
 *
 * Boot sequence:
 *
 * 1. Side-effect import of `data/db.ts` opens the libSQL database and runs migrations.
 * 2. Storage adapter and write buffer are initialised by the runtime module.
 * 3. h3 app is constructed and routes are wired.
 * 4. `serve` binds to the resolved port.
 *
 * Phase 1: in-memory storage adapter only; the worker runs synchronously
 * inside each route handler. Phase 2 swaps the adapter to S3-compatible
 * and the dispatcher to a background worker pool.
 */

import { logger, } from '@monochromatic-dev/logger-core/logger';
import { tagged, } from '@monochromatic-dev/logger-core/tagged';
import {
  H3,
  serve,
} from 'h3';

// oxlint-disable-next-line import/no-unassigned-import -- side effect: opens libSQL and runs migrations
import './data/db.ts';

import { getArgumentValue, } from './lib/args.ts';
import { dispatchAndFlush, } from './server/dispatch-and-flush.ts';
import {
  authHandler,
  createCommentHandler,
  createIssueHandler,
  filterListHandler,
  gitInfoRefsHandler,
  gitReceivePackHandler,
  gitUploadPackHandler,
  issueDetailHandler,
  labelIssueHandler,
  meDeltaHandler,
  rawFragmentHandler,
} from './server/routes.ts';
import {
  getEventCursor,
  setEventCursor,
  storage,
  writeBuffer,
} from './server/runtime.ts';

/**
 * Tagged logger for the server boot.
 */
const l = tagged({
  tag: 'server',
  l: logger,
},);

/**
 * Default listen port when not overridden.
 */
const DEFAULT_PORT = 3_000;

/**
 * Decimal radix for `parseInt`.
 */
const DECIMAL_RADIX = 10;

/**
 * Resolves the listen port from CLI argument, environment, or default.
 *
 * @returns parsed port
 */
function resolvePort(): number {
  /**
   * `--port=N` CLI argument when supplied; highest priority source.
   */
  const argumentPort = getArgumentValue('port',);
  /**
   * `PORT` environment variable; second priority source.
   */
  const environmentPort = process.env
    .PORT;
  /**
   * Selected raw string; `undefined` falls through to the compile-time default.
   */
  const rawPort = argumentPort ?? environmentPort;
  if (rawPort === undefined)
    return DEFAULT_PORT;
  /**
   * Parsed integer; NaN falls back to `DEFAULT_PORT`.
   */
  const parsedPort = Number.parseInt(
    rawPort,
    DECIMAL_RADIX,
  );
  return Number.isNaN(parsedPort,) ? DEFAULT_PORT : parsedPort;
}

/**
 * h3 application instance routing HTTP requests to handlers.
 */
const app = new H3();

//region Read routes

app.get(
  '/_fragments/**',
  rawFragmentHandler,
);
app.get(
  '/:owner/:repo/issues/:number',
  issueDetailHandler,
);
app.get(
  '/:owner/:repo/issues',
  filterListHandler,
);

//endregion

//region Write routes

app.post(
  '/api/repos/:owner/:repo/issues',
  createIssueHandler,
);
app.post(
  '/api/repos/:owner/:repo/issues/:number/comments',
  createCommentHandler,
);
app.post(
  '/api/repos/:owner/:repo/issues/:number/labels/:label',
  labelIssueHandler,
);

//endregion

//region Git smart-HTTP routes

app.get(
  '/:owner/:repo/info/refs',
  gitInfoRefsHandler,
);
app.post(
  '/:owner/:repo/git-upload-pack',
  gitUploadPackHandler,
);
app.post(
  '/:owner/:repo/git-receive-pack',
  gitReceivePackHandler,
);

//endregion

//region Auth routes

app.all(
  '/api/auth/**',
  authHandler,
);

//endregion

//region Per-viewer delta route

app.get(
  '/api/me/delta',
  meDeltaHandler,
);

//endregion

// Drain every event in the libSQL log into the in-memory storage adapter
// before accepting requests. Phase 1's storage adapter is process-local
// in-memory, so a freshly booted server starts with empty storage even
// if the database already has many events from a prior `forge:seed` run.
l.info('warming fragment cache from event log',);
/**
 * Highest `events.id` processed during the boot-time warm-up pass.
 */
const warmCursor = await dispatchAndFlush({
  afterEventId: getEventCursor(),
  storage,
  writeBuffer,
},);
setEventCursor(warmCursor,);
l.info(`fragment cache warmed; highest event id: ${String(warmCursor,)}`,);

/**
 * Resolved listen port, taken from `--port=` argv, `PORT` env, or the default.
 */
const port = resolvePort();
serve(
  app,
  {
    port,
    hostname: '0.0.0.0',
  },
);
l.info(`forge listening on http://localhost:${String(port,)}`,);
