import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { parseIncludedResponse, } from '../dist/final/node/index.mjs';

await describe({
  name: parseIncludedResponse.name,
  children: [
    it({
      name: 'parses status headers and JSON body from gh include output',
      fn: async () => {
        /**
         Representative `gh api --include` response using CRLF delimiters.
         */
        const stdout = [
          'HTTP/2.0 201 Created',
          'Content-Type: application/json; charset=utf-8',
          'X-RateLimit-Remaining: 4999',
          'X-RateLimit-Reset: 2000000000',
          '',
          '{"number":42,"html_url":"https://github.com/owner/repo/issues/42"}',
        ].join('\r\n',);

        expect(parseIncludedResponse({ stdout, },),).toStrictEqual({
          status: 201,
          headers: {
            'content-type': 'application/json; charset=utf-8',
            'x-ratelimit-remaining': '4999',
            'x-ratelimit-reset': '2000000000',
          },
          body: {
            number: 42,
            html_url: 'https://github.com/owner/repo/issues/42',
          },
        },);
      },
    },),
  ],
},);
