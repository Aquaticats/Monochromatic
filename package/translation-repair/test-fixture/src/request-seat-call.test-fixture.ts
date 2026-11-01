import {
  isSpread,
  namedUnderKey,
} from './reask-role-call.test-fixture.ts';
import {
  afterBracket,
  propertiesOf,
  type Token,
  tokensOf,
} from './reask-role-tokens.test-fixture.ts';

//region Seat named at a model request
// WHICH SEAT A REQUEST TO A MODEL CLIENT NAMES, READ FROM THE SOURCE TEXT (ledger B1373, B1412). The router holds a model
// off the provider its card names for the role of the request's seat, and a request that names no seat is held to the
// holds every role honours; a caller that names a seat or a measurement keeps the holds of that role or is exempt. A new
// caller that leaves the seat out would therefore be held to the widest rule without having been asked, so this reads
// every request an object literal builds and says whether it names its seat.
//
// THE COMPILER AND THIS SCAN SHARE THE WORK (ledger B1412). A literal that builds a request (it names `modelId`) is
// checked against `SeatedChatTextRequest` or `SeatedChatJsonRequest` with `satisfies`, whose seat is required and never
// undefined, so a seat left out, a conditional seat and a spread of an unrelated object fail to type-check;
// `requestsNotCheckedAsSeated` finds a builder that dropped the `satisfies`. The reading of `reaskSeat` is the backstop
// for a builder in a file the compiler check does not reach, and for a seat the type accepts but a caller should not
// write (a seat read off another request, which may be absent).
//
// A REQUEST THE CALL PASSES ON IS NOT BUILT THERE. A wrapper that hands on the request it was given (`inner.chatText(
// request)`, `client.chatText({ ...request, reuseScope })`) names the seat of the caller above it, and the scan reads
// that caller where it builds the request. A spread is read as a request passed on only where the literal names none of
// the keys a request requires: the compiler then proved the spread supplies them, so it is a request. A literal that
// names every required key and spreads something besides builds a new request from it, and a spread of anything but a
// plain name can carry anything; both read as no seat, since a conditional spread is how a seat once became optional.
//
// THE REACH, as the ledger states it: a request built in a variable and passed by name, a seat that is named and wrong,
// a client method called through an alias or a destructured binding (`const { chatText } = client`), and a request
// passed on whose seat the wrapper above it dropped (`const { reaskSeat, ...rest } = request`) are out of the scan's
// reach.

/**
 The methods of a model client that take a request.
 */
const REQUEST_METHODS: ReadonlySet<string> = new Set([
  'chatJson',
  'chatText',
]);

/**
 The keys a request must name for the compiler to accept it, which a spread can supply in their place. The source of
 truth is `ChatTextRequest` in `provider/src/chat-contract.ts`, and the unit test of the scan reads the type to hold this
 list to it.
 */
export const REQUIRED_REQUEST_KEYS: readonly string[] = [
  'modelId',
  'messages',
  'signal',
];

/**
 The three dots that open a spread property.
 */
const SPREAD_MARK = '...';

/**
 The key a request names its seat under.
 */
const SEAT_KEY = 'reaskSeat';

/**
 The prefix of the types a builder checks its literal against.
 */
const SEATED_TYPE_PREFIX = 'Seated';

/**
 What a request that is handed on, or built by a spread of a request alone, is read as.
 */
export const REQUEST_PASSED_ON = 'passes a request on';

/**
 The prefix of a reading of a seat that the literal writes but that may be absent when the call runs.
 */
export const SEAT_MAY_BE_ABSENT = 'may be absent: ';

/**
 One call of a request method as the scan reads it.
 */
type RequestCall = {
  /**
   Tokens of the whole file, which the argument is read from.
   */
  readonly tokens: readonly Token[];

  /**
   Properties of the argument object literal, or none where the call hands a name on.
   */
  readonly properties: readonly (readonly Token[])[];

  /**
   Whether the argument is a literal and not a name.
   */
  readonly isLiteral: boolean;

  /**
   Whether the literal is followed by `satisfies` and a seated request type.
   */
  readonly checkedAsSeated: boolean;
};

