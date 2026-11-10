/**
 Regex-free scans over the text of a generated design review.
 
 The review builders in `package/music-player/design` validate their own output
 before it is committed. Every check in this module answers
 `package/config/oxlint`'s `no-restricted-syntax/no-regex`: that rule asks for an
 index scan, a parser or a string API instead of a pattern, and these functions
 are the index scans the validators share.
 
 Each scan walks its input once from left to right and keeps no backtracking
 state, so its cost is linear in the document length for any input.

 @module
 */

//region Constants

/**
 Literal that opens an embedded base64 PNG payload inside a `data:` URL.
 */
const PNG_DATA_URL_PREFIX = 'data:image/png;base64,';

/**
 Characters the base64 alphabet adds to the ASCII letters and digits: the two
 extra symbols and the padding character.
 */
const BASE64_SYMBOLS = '+/=';

//endregion Constants

//region Base64 payloads

/**
 Whether one UTF-16 code unit belongs to the base64 alphabet.
 
 @param character - single code unit to classify
 
 @returns whether the code unit can appear inside a base64 payload
 
 @example
 ```ts
 isBase64Character({ character: 'A' }); // true
 isBase64Character({ character: '!' }); // false
 ```
 */
function isBase64Character(
  { character, }: { readonly character: string; },
): boolean {
  if (character
    === '')
    return false;
  if ((character
    >= 'A') && (character
      <= 'Z'))
    return true;
  if ((character
    >= 'a') && (character
      <= 'z'))
    return true;
  if ((character
    >= '0') && (character
      <= '9'))
    return true;
  return BASE64_SYMBOLS.includes(character,);
}

/**
 Every base64 PNG payload a document embeds, in document order.
 
 The regular expression this replaced,
 `[...text.matchAll(/data:image\/png;base64,([A-Za-z0-9+/=]+)/g)]`,
 reported the maximal base64 run after each prefix and resumed after that run, so
 a payload was never reported twice and a prefix with no base64 character after it
 reported nothing. This scan keeps both properties: the run is maximal because the
 walk stops at the first code unit outside the alphabet, and the next search
 resumes one code unit past a prefix that carried no payload, which is the
 position the pattern engine itself retried from.
 
 @param text - document text whose embedded payloads are collected
 
 @returns payload text without the `data:` URL prefix, in document order
 
 @example
 ```ts
 findPngDataPayloads({ text: 'src="data:image/png;base64,AAEC"' }); // ['AAEC']
 ```
 */
export function findPngDataPayloads(
  { text, }: { readonly text: string; },
): readonly string[] {
  /**
   Payloads found so far, in document order.
   */
  const payloads: string[] = [];
  /**
   Position the next prefix search starts from, past everything already decided.
   
   A `for` initializer rather than a function-root `let`: the counter belongs to
   the loop, and `no-restricted-syntax/no-function-root-let` keeps a mutable root
   binding from leaking into every statement after it.
   */
  for (let searchFrom = 0; searchFrom < text.length;) {
    /**
     Position of the next `data:` URL prefix, or -1 when the document holds none.
     */
    const prefixAt = text.indexOf(
      PNG_DATA_URL_PREFIX,
      searchFrom,
    );
    if (prefixAt
      === (-1))
      break;
    /**
     First code unit after the prefix, where a payload can begin.
     */
    const payloadStart = prefixAt + PNG_DATA_URL_PREFIX.length;
    /**
     End of the maximal base64 run, equal to `payloadStart` for an empty run.
     */
    let payloadEnd = payloadStart;
    while ((payloadEnd
      < text.length)
      && isBase64Character({ character: text.charAt(payloadEnd,), },))
      payloadEnd += 1;
    if (payloadEnd
      === payloadStart) {
      searchFrom = payloadStart;
      continue;
    }
    payloads.push(text.slice(
      payloadStart,
      payloadEnd,
    ),);
    searchFrom = payloadEnd;
  }
  return payloads;
}

//endregion Base64 payloads
