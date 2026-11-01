/**
 Guards that every request a production caller builds for a model client names its seat (ledger B1373).

 The router holds a model off the provider its card names for the role of the request's seat, and since this ledger entry
 a request that names no seat is held to the holds every role honours on its first ask too. A caller that leaves the seat
 out is therefore held to the widest rule without having been asked, and a caller that is a measurement must say so
 (`reask: 'measured-model'`) to stay exempt. The model health probe and the picture reader named no seat before this
 scan, and the first ask of each was served on whatever provider the policy chose.

 WHAT THE SCAN READS, in package source (not tests and fixtures): every `chatText` and `chatJson` called as a method
 (dotted or by a bracketed name) with an object literal for its request, and the value that literal names under
 `reaskSeat`. A request that is handed on (`inner.chatText(request)`) or built by a spread of a plain name in a literal
 that names none of the keys a request requires (`{ ...request, reuseScope }`) names the seat of the caller above it and
 is read where that caller builds it: the type check proved the spread supplies those keys, so it is a request.

 THE COMPILER SHARES THE WORK (ledger B1412). A literal that builds a request, one that names `modelId`, is followed by
 `satisfies SeatedChatTextRequest` or `satisfies SeatedChatJsonRequest`, whose seat is required and never undefined; the
 scan finds a builder that has no such check. A required seat on `ChatTextRequest` itself would touch every request
 literal of the tests and the fixtures (B1373 measured 219 in 49 files) and would still not reach a wrapper, whose
 request arrives typed as the loose request.

 OUT OF THE SCAN'S REACH: a request built in a variable and passed by name, which is read as handed on; a seat that
 is named and wrong; a client method called through an alias or a destructured binding; and a request passed on whose
 seat a wrapper dropped before passing it (`const { reaskSeat, ...rest } = request`).

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
  afterBracket,
  tokensOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-tokens.test-fixture.ts';
import {
  REQUEST_PASSED_ON,
  REQUIRED_REQUEST_KEYS,
  requestsNotCheckedAsSeated,
  SEAT_MAY_BE_ABSENT,
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
    || seat.startsWith(SEAT_MAY_BE_ABSENT,)
    || seat.endsWith(' then a spread',);
}

/**
 Names of the properties a request type requires, read from the source of its declaration.

 @param source - text of the file that declares `ChatTextRequest`

 @returns Names of the properties written without a question mark, in source order

 @example
 ```ts
 const required = requiredKeysOfChatTextRequest({ source, });
 ```
 */