/**
 Whether a property is a spread of a plain name, `...request`.

 @param property - tokens of one property

 @returns True when it is three dots and one word, and nothing else

 @example
 ```ts
 const plain = isPlainSpread(property,);
 ```
 */
function isPlainSpread(property: readonly Token[],): boolean {
  return isSpread(property,)
    && (property.length === (SPREAD_MARK.length + 1))
    && (property[SPREAD_MARK.length]
      ?.kind
      === 'word');
}

/**
 Whether a property is written under a key a request must name.

 @param property - tokens of one property

 @returns True when its first token is one of the required keys

 @example
 ```ts
 const names = namesRequiredKey(property,);
 ```
 */
function namesRequiredKey(property: readonly Token[],): boolean {
  return REQUIRED_REQUEST_KEYS.includes(property[0]
    ?.text
    ?? '',);
}

/**
 Reads each call of a request method in a source text.

 @param source - text of the file

 @returns One entry per call of `chatText` or `chatJson` as a method, dotted or bracketed, in source order

 @example
 ```ts
 const calls = requestCallsIn({ source, },);
 ```
 */
function requestCallsIn({ source, }: { readonly source: string; },): readonly RequestCall[] {
  /**
   Tokens of the file, comments dropped.
   */
  const tokens = tokensOf({ source, },);

  /**
   What each call hands over.
   */
  const calls: RequestCall[] = [];
  for (let at = 0; at < tokens.length; at += 1) {
    /**
     The token under question.
     */
    const token = tokens[at];
    if ((token === undefined) || (!REQUEST_METHODS.has(token.text,)))
      continue;
    // A METHOD CALL ONLY: a declaration (`function chatText`), a property of an object (`chatText: ...`) and a bare call
    // of a local function are no request to a client. The method is named after a dot (`client.chatText(`), or by a
    // string in brackets (`client['chatText'](`), which reads the same.
    /**
     The token before the name.
     */
    const before = tokens[at - 1];
    /**
     The token after the name.
     */
    const behind = tokens[at + 1];
    /**
     Whether the name is written as a string in brackets.
     */
    const bracketed = (token.kind === 'string')
      && (before?.text === '[')
      && (behind?.text === ']');
    /**
     Whether the name is written as a member after a dot.
     */
    const dotted = (token.kind === 'word') && (before?.text === '.');
    if ((!bracketed) && (!dotted))
      continue;

    /**
     Index of the token after the name and its type arguments.
     */
    const afterName = bracketed ? at + 2 : ((behind?.text === '<') ? afterBracket({
      tokens,
      open: at + 1,
      angled: true,
    },) : at + 1);
    if (tokens[afterName]
      ?.text
      !== '(')
      continue;
    if (tokens[afterName + 1]
      ?.text
      !== '{') {
      calls.push({
        tokens,
        properties: [],
        isLiteral: false,
        checkedAsSeated: false,
      },);
      continue;
    }

    /**
     Index one past the closing brace of the argument object.
     */
    const afterObject = afterBracket({
      tokens,
      open: afterName + 1,
      angled: false,
    },);
    calls.push({
      tokens,
      properties: propertiesOf({
        body: tokens.slice(
          afterName + 2,
          afterObject - 1,
        ),
      },),
      isLiteral: true,
      checkedAsSeated: (tokens[afterObject]
        ?.text
        === 'satisfies') && (tokens[afterObject + 1]
          ?.text
          ?.startsWith(SEATED_TYPE_PREFIX,)
          === true),
    },);
  }
  return calls;
}

