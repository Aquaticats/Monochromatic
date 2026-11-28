/**
 Unit tests for the OVHcloud request core: the signature, the clock correction,
 masked paths, OVHcloud's own message on a failure, and the rule that no key
 value ever reaches a message, a log line or a stream.

 Every body here is a stand-in written by hand from OVHcloud's schema of the
 cloud API, never a recording, so nothing in this file claims what OVHcloud really
 answers. The one value compared against a fixed expectation is the signature,
 whose expected digest was computed a second way:

 ```sh
 printf '%s' 'standin-as-1111-not-a-real-secret+standin-ck-2222-not-a-real-key+POST+https://ca.api.ovh.com/v1/cloud/project/mvm-stand-in-project/instance+{"name":"mvm-dev","monthlyBilling":false}+1800000000' | sha1sum
 # => 0d5de8773c006c41674f43c9c8305212504d3bc6
 ```

 and, for a request with no body,

 ```sh
 printf '%s' 'standin-as-1111-not-a-real-secret+standin-ck-2222-not-a-real-key+GET+https://ca.api.ovh.com/v1/cloud/project/mvm-stand-in-project/instance++1800000000' | sha1sum
 # => 82035ca39ba3b2aeab973317e4cd117a8b07e3c6
 ```

 @module
 */

import {
  caughtValueStack,
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  AUTH_TIME_PATH,
  forgetServerClockOffset,
  MASKED_PROJECT,
  maskProjectId,
  ovhList,
  OvhApiError,
  ovhRequest,
  signRequest,
  signaturePayload,
} from '../../../dist/final/node/index.mjs';
import {
  cloudPath,
  errorFrom,
  failureAnswer,
  installEnv,
  installRawStandIn,
  installStandIn,
  jsonAnswer,
  STAND_IN_APPLICATION_SECRET,
  STAND_IN_CONSUMER_KEY,
  STAND_IN_PROJECT,
  STAND_IN_SERVER_TIME,
} from './fixture.ovh-api-stand-in.ts';

//region Fixed expectations

/**
 Digest of the fixed POST payload, computed with `sha1sum` as shown in this
 file's top doc.
 */
const POST_SIGNATURE_DIGEST = '0d5de8773c006c41674f43c9c8305212504d3bc6';

/**
 Digest of the fixed GET payload with an empty body, computed with `sha1sum` as
 shown in this file's top doc.
 */
const GET_SIGNATURE_DIGEST = '82035ca39ba3b2aeab973317e4cd117a8b07e3c6';

/**
 The fixed target both digests cover.
 */
const FIXED_TARGET = `https://ca.api.ovh.com${cloudPath('/instance',)}`;

/**
 The fixed timestamp both digests cover.
 */
const FIXED_TIMESTAMP = 1_800_000_000;

//endregion Fixed expectations

