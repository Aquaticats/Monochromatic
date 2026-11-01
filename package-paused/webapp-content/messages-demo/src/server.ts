/**
 * Application entry point.
 *
 * Boot sequence:
 *
 * 1. Side-effect import of `./lib/db.ts` opens the SQLite database and
 *    runs migrations (also seeds the three identities the dropdown shows).
 * 2. CSS is compiled from `src/client/styles.css` -> `dist/css/styles.css`
 *    so a fresh checkout works without a separate `mise run build:css`.
 * 3. h3 app is constructed and the route table is wired.
 * 4. `serve` binds to the resolved port.
 *
 * Client JS bundles are built separately via `mise run build:js:client`
 * (tsdown). The server logs a hint if the bundles are missing.
 */

import { buildCss as buildCSS, } from '@monochromatic-dev/css-cli/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  defineHandler,
  H3,
  HTTPError,
  redirect,
  serve,
} from 'h3';
import { existsSync, } from 'node:fs';

// oxlint-disable-next-line import/no-unassigned-import -- side-effect: opens SQLite and runs migrations
import './lib/db.ts';

import {
  ARG_ABSENT,
  getArgumentValue,
} from './lib/args.ts';
import {
  HTTP_BAD_REQUEST,
  HTTP_FOUND,
} from './lib/http.ts';
import {
  cancelDraftHandler,
  createDraftHandler,
  finalizeDraftHandler,
  putChunkHandler,
} from './server/api/drafts.ts';
import { importHandler, } from './server/api/import.ts';
import {
  deleteMessageHandler,
  editMessageHandler,
} from './server/api/messages.ts';
import {
  renderChunkMd,
  renderChunkRaw,
} from './server/pages/chunk-data.ts';
import { renderEditPage, } from './server/pages/edit.ts';
import { renderFeed, } from './server/pages/feed.ts';
import { renderMessageChunk, } from './server/pages/message.ts';
import {
  ifNoneMatchInput,
  paramsInput,
  parseId,
  requireParam,
} from './server/route-params.ts';
import { staticHandler, } from './server/static.ts';

/**
 * Tagged logger for the server boot.
 */
const l = tagged({
  tag: 'server',
  l: logger,
},);

/**
 * Default HTTP port when neither `--port=` nor `PORT` env var is provided.
 */
const DEFAULT_PORT = 3_000;

/**
 * Radix used for explicit decimal integer parsing.
 */
const DECIMAL_RADIX = 10;

/**
 * Resolves the listen port from CLI argument, environment, or default.
 *
 * @returns parsed port
 */
function resolvePort(): number {
  /**
   * CLI override; preferred over the env var so the developer's `--port=` wins in foreground runs.
   */
  const argumentPort = getArgumentValue('port',);
  /**
   * Fallback environment value; used when the CLI did not supply one.
   */
  const environmentPort = process.env
    .PORT;
  /**
   * Resolved precedence: CLI \> env; absent on both falls through to the default.
   */
  const rawPort = argumentPort !== ARG_ABSENT
    ? argumentPort
    : environmentPort;
  if (rawPort === undefined)
    return DEFAULT_PORT;
  /**
   * Parsed value; NaN signals a malformed input, in which case the default wins.
   */
  const parsedPort = Number.parseInt(
    rawPort,
    DECIMAL_RADIX,
  );
  return Number.isNaN(parsedPort,) ? DEFAULT_PORT : parsedPort;
}

/**
 * Compiles `src/client/styles.css` to `dist/css/styles.css`. Skipped
 * silently if the source does not exist yet (very first boot before
 * `client/styles.css` has been written).
 */
async function compileCss(): Promise<void> {
  if (!existsSync('./src/client/styles.css',)) {
    l.info('skipping CSS build: src/client/styles.css missing',);
    return;
  }
  await buildCSS({
    input: './src/client/styles.css',
    output: './dist/css/styles.css',
  },);
}

await compileCss();

if (!existsSync('./dist/client/index.js',)) {
  l.warn(
    'dist/client/index.js missing: run `mise run build:js:client` to enable the composer',
  );
}

/**
 * h3 application instance routing HTTP requests to handlers.
 */
const app = new H3();

//region Read pages

app.get(
  '/',
  defineHandler(
    async function handleFeed(event,) {
      return await renderFeed({ ...ifNoneMatchInput(event.req
        .headers,), },);
    },
  ),
);

