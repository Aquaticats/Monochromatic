/**
 Tests for the gateway's size refusal, which arrives disguised as a parse
 failure.

 THE DISCRIMINATORS ARE THE POINT. Re-raising every `400` as a size problem
 would be worse than saying nothing, because a body we genuinely malformed
 would then be reported as too big and whoever chased it would go looking for
 a limit rather than for their own bug. Three cases here each break one of the
 three signals and demand the plain failure back.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  failureForReply,
  SyntheticRequestTooLargeError,
} from '../dist/final/node/index.mjs';

/**
 Largest body measured to pass, mirrored from the module under test.

 SPELLED OUT RATHER THAN IMPORTED, deliberately. The module keeps this
 private, and a test that read the same constant it asserts against would
 agree with any value the module happened to hold, including a wrong one.
 */
const PASSING_BODY_BYTES = 10_485_760;

/**
 What the gateway says when a body exceeds its cap, offset and all.
 */
const PARSE_FAILURE = 'Could not parse request as valid JSON. Unterminated string in JSON at position 10444203';

await describe({
  name: failureForReply.name,
  children: [
    it({
      name: 'NAMES SIZE WHEN ALL THREE SIGNALS AGREE, which is the whole reason this exists. The '
        + 'gateway answers an oversize body with a parse failure at a byte offset, so the message '
        + 'describes our JSON rather than its own limit and a reader chases an encoder bug that is '
        + 'not there',
      fn: async () => {
        /**
         Failure built for a body over the passing size.
         */
        const failure = failureForReply({
          provider: 'synthetic',
          status: 400,
          bodyText: PARSE_FAILURE,
          requestBodyBytes: PASSING_BODY_BYTES + 1,
        },);

        expect(failure instanceof SyntheticRequestTooLargeError,).toBe(true,);
        if (!(failure instanceof SyntheticRequestTooLargeError))
          throw new Error('size failure by construction',);
        expect(failure.bodyBytes,).toBe(PASSING_BODY_BYTES + 1,);
        expect(failure.passingBodyBytes,).toBe(PASSING_BODY_BYTES,);
      },
    },),

    it({
      name: 'STAYS A PLAIN ERROR FOR A SMALL BODY CARRYING THE SAME MESSAGE, because a parse '
        + 'failure on a body well under the cap is a body we actually malformed. Reporting that as '
        + 'too large would send whoever chases it looking for a limit instead of for their own '
        + 'defect, which is the exact confusion this module was built to end',
      fn: async () => {
        /**
         Failure built for a small body the gateway could not parse.
         */
        const failure = failureForReply({
          provider: 'synthetic',
          status: 400,
          bodyText: PARSE_FAILURE,
          requestBodyBytes: 4_096,
        },);

        expect(failure instanceof SyntheticRequestTooLargeError,).toBe(false,);
        expect(failure instanceof ProviderHttpError,).toBe(true,);
      },
    },),

    it({
      name: 'STAYS A PLAIN ERROR FOR AN OVERSIZE BODY REFUSED FOR SOMETHING ELSE, since a big '
        + 'request can be rejected for a bad model name or a malformed parameter like any other. '
        + 'Size is only the explanation when the gateway offers the explanation it offers for size',
      fn: async () => {
        /**
         Failure built for a large body rejected on its content.
         */
        const failure = failureForReply({
          provider: 'synthetic',
          status: 400,
          bodyText: '{"error":{"message":"Unknown model"}}',
          requestBodyBytes: PASSING_BODY_BYTES * 2,
        },);

        expect(failure instanceof SyntheticRequestTooLargeError,).toBe(false,);
        expect(failure instanceof ProviderHttpError,).toBe(true,);
      },
    },),

    it({
      name: 'STAYS A PLAIN ERROR ON ANY OTHER STATUS, so a throttle or an upstream fault carrying '
        + 'an oversize body is still the throttle it was. The retry layer branches on status and '
        + 'nothing here may quietly change what it sees',
      fn: async () => {
        /**
         Failure built for a throttle on a large body.
         */
        const failure = failureForReply({
          provider: 'synthetic',
          status: 429,
          bodyText: PARSE_FAILURE,
          requestBodyBytes: PASSING_BODY_BYTES * 2,
        },);

        expect(failure instanceof SyntheticRequestTooLargeError,).toBe(false,);
        expect(failure.status,).toBe(429,);
      },
    },),

    it({
      name: 'CARRIES BOTH NUMBERS AND A WAY OUT IN THE MESSAGE, because whoever reads this in a log '
        + 'gets one line and no debugger. It states what was sent, what passes, how far over the '
        + 'request was, what to do about it, and where the measurement lives, and it keeps the '
        + 'gateway\'s own words OFF the line, which a class that declares its message quote-free '
        + 'may not repeat',
      fn: async () => {
        /**
         Message composed for a body 699575 bytes past the passing size.
         */
        const { message, } = failureForReply({
          provider: 'synthetic',
          status: 400,
          bodyText: PARSE_FAILURE,
          requestBodyBytes: 11_185_335,
        },);

        expect(message.includes('11185335',),).toBe(true,);
        expect(message.includes('10485760',),).toBe(true,);
        expect(message.includes('699575',),).toBe(true,);
        expect(message.includes('Send less in one call',),).toBe(true,);
        expect(
          message.includes('doc/troubleshooting/synthetic-request-body-size-cap.md',),
        ).toBe(true,);
        expect(message.includes('Unterminated string',),).toBe(false,);
      },
    },),
    it({
      name: 'CARRIES THE PROVIDER THAT SERVED THE CALL on every failure, and READS THE SIZE ONLY ON SYNTHETIC, whose '
        + 'gateway the cap and its parse-failure words were measured on: the same reply from another provider stays '
        + 'a plain failure rather than claiming a size nobody measured there',
      fn: async () => {
        /**
         Plain failure for a throttle on Bedrock.
         */
        const plain = failureForReply({
          provider: 'bedrock',
          status: 429,
          bodyText: 'too many requests',
          requestBodyBytes: 4_096,
        },);
        /**
         Reply on Charm Hyper with every size signal Synthetic's gateway gives.
         */
        const elsewhere = failureForReply({
          provider: 'hyper',
          status: 400,
          bodyText: PARSE_FAILURE,
          requestBodyBytes: PASSING_BODY_BYTES + 1,
        },);
        /**
         The same reply on Synthetic.
         */
        const onSynthetic = failureForReply({
          provider: 'synthetic',
          status: 400,
          bodyText: PARSE_FAILURE,
          requestBodyBytes: PASSING_BODY_BYTES + 1,
        },);

        expect(plain.provider,).toBe('bedrock',);
        expect(elsewhere.provider,).toBe('hyper',);
        expect(elsewhere instanceof SyntheticRequestTooLargeError,).toBe(false,);
        expect(elsewhere.message.includes('measured to pass',),).toBe(false,);
        expect(onSynthetic.provider,).toBe('synthetic',);
        expect(onSynthetic instanceof SyntheticRequestTooLargeError,).toBe(true,);
        expect(onSynthetic.message.startsWith('The Synthetic API refused a request body of',),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS whatever the gateway echoed on the excerpt for the log and out of the message, so a '
        + 'gateway that repeats request bytes cannot put corpus text on a CLI line',
      fn: async () => {
        /**
         Refusal built from a gateway body that echoes something of ours.
         */
        const failure = failureForReply({
          provider: 'synthetic',
          status: 400,
          bodyText: `${PARSE_FAILURE} whiskers-gateway-echo`,
          requestBodyBytes: 11_185_335,
        },);
        expect(failure.message.includes('whiskers-gateway-echo',),).toBe(false,);
        expect(
          (failure instanceof ProviderHttpError) ? failure.bodyExcerpt.includes('whiskers-gateway-echo',) : false,
        ).toBe(true,);
      },
    },),
  ],
},);