await describe({
  name: 'ovh request core',
  concurrency: 1,
  children: [
    //region Signature

    it({
      name: 'signaturePayload joins the six fields OVHcloud digests',
      fn: async () => {
        expect(
          signaturePayload({
            applicationSecret: 'as',
            body: '',
            consumerKey: 'ck',
            method: 'get',
            target: 'https://ca.api.ovh.com/v1/cloud/project/p/instance',
            timestamp: 1,
          },),
        ).toBe('as+ck+GET+https://ca.api.ovh.com/v1/cloud/project/p/instance++1',);
      },
    },),
    it({
      name: 'signRequest signs a POST body with the digest sha1sum gave for the same fields',
      fn: async () => {
        expect(
          signRequest({
            applicationKey: 'ak',
            applicationSecret: STAND_IN_APPLICATION_SECRET,
            body: '{"name":"mvm-dev","monthlyBilling":false}',
            consumerKey: STAND_IN_CONSUMER_KEY,
            method: 'post',
            target: FIXED_TARGET,
            timestamp: FIXED_TIMESTAMP,
          },),
        ).toEqual({
          'X-Ovh-Application': 'ak',
          'X-Ovh-Consumer': STAND_IN_CONSUMER_KEY,
          'X-Ovh-Signature': `$1$${POST_SIGNATURE_DIGEST}`,
          'X-Ovh-Timestamp': String(FIXED_TIMESTAMP,),
        },);
      },
    },),
    it({
      name: 'signRequest signs a GET with an empty body with the digest sha1sum gave for the same fields',
      fn: async () => {
        expect(
          signRequest({
            applicationKey: 'ak',
            applicationSecret: STAND_IN_APPLICATION_SECRET,
            body: '',
            consumerKey: STAND_IN_CONSUMER_KEY,
            method: 'GET',
            target: FIXED_TARGET,
            timestamp: FIXED_TIMESTAMP,
          },)['X-Ovh-Signature'],
        ).toBe(`$1$${GET_SIGNATURE_DIGEST}`,);
      },
    },),

    //endregion Signature

    //region Host clock

    it({
      name: 'ovhRequest corrects the timestamp with the API host clock and reads that clock once',
      fn: async () => {
        await using _env = await installEnv();
        forgetServerClockOffset();
        using standIn = installStandIn(() => jsonAnswer([],),);
        await ovhRequest({
          method: 'GET',
          path: cloudPath('/instance',),
        },);
        await ovhRequest({
          method: 'GET',
          path: cloudPath('/instance',),
        },);
        const clockReads = standIn.calls.filter(function isClock(call,) {
          return call.path === AUTH_TIME_PATH;
        },);
        expect(clockReads.length,).toBe(1,);
        const instanceReads = standIn.calls.filter(function isInstanceRead(call,) {
          return call.path === cloudPath('/instance',);
        },);
        const [first,] = instanceReads;
        if (first === undefined) {
          throw new Error('expected the instance list to be requested',);
        }
        expect(instanceReads.length,).toBe(2,);
        const sent = Number(first.headers['X-Ovh-Timestamp'],);
        expect(sent,).toBeGreaterThanOrEqual(STAND_IN_SERVER_TIME,);
        expect(sent,).toBeLessThanOrEqual(STAND_IN_SERVER_TIME + 1,);
        expect(first.headers['X-Ovh-Application'],).toBeDefined();
        expect(first.headers['X-Ovh-Signature'],).toContain('$1$',);
        forgetServerClockOffset();
      },
    },),
    it({
      name: 'ovhRequest reports a clock endpoint that refuses to answer',
      fn: async () => {
        await using _env = await installEnv();
        forgetServerClockOffset();
        using _standIn = installRawStandIn(() => new Response(
          'no clock here',
          { status: 503, },
        ),);
        const error = await errorFrom({
          run: async function request() {
            await ovhRequest({
              method: 'GET',
              path: cloudPath('/instance',),
            },);
          },
        },);
        forgetServerClockOffset();
        if (!(error instanceof OvhApiError)) {
          throw new Error(`expected an OvhApiError, got ${String(error)}`,);
        }
        expect(error.status,).toBe(503,);
        expect(error.message,).toContain('could not read the API host clock',);
      },
    },),

    //endregion Host clock

    //region Masking

    it({
      name: 'maskProjectId replaces every occurrence of the project identifier',
      fn: async () => {
        expect(
          maskProjectId({
            path: `/v1/cloud/project/${STAND_IN_PROJECT}/instance/${STAND_IN_PROJECT}`,
            projectId: STAND_IN_PROJECT,
          },),
        ).toBe(`/v1/cloud/project/${MASKED_PROJECT}/instance/${MASKED_PROJECT}`,);
        expect(
          maskProjectId({
            path: '/v1/cloud/project/x/instance',
            projectId: '',
          },),
        ).toBe('/v1/cloud/project/x/instance',);
      },
    },),
    it({
      name: 'a failed request names its method, its masked path, its status and OVHcloud own message',
      fn: async () => {
        await using _env = await installEnv();
        forgetServerClockOffset();
        using _standIn = installStandIn(() => failureAnswer({
          message: 'This flavor is not available in this region',
          status: 400,
        },),);
        const error = await errorFrom({
          run: async function request() {
            await ovhRequest({
              method: 'POST',
              path: cloudPath('/instance',),
              body: { name: 'mvm-dev', },
            },);
          },
        },);
        if (!(error instanceof OvhApiError)) {
          throw new Error(`expected an OvhApiError, got ${String(error)}`,);
        }
        expect(error.status,).toBe(400,);
        expect(error.errorCode,).toBe('INVALID_PARAMETER',);
        expect(error.path,).toBe(`/v1/cloud/project/${MASKED_PROJECT}/instance`,);
        expect(error.message,).toContain('POST',);
        expect(error.message,).toContain('400',);
        expect(error.message,).toContain('This flavor is not available in this region',);
        expect(error.message,).not.toContain(STAND_IN_PROJECT,);
        expect(error.message,).not.toContain(STAND_IN_APPLICATION_SECRET,);
      },
    },),
    it({
      name: 'a failure whose body is not JSON still names the status and the status text',
      fn: async () => {
        await using _env = await installEnv();
        forgetServerClockOffset();
        using _standIn = installStandIn(() => new Response(
          'the gateway gave up',
          {
            status: 502,
            statusText: 'Bad Gateway',
          },
        ),);
        const error = await errorFrom({
          run: async function request() {
            await ovhRequest({
              method: 'GET',
              path: cloudPath('/instance',),
            },);
          },
        },);
        expect(caughtValueText(error,),).toContain('Bad Gateway',);
      },
    },),

    //endregion Masking

    //region Keys never leave

    it({
      name: 'no key value reaches a stream, a log line, a message or a thrown error',
      fn: async ctx => {
        await using _env = await installEnv();
        forgetServerClockOffset();
        using _standIn = installStandIn(() => failureAnswer({
          message: 'the project quota is exhausted',
          status: 400,
        },),);
        /**
         Everything written to either standard stream, or to a console method,
         while the failing call runs.
         */
        const stderr = ctx.sinon.stub(process.stderr, 'write',);
        stderr.returns(true,);
        const stdout = ctx.sinon.stub(process.stdout, 'write',);
        stdout.returns(true,);
        const consoleLog = ctx.sinon.stub(console, 'log',);
        const consoleError = ctx.sinon.stub(console, 'error',);
        const error = await errorFrom({
          run: async function request() {
            await ovhRequest({
              method: 'GET',
              path: cloudPath('/instance',),
            },);
          },
        },);
        // Positive control: the capture can show a value, so a missing key below means absence, not a blind probe.
        process.stderr.write('capture-probe',);
        /**
         Everything the capture collected, as one text.
         */
        const printed = [
          ...stderr.args,
          ...stdout.args,
          ...consoleLog.args,
          ...consoleError.args,
        ]
          .map(function asText(args,) {
            return args.map(String,)
              .join(' ',);
          },)
          .join('\n',);
        expect(printed,).toContain('capture-probe',);
        /**
         Everything the thrown error carries, message and stack together.
         */
        const thrown = caughtValueStack(error,);
        for (const value of [
          STAND_IN_APPLICATION_SECRET,
          STAND_IN_CONSUMER_KEY,
          STAND_IN_PROJECT,
        ]) {
          expect(printed,).not.toContain(value,);
          expect(thrown,).not.toContain(value,);
        }
        expect(thrown,).toContain('the project quota is exhausted',);
      },
    },),

    //endregion Keys never leave

    //region Reads

    it({
      name: 'ovhList returns the owned instances with their status lower-cased',
      fn: async () => {
        await using _env = await installEnv();
        forgetServerClockOffset();
        using _standIn = installStandIn(() => jsonAnswer([
          {
            id: 'instance-1',
            name: 'mvm-dev',
            region: 'bhs5',
            status: 'ACTIVE',
            imageId: 'image-1',
            flavorId: 'flavor-1',
            ipAddresses: [],
          },
        ],),);
        expect(await ovhList(),).toEqual([{ name: 'dev', state: 'active', },],);
      },
    },),

    //endregion Reads
  ],
},);