app.get(
  '/p/:cursor',
  defineHandler(
    async function handleFeedPage(event,) {
      /**
       * Required `:cursor` path param; bails to 400 when missing.
       */
      const cursor = requireParam({
        ...paramsInput(event.context
          .params,),
        name: 'cursor',
      },);
      return await renderFeed({
        cursorToken: cursor,
        ...ifNoneMatchInput(event.req
          .headers,),
      },);
    },
  ),
);

app.get(
  '/m/:id',
  defineHandler(
    function handleMessageRoot(event,) {
      /**
       * Parsed `:id` param; redirect target uses this in the chunk-0 URL.
       */
      const id = parseId({
        ...paramsInput(event.context
          .params,),
        name: 'id',
      },);
      return redirect(
        `/m/${String(id,)}/c/0`,
        HTTP_FOUND,
      );
    },
  ),
);

app.get(
  '/m/:id/c/:idx',
  defineHandler(
    async function handleMessageChunk(event,) {
      /**
       * Parsed `:id` param; consumed by `renderMessageChunk` as the message id.
       */
      const id = parseId({
        ...paramsInput(event.context
          .params,),
        name: 'id',
      },);
      /**
       * Parsed `:idx` param; chunk index inside the message.
       */
      const idx = parseId({
        ...paramsInput(event.context
          .params,),
        name: 'idx',
        min: 0,
      },);
      return await renderMessageChunk({
        messageId: id,
        chunkIndex: idx,
        ...ifNoneMatchInput(event.req
          .headers,),
      },);
    },
  ),
);

app.get(
  '/m/:id/c/:idx/raw',
  defineHandler(
    async function handleChunkRaw(event,) {
      /**
       * Parsed `:id` param; consumed by `renderChunkRaw` as the message id.
       */
      const id = parseId({
        ...paramsInput(event.context
          .params,),
        name: 'id',
      },);
      /**
       * Parsed `:idx` param; chunk index inside the message.
       */
      const idx = parseId({
        ...paramsInput(event.context
          .params,),
        name: 'idx',
        min: 0,
      },);
      return await renderChunkRaw({
        messageId: id,
        chunkIndex: idx,
        ...ifNoneMatchInput(event.req
          .headers,),
      },);
    },
  ),
);

app.get(
  '/m/:id/c/:idx/md',
  defineHandler(
    async function handleChunkMd(event,) {
      /**
       * Parsed `:id` param; consumed by `renderChunkMd` as the message id.
       */
      const id = parseId({
        ...paramsInput(event.context
          .params,),
        name: 'id',
      },);
      /**
       * Parsed `:idx` param; chunk index inside the message.
       */
      const idx = parseId({
        ...paramsInput(event.context
          .params,),
        name: 'idx',
        min: 0,
      },);
      return await renderChunkMd({
        messageId: id,
        chunkIndex: idx,
        ...ifNoneMatchInput(event.req
          .headers,),
      },);
    },
  ),
);

app.get(
  '/m/:id/edit',
  defineHandler(
    async function handleEdit(event,) {
      /**
       * Parsed `:id` param; consumed by `renderEditPage`.
       */
      const id = parseId({
        ...paramsInput(event.context
          .params,),
        name: 'id',
      },);
      return await renderEditPage(id,);
    },
  ),
);

//endregion

//region Write API

app.post(
  '/api/drafts',
  createDraftHandler,
);
app.put(
  '/api/drafts/:id/chunks/:seq',
  putChunkHandler,
);
app.post(
  '/api/drafts/:id/finalize',
  finalizeDraftHandler,
);
app.delete(
  '/api/drafts/:id',
  cancelDraftHandler,
);
app.post(
  '/api/messages/:id/edit',
  editMessageHandler,
);
app.post(
  '/api/messages/:id/delete',
  deleteMessageHandler,
);
app.post(
  '/api/import',
  importHandler,
);

/**
 * The HTML composer form points its `action` at this endpoint as a
 * fallback for the (unsupported) no-JS path. We always 400, because JS
 * is required.
 */
app.post(
  '/api/composer-noscript',
  defineHandler(
    function handleNoScriptPost() {
      throw new HTTPError({
        status: HTTP_BAD_REQUEST,
        message: 'JavaScript is required to send messages. Enable JS and try again.',
      },);
    },
  ),
);

//endregion

//region Static assets

app.get(
  '/dist/**',
  staticHandler,
);

//endregion

/**
 * Resolved listen port, taken from `--port=` argv or `3000`.
 */
const port = resolvePort();
serve(
  app,
  {
    port,
    hostname: '0.0.0.0',
  },
);
l.info(`listening on http://localhost:${String(port,)}`,);
