/**
 Guards that every request a production caller builds for a model client names its seat (ledger B1373).

 The router holds a model off the provider its card names for the role of the request's seat, and since this ledger entry
 a request that names no seat is held to the holds every role honours on its first ask too. A caller that leaves the seat
 out is therefore held to the widest rule without having been asked, and a caller that is a measurement must say so
 (`reask: 'measured-model'`) to stay exempt. The model health probe and the picture reader named no seat before this
 scan, and the first ask of each was served on whatever provider the policy chose.

 WHAT THE SCAN READS, in package source (not tests and fixtures): every `chatText` and `chatJson` called as a method with
 an object literal for its request, and the value that literal names under `reaskSeat`. A request that is handed on
 (`inner.chatText(request)`) or built by spreading a plain name alone (`{ ...request, reuseScope }`) names the seat of the
 caller above it and is read where that caller builds it.

 OUT OF THE SCAN'S REACH: a request built in a variable and passed by name, which is read as handed on; a seat that
 is named and wrong; and a client method called through an alias.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';
import {
  REQUEST_PASSED_ON,
  seatsNamedAtRequestCalls,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/request-seat-call.test-fixture.ts';

//region Request seat scan

/**
 Whether a reading says the request names no seat: no key, a key that only a spread may replace, or a key written twice.

 @param seat - reading of one request

 @returns True when the request does not name its seat once and plainly

 @example
 ```ts
 const lacking = namesNoSeat('none',);
 ```
 */
function namesNoSeat(seat: string,): boolean {
  return (seat === 'none')
    || (seat === 'only a spread')
    || seat.startsWith('duplicate',)
    || seat.endsWith(' then a spread',);
}

/**
 Reads the seats a planted source names at its requests.

 @param source - text of the planted file

 @returns One reading per request, in source order

 @example
 ```ts
 const seats = seatsOf('client.chatJson({ modelId, reaskSeat, },);');
 ```
 */
function seatsOf(source: string,): readonly string[] {
  return seatsNamedAtRequestCalls({ source, },);
}

await describe({
  name: 'the seat every production request to a model client names',
  concurrency: 1,
  children: [
    it({
      name: 'FINDS A REQUEST THAT NAMES NO SEAT, whichever method and however the literal is laid out',
      fn: async () => {
        expect(seatsOf('await client.chatJson({ modelId, messages, validate, },);',),).toEqual(['none',],);
        expect(seatsOf('const reply = await this.client\n  .chatText<Reply,>({\n    modelId,\n    signal,\n  },);',),).toEqual(['none',],);
        expect(seatsOf('client.chatJson({ modelId, ...(seat === undefined ? {} : { reaskSeat: seat, }), },);',),).toEqual([
          'only a spread',
        ],);
        expect(seatsOf('client.chatText({ reaskSeat: a, reaskSeat: b, },);',),).toEqual(['duplicate reaskSeat: a, b',],);
        expect(seatsOf('client.chatText({ modelId, ...pick(request,), },);',),).toEqual(['only a spread',],);
      },
    },),

    it({
      name: 'READS A REQUEST THAT NAMES ITS SEAT, OR HANDS ON ONE THAT DOES, as naming it',
      fn: async () => {
        expect(seatsOf('client.chatJson({ modelId, reaskSeat, },);',),).toEqual(['reaskSeat',],);
        expect(seatsOf('client.chatText({ modelId, reaskSeat: seat, },);',),).toEqual(['seat',],);
        expect(seatsOf('inner.chatText(request,);',),).toEqual([REQUEST_PASSED_ON,],);
        expect(seatsOf('client.chatText({ ...request, reuseScope, },);',),).toEqual([REQUEST_PASSED_ON,],);
        expect(seatsOf('client.chatJson<Verdict,>({ ...request, },);',),).toEqual([REQUEST_PASSED_ON,],);
      },
    },),

    it({
      name: 'READS NO REQUEST WHERE THE NAME IS A DECLARATION, A PROPERTY, A COMMENT OR A STRING',
      fn: async () => {
        expect(seatsOf('async function chatText(request) { return 1; }',),).toEqual([],);
        expect(seatsOf('const client = { chatText: async () => 1, chatJson: window.chatJson, };',),).toEqual([],);
        expect(seatsOf('// await client.chatJson({ modelId, },);\n/* client.chatText({ modelId, },); */',),).toEqual([],);
        expect(seatsOf('const text = "client.chatText({ modelId, },)";',),).toEqual([],);
        expect(seatsOf('chatText({ modelId, },);',),).toEqual([],);
      },
    },),

    it({
      name: 'FINDS NO PRODUCTION REQUEST THAT NAMES NO SEAT (ledger B1373), after reading some: the stage call, the picture '
        + 'reader and the model health probe each build a request',
      fn: async () => {
        /**
         Production files and the readings of their requests.
         */
        const readings = (await readPackageSource())
          .filter(function isProduction(file,): boolean {
            return !file.isTest;
          },)
          .map(function read(file,) {
            return { path: file.path, seats: seatsOf(file.text,), };
          },);
        // The positive control: the scan reaches the three builders this entry named, so a change to how it reads
        // cannot leave it empty.
        for (const builder of ['stage-call.ts', 'image-reading-stage.ts', 'model-health-probe.ts',]) {
          expect(readings.some(function isBuilder(reading,): boolean {
            return reading.path.endsWith(`/${builder}`,) && (reading.seats.length > 0);
          },),).toBe(true,);
        }
        expectNoFindings({
          findings: readings.flatMap(function unnamed(reading,): readonly string[] {
            return reading.seats
              .filter(namesNoSeat,)
              .map(function toFinding(seat,): string {
                return `${reading.path}: a request to a model client reads as ${seat} under reaskSeat`;
              },);
          },),
        },);
      },
    },),
  ],
},);

//endregion Request seat scan
