import { tagged, } from '@monochromatic-dev/logger-core/tagged';
import {
  defineHandler,
  H3,
  serve,
} from 'h3';
import { indexHtml, } from './asset.ts';
import { l as parentLogger, } from './log.ts';
import { PORT, } from './port.ts';

/**
 * Tagged logger for the server subsystem.
 */
const l = tagged({
  tag: 'server',
  l: parentLogger,
},);

/**
 * H3 application instance for the exa-search server.
 */
const app = new H3();

/**
 * Builds a Response containing the exa-search HTML page.
 *
 * @returns HTML Response with status 200
 */
function serveIndexHtml(): Response {
  return new Response(
    indexHtml,
    {
      status: 200,
      headers: { 'content-type': 'text/html', },
    },
  );
}

/**
 * Serves the single-page exa-search application HTML.
 */
app.get(
  '/',
  defineHandler(serveIndexHtml,),
);

/**
 * Running HTTP server instance.
 */
const server = serve(
  app,
  { port: PORT, },
);

l.info(`listening on ${server.url}`,);
