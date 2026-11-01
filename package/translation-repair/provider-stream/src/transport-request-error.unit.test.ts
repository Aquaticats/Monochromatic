/**
 Tests for the failure the transport throws when the runtime rejects a request:
 an authored account that quotes nothing the runtime wrote.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure';
import { TransportRequestFailedError, } from '../dist/final/node/index.mjs';
import {
  quotingFailure,
  WHISKER_KEY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';

await describe({
  name: TransportRequestFailedError.name,
  children: [
    it({
      name: 'STATES THE ACCOUNT ITSELF and repeats it through the refusal printer, never the runtime\'s words',
      fn: async () => {
        /**
         Rejection whose message quotes the stand-in key.
         */
        const cause = quotingFailure();
        /**
         Failure built over it.
         */
        const failure = new TransportRequestFailedError({
          label: 'hf:whiskers',
          cause,
        },);
        expect(failure.message.includes(WHISKER_KEY,),).toBe(false,);
        expect(refusalText({ error: failure, },),).toBe(failure.message,);
        expect(failure.cause,).toBe(cause,);
        expect(String(failure,),).toBe(
          'TransportRequestFailedError: hf:whiskers: the request failed before any answer came back, either '
            + 'because the network could not be reached or because the transport refused to send it (a header '
            + 'value it cannot carry is one such refusal); check the connection and the key',
        );
      },
    },),
  ],
},);
