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
// WHICH SEAT A REQUEST TO A MODEL CLIENT NAMES, READ FROM THE SOURCE TEXT (ledger B1373). The router holds a model off
// the provider its card names for the role of the request's seat, and a request that names no seat is held to the holds
// every role honours; a caller that names a seat or a measurement keeps the holds of that role or is exempt. A new
// caller that leaves the seat out would therefore be held to the widest rule without having been asked, so this reads
// every request an object literal builds and says whether it names its seat.
//
// A REQUEST THE CALL PASSES ON IS NOT BUILT THERE. A wrapper that hands on the request it was given (`inner.chatText(
// request)`, `client.chatText({ ...request, reuseScope })`) names the seat of the caller above it, and the scan reads
// that caller where it builds the request. A spread of anything but a plain name can carry anything and is read as no
// seat, since a conditional spread is how a seat once became optional.

/**
 The methods of a model client that take a request.
 */
const REQUEST_METHODS: ReadonlySet<string> = new Set([
  'chatJson',
  'chatText',
]);

/**
 The three dots that open a spread property.
 */
const SPREAD_MARK = '...';

/**
 The key a request names its seat under.
 */
const SEAT_KEY = 'reaskSeat';

/**
 What a request that is handed on, or built by a spread of a request alone, is read as.
 */
export const REQUEST_PASSED_ON = 'passes a request on';

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
 Reads the seat each request an object literal builds names, at the calls of the client methods that take a request.

 @param source - text of the file

 @returns One entry per call of `chatText` or `chatJson` as a method, in source order: the value named under the seat
 key, `passes a request on` where the call hands on a name or spreads a plain name alone, `none` where it names no seat,
 `only a spread` where the only trace of one is a spread that is no plain name, and `duplicate reaskSeat: a, b` where the
 key is written twice; empty where the file has no such call

 @example
 ```ts
 expect(seatsNamedAtRequestCalls({ source, },),).toEqual(['reaskSeat',],);
 ```
 */
export function seatsNamedAtRequestCalls({ source, }: { readonly source: string; },): readonly string[] {
  /**
   Tokens of the file, comments dropped.
   */
  const tokens = tokensOf({ source, },);

  /**
   What each call names.
   */
  const named: string[] = [];
  for (let at = 0; at < tokens.length; at += 1) {
    /**
     The token under question.
     */
    const token = tokens[at];
    if ((token?.kind !== 'word') || (!REQUEST_METHODS.has(token.text,)))
      continue;
    // A METHOD CALL ONLY: a declaration (`function chatText`), a property of an object (`chatText: ...`) and a bare call
    // of a local function are no request to a client.
    if (tokens[at - 1]
      ?.text
      !== '.')
      continue;

    /**
     Index of the token after the name and its type arguments.
     */
    const afterName = (tokens[at + 1]
      ?.text
      === '<') ? afterBracket({
        tokens,
        open: at + 1,
        angled: true,
      },) : at + 1;
    if (tokens[afterName]
      ?.text
      !== '(')
      continue;
    if (tokens[afterName + 1]
      ?.text
      !== '{') {
      named.push(REQUEST_PASSED_ON,);
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
    /**
     Properties of the argument object.
     */
    const properties = propertiesOf({
      body: tokens.slice(
        afterName + 2,
        afterObject - 1,
      ),
    },);
    /**
     What the seat key is given, as the other readers of a call's literal read it.
     */
    const seat = namedUnderKey({
      properties,
      key: SEAT_KEY,
    },);
    /**
     Whether every spread of the literal is a plain name, which hands a request on.
     */
    const spreadsAreRequests = properties
      .filter(function opensWithSpread(property,): boolean {
        return isSpread(property,);
      },)
      .every(function handsARequestOn(property,): boolean {
        return isPlainSpread(property,);
      },);
    named.push((seat === 'only a spread') && spreadsAreRequests ? REQUEST_PASSED_ON : seat,);
  }
  return named;
}

//endregion Seat named at a model request