/**
 Reads what the seat key of a literal is given, apart from the readings of its absence.

 @param properties - properties of the call's argument object

 @returns The reading of `namedUnderKey`, or the same text behind the mark of a seat that may be absent where the value
 written is `undefined`, a conditional, or a seat read off another object (which is optional on every request)

 @example
 ```ts
 const seat = seatReadingOf({ properties, },);
 ```
 */
function seatReadingOf({ properties, }: { readonly properties: readonly (readonly Token[])[]; },): string {
  /**
   What the seat key is given, as the other readers of a call's literal read it.
   */
  const seat = namedUnderKey({
    properties,
    key: SEAT_KEY,
  },);
  /**
   Whether the value written can be absent when the call runs: the word `undefined`, a `?` of a conditional or of a
   fallback, or a member read of another object, whose `reaskSeat` is optional.
   */
  const isUndefined = seat === 'undefined';
  /**
   Whether it holds a conditional or a fallback.
   */
  const isConditional = seat.includes('?',);
  /**
   Whether it reads the seat off another object.
   */
  const isReadOff = seat.endsWith(`.${SEAT_KEY}`,);
  return (isUndefined
    || isConditional
    || isReadOff)
    ? `${SEAT_MAY_BE_ABSENT}${seat}`
    : seat;
}

/**
 Reads the seat each request an object literal builds names, at the calls of the client methods that take a request.

 @param source - text of the file

 @returns One entry per call of `chatText` or `chatJson` as a method, in source order: the value named under the seat
 key, `passes a request on` where the call hands on a name or spreads a plain name alone and names no key a request
 requires, `none` where it names no seat, `only a spread` where the only trace of one is a spread that is no such pass,
 `may be absent: <value>` where the value written can be absent, and `duplicate reaskSeat: a, b` where the key is
 written twice; empty where the file has no such call

 @example
 ```ts
 expect(seatsNamedAtRequestCalls({ source, },),).toEqual(['reaskSeat',],);
 ```
 */
export function seatsNamedAtRequestCalls({ source, }: { readonly source: string; },): readonly string[] {
  return requestCallsIn({ source, },)
    .map(function seatOf(call,): string {
      if (!call.isLiteral)
        return REQUEST_PASSED_ON;
      /**
       Whether every spread of the literal is a plain name, which hands a request on.
       */
      const spreadsAreRequests = call.properties
        .filter(function opensWithSpread(property,): boolean {
          return isSpread(property,);
        },)
        .every(function handsARequestOn(property,): boolean {
          return isPlainSpread(property,);
        },);
      /**
       Whether the literal builds the request itself, so no spread of it is proved a request by the type check.
       */
      const buildsItself = call.properties
        .some(namesRequiredKey,);
      /**
       What the literal gives the seat key.
       */
      const seat = seatReadingOf({ properties: call.properties, },);
      /**
       Whether the spread alone stands for the request.
       */
      const spreadIsTheRequest = (seat === 'only a spread')
        && spreadsAreRequests
        && (!buildsItself);
      return spreadIsTheRequest ? REQUEST_PASSED_ON : seat;
    },);
}

/**
 Counts the calls whose literal builds a request and is not checked against a seated request type.

 @param source - text of the file

 @returns For each such call, in source order, the reading of its seat; empty where every builder carries its check

 @example
 ```ts
 expect(requestsNotCheckedAsSeated({ source, },),).toEqual([],);
 ```
 */
export function requestsNotCheckedAsSeated({ source, }: { readonly source: string; },): readonly string[] {
  return requestCallsIn({ source, },)
    .filter(function buildsWithoutTheCheck(call,): boolean {
      /**
       Whether the literal names a key a request requires, which makes it a builder.
       */
      const builds = call.properties
        .some(namesRequiredKey,);
      return call.isLiteral
        && builds
        && (!call.checkedAsSeated);
    },)
    .map(function seatOf(call,): string {
      return seatReadingOf({ properties: call.properties, },);
    },);
}

//endregion Seat named at a model request
