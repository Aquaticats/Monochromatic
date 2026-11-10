/**
 Regression coverage for an OPML source whose fetch rejects: the helper
 documented discarding unreachable sources, so a rejected `fetch` must not
 reject the whole text list it joins.
 */

import { logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { getOPMLTexts, } from './opml-text.ts';

/**
 Source URL whose host cannot be reached, so `fetch` rejects without a
 response.
 */
const UNREACHABLE_URL = 'https://unreachable.example/list.opml';

/**
 Source URL that answers with an OPML body.
 */
const READABLE_URL = 'https://readable.example/list.opml';

/**
 OPML document the readable source returns, asserted on the surviving text.
 */
const READABLE_OPML = '<?xml version="1.0"?><opml version="2.0"><body/></opml>';

/**
 Message a stubbed rejection carries, so a passing assertion proves the caught
 text reached the log instead of an empty object.
 */
const FETCH_FAILURE_MESSAGE = 'fetch failed: getaddrinfo ENOTFOUND unreachable.example';

await describe({
  name: getOPMLTexts.name,
  concurrency: 1,
  children: [
    it({
      name: 'discards an OPML source whose fetch rejects instead of rejecting the list',
      fn: async ctx => {
        /**
         Rendered warn lines the console sink emits while the stub is active.
         */
        const lines: string[] = [];
        ctx.sinon
          .stub(console, 'warn',)
          .callsFake((...args: unknown[]) => {
            lines.push(args.map(String,).join(' ',),);
          },);
        ctx.sinon
          .stub(globalThis, 'fetch',)
          .value(async function stubUnreachableFetch(url: string,) {
            if (url === UNREACHABLE_URL)
              throw new TypeError(FETCH_FAILURE_MESSAGE,);
            return new Response(READABLE_OPML, { status: 200, },);
          } as typeof fetch,);

        const texts = await getOPMLTexts([UNREACHABLE_URL, READABLE_URL,],);
        await logger.flush();

        expect(texts,).toEqual([READABLE_OPML,],);
        expect(lines.join('\n',),).toContain(FETCH_FAILURE_MESSAGE,);
      },
    },),
  ],
},);
