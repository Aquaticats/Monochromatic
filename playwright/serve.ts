import { readFile, } from 'node:fs/promises';
import { extname, } from 'node:path';

import {
  defineHandler,
  H3,
  serve,
} from 'h3';

/**
 Map file extensions to MIME types for browser test assets.
 */
const mimeTypes: Record<string, string> = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.mjs': 'application/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.map': 'application/json',
};

/**
 Test-harness HTTP server; routes serve the harness HTML and built dist assets to Playwright.
 */
const app = new H3();

app.all(
  '/**',
  defineHandler(async function serveTestHarness(event,) {
    /**
     Request URL path; routed below by prefix match.
     */
    const { pathname, } = event.url;

    if ((pathname === '/') || (pathname === '/test-harness.html')) {
      return new Response(
        await readFile('playwright/test-harness.html',),
        {
          headers: { 'content-type': 'text/html', },
        },
      );
    }

    if (pathname.startsWith('/dist/module-test/',)) {
      /**
       Browser acceptance imports only this package's built fixture assets.
       */
      const contentType = mimeTypes[extname(pathname,)] ?? 'application/octet-stream';
      return new Response(
        await readFile(`package/test-harness/core/dist/${pathname.slice('/dist/module-test/'.length,)}`,),
        { headers: { 'content-type': contentType, }, },
      );
    }

    if (pathname.startsWith('/dist/module-logger.fuzz/',)) {
      /**
       Resolved MIME type for the requested sidecar asset; falls back to a safe binary type.
       */
      const contentType = mimeTypes[extname(pathname,)]
        ?? 'application/octet-stream';
      return new Response(
        await readFile(
          `package/logger/core.fuzz/dist/${pathname.slice('/dist/module-logger.fuzz/'.length,)}`,
        ),
        {
          headers: { 'content-type': contentType, },
        },
      );
    }

    if (pathname.startsWith('/dist/module-fs-path/',)) {
      /**
       Resolved MIME type for the requested fs-path asset; falls back to a safe binary type.
       */
      const contentType = mimeTypes[extname(pathname,)]
        ?? 'application/octet-stream';
      return new Response(
        await readFile(
          `package/fs-path/core/dist/${pathname.slice('/dist/module-fs-path/'.length,)}`,
        ),
        {
          headers: { 'content-type': contentType, },
        },
      );
    }

    if (pathname.startsWith('/dist/module-logger/',)) {
      /**
       Resolved MIME type for the requested asset; falls back to a safe binary type.
       */
      const contentType = mimeTypes[extname(pathname,)]
        ?? 'application/octet-stream';
      return new Response(
        await readFile(
          `package/logger/core/dist/${pathname.slice('/dist/module-logger/'.length,)}`,
        ),
        {
          headers: { 'content-type': contentType, },
        },
      );
    }

    return new Response(
      'Not found',
      { status: 404, },
    );
  },),
);

serve(
  app,
  { port: 3_005, },
);
