/**
 Tests for the log of the limits a Claude key states: the first reply that carries any is logged once, under the tag it
 is given, and nothing else is.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { infoLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import {
  limitsLoggerOnce,
  RATE_LIMIT_HEADER_PREFIX,
} from '../dist/final/node/index.mjs';

await describe({
  name: limitsLoggerOnce.name,
  // SEQUENTIAL: each case diverts the process-wide console.
  concurrency: 1,
  children: [
    it({
      name: 'LOGS THE LIMITS OF THE FIRST REPLY THAT STATES ANY, once: a reply with none is passed over, the next one '
        + 'that states some is logged, and a third is not',
      fn: async () => {
        const logLimits = limitsLoggerOnce({ tag: 'the-cat-client', },);
        const { logged, } = await infoLinesDuring({
          run: async () => {
            logLimits({ status: 200, bodyText: '', headers: { 'content-type': 'text/event-stream', }, },);
            logLimits({
              status: 200,
              bodyText: '',
              headers: { [`${RATE_LIMIT_HEADER_PREFIX}requests-limit`]: '1000', 'retry-after': '1', },
            },);
            logLimits({
              status: 200,
              bodyText: '',
              headers: { [`${RATE_LIMIT_HEADER_PREFIX}requests-limit`]: '2000', },
            },);
          },
        },);
        expect(logged.length,).toBe(1,);
        expect(logged[0],).toContain('[the-cat-client]',);
        expect(logged[0],).toContain('rate limits on the first reply: anthropic-ratelimit-requests-limit=1000',);
        expect(logged[0],).not.toContain('retry-after',);
      },
    },),
  ],
},);