function requiredKeysOfChatTextRequest({ source, }: { readonly source: string; },): readonly string[] {
  /**
   Tokens of the file, comments dropped.
   */
  const tokens = tokensOf({ source, },);
  /**
   Index of the name in `type ChatTextRequest = {`.
   */
  const named = tokens.findIndex(function isDeclaredName(token, at,): boolean {
    return (token.text === 'ChatTextRequest') && (tokens[at - 1]
      ?.text
      === 'type');
  },);
  /**
   Index of the brace that opens the type.
   */
  const open = tokens.findIndex(function isOpening(token, at,): boolean {
    return (at > named) && (token.text === '{');
  },);
  /**
   Index one past the brace that closes it.
   */
  const end = afterBracket({
    tokens,
    open,
    angled: false,
  },);
  /**
   Names of required properties found so far.
   */
  const required: string[] = [];
  /**
   Bracket depth inside the type; its own properties sit at one.
   */
  let depth = 0;
  /**
   Whether the next word at depth one begins a property.
   */
  let atPropertyStart = false;
  for (const [at, token,] of tokens.slice(
    open,
    end,
  ).entries()) {
    if ((token.text === '{') || (token.text === '(') || (token.text === '['))
      depth += 1;
    if ((token.text === '}') || (token.text === ')') || (token.text === ']'))
      depth -= 1;
    if ((depth === 1) && ((token.text === '{') || (token.text === ';'))) {
      atPropertyStart = true;
      continue;
    }
    if ((depth !== 1) || (!atPropertyStart) || (token.kind !== 'word') || (token.text === 'readonly'))
      continue;
    atPropertyStart = false;
    if (tokens[open + at + 1]
      ?.text
      === ':')
      required.push(token.text,);
  }
  return required;
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
      name: 'FINDS A SPREAD OF A PLAIN NAME THAT IS NO REQUEST, a seat that may be absent and a call by a bracketed '
        + 'name (ledger B1412)',
      fn: async () => {
        // The spread of an unrelated object into a literal that names every key a request requires proves nothing: the
        // seat is in the object or it is not.
        expect(seatsOf('const options = { exchangeTimeoutMs: 1000, };\nclient.chatText({ modelId, messages, signal, ...options, },);',),)
          .toEqual(['only a spread',],);
        expect(seatsOf('client.chatText({ modelId, messages, signal, reaskSeat: flag ? seat : undefined, },);',),)
          .toEqual(['may be absent: flag?seat:undefined',],);
        expect(seatsOf('client.chatText({ modelId, reaskSeat: undefined, },);',),).toEqual(['may be absent: undefined',],);
        expect(seatsOf('client.chatText({ modelId, reaskSeat: request.reaskSeat, },);',),)
          .toEqual(['may be absent: request.reaskSeat',],);
        expect(seatsOf('client[\'chatText\']({ modelId, },);',),).toEqual(['none',],);
        expect(seatsOf('client?.chatJson({ modelId, },);',),).toEqual(['none',],);
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
      name: 'READS A SPREAD OF A REAL REQUEST AS PASSED ON ONLY WHERE THE LITERAL BUILDS NOTHING BESIDE IT (ledger B1412)',
      fn: async () => {
        expect(seatsOf('client.chatText({ ...request, reuseScope, },);',),).toEqual([REQUEST_PASSED_ON,],);
        expect(seatsOf('client.chatText({ ...request, exchangeTimeoutMs, },);',),).toEqual([REQUEST_PASSED_ON,],);
        // A literal that names a key a request requires builds a request, so the spread beside it proves nothing.
        for (const key of REQUIRED_REQUEST_KEYS) {
          expect(seatsOf(`client.chatText({ ${key}, ...request, },);`,),).toEqual(['only a spread',],);
        }
      },
    },),

    it({
      name: 'FINDS A BUILDER THAT IS NOT CHECKED AGAINST A SEATED REQUEST TYPE and reads a wrapper and a checked builder '
        + 'as having nothing to check (ledger B1412)',
      fn: async () => {
        expect(requestsNotCheckedAsSeated({ source: 'client.chatText({ modelId, reaskSeat, },);', },),).toEqual([
          'reaskSeat',
        ],);
        expect(requestsNotCheckedAsSeated({ source: 'client.chatText({ modelId, reaskSeat, } satisfies Request,);', },),)
          .toEqual(['reaskSeat',],);
        expect(requestsNotCheckedAsSeated({
          source: 'client.chatText({ modelId, reaskSeat, } satisfies SeatedChatTextRequest,);',
        },),).toEqual([],);
        expect(requestsNotCheckedAsSeated({
          source: 'client.chatJson({ modelId, reaskSeat, validate, } satisfies SeatedChatJsonRequest<Verdict>,);',
        },),).toEqual([],);
        expect(requestsNotCheckedAsSeated({
          source: 'client.chatText({ ...request, reuseScope, },);\ninner.chatText(request,);',
        },),).toEqual([],);
      },
    },),

    it({
      name: 'HOLDS THE KEYS THE SCAN CALLS REQUIRED TO THE KEYS THE REQUEST TYPE REQUIRES (ledger B1412), and the type has '
        + 'a required key to compare',
      fn: async () => {
        /**
         The file that declares the request type.
         */
        const contract = (await readPackageSource()).find(function isContract(file,): boolean {
          return file.path.endsWith('/chat-contract.ts',) && file.path.startsWith('provider/',);
        },);
        /**
         Keys the type requires.
         */
        const required = requiredKeysOfChatTextRequest({ source: contract?.text ?? '', },);
        expect(required.length,).toBeGreaterThan(0,);
        expect(required.toSorted(),).toEqual(REQUIRED_REQUEST_KEYS.toSorted(),);
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
            return {
              path: file.path,
              seats: seatsOf(file.text,),
              unchecked: requestsNotCheckedAsSeated({ source: file.text, },),
            };
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
        expectNoFindings({
          findings: readings.flatMap(function unchecked(reading,): readonly string[] {
            return reading.unchecked
              .map(function toFinding(seat,): string {
                return `${reading.path}: a request literal naming its model and its seat as ${seat} is not checked with `
                  + '`satisfies` against a seated request type';
              },);
          },),
        },);
      },
    },),
  ],
},);

//endregion Request seat scan
